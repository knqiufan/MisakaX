use std::path::Path;

use tauri::State;

use crate::db::models::{ExportData, ExportSession, ImportResult, MessageSearchResult, Session};
use crate::db::repository::{MessageRepo, SessionRepo, WorkspaceRepo};
use crate::AppState;

// ─── create_session Command ──────────────────────────────────────────

#[tauri::command]
pub fn create_session(
    state: State<'_, AppState>,
    title: Option<String>,
    model: Option<String>,
    working_directory: Option<String>,
) -> Result<Session, String> {
    let validated_dir = validate_working_dir(working_directory.as_deref())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();

    let session = SessionRepo::create(
        &conn,
        &id,
        title.as_deref(),
        model.as_deref(),
        validated_dir.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    if let Some(ref dir) = validated_dir {
        record_directory_usage_internal(&conn, dir);
    }

    Ok(session)
}

// ─── list_sessions Command ───────────────────────────────────────────

#[tauri::command]
pub fn list_sessions(
    state: State<'_, AppState>,
    status: Option<String>,
) -> Result<Vec<Session>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::list(&conn, status.as_deref()).map_err(|e| e.to_string())
}

// ─── update_session Command ──────────────────────────────────────────

#[tauri::command]
pub fn update_session(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    model: Option<String>,
    pinned: Option<bool>,
    status: Option<String>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::update(
        &conn,
        &id,
        title.as_deref(),
        model.as_deref(),
        pinned,
        status.as_deref(),
    )
    .map_err(|e| e.to_string())
}

// ─── delete_session Command ──────────────────────────────────────────

#[tauri::command]
pub fn delete_session(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::delete(&conn, &id).map_err(|e| e.to_string())
}

// ─── search_sessions Command ─────────────────────────────────────────

#[tauri::command]
pub fn search_sessions(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<Session>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::search(&conn, &query).map_err(|e| e.to_string())
}

// ─── update_session_working_dir Command ──────────────────────────────

#[tauri::command]
pub fn update_session_working_dir(
    state: State<'_, AppState>,
    session_id: String,
    working_directory: Option<String>,
) -> Result<(), String> {
    let validated_dir = validate_working_dir(working_directory.as_deref())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    SessionRepo::update_working_directory(&conn, &session_id, validated_dir.as_deref())
        .map_err(|e| e.to_string())?;

    if let Some(ref dir) = validated_dir {
        record_directory_usage_internal(&conn, dir);
    }

    Ok(())
}

// ─── get_session Command ─────────────────────────────────────────────

#[tauri::command]
pub fn get_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Session, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::find_by_id(&conn, &session_id).map_err(|e| e.to_string())
}

// ─── pin_session Command ─────────────────────────────────────────────

#[tauri::command]
pub fn pin_session(
    state: State<'_, AppState>,
    id: String,
    pinned: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::pin_session(&conn, &id, pinned).map_err(|e| e.to_string())
}

// ─── archive_session Command ────────────────────────────────────────

#[tauri::command]
pub fn archive_session(
    state: State<'_, AppState>,
    id: String,
    archived: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if archived {
        SessionRepo::archive_session(&conn, &id)
    } else {
        SessionRepo::unarchive_session(&conn, &id)
    }
    .map_err(|e| e.to_string())
}

// ─── set_session_group Command ──────────────────────────────────────

#[tauri::command]
pub fn set_session_group(
    state: State<'_, AppState>,
    id: String,
    group: Option<String>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::set_group(&conn, &id, group.as_deref()).map_err(|e| e.to_string())
}

// ─── list_session_groups Command ────────────────────────────────────

#[tauri::command]
pub fn list_session_groups(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::list_groups(&conn).map_err(|e| e.to_string())
}

// ─── search_messages Command (FTS5) ─────────────────────────────────

#[tauri::command]
pub fn search_messages(
    state: State<'_, AppState>,
    query: String,
    session_id: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<MessageSearchResult>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    MessageRepo::search_fts(&conn, &query, session_id.as_deref(), limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

// ─── export_sessions Command ────────────────────────────────────────

#[tauri::command]
pub fn export_sessions(
    state: State<'_, AppState>,
    session_ids: Vec<String>,
    file_path: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut export_sessions = Vec::with_capacity(session_ids.len());
    for sid in &session_ids {
        let session = SessionRepo::find_by_id(&conn, sid).map_err(|e| e.to_string())?;
        let messages =
            MessageRepo::find_recent(&conn, sid, u32::MAX).map_err(|e| e.to_string())?;
        export_sessions.push(ExportSession { session, messages });
    }

    let export_data = ExportData {
        version: 1,
        exported_at: chrono::Utc::now().to_rfc3339(),
        app: "MisakaX".to_string(),
        sessions: export_sessions,
    };

    let json = serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string())?;
    std::fs::write(&file_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

// ─── import_sessions Command ────────────────────────────────────────

#[tauri::command]
pub fn import_sessions(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let json = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let data: ExportData = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut imported = 0u32;
    let mut skipped = 0u32;
    let mut errors: Vec<String> = Vec::new();

    for es in &data.sessions {
        let exists = SessionRepo::find_by_id(&conn, &es.session.id).is_ok();
        if exists {
            skipped += 1;
            continue;
        }

        if let Err(e) = import_single_session(&conn, es) {
            errors.push(format!("Session {}: {}", es.session.id, e));
            continue;
        }
        imported += 1;
    }

    Ok(ImportResult {
        imported_count: imported,
        skipped_count: skipped,
        errors,
    })
}

// ─── 内部辅助 ────────────────────────────────────────────────────────

fn import_single_session(
    conn: &rusqlite::Connection,
    es: &ExportSession,
) -> Result<(), String> {
    let s = &es.session;
    conn.execute(
        "INSERT INTO sessions (id, title, model, system_prompt, working_directory, project_name,
            status, mode, total_input_tokens, total_output_tokens,
            last_message_at, pinned, group_name, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        rusqlite::params![
            s.id,
            s.title,
            s.model,
            s.system_prompt,
            s.working_directory,
            s.project_name,
            s.status,
            s.mode,
            s.total_input_tokens,
            s.total_output_tokens,
            s.last_message_at,
            s.pinned as i32,
            s.group_name,
            s.created_at,
            s.updated_at,
        ],
    )
    .map_err(|e| e.to_string())?;

    for m in &es.messages {
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, token_usage, model,
                thinking_content, attachments, status, tool_calls, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            rusqlite::params![
                m.id,
                m.session_id,
                m.role,
                m.content,
                m.token_usage,
                m.model,
                m.thinking_content,
                m.attachments,
                m.status,
                m.tool_calls,
                m.created_at,
            ],
        )
        .map_err(|e| e.to_string())?;

        if !m.content.is_empty() {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO messages_fts (rowid, content, session_id, role)
                 VALUES ((SELECT rowid FROM messages WHERE id = ?1), ?2, ?3, ?4)",
                rusqlite::params![m.id, m.content, m.session_id, m.role],
            );
        }
    }
    Ok(())
}

fn validate_working_dir(dir: Option<&str>) -> Result<Option<String>, String> {
    match dir {
        None | Some("") => Ok(None),
        Some(path) => {
            let p = Path::new(path);
            if !p.exists() || !p.is_dir() {
                return Err(format!(
                    "Working directory does not exist or is not a directory: {}",
                    path
                ));
            }
            Ok(Some(path.to_string()))
        }
    }
}

fn record_directory_usage_internal(conn: &rusqlite::Connection, dir: &str) {
    let id = uuid::Uuid::new_v4().to_string();
    let display_name = Path::new(dir)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    if let Err(e) = WorkspaceRepo::record_usage(conn, &id, dir, display_name.as_deref()) {
        tracing::warn!("Failed to record directory usage: {}", e);
    }
}
