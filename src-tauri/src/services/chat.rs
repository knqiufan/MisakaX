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
use crate::AppState;

/// 将附件 JSON 字符串反序列化为 MessageAttachment 列表
pub(crate) fn parse_attachments_json(json: Option<&str>) -> Option<Vec<MessageAttachment>> {
    json.and_then(|s| serde_json::from_str::<Vec<MessageAttachment>>(s).ok())
        .filter(|attachments| !attachments.is_empty())
}

// ─── 模型标识解析 ───────────────────────────────────────────────────────

/// 模型标识解析结果
#[derive(Debug)]
pub struct ModelSpec {
    pub config_id: String,
    pub model_id: String,
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
    model_spec: &ModelSpec,
    llm_config: Option<LlmConfig>,
    abort_flag: Arc<AtomicBool>,
    assistant_msg_id: &str,
) -> Result<(StreamResult, Option<String>), String> {
    let request = build_agent_chat_request(
        session,
        history,
        user_content,
        &model_spec.model_id,
        llm_config,
    );
    let response = state.sidecar_client.stream(&request).await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Sidecar stream HTTP {status}: {body}"));
    }

    let result =
        consume_sidecar_stream(app, response, &session.id, assistant_msg_id, abort_flag).await?;
    Ok((result, None))
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
    model_spec: &ModelSpec,
    llm_config: Option<LlmConfig>,
    abort_flag: Arc<AtomicBool>,
    assistant_msg_id: &str,
) -> Result<(StreamResult, Option<String>), String> {
    // Fallback path only: Sidecar agents call MCP through mcp_bridge_tool.
    inject_mcp_prompt(state, &mut session);

    let (router_config, decrypted_key) = load_and_decrypt_config(state, &model_spec.config_id)?;
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
        &model_spec.model_id,
        abort_flag,
        assistant_msg_id,
    )
    .await
}

// ─── 请求构造 ───────────────────────────────────────────────────────────

/// Build AgentChatRequest for Sidecar (text-only; attachments stay on Rig path).
pub fn build_agent_chat_request(
    session: &Session,
    history: &[Message],
    user_content: &str,
    model_id: &str,
    llm_config: Option<LlmConfig>,
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
        },
        session_id: Some(session.id.clone()),
        working_dir: session.working_directory.clone(),
    }
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

pub(crate) fn update_assistant_message(
    state: &AppState,
    msg_id: &str,
    result: &StreamResult,
    tool_calls_json: Option<&str>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let usage_json = result
        .usage
        .as_ref()
        .map(|u| serde_json::to_string(u).unwrap_or_default());

    let thinking = if result.thinking.is_empty() {
        None
    } else {
        Some(result.thinking.as_str())
    };

    MessageRepo::update_assistant_content(
        &db,
        msg_id,
        &result.content,
        thinking,
        usage_json.as_deref(),
        result.was_aborted,
        tool_calls_json,
    )
    .map_err(|e| e.to_string())
}

pub(crate) fn update_session_stats(
    state: &AppState,
    session_id: &str,
    result: &StreamResult,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let (input_tokens, output_tokens) = match &result.usage {
        Some(usage) => (Some(usage.input_tokens), Some(usage.output_tokens)),
        None => (None, None),
    };

    SessionRepo::update_stats(&db, session_id, input_tokens, output_tokens)
        .map_err(|e| e.to_string())
}
