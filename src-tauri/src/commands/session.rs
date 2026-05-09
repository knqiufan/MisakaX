use std::path::Path;

use tauri::State;

use crate::db::models::Session;
use crate::db::repository::{SessionRepo, WorkspaceRepo};
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

// ─── 内部辅助 ────────────────────────────────────────────────────────

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
