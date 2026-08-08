use std::path::{Path, PathBuf};

use tauri::State;

use crate::config;
use crate::db::models::{ExportData, ExportSession, ImportResult, MessageSearchResult, Session};
use crate::db::repository::{
    ArtifactRepo, MessageBlockRepo, MessageRepo, SessionRepo, WorkspaceRepo,
};
use crate::services::artifacts::RetentionState;
use crate::AppState;

pub const WORKSPACE_KIND_DEFAULT: &str = "default";
pub const WORKSPACE_KIND_CUSTOM: &str = "custom";

/// Resolved working directory + kind for session create/update.
pub struct ResolvedWorkspace {
    pub path: String,
    pub kind: String,
}

/// Resolve an explicit path or fall back to the app-managed default workspace.
pub fn resolve_workspace(dir: Option<&str>) -> Result<ResolvedWorkspace, String> {
    match dir {
        None | Some("") => {
            let path = ensure_default_workspace_path()?;
            Ok(ResolvedWorkspace {
                path,
                kind: WORKSPACE_KIND_DEFAULT.to_string(),
            })
        }
        Some(path) => {
            let canonical = validate_existing_dir(path)?;
            Ok(ResolvedWorkspace {
                path: canonical,
                kind: WORKSPACE_KIND_CUSTOM.to_string(),
            })
        }
    }
}

fn ensure_default_workspace_path() -> Result<String, String> {
    let path = config::default_workspace_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

fn validate_existing_dir(path: &str) -> Result<String, String> {
    let p = Path::new(path);
    if !p.exists() || !p.is_dir() {
        return Err(format!(
            "Working directory does not exist or is not a directory: {}",
            path
        ));
    }
    match std::fs::canonicalize(p) {
        Ok(canonical) => Ok(strip_windows_unc_prefix(canonical)),
        Err(_) => Ok(path.to_string()),
    }
}

fn strip_windows_unc_prefix(path: PathBuf) -> String {
    let s = path.to_string_lossy().to_string();
    // Windows canonicalize yields \\?\C:\...
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        s
    }
}

// ─── create_session Command ──────────────────────────────────────────

#[tauri::command]
pub fn create_session(
    state: State<'_, AppState>,
    title: Option<String>,
    model: Option<String>,
    working_directory: Option<String>,
) -> Result<Session, String> {
    let workspace = resolve_workspace(working_directory.as_deref())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();

    let session = SessionRepo::create_with_workspace(
        &conn,
        &id,
        title.as_deref(),
        model.as_deref(),
        Some(workspace.path.as_str()),
        &workspace.kind,
    )
    .map_err(|e| e.to_string())?;

    WorkspaceRepo::restore_workspace(&conn, &workspace.path).map_err(|e| e.to_string())?;

    record_directory_usage_internal(&conn, &workspace.path);

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
pub async fn delete_session(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let _binding_guard = state.workspace_terminal_guard.lock().await;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::delete(&conn, &id).map_err(|e| e.to_string())?;
    drop(conn);
    state.terminal_manager.kill_chat_session(&id);
    state.workspace_context.unbind_session(&id);
    Ok(())
}

// ─── search_sessions Command ─────────────────────────────────────────

#[tauri::command]
pub fn search_sessions(state: State<'_, AppState>, query: String) -> Result<Vec<Session>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::search(&conn, &query).map_err(|e| e.to_string())
}

// ─── update_session_working_dir Command ──────────────────────────────

#[tauri::command]
pub async fn update_session_working_dir(
    state: State<'_, AppState>,
    session_id: String,
    working_directory: Option<String>,
) -> Result<(), String> {
    let _binding_guard = state.workspace_terminal_guard.lock().await;
    let workspace = resolve_workspace(working_directory.as_deref())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    SessionRepo::update_working_directory_with_kind(
        &conn,
        &session_id,
        Some(workspace.path.as_str()),
        &workspace.kind,
    )
    .map_err(|e| e.to_string())?;

    record_directory_usage_internal(&conn, &workspace.path);
    drop(conn);
    state.workspace_context.unbind_session(&session_id);
    Ok(())
}

// ─── get_session Command ─────────────────────────────────────────────

#[tauri::command]
pub fn get_session(state: State<'_, AppState>, session_id: String) -> Result<Session, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::find_by_id(&conn, &session_id).map_err(|e| e.to_string())
}

// ─── pin_session Command ─────────────────────────────────────────────

#[tauri::command]
pub fn pin_session(state: State<'_, AppState>, id: String, pinned: bool) -> Result<(), String> {
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
pub fn list_session_groups(state: State<'_, AppState>) -> Result<Vec<String>, String> {
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
    export_sessions_to_file(&conn, &session_ids, Path::new(&file_path))
}

pub fn export_sessions_to_file(
    conn: &rusqlite::Connection,
    session_ids: &[String],
    file_path: &Path,
) -> Result<(), String> {
    let mut export_sessions = Vec::with_capacity(session_ids.len());
    for sid in session_ids {
        let session = SessionRepo::find_by_id(conn, sid).map_err(|e| e.to_string())?;
        let mut messages =
            MessageRepo::find_recent(conn, sid, u32::MAX).map_err(|e| e.to_string())?;
        for message in &mut messages {
            message.blocks =
                MessageBlockRepo::find_by_message(conn, &message.id).map_err(|e| e.to_string())?;
        }
        export_sessions.push(ExportSession { session, messages });
    }

    let export_data = ExportData {
        version: 1,
        exported_at: chrono::Utc::now().to_rfc3339(),
        app: "MisakaX".to_string(),
        sessions: export_sessions,
        artifact_manifest: ArtifactRepo::find_by_sessions(conn, session_ids)
            .map_err(|e| e.to_string())?,
    };

    let json = serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string())?;
    std::fs::write(file_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

// ─── import_sessions Command ────────────────────────────────────────

#[tauri::command]
pub fn import_sessions(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let result = import_sessions_from_file(&conn, Path::new(&file_path))?;
    // Normalize any imported sessions that still lack a working directory.
    if let Ok(default_dir) = ensure_default_workspace_path() {
        let _ = SessionRepo::backfill_null_workspaces(&conn, &default_dir);
        record_directory_usage_internal(&conn, &default_dir);
    }
    Ok(result)
}

pub fn import_sessions_from_file(
    conn: &rusqlite::Connection,
    file_path: &Path,
) -> Result<ImportResult, String> {
    let json = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let data: ExportData = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let mut imported = 0u32;
    let mut skipped = 0u32;
    let mut errors: Vec<String> = Vec::new();

    for es in &data.sessions {
        let exists = SessionRepo::find_by_id(conn, &es.session.id).is_ok();
        if exists {
            skipped += 1;
            continue;
        }

        if let Err(e) = import_single_session(conn, es) {
            errors.push(format!("Session {}: {}", es.session.id, e));
            continue;
        }
        imported += 1;
    }

    for mut artifact in data.artifact_manifest {
        if artifact.owner_session_id.is_empty()
            || ArtifactRepo::find_by_id(conn, &artifact.artifact_id).is_ok()
        {
            continue;
        }
        // A session export intentionally contains a manifest, not binary bytes.
        // Imported records stay inspectable but cannot be read until a future
        // import flow transfers verified content into the artifact store.
        artifact.retention_state = RetentionState::Expired;
        let _ = ArtifactRepo::insert(conn, &artifact);
    }

    Ok(ImportResult {
        imported_count: imported,
        skipped_count: skipped,
        errors,
    })
}

/// Idempotent backfill for sessions with NULL/empty working directories.
#[tauri::command]
pub fn backfill_session_workspaces(state: State<'_, AppState>) -> Result<usize, String> {
    let default_dir = ensure_default_workspace_path()?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let count =
        SessionRepo::backfill_null_workspaces(&conn, &default_dir).map_err(|e| e.to_string())?;
    if count > 0 {
        record_directory_usage_internal(&conn, &default_dir);
    }
    Ok(count)
}

// ─── 内部辅助 ────────────────────────────────────────────────────────

fn import_single_session(conn: &rusqlite::Connection, es: &ExportSession) -> Result<(), String> {
    let mut session = es.session.clone();
    if session
        .working_directory
        .as_deref()
        .unwrap_or("")
        .is_empty()
    {
        let default_dir = ensure_default_workspace_path()?;
        session.working_directory = Some(default_dir.clone());
        session.workspace_kind = WORKSPACE_KIND_DEFAULT.to_string();
        let name = Path::new(&default_dir)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workspace")
            .to_string();
        session.project_name = Some(name);
    } else if session.workspace_kind.is_empty() {
        session.workspace_kind = WORKSPACE_KIND_CUSTOM.to_string();
    }

    SessionRepo::import(conn, &session).map_err(|e| e.to_string())?;

    for m in &es.messages {
        MessageRepo::import(conn, m).map_err(|e| e.to_string())?;
        for block in &m.blocks {
            MessageBlockRepo::insert(conn, block).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
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
