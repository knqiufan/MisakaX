//! Chat IPC commands — thin delegates over [`crate::services::chat`].
//!
//! This module only owns request/response DTOs and the `#[tauri::command]`
//! surface; orchestration (sidecar/rig routing, MCP tool loop, persistence)
//! lives in the service layer.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::db::models::Message;
use crate::db::repository::{MessageRepo, SessionRepo};
use crate::services::chat;
use crate::services::llm::backend::MessageAttachment;
use crate::services::llm::config::LlmConfig;
use crate::services::llm::RigBackend;
use crate::AppState;

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
    #[serde(default)]
    pub selected_skill_ids: Vec<String>,
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

    let (session, history) = chat::load_session_context(&state, &request.session_id)?;
    let selected =
        chat::resolve_model_spec(request.model_override.as_deref(), session.model.as_deref())?;
    let thinking_enabled = request
        .llm_config
        .as_ref()
        .map(|c| c.thinking_enabled)
        .unwrap_or(true);
    let use_sidecar = chat::read_use_sidecar(&state);
    let turn = chat::resolve_turn_model(&state, &selected, thinking_enabled, use_sidecar)?;
    let selected_skills = chat::resolve_selected_skill_ids(&state, &request.selected_skill_ids)?;
    let selected_skill_ids = selected_skills
        .iter()
        .map(|skill| skill.slug.clone())
        .collect::<Vec<_>>();

    chat::save_user_message(
        &state,
        &user_msg_id,
        &request.session_id,
        &request.content,
        request.attachments.as_deref(),
    )?;
    chat::save_message_skill_selection(&state, &user_msg_id, &selected_skill_ids)?;
    chat::create_assistant_placeholder(
        &state,
        &assistant_msg_id,
        &request.session_id,
        &turn.effective,
    )?;

    let abort_flag = state.stream_registry.register(&request.session_id);
    let turn_result = if use_sidecar {
        chat::send_via_sidecar(
            &app,
            &state,
            &session,
            &history,
            &request.content,
            &turn,
            request.llm_config.clone(),
            selected_skills,
            abort_flag,
            &assistant_msg_id,
        )
        .await
    } else {
        chat::send_via_rig(
            &app,
            &state,
            session,
            history,
            request.content.clone(),
            request.attachments.clone(),
            &turn,
            request.llm_config,
            abort_flag,
            &assistant_msg_id,
        )
        .await
    };
    state.stream_registry.unregister(&request.session_id);

    let (result, tool_calls_json) = turn_result?;
    chat::update_assistant_message(
        &state,
        &assistant_msg_id,
        &result,
        tool_calls_json.as_deref(),
    )?;
    chat::update_session_stats(&state, &request.session_id, &result)?;

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
    llm_config: Option<LlmConfig>,
) -> Result<SendMessageResult, String> {
    let regen_ctx = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        MessageRepo::find_regeneration_context(&db, &session_id, &message_id)
            .map_err(|e| e.to_string())?
    };

    let (session, _) = chat::load_session_context(&state, &session_id)?;
    let selected = chat::resolve_model_spec(None, session.model.as_deref())?;
    let thinking_enabled = llm_config
        .as_ref()
        .map(|c| c.thinking_enabled)
        .unwrap_or(true);
    let use_sidecar = chat::read_use_sidecar(&state);
    let turn = chat::resolve_turn_model(&state, &selected, thinking_enabled, use_sidecar)?;
    let selected_skill_ids = chat::load_message_skill_selection(&state, &regen_ctx.user_msg_id)?;
    let selected_skills = chat::resolve_selected_skill_ids(&state, &selected_skill_ids)?;

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        MessageRepo::delete_from(&db, &session_id, &message_id).map_err(|e| e.to_string())?;
    }

    let assistant_msg_id = uuid::Uuid::new_v4().to_string();
    chat::create_assistant_placeholder(&state, &assistant_msg_id, &session_id, &turn.effective)?;

    let abort_flag = state.stream_registry.register(&session_id);
    let attachments = chat::parse_attachments_json(regen_ctx.user_attachments.as_deref());
    let turn_result = if use_sidecar {
        chat::send_via_sidecar(
            &app,
            &state,
            &session,
            &regen_ctx.messages_before,
            &regen_ctx.user_content,
            &turn,
            llm_config.clone(),
            selected_skills,
            abort_flag,
            &assistant_msg_id,
        )
        .await
    } else {
        chat::send_via_rig(
            &app,
            &state,
            session,
            regen_ctx.messages_before,
            regen_ctx.user_content.clone(),
            attachments,
            &turn,
            llm_config,
            abort_flag,
            &assistant_msg_id,
        )
        .await
    };
    state.stream_registry.unregister(&session_id);

    let (result, tool_calls_json) = turn_result?;
    chat::update_assistant_message(
        &state,
        &assistant_msg_id,
        &result,
        tool_calls_json.as_deref(),
    )?;
    chat::update_session_stats(&state, &session_id, &result)?;

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
        chat::resolve_model_spec(request.model_override.as_deref(), session.model.as_deref())?;
    chat::ensure_model_enabled(&state, &model_spec)?;
    let (router_config, decrypted_key) =
        chat::load_and_decrypt_config(&state, &model_spec.config_id)?;

    let llm_config = LlmConfig {
        temperature: 0.3,
        max_tokens: Some(32),
        ..LlmConfig::default()
    }
    .sanitized();

    let backend = RigBackend::from_config(&router_config, &decrypted_key, llm_config)
        .map_err(|e| format!("Failed to create backend: {e}"))?;

    let prompt = chat::build_title_prompt(&request.first_message);
    let title = backend
        .prompt_once(&model_spec.model_id, &prompt)
        .await
        .map_err(|e| format!("Title generation failed: {e}"))?;
    let title = chat::sanitize_session_title(&title);

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
) -> Result<Vec<Message>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(50).min(200);

    let messages = if let Some(ref bid) = before_id {
        MessageRepo::find_before(&db, &session_id, bid, limit)
    } else {
        MessageRepo::find_recent(&db, &session_id, limit)
    };

    messages.map_err(|e| e.to_string())
}
