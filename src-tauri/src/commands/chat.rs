use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::crypto;
use crate::db::repository::{MessageRepo, RouterConfigRepo, SessionRepo};
use crate::services::llm::backend::MessageAttachment;
use crate::services::llm::config::LlmConfig;
use crate::services::llm::{RigBackend, StreamResult};
use crate::services::mcp::{McpToolLoop, MAX_TOOL_ROUNDS};
use crate::services::mcp_bridge::McpToolBridge;
use crate::services::sidecar_client::{AgentChatConfig, AgentChatMessage, AgentChatRequest};
use crate::services::sidecar_sse::consume_sidecar_stream;
use crate::AppState;

/// 将附件 JSON 字符串反序列化为 MessageAttachment 列表
fn parse_attachments_json(json: Option<&str>) -> Option<Vec<MessageAttachment>> {
    json.and_then(|s| serde_json::from_str::<Vec<MessageAttachment>>(s).ok())
        .filter(|attachments| !attachments.is_empty())
}

// ─── 请求/响应类型 ─────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SendMessageResult {
    pub user_message_id: String,
    pub assistant_message_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub session_id: String,
    pub content: String,
    #[serde(default, alias = "images")]
    pub attachments: Option<Vec<MessageAttachment>>,
    pub model_override: Option<String>,
    pub llm_config: Option<LlmConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateSessionTitleRequest {
    pub session_id: String,
    pub first_message: String,
    pub model_override: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GenerateSessionTitleResult {
    pub title: String,
}

// ─── send_message Command ──────────────────────────────────────────────

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    state: State<'_, AppState>,
    request: SendMessageRequest,
) -> Result<SendMessageResult, String> {
    let user_msg_id = uuid::Uuid::new_v4().to_string();
    let assistant_msg_id = uuid::Uuid::new_v4().to_string();

    save_user_message(&state, &user_msg_id, &request)?;
    let (session, history) = load_session_context(&state, &request.session_id)?;
    let model_spec =
        resolve_model_spec(request.model_override.as_deref(), session.model.as_deref())?;
    ensure_model_enabled(&state, &model_spec)?;
    create_assistant_placeholder(&state, &assistant_msg_id, &request.session_id, &model_spec)?;

    let abort_flag = state.stream_registry.register(&request.session_id);
    let use_sidecar = read_use_sidecar(&state);
    let turn = if use_sidecar {
        send_via_sidecar(
            &app,
            &state,
            &session,
            &history,
            &request.content,
            &model_spec,
            request.llm_config.clone(),
            abort_flag,
            &assistant_msg_id,
        )
        .await
    } else {
        send_via_rig(
            &app,
            &state,
            session,
            history,
            request.content.clone(),
            request.attachments.clone(),
            &model_spec,
            request.llm_config,
            abort_flag,
            &assistant_msg_id,
        )
        .await
    };
    state.stream_registry.unregister(&request.session_id);

    let (result, tool_calls_json) = turn?;
    update_assistant_message(
        &state,
        &assistant_msg_id,
        &result,
        tool_calls_json.as_deref(),
    )?;
    update_session_stats(&state, &request.session_id, &result)?;

    Ok(SendMessageResult {
        user_message_id: user_msg_id,
        assistant_message_id: assistant_msg_id,
    })
}

// ─── stop_generation Command ───────────────────────────────────────────

#[tauri::command]
pub async fn stop_generation(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<bool, String> {
    let aborted = state.stream_registry.abort(&session_id);
    if !aborted {
        tracing::info!(session_id = %session_id, "No active stream to abort");
    }
    Ok(aborted)
}

// ─── regenerate_message Command ────────────────────────────────────────

#[tauri::command]
pub async fn regenerate_message(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    message_id: String,
) -> Result<SendMessageResult, String> {
    let regen_ctx = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        MessageRepo::find_regeneration_context(&db, &session_id, &message_id)
            .map_err(|e| e.to_string())?
    };

    let (session, _) = load_session_context(&state, &session_id)?;
    let model_spec = resolve_model_spec(None, session.model.as_deref())?;
    ensure_model_enabled(&state, &model_spec)?;

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        MessageRepo::delete_from(&db, &session_id, &message_id).map_err(|e| e.to_string())?;
    }

    let assistant_msg_id = uuid::Uuid::new_v4().to_string();
    create_assistant_placeholder(&state, &assistant_msg_id, &session_id, &model_spec)?;

    let abort_flag = state.stream_registry.register(&session_id);
    let attachments = parse_attachments_json(regen_ctx.user_attachments.as_deref());
    let use_sidecar = read_use_sidecar(&state);
    let turn = if use_sidecar {
        send_via_sidecar(
            &app,
            &state,
            &session,
            &regen_ctx.messages_before,
            &regen_ctx.user_content,
            &model_spec,
            None,
            abort_flag,
            &assistant_msg_id,
        )
        .await
    } else {
        send_via_rig(
            &app,
            &state,
            session,
            regen_ctx.messages_before,
            regen_ctx.user_content.clone(),
            attachments,
            &model_spec,
            None,
            abort_flag,
            &assistant_msg_id,
        )
        .await
    };
    state.stream_registry.unregister(&session_id);

    let (result, tool_calls_json) = turn?;
    update_assistant_message(
        &state,
        &assistant_msg_id,
        &result,
        tool_calls_json.as_deref(),
    )?;
    update_session_stats(&state, &session_id, &result)?;

    Ok(SendMessageResult {
        user_message_id: regen_ctx.user_msg_id,
        assistant_message_id: assistant_msg_id,
    })
}

// ─── generate_session_title Command ────────────────────────────────────

/// Lightweight Rig-only title generation (does not use Sidecar).
#[tauri::command]
pub async fn generate_session_title(
    state: State<'_, AppState>,
    request: GenerateSessionTitleRequest,
) -> Result<GenerateSessionTitleResult, String> {
    let session = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        SessionRepo::find_by_id(&db, &request.session_id).map_err(|e| e.to_string())?
    };
    let model_spec =
        resolve_model_spec(request.model_override.as_deref(), session.model.as_deref())?;
    ensure_model_enabled(&state, &model_spec)?;
    let (router_config, decrypted_key) = load_and_decrypt_config(&state, &model_spec.config_id)?;

    let llm_config = LlmConfig {
        temperature: 0.3,
        max_tokens: Some(32),
        ..LlmConfig::default()
    }
    .sanitized();

    let backend = RigBackend::from_config(&router_config, &decrypted_key, llm_config)
        .map_err(|e| format!("Failed to create backend: {e}"))?;

    let prompt = build_title_prompt(&request.first_message);
    let title = backend
        .prompt_once(&model_spec.model_id, &prompt)
        .await
        .map_err(|e| format!("Title generation failed: {e}"))?;
    let title = sanitize_session_title(&title);

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        SessionRepo::update(&db, &request.session_id, Some(&title), None, None, None)
            .map_err(|e| e.to_string())?;
    }

    Ok(GenerateSessionTitleResult { title })
}

// ─── get_messages Command ──────────────────────────────────────────────

#[tauri::command]
pub fn get_messages(
    state: State<'_, AppState>,
    session_id: String,
    limit: Option<u32>,
    before_id: Option<String>,
) -> Result<Vec<crate::db::models::Message>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(50).min(200);

    let messages = if let Some(ref bid) = before_id {
        MessageRepo::find_before(&db, &session_id, bid, limit)
    } else {
        MessageRepo::find_recent(&db, &session_id, limit)
    };

    messages.map_err(|e| e.to_string())
}

// ─── Path helpers ──────────────────────────────────────────────────────

fn read_use_sidecar(state: &AppState) -> bool {
    state.config.lock().map(|c| c.use_sidecar).unwrap_or(true)
}

/// Sidecar chat path — Agent owns MCP via mcp_bridge; do NOT inject MCP prompt.
#[allow(clippy::too_many_arguments)]
async fn send_via_sidecar(
    app: &AppHandle,
    state: &AppState,
    session: &crate::db::models::Session,
    history: &[crate::db::models::Message],
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
async fn send_via_rig(
    app: &AppHandle,
    state: &AppState,
    mut session: crate::db::models::Session,
    history: Vec<crate::db::models::Message>,
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

/// Build AgentChatRequest for Sidecar (text-only; attachments stay on Rig path).
pub fn build_agent_chat_request(
    session: &crate::db::models::Session,
    history: &[crate::db::models::Message],
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
#[cfg_attr(feature = "test-private", allow(dead_code))]
pub fn build_agent_messages(
    session: &crate::db::models::Session,
    history: &[crate::db::models::Message],
) -> Vec<AgentChatMessage> {
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

// ─── 内部辅助函数（薄层编排） ──────────────────────────────────────────

/// 模型标识解析结果
#[derive(Debug)]
#[cfg_attr(feature = "test-private", allow(dead_code))]
pub struct ModelSpec {
    pub config_id: String,
    pub model_id: String,
}

/// 解析模型标识 — 格式为 "config_id:model_id"
#[cfg_attr(feature = "test-private", allow(dead_code))]
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

/// 将已连接 MCP Server 的工具描述注入 session 的 system prompt（无工具则不改）
///
/// Only used by the Rig fallback path.
fn inject_mcp_prompt(state: &AppState, session: &mut crate::db::models::Session) {
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
    session: &crate::db::models::Session,
    history: Vec<crate::db::models::Message>,
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

fn save_user_message(
    state: &AppState,
    msg_id: &str,
    request: &SendMessageRequest,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let attachments_json = request
        .attachments
        .as_ref()
        .map(|attachments| serde_json::to_string(attachments).unwrap_or_default());

    MessageRepo::insert_user_message(
        &db,
        msg_id,
        &request.session_id,
        &request.content,
        attachments_json.as_deref(),
    )
    .map_err(|e| e.to_string())
}

fn load_session_context(
    state: &AppState,
    session_id: &str,
) -> Result<(crate::db::models::Session, Vec<crate::db::models::Message>), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let session = SessionRepo::find_by_id(&db, session_id).map_err(|e| e.to_string())?;
    let messages = MessageRepo::find_recent(&db, session_id, 50).map_err(|e| e.to_string())?;

    Ok((session, messages))
}

fn load_and_decrypt_config(
    state: &AppState,
    config_id: &str,
) -> Result<(crate::db::models::RouterConfig, String), String> {
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

fn ensure_model_enabled(state: &AppState, model_spec: &ModelSpec) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ensure_model_enabled_for_config(&db, &model_spec.config_id, &model_spec.model_id)
}

#[cfg_attr(feature = "test-private", allow(dead_code))]
pub fn ensure_model_enabled_for_config(
    conn: &rusqlite::Connection,
    config_id: &str,
    model_id: &str,
) -> Result<(), String> {
    let exists = conn
        .query_row(
            "SELECT COUNT(*)
             FROM custom_models
             WHERE router_config_id = ?1 AND model_id = ?2 AND enabled = 1",
            rusqlite::params![config_id, model_id],
            |row| row.get::<_, i32>(0).map(|count| count > 0),
        )
        .map_err(|e| e.to_string())?;

    if exists {
        Ok(())
    } else {
        Err(format!(
            "Model is not enabled for this provider: {model_id}"
        ))
    }
}

fn create_assistant_placeholder(
    state: &AppState,
    msg_id: &str,
    session_id: &str,
    model_spec: &ModelSpec,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    MessageRepo::insert_assistant_placeholder(&db, msg_id, session_id, &model_spec.model_id)
        .map_err(|e| e.to_string())
}

fn update_assistant_message(
    state: &AppState,
    msg_id: &str,
    result: &crate::services::llm::StreamResult,
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

fn update_session_stats(
    state: &AppState,
    session_id: &str,
    result: &crate::services::llm::StreamResult,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let (input_tokens, output_tokens) = match &result.usage {
        Some(usage) => (Some(usage.input_tokens), Some(usage.output_tokens)),
        None => (None, None),
    };

    SessionRepo::update_stats(&db, session_id, input_tokens, output_tokens)
        .map_err(|e| e.to_string())
}
