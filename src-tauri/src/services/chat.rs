//! Chat orchestration service.
//!
//! The command layer (`commands::chat`) is a thin IPC delegate; the real
//! orchestration lives here — routing a turn through the Sidecar or the Rig
//! fallback, driving the MCP tool loop, and persisting messages/stats.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use tauri::AppHandle;

use crate::crypto;
use crate::db::models::{Message, RouterConfig, Session};
use crate::db::repository::{CustomModelRepo, MessageRepo, RouterConfigRepo, SessionRepo};
use crate::services::llm::backend::MessageAttachment;
use crate::services::llm::config::LlmConfig;
use crate::services::llm::{RigBackend, StreamResult};
use crate::services::mcp::{McpToolLoop, MAX_TOOL_ROUNDS};
use crate::services::mcp_bridge::McpToolBridge;
use crate::services::sidecar_client::{AgentChatConfig, AgentChatMessage, AgentChatRequest};
use crate::services::sidecar_sse::consume_sidecar_stream;
use crate::services::skills::types::{MessageSkillSelection, SkillActivationView};
use crate::services::thinking_capabilities::lookup_thinking_capability;
use crate::services::usage::collector::ensure_fallback_capture;
use crate::services::usage::finalize::{
    emit_usage_recorded, finalize_turn, FinalizeTurnOutcome, FinalizeTurnRequest,
};
use crate::services::usage::UsageOperationKind;
use crate::AppState;

/// 将附件 JSON 字符串反序列化为 MessageAttachment 列表
pub(crate) fn parse_attachments_json(json: Option<&str>) -> Option<Vec<MessageAttachment>> {
    json.and_then(|s| serde_json::from_str::<Vec<MessageAttachment>>(s).ok())
        .filter(|attachments| !attachments.is_empty())
}

pub(crate) fn resolve_selected_skill_ids(
    state: &AppState,
    requested: &[String],
) -> Result<(SkillActivationView, Vec<MessageSkillSelection>), String> {
    let unique = normalize_skill_ids(requested)?;
    let db = state.db.lock().map_err(|error| error.to_string())?;
    crate::services::skills::registry::resolve_selection(&db, &unique, None)
        .map_err(|error| error.to_string())
}

pub(crate) fn save_message_skill_selection(
    state: &AppState,
    message_id: &str,
    selections: &[MessageSkillSelection],
) -> Result<(), String> {
    let db = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::repository::SkillSourceRepo::replace_message_selection(&db, message_id, selections)
        .map_err(|error| error.to_string())
}

pub(crate) fn load_message_skill_selection(
    state: &AppState,
    message_id: &str,
) -> Result<Vec<MessageSkillSelection>, String> {
    let db = state.db.lock().map_err(|error| error.to_string())?;
    crate::db::repository::SkillSourceRepo::message_selection(&db, message_id)
        .map_err(|error| error.to_string())
}

fn normalize_skill_ids(requested: &[String]) -> Result<Vec<String>, String> {
    if requested.len() > 20 {
        return Err("At most 20 Skills can be selected for one message".to_string());
    }
    let mut unique = Vec::new();
    for raw in requested {
        let identifier = raw.trim();
        if identifier.is_empty() {
            return Err("Skill identifier cannot be empty".to_string());
        }
        if !unique.iter().any(|item| item == identifier) {
            unique.push(identifier.to_string());
        }
    }
    Ok(unique)
}

// ─── 模型标识解析 ───────────────────────────────────────────────────────

/// 模型标识解析结果
#[derive(Debug, Clone)]
pub struct ModelSpec {
    pub config_id: String,
    pub model_id: String,
}

/// Resolved model + thinking strategy for a single turn.
#[derive(Debug, Clone)]
pub struct TurnModel {
    /// User-selected model (session default is not overwritten).
    pub selected: ModelSpec,
    /// Model actually invoked for this turn.
    pub effective: ModelSpec,
    pub thinking_enabled: bool,
    pub uses_native_control: bool,
    pub vendor: Option<String>,
    /// `enabled` | `disabled` when native control applies; otherwise None.
    pub thinking_mode: Option<String>,
}

/// 解析模型标识 — 格式为 "config_id:model_id"
pub fn resolve_model_spec(
    model_override: Option<&str>,
    session_model: Option<&str>,
) -> Result<ModelSpec, String> {
    let raw = model_override
        .or(session_model)
        .ok_or("No model specified: provide model_override or set session model")?;

    if let Some((config_id, model_id)) = raw.split_once(':') {
        Ok(ModelSpec {
            config_id: config_id.to_string(),
            model_id: model_id.to_string(),
        })
    } else {
        Err(format!(
            "Invalid model format '{}'. Expected 'config_id:model_id'",
            raw
        ))
    }
}

/// Resolve which model to call and how thinking is controlled for this turn.
///
/// Must run before persisting user/assistant placeholders so a missing
/// fallback does not leave orphan messages.
pub fn resolve_turn_model(
    state: &AppState,
    selected: &ModelSpec,
    thinking_enabled: bool,
    use_sidecar: bool,
) -> Result<TurnModel, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let router =
        RouterConfigRepo::find_by_id(&db, &selected.config_id).map_err(|e| e.to_string())?;
    let custom = CustomModelRepo::find_enabled(&db, &selected.config_id, &selected.model_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            format!(
                "Model is not enabled for this provider: {}",
                selected.model_id
            )
        })?;

    let vendor = router
        .vendor
        .clone()
        .unwrap_or_else(|| router.provider.clone());
    let catalog = lookup_thinking_capability(&vendor, &selected.model_id);
    let supports_thinking = custom.supports_thinking || catalog.supports_thinking;

    if thinking_enabled || !supports_thinking {
        return Ok(TurnModel {
            selected: selected.clone(),
            effective: selected.clone(),
            thinking_enabled,
            uses_native_control: false,
            vendor: Some(vendor),
            thinking_mode: if thinking_enabled && catalog.native_control.is_some() {
                Some("enabled".to_string())
            } else {
                None
            },
        });
    }

    // Thinking toggled OFF on a thinking-capable model.
    if use_sidecar && catalog.native_control.is_some() {
        return Ok(TurnModel {
            selected: selected.clone(),
            effective: selected.clone(),
            thinking_enabled: false,
            uses_native_control: true,
            vendor: Some(vendor),
            thinking_mode: Some("disabled".to_string()),
        });
    }

    let Some(fallback_id) = custom.thinking_off_model_id.filter(|s| !s.is_empty()) else {
        return Err(format!(
            "Thinking is off, but model '{}' cannot natively disable thinking and no \
             same-provider non-thinking alternate (thinking_off_model_id) is configured. \
             Open Provider settings and pick a non-thinking fallback model.",
            selected.model_id
        ));
    };

    let fallback = CustomModelRepo::find_enabled(&db, &selected.config_id, &fallback_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            format!(
                "Configured thinking-off model '{fallback_id}' is missing or disabled \
                 for this provider."
            )
        })?;

    if fallback.supports_thinking {
        return Err(format!(
            "Configured thinking-off model '{fallback_id}' still supports thinking. \
             Choose a non-thinking alternate in Provider settings."
        ));
    }

    Ok(TurnModel {
        selected: selected.clone(),
        effective: ModelSpec {
            config_id: selected.config_id.clone(),
            model_id: fallback.model_id,
        },
        thinking_enabled: false,
        uses_native_control: false,
        vendor: Some(vendor),
        thinking_mode: None,
    })
}

// ─── Sidecar / Rig 路由 ─────────────────────────────────────────────────

pub(crate) fn read_use_sidecar(state: &AppState) -> bool {
    state.config.lock().map(|c| c.use_sidecar).unwrap_or(true)
}

/// Sidecar chat path — Agent owns MCP via mcp_bridge; do NOT inject MCP prompt.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn send_via_sidecar(
    app: &AppHandle,
    state: &AppState,
    session: &Session,
    history: &[Message],
    user_content: &str,
    turn: &TurnModel,
    llm_config: Option<LlmConfig>,
    skill_activation: SkillActivationView,
    selected_skills: Vec<MessageSkillSelection>,
    abort_flag: Arc<AtomicBool>,
    assistant_msg_id: &str,
) -> Result<(StreamResult, Option<String>), String> {
    if !state.sidecar.is_ready_for_chat() {
        return Err(state.sidecar.not_ready_message());
    }
    let (router_config, decrypted_key) = load_and_decrypt_config(state, &turn.effective.config_id)?;
    let mut request = build_agent_chat_request_for_turn(
        session,
        history,
        user_content,
        turn,
        llm_config,
        &router_config,
        &decrypted_key,
    );
    request.selected_skill_ids = selected_skills
        .iter()
        .map(|skill| skill.skill_id.clone())
        .collect();
    request.skill_activation = Some(skill_activation);
    let response = state.sidecar_client.stream(&request).await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Sidecar stream HTTP {status}: {body}"));
    }

    let outcome =
        consume_sidecar_stream(app, response, &session.id, assistant_msg_id, abort_flag).await?;
    let tool_calls_json = if outcome.tool_calls.is_empty() {
        None
    } else {
        Some(
            serde_json::to_string(&outcome.tool_calls)
                .map_err(|e| format!("Failed to serialize tool calls: {e}"))?,
        )
    };
    Ok((outcome.result, tool_calls_json))
}

/// Rig fallback path — keeps MCP prompt injection and McpToolLoop.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn send_via_rig(
    app: &AppHandle,
    state: &AppState,
    mut session: Session,
    history: Vec<Message>,
    user_content: String,
    attachments: Option<Vec<MessageAttachment>>,
    turn: &TurnModel,
    llm_config: Option<LlmConfig>,
    abort_flag: Arc<AtomicBool>,
    assistant_msg_id: &str,
) -> Result<(StreamResult, Option<String>), String> {
    // Fallback path only: Sidecar agents call MCP through mcp_bridge_tool.
    inject_mcp_prompt(state, &mut session);

    let (router_config, decrypted_key) = load_and_decrypt_config(state, &turn.effective.config_id)?;
    let llm_config = llm_config.unwrap_or_default().sanitized();
    let backend = RigBackend::from_config(&router_config, &decrypted_key, llm_config)
        .map_err(|e| format!("Failed to create backend: {e}"))?;

    run_assistant_turn(
        app,
        state,
        &backend,
        &session,
        history,
        user_content,
        attachments,
        &turn.effective.model_id,
        abort_flag,
        assistant_msg_id,
    )
    .await
}

// ─── 请求构造 ───────────────────────────────────────────────────────────

/// Build AgentChatRequest for Sidecar (text-only; attachments stay on Rig path).
///
/// Includes provider binding so Python uses the same protocol/base_url/key as Rig.
pub fn build_agent_chat_request(
    session: &Session,
    history: &[Message],
    user_content: &str,
    model_id: &str,
    llm_config: Option<LlmConfig>,
    router: &RouterConfig,
    api_key: &str,
) -> AgentChatRequest {
    let llm_config = llm_config.unwrap_or_default().sanitized();
    let mut messages = build_agent_messages(session, history);
    messages.push(AgentChatMessage {
        role: "user".to_string(),
        content: user_content.to_string(),
        name: None,
        tool_call_id: None,
    });

    AgentChatRequest {
        messages,
        config: AgentChatConfig {
            model: Some(model_id.to_string()),
            temperature: llm_config.temperature,
            max_tokens: llm_config.max_tokens,
            stream: true,
            provider: Some(router.provider.clone()),
            vendor: router.vendor.clone(),
            api_compat: router.api_compat.clone(),
            base_url: router.base_url.clone(),
            api_key: Some(api_key.to_string()),
            thinking_enabled: Some(llm_config.thinking_enabled),
            thinking_mode: None,
        },
        session_id: Some(session.id.clone()),
        working_dir: session.working_directory.clone(),
        agent_mode: llm_config.agent_mode.clone(),
        selected_skill_ids: Vec::new(),
        skill_activation: None,
    }
}

/// Build AgentChatRequest using a resolved [`TurnModel`].
pub fn build_agent_chat_request_for_turn(
    session: &Session,
    history: &[Message],
    user_content: &str,
    turn: &TurnModel,
    llm_config: Option<LlmConfig>,
    router: &RouterConfig,
    api_key: &str,
) -> AgentChatRequest {
    let mut request = build_agent_chat_request(
        session,
        history,
        user_content,
        &turn.effective.model_id,
        llm_config,
        router,
        api_key,
    );
    request.config.vendor = turn.vendor.clone().or(router.vendor.clone());
    request.config.thinking_enabled = Some(turn.thinking_enabled);
    request.config.thinking_mode = turn.thinking_mode.clone();
    request
}

/// Convert session system prompt + history into Sidecar chat messages.
pub fn build_agent_messages(session: &Session, history: &[Message]) -> Vec<AgentChatMessage> {
    let mut messages = Vec::new();
    if let Some(system) = session.system_prompt.as_deref() {
        if !system.trim().is_empty() {
            messages.push(AgentChatMessage {
                role: "system".to_string(),
                content: system.to_string(),
                name: None,
                tool_call_id: None,
            });
        }
    }

    for msg in history {
        let role = msg.role.as_str();
        if !matches!(role, "user" | "assistant" | "system" | "tool") {
            continue;
        }
        messages.push(AgentChatMessage {
            role: role.to_string(),
            content: msg.content.clone(),
            name: None,
            tool_call_id: None,
        });
    }
    messages
}

pub fn build_title_prompt(first_message: &str) -> String {
    format!("Generate a concise title (5-10 words) for: {first_message}")
}

pub fn sanitize_session_title(raw: &str) -> String {
    let trimmed = raw
        .lines()
        .next()
        .unwrap_or(raw)
        .trim()
        .trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .trim();
    if trimmed.is_empty() {
        "New Chat".to_string()
    } else {
        trimmed.chars().take(80).collect()
    }
}

// ─── 内部辅助（薄层编排） ──────────────────────────────────────────────

/// 将已连接 MCP Server 的工具描述注入 session 的 system prompt（无工具则不改）
///
/// Only used by the Rig fallback path.
fn inject_mcp_prompt(state: &AppState, session: &mut Session) {
    let bridge = McpToolBridge::new(Arc::clone(&state.mcp_manager));
    if let Some(tool_desc) = bridge.tool_descriptions() {
        let base_prompt = session.system_prompt.clone().unwrap_or_default();
        session.system_prompt = Some(format!("{base_prompt}{tool_desc}"));
    }
}

/// 执行一轮 assistant 回复：有 MCP 工具时走多轮工具循环，否则单次流式
///
/// 返回 `(StreamResult, tool_calls_json)`；`tool_calls_json` 供 finalize 写入 DB。
#[allow(clippy::too_many_arguments)]
async fn run_assistant_turn(
    app: &AppHandle,
    state: &AppState,
    backend: &RigBackend,
    session: &Session,
    history: Vec<Message>,
    user_content: String,
    attachments: Option<Vec<MessageAttachment>>,
    model_id: &str,
    abort_flag: Arc<AtomicBool>,
    assistant_msg_id: &str,
) -> Result<(StreamResult, Option<String>), String> {
    let has_tools = McpToolBridge::new(Arc::clone(&state.mcp_manager)).has_tools();

    if has_tools {
        let tool_loop = McpToolLoop::new(
            app,
            &state.db,
            &state.mcp_manager,
            backend,
            session,
            model_id,
            abort_flag,
            assistant_msg_id,
            MAX_TOOL_ROUNDS,
        );
        let outcome = tool_loop.run(history, user_content, attachments).await?;
        let tool_calls_json = match outcome.tool_calls.is_empty() {
            true => None,
            false => Some(serde_json::to_string(&outcome.tool_calls).map_err(|e| e.to_string())?),
        };
        return Ok((outcome.result, tool_calls_json));
    }

    use crate::services::llm::ChatBackend;
    let result = backend
        .send_and_stream(
            app,
            session,
            &history,
            &user_content,
            &attachments,
            model_id,
            abort_flag,
            assistant_msg_id,
        )
        .await
        .map_err(|e| format!("Stream error: {e}"))?;
    Ok((result, None))
}

// ─── 持久化辅助 ─────────────────────────────────────────────────────────

pub(crate) fn save_user_message(
    state: &AppState,
    msg_id: &str,
    session_id: &str,
    content: &str,
    attachments: Option<&[MessageAttachment]>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let attachments_json = attachments.map(|a| serde_json::to_string(a).unwrap_or_default());

    MessageRepo::insert_user_message(
        &db,
        msg_id,
        session_id,
        content,
        attachments_json.as_deref(),
    )
    .map_err(|e| e.to_string())
}

pub(crate) fn load_session_context(
    state: &AppState,
    session_id: &str,
) -> Result<(Session, Vec<Message>), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let session = SessionRepo::find_by_id(&db, session_id).map_err(|e| e.to_string())?;
    let messages = MessageRepo::find_recent(&db, session_id, 50).map_err(|e| e.to_string())?;

    Ok((session, messages))
}

pub(crate) fn load_and_decrypt_config(
    state: &AppState,
    config_id: &str,
) -> Result<(RouterConfig, String), String> {
    let config = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        RouterConfigRepo::find_by_id(&db, config_id).map_err(|e| e.to_string())?
    };

    let encrypted = config
        .api_key_encrypted
        .as_deref()
        .ok_or("No API key configured for this router config")?;
    let decrypted_key = crypto::decrypt(encrypted).map_err(|e| e.to_string())?;

    Ok((config, decrypted_key))
}

pub(crate) fn ensure_model_enabled(state: &AppState, model_spec: &ModelSpec) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_model_enabled_for_config(&db, &model_spec.config_id, &model_spec.model_id)
}

/// 校验指定 router config 下该模型已启用（业务规则层；数据访问走 CustomModelRepo）。
pub fn ensure_model_enabled_for_config(
    conn: &rusqlite::Connection,
    config_id: &str,
    model_id: &str,
) -> Result<(), String> {
    if CustomModelRepo::is_enabled(conn, config_id, model_id).map_err(|e| e.to_string())? {
        Ok(())
    } else {
        Err(format!(
            "Model is not enabled for this provider: {model_id}"
        ))
    }
}

pub(crate) fn create_assistant_placeholder(
    state: &AppState,
    msg_id: &str,
    session_id: &str,
    model_spec: &ModelSpec,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    MessageRepo::insert_assistant_placeholder(&db, msg_id, session_id, &model_spec.model_id)
        .map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn finalize_assistant_turn(
    app: &AppHandle,
    state: &AppState,
    session_id: &str,
    assistant_message_id: &str,
    turn: &TurnModel,
    operation_kind: UsageOperationKind,
    input_segments: &[String],
    has_image_attachments: bool,
    call_started: bool,
    mut result: StreamResult,
    tool_calls_json: Option<String>,
) -> Result<FinalizeTurnOutcome, String> {
    let input_refs: Vec<&str> = input_segments.iter().map(String::as_str).collect();
    ensure_fallback_capture(
        &mut result.usage_captures,
        turn.vendor.as_deref(),
        Some(&turn.effective.model_id),
        &input_refs,
        &result.content,
        has_image_attachments,
        tool_calls_json.is_some(),
        call_started,
    );
    let request = FinalizeTurnRequest {
        operation_key: format!("assistant:{assistant_message_id}"),
        operation_kind,
        session_id: Some(session_id.to_string()),
        message_id: Some(assistant_message_id.to_string()),
        selected_model_id: Some(turn.selected.model_id.clone()),
        effective_provider_config_id: Some(turn.effective.config_id.clone()),
        effective_model_id: Some(turn.effective.model_id.clone()),
        vendor_id: turn.vendor.clone(),
        content: result.content,
        thinking: result.thinking,
        tool_calls_json,
        captures: result.usage_captures,
        was_aborted: result.was_aborted,
        stream_error: result.stream_error,
        session_title: None,
    };
    let mut db = state.db.lock().map_err(|error| error.to_string())?;
    let outcome = finalize_turn(&mut db, &request).map_err(|error| error.to_string())?;
    drop(db);
    emit_usage_recorded(app, &outcome.recorded);
    Ok(outcome)
}
