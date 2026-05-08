use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::crypto;
use crate::db::repository::{MessageRepo, RouterConfigRepo, SessionRepo};
use crate::services::llm::backend::ImageAttachment;
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
    pub images: Option<Vec<ImageAttachment>>,
    pub model_override: Option<String>,
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

    // Step 1: 保存用户消息
    save_user_message(&state, &user_msg_id, &request)?;

    // Step 2: 加载会话 + 历史消息
    let (session, history) = load_session_context(&state, &request.session_id)?;

    // Step 3: 解析模型标识
    let model_spec = resolve_model_spec(
        request.model_override.as_deref(),
        session.model.as_deref(),
    )?;

    // Step 4: 读取 RouterConfig，解密 API Key
    let (router_config, decrypted_key) =
        load_and_decrypt_config(&state, &model_spec.config_id)?;

    // Step 5: 创建 assistant 消息占位
    create_assistant_placeholder(
        &state,
        &assistant_msg_id,
        &request.session_id,
        &model_spec,
    )?;

    // Step 6: 注册流，构建 Backend，执行流式调用
    let abort_flag = state.stream_registry.register(&request.session_id);

    let backend = RigBackend::from_config(
        &router_config,
        &decrypted_key,
        LlmConfig::default().sanitized(),
    )
    .map_err(|e| format!("Failed to create backend: {e}"))?;

    let stream_result = {
        use crate::services::llm::ChatBackend;
        backend
            .send_and_stream(
                &app,
                &session,
                &history,
                &request.content,
                &request.images,
                &model_spec.model_id,
                abort_flag,
                &assistant_msg_id,
            )
            .await
            .map_err(|e| format!("Stream error: {e}"))
    };

    state.stream_registry.unregister(&request.session_id);

    // Step 7: 更新 assistant 消息内容
    let result = stream_result?;
    update_assistant_message(&state, &assistant_msg_id, &result)?;

    // Step 8: 更新 session 统计
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
    // Step 1: 加载 regeneration 上下文
    let (user_content, user_msg_id, messages_before) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        MessageRepo::find_regeneration_context(&db, &session_id, &message_id)
            .map_err(|e| e.to_string())?
    };

    // Step 2: 删除目标消息及之后的所有消息
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        MessageRepo::delete_from(&db, &session_id, &message_id)
            .map_err(|e| e.to_string())?;
    }

    // Step 3: 复用核心流程
    let assistant_msg_id = uuid::Uuid::new_v4().to_string();
    let (session, _) = load_session_context(&state, &session_id)?;
    let model_spec = resolve_model_spec(None, session.model.as_deref())?;
    let (router_config, decrypted_key) =
        load_and_decrypt_config(&state, &model_spec.config_id)?;

    create_assistant_placeholder(
        &state,
        &assistant_msg_id,
        &session_id,
        &model_spec,
    )?;

    let abort_flag = state.stream_registry.register(&session_id);

    let backend = RigBackend::from_config(
        &router_config,
        &decrypted_key,
        LlmConfig::default().sanitized(),
    )
    .map_err(|e| format!("Failed to create backend: {e}"))?;

    let stream_result = {
        use crate::services::llm::ChatBackend;
        backend
            .send_and_stream(
                &app,
                &session,
                &messages_before,
                &user_content,
                &None,
                &model_spec.model_id,
                abort_flag,
                &assistant_msg_id,
            )
            .await
            .map_err(|e| format!("Stream error: {e}"))
    };

    state.stream_registry.unregister(&session_id);

    let result = stream_result?;
    update_assistant_message(&state, &assistant_msg_id, &result)?;
    update_session_stats(&state, &session_id, &result)?;

    Ok(SendMessageResult {
        user_message_id: user_msg_id,
        assistant_message_id: assistant_msg_id,
    })
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

fn save_user_message(
    state: &AppState,
    msg_id: &str,
    request: &SendMessageRequest,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let attachments_json = request
        .images
        .as_ref()
        .map(|imgs| serde_json::to_string(imgs).unwrap_or_default());

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

    let session = SessionRepo::find_by_id(&db, session_id)
        .map_err(|e| e.to_string())?;
    let messages = MessageRepo::find_recent(&db, session_id, 50)
        .map_err(|e| e.to_string())?;

    Ok((session, messages))
}

fn load_and_decrypt_config(
    state: &AppState,
    config_id: &str,
) -> Result<(crate::db::models::RouterConfig, String), String> {
    let config = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        RouterConfigRepo::find_by_id(&db, config_id)
            .map_err(|e| e.to_string())?
    };

    let encrypted = config
        .api_key_encrypted
        .as_deref()
        .ok_or("No API key configured for this router config")?;
    let decrypted_key = crypto::decrypt(encrypted).map_err(|e| e.to_string())?;

    Ok((config, decrypted_key))
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
