//! Chat IPC commands — thin delegates over [`crate::services::chat`].
//!
//! This module only owns request/response DTOs and the `#[tauri::command]`
//! surface; orchestration (sidecar/rig routing, MCP tool loop, persistence)
//! lives in the service layer.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::db::models::Message;
use crate::db::repository::{MessageBlockRepo, MessageRepo, SessionRepo};
use crate::services::chat;
use crate::services::llm::backend::MessageAttachment;
use crate::services::llm::config::LlmConfig;
use crate::services::llm::{RigBackend, StreamResult};
use crate::services::usage::collector::{ensure_fallback_capture, provider_capture};
use crate::services::usage::finalize::{emit_usage_recorded, finalize_turn, FinalizeTurnRequest};
use crate::services::usage::UsageOperationKind;
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
    let operation_kind = if request
        .llm_config
        .as_ref()
        .is_some_and(|config| config.agent_mode == "research")
    {
        UsageOperationKind::Research
    } else {
        UsageOperationKind::Chat
    };
    let turn = chat::resolve_turn_model(&state, &selected, thinking_enabled, use_sidecar)?;
    let (skill_activation, selected_skills) =
        chat::resolve_selected_skill_ids(&state, &request.selected_skill_ids)?;

    chat::save_user_message(
        &state,
        &user_msg_id,
        &request.session_id,
        &request.content,
        request.attachments.as_deref(),
    )?;
    chat::save_message_skill_selection(&state, &user_msg_id, &selected_skills)?;
    chat::create_assistant_placeholder(
        &state,
        &assistant_msg_id,
        &request.session_id,
        &turn.effective,
    )?;

    let mut estimator_inputs: Vec<String> = session
        .system_prompt
        .iter()
        .cloned()
        .chain(history.iter().map(|message| message.content.clone()))
        .collect();
    estimator_inputs.push(request.content.clone());
    let has_image_attachments = request.attachments.as_ref().is_some_and(|attachments| {
        attachments
            .iter()
            .any(|attachment| matches!(attachment, MessageAttachment::Image { .. }))
    });
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
            skill_activation,
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

    let (result, tool_calls_json, call_started) = match turn_result {
        Ok((result, tool_calls_json)) => (result, tool_calls_json, true),
        Err(error) => (failed_stream_result(&error), None, false),
    };
    let stream_error = result.stream_error.clone();
    chat::finalize_assistant_turn(
        &app,
        &state,
        &request.session_id,
        &assistant_msg_id,
        &turn,
        operation_kind,
        &estimator_inputs,
        has_image_attachments,
        call_started,
        result,
        tool_calls_json,
    )?;
    if let Some(error) = stream_error {
        return Err(error);
    }

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
    let skill_snapshots = chat::load_message_skill_selection(&state, &regen_ctx.user_msg_id)?;
    let (skill_activation, selected_skills) = {
        let db = state.db.lock().map_err(|error| error.to_string())?;
        crate::services::skills::registry::resolve_snapshot_selection(&db, &skill_snapshots)
            .map_err(|error| error.to_string())?
    };

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        MessageRepo::delete_from(&db, &session_id, &message_id).map_err(|e| e.to_string())?;
    }

    let assistant_msg_id = uuid::Uuid::new_v4().to_string();
    chat::create_assistant_placeholder(&state, &assistant_msg_id, &session_id, &turn.effective)?;

    let mut estimator_inputs: Vec<String> = session
        .system_prompt
        .iter()
        .cloned()
        .chain(
            regen_ctx
                .messages_before
                .iter()
                .map(|message| message.content.clone()),
        )
        .collect();
    estimator_inputs.push(regen_ctx.user_content.clone());
    let attachments = chat::parse_attachments_json(regen_ctx.user_attachments.as_deref());
    let has_image_attachments = attachments.as_ref().is_some_and(|items| {
        items
            .iter()
            .any(|attachment| matches!(attachment, MessageAttachment::Image { .. }))
    });
    let operation_kind = if llm_config
        .as_ref()
        .is_some_and(|config| config.agent_mode == "research")
    {
        UsageOperationKind::Research
    } else {
        UsageOperationKind::Chat
    };
    let abort_flag = state.stream_registry.register(&session_id);
    let turn_result = if use_sidecar {
        chat::send_via_sidecar(
            &app,
            &state,
            &session,
            &regen_ctx.messages_before,
            &regen_ctx.user_content,
            &turn,
            llm_config.clone(),
            skill_activation,
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

    let (result, tool_calls_json, call_started) = match turn_result {
        Ok((result, tool_calls_json)) => (result, tool_calls_json, true),
        Err(error) => (failed_stream_result(&error), None, false),
    };
    let stream_error = result.stream_error.clone();
    chat::finalize_assistant_turn(
        &app,
        &state,
        &session_id,
        &assistant_msg_id,
        &turn,
        operation_kind,
        &estimator_inputs,
        has_image_attachments,
        call_started,
        result,
        tool_calls_json,
    )?;
    if let Some(error) = stream_error {
        return Err(error);
    }

    Ok(SendMessageResult {
        user_message_id: regen_ctx.user_msg_id,
        assistant_message_id: assistant_msg_id,
    })
}

// ─── generate_session_title Command ────────────────────────────────────

/// Lightweight Rig-only title generation (does not use Sidecar).
#[tauri::command]
pub async fn generate_session_title(
    app: AppHandle,
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
    let prompt_outcome = backend
        .prompt_once(&model_spec.model_id, &prompt)
        .await
        .map_err(|e| format!("Title generation failed: {e}"))?;
    let title = chat::sanitize_session_title(&prompt_outcome.output);
    let mut captures = prompt_outcome
        .usage
        .map(|usage| {
            provider_capture(
                "rig:title",
                Some(model_spec.model_id.clone()),
                usage.input_tokens,
                usage.output_tokens,
                usage.total_tokens,
                usage.cache_read_tokens,
                usage.cache_creation_tokens,
                usage.reasoning_tokens,
            )
        })
        .into_iter()
        .collect::<Vec<_>>();
    ensure_fallback_capture(
        &mut captures,
        Some(&router_config.provider),
        Some(&model_spec.model_id),
        &[prompt.as_str()],
        &prompt_outcome.output,
        false,
        false,
        true,
    );
    let finalize_request = FinalizeTurnRequest {
        operation_key: format!("session_title:{}", uuid::Uuid::new_v4()),
        operation_kind: UsageOperationKind::SessionTitle,
        session_id: Some(request.session_id.clone()),
        message_id: None,
        selected_model_id: Some(model_spec.model_id.clone()),
        effective_provider_config_id: Some(model_spec.config_id.clone()),
        effective_model_id: Some(model_spec.model_id.clone()),
        vendor_id: router_config.vendor.clone(),
        content: String::new(),
        thinking: String::new(),
        tool_calls_json: None,
        captures,
        was_aborted: false,
        stream_error: None,
        session_title: Some(title.clone()),
    };
    let mut db = state.db.lock().map_err(|error| error.to_string())?;
    let finalized = finalize_turn(&mut db, &finalize_request).map_err(|error| error.to_string())?;
    drop(db);
    emit_usage_recorded(&app, &finalized.recorded);

    Ok(GenerateSessionTitleResult { title })
}

fn failed_stream_result(error: &str) -> StreamResult {
    StreamResult {
        content: String::new(),
        thinking: String::new(),
        usage: None,
        usage_captures: Vec::new(),
        was_aborted: false,
        stream_error: Some(error.to_string()),
    }
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

    let mut messages = if let Some(ref bid) = before_id {
        MessageRepo::find_before(&db, &session_id, bid, limit)
    } else {
        MessageRepo::find_recent(&db, &session_id, limit)
    }
    .map_err(|error| error.to_string())?;

    if state.feature_flags.rich_content_render {
        let ids = messages
            .iter()
            .map(|message| message.id.clone())
            .collect::<Vec<_>>();
        let blocks =
            MessageBlockRepo::find_by_messages(&db, &ids).map_err(|error| error.to_string())?;
        for message in &mut messages {
            message.blocks = blocks
                .iter()
                .filter(|block| block.message_id == message.id)
                .cloned()
                .collect();
        }
    }

    Ok(messages)
}
