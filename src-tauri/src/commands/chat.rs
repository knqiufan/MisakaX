use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::crypto;
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

    // Step 1: {lock db} 保存用户消息 {unlock db}
    save_user_message(&state, &user_msg_id, &request)?;

    // Step 2: {lock db} 加载会话 + 历史消息 {unlock db}
    let (session, history) = load_session_context(&state, &request.session_id)?;

    // Step 3: 解析模型标识 — model_override > session.model > default
    let model_spec = resolve_model_spec(
        request.model_override.as_deref(),
        session.model.as_deref(),
    )?;

    // Step 4: {lock db} 读取 RouterConfig {unlock db}, 解密 API Key
    let (router_config, decrypted_key) =
        load_and_decrypt_config(&state, &model_spec.config_id)?;

    // Step 5: 创建 assistant 消息占位（status=streaming）
    create_assistant_placeholder(&state, &assistant_msg_id, &request.session_id, &model_spec)?;

    // Step 6: 注册流，构建 Backend，执行流式调用（全程无锁）
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

    // Step 7: {lock db} 更新 assistant 消息内容 + usage {unlock db}
    let result = stream_result?;
    update_assistant_message(&state, &assistant_msg_id, &result)?;

    // Step 8: {lock db} 更新 session 的 last_message_at 和 token 计数 {unlock db}
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
    // Step 1: {lock db} 读取要重新生成的消息，获取其前一条用户消息 {unlock db}
    let (user_content, user_msg_id, messages_before) =
        load_regeneration_context(&state, &session_id, &message_id)?;

    // Step 2: {lock db} 删除原 assistant 消息及其之后的所有消息 {unlock db}
    delete_messages_from(&state, &session_id, &message_id)?;

    // Step 3: 构造新请求，复用 send_message 逻辑的核心流程
    let assistant_msg_id = uuid::Uuid::new_v4().to_string();
    let (session, _) = load_session_context(&state, &session_id)?;
    let model_spec = resolve_model_spec(None, session.model.as_deref())?;
    let (router_config, decrypted_key) =
        load_and_decrypt_config(&state, &model_spec.config_id)?;

    create_assistant_placeholder(&state, &assistant_msg_id, &session_id, &model_spec)?;

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

    if let Some(ref bid) = before_id {
        let created_at: String = db
            .query_row(
                "SELECT created_at FROM messages WHERE id = ?1",
                [bid],
                |row| row.get(0),
            )
            .map_err(|e| format!("Message not found: {e}"))?;

        let mut stmt = db
            .prepare(
                "SELECT id, session_id, role, content, token_usage, model,
                        thinking_content, attachments, status, created_at
                 FROM messages
                 WHERE session_id = ?1 AND created_at < ?2
                 ORDER BY created_at DESC
                 LIMIT ?3",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(
                rusqlite::params![session_id, created_at, limit],
                map_message_row,
            )
            .map_err(|e| e.to_string())?;

        collect_messages_reversed(rows)
    } else {
        let mut stmt = db
            .prepare(
                "SELECT id, session_id, role, content, token_usage, model,
                        thinking_content, attachments, status, created_at
                 FROM messages
                 WHERE session_id = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(
                rusqlite::params![session_id, limit],
                map_message_row,
            )
            .map_err(|e| e.to_string())?;

        collect_messages_reversed(rows)
    }
}

// ─── 内部辅助函数 ──────────────────────────────────────────────────────

/// 模型标识解析结果
#[derive(Debug)]
#[cfg_attr(feature = "test-private", allow(dead_code))]
pub struct ModelSpec {
    pub config_id: String,
    pub model_id: String,
}

/// 解析模型标识 — 格式为 "config_id:model_id"
///
/// 优先级：model_override > session.model > 返回错误
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

/// Step 1: 保存用户消息到 SQLite
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

    db.execute(
        "INSERT INTO messages (id, session_id, role, content, attachments, status)
         VALUES (?1, ?2, 'user', ?3, ?4, 'complete')",
        rusqlite::params![msg_id, request.session_id, request.content, attachments_json],
    )
    .map_err(|e| format!("Failed to save user message: {e}"))?;

    sync_message_fts(&db, msg_id, &request.content, &request.session_id, "user");

    Ok(())
}

/// Step 2: 加载会话信息和历史消息
fn load_session_context(
    state: &AppState,
    session_id: &str,
) -> Result<(crate::db::models::Session, Vec<crate::db::models::Message>), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let session = db
        .query_row(
            "SELECT id, title, model, system_prompt, working_directory, project_name,
                    status, mode, created_at, updated_at
             FROM sessions WHERE id = ?1",
            [session_id],
            |row| {
                Ok(crate::db::models::Session {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    model: row.get(2)?,
                    system_prompt: row.get(3)?,
                    working_directory: row.get(4)?,
                    project_name: row.get(5)?,
                    status: row.get(6)?,
                    mode: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        )
        .map_err(|e| format!("Session not found: {e}"))?;

    let messages = load_recent_messages(&db, session_id, 50)?;

    Ok((session, messages))
}

/// 加载最近 N 条消息（按时间正序）
fn load_recent_messages(
    db: &rusqlite::Connection,
    session_id: &str,
    limit: u32,
) -> Result<Vec<crate::db::models::Message>, String> {
    let mut stmt = db
        .prepare(
            "SELECT id, session_id, role, content, token_usage, model,
                    thinking_content, attachments, status, created_at
             FROM messages
             WHERE session_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![session_id, limit], map_message_row)
        .map_err(|e| e.to_string())?;

    collect_messages_reversed(rows)
}

/// Step 4: 读取 RouterConfig 并解密 API Key
fn load_and_decrypt_config(
    state: &AppState,
    config_id: &str,
) -> Result<(crate::db::models::RouterConfig, String), String> {
    let config = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT id, name, provider, api_key_encrypted, model, base_url,
                    config_json, is_active, created_at, api_compat
             FROM router_configs WHERE id = ?1",
            [config_id],
            |row| {
                Ok(crate::db::models::RouterConfig {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    provider: row.get(2)?,
                    api_key_encrypted: row.get(3)?,
                    model: row.get(4)?,
                    base_url: row.get(5)?,
                    config_json: row.get(6)?,
                    is_active: row.get::<_, i32>(7)? != 0,
                    created_at: row.get(8)?,
                    api_compat: row.get(9)?,
                })
            },
        )
        .map_err(|e| format!("Router config not found: {e}"))?
    };

    let encrypted = config
        .api_key_encrypted
        .as_deref()
        .ok_or("No API key configured for this router config")?;
    let decrypted_key = crypto::decrypt(encrypted).map_err(|e| e.to_string())?;

    Ok((config, decrypted_key))
}

/// Step 5: 创建 assistant 消息占位
fn create_assistant_placeholder(
    state: &AppState,
    msg_id: &str,
    session_id: &str,
    model_spec: &ModelSpec,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO messages (id, session_id, role, content, model, status)
         VALUES (?1, ?2, 'assistant', '', ?3, 'streaming')",
        rusqlite::params![msg_id, session_id, model_spec.model_id],
    )
    .map_err(|e| format!("Failed to create assistant placeholder: {e}"))?;
    Ok(())
}

/// Step 7: 更新 assistant 消息内容和用量
fn update_assistant_message(
    state: &AppState,
    msg_id: &str,
    result: &crate::services::llm::StreamResult,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let status = if result.was_aborted {
        "aborted"
    } else {
        "complete"
    };

    let usage_json = result
        .usage
        .as_ref()
        .map(|u| serde_json::to_string(u).unwrap_or_default());

    let thinking = if result.thinking.is_empty() {
        None
    } else {
        Some(&result.thinking)
    };

    db.execute(
        "UPDATE messages SET content = ?1, status = ?2, token_usage = ?3,
                thinking_content = ?4
         WHERE id = ?5",
        rusqlite::params![result.content, status, usage_json, thinking, msg_id],
    )
    .map_err(|e| format!("Failed to update assistant message: {e}"))?;

    let session_id: String = db
        .query_row(
            "SELECT session_id FROM messages WHERE id = ?1",
            [msg_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    sync_message_fts(&db, msg_id, &result.content, &session_id, "assistant");

    Ok(())
}

/// Step 8: 更新 session 统计
fn update_session_stats(
    state: &AppState,
    session_id: &str,
    result: &crate::services::llm::StreamResult,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    if let Some(ref usage) = result.usage {
        db.execute(
            "UPDATE sessions
             SET total_input_tokens = total_input_tokens + ?1,
                 total_output_tokens = total_output_tokens + ?2,
                 last_message_at = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = ?3",
            rusqlite::params![usage.input_tokens, usage.output_tokens, session_id],
        )
        .map_err(|e| format!("Failed to update session stats: {e}"))?;
    } else {
        db.execute(
            "UPDATE sessions
             SET last_message_at = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
            [session_id],
        )
        .map_err(|e| format!("Failed to update session timestamp: {e}"))?;
    }

    Ok(())
}

/// 加载重新生成上下文：找到目标消息前的用户消息及更早的历史
fn load_regeneration_context(
    state: &AppState,
    session_id: &str,
    target_message_id: &str,
) -> Result<(String, String, Vec<crate::db::models::Message>), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let target_created_at: String = db
        .query_row(
            "SELECT created_at FROM messages WHERE id = ?1 AND session_id = ?2",
            rusqlite::params![target_message_id, session_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Target message not found: {e}"))?;

    let (user_content, user_msg_id) = db
        .query_row(
            "SELECT content, id FROM messages
             WHERE session_id = ?1 AND role = 'user' AND created_at < ?2
             ORDER BY created_at DESC LIMIT 1",
            rusqlite::params![session_id, target_created_at],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|e| format!("No user message found before target: {e}"))?;

    let user_created_at: String = db
        .query_row(
            "SELECT created_at FROM messages WHERE id = ?1",
            [&user_msg_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = db
        .prepare(
            "SELECT id, session_id, role, content, token_usage, model,
                    thinking_content, attachments, status, created_at
             FROM messages
             WHERE session_id = ?1 AND created_at < ?2
             ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(
            rusqlite::params![session_id, user_created_at],
            map_message_row,
        )
        .map_err(|e| e.to_string())?;

    let messages_before: Vec<crate::db::models::Message> = rows
        .filter_map(|r| r.ok())
        .collect();

    Ok((user_content, user_msg_id, messages_before))
}

/// 删除指定消息及其之后的所有消息
fn delete_messages_from(
    state: &AppState,
    session_id: &str,
    message_id: &str,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let created_at: String = db
        .query_row(
            "SELECT created_at FROM messages WHERE id = ?1 AND session_id = ?2",
            rusqlite::params![message_id, session_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Message not found: {e}"))?;

    db.execute(
        "DELETE FROM messages WHERE session_id = ?1 AND created_at >= ?2",
        rusqlite::params![session_id, created_at],
    )
    .map_err(|e| format!("Failed to delete messages: {e}"))?;

    Ok(())
}

/// FTS 同步写入
fn sync_message_fts(
    db: &rusqlite::Connection,
    msg_id: &str,
    content: &str,
    session_id: &str,
    role: &str,
) {
    if content.is_empty() {
        return;
    }
    if let Err(e) = db.execute(
        "INSERT OR REPLACE INTO messages_fts (rowid, content, session_id, role)
         VALUES ((SELECT rowid FROM messages WHERE id = ?1), ?2, ?3, ?4)",
        rusqlite::params![msg_id, content, session_id, role],
    ) {
        tracing::warn!(error = %e, "FTS sync failed for message {}", msg_id);
    }
}

/// 映射单行消息查询结果
fn map_message_row(row: &rusqlite::Row) -> rusqlite::Result<crate::db::models::Message> {
    Ok(crate::db::models::Message {
        id: row.get(0)?,
        session_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        token_usage: row.get(4)?,
        model: row.get(5)?,
        thinking_content: row.get(6)?,
        attachments: row.get(7)?,
        status: row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "complete".to_string()),
        created_at: row.get(9)?,
    })
}

/// 收集查询结果并反转为时间正序
fn collect_messages_reversed(
    rows: rusqlite::MappedRows<
        '_,
        impl FnMut(&rusqlite::Row) -> rusqlite::Result<crate::db::models::Message>,
    >,
) -> Result<Vec<crate::db::models::Message>, String> {
    let mut messages: Vec<crate::db::models::Message> = rows
        .filter_map(|r| r.ok())
        .collect();
    messages.reverse();
    Ok(messages)
}
