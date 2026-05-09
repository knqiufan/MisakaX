use std::fs;
use std::path::Path;

use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::db::repository::workspace_repo::{DirectoryInfo, RecentDirectory, WorkspaceRepo};
use crate::AppState;

// ─── browse_directory Command ─────────────────────────────────────────

#[tauri::command]
pub async fn browse_directory(
    app: tauri::AppHandle,
    start_path: Option<String>,
) -> Result<Option<String>, String> {
    let mut builder = app.dialog().file();

    if let Some(ref dir) = start_path {
        let p = Path::new(dir);
        if p.is_dir() {
            builder = builder.set_directory(p);
        }
    }

    builder = builder.set_title("Select Working Directory");

    let result = builder
        .blocking_pick_folder()
        .map(|fp| fp.to_string());

    Ok(result)
}

// ─── validate_directory Command ───────────────────────────────────────

#[tauri::command]
pub fn validate_directory(path: String) -> Result<DirectoryInfo, String> {
    let p = Path::new(&path);
    let exists = p.exists() && p.is_dir();

    if !exists {
        return Ok(DirectoryInfo {
            path: path.clone(),
            name: extract_dir_name(&path),
            exists: false,
            readable: false,
            writable: false,
            file_count: None,
        });
    }

    let readable = fs::read_dir(p).is_ok();
    let writable = check_writable(p);
    let file_count = count_top_level_entries(p);

    Ok(DirectoryInfo {
        path: path.clone(),
        name: extract_dir_name(&path),
        exists: true,
        readable,
        writable,
        file_count: Some(file_count),
    })
}

// ─── get_recent_directories Command ───────────────────────────────────

#[tauri::command]
pub fn get_recent_directories(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<RecentDirectory>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(20);
    WorkspaceRepo::find_recent(&conn, limit).map_err(|e| e.to_string())
}

// ─── record_directory_usage Command ───────────────────────────────────

#[tauri::command]
pub fn record_directory_usage(
    state: State<'_, AppState>,
    path: String,
    display_name: Option<String>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    WorkspaceRepo::record_usage(
        &conn,
        &id,
        &path,
        display_name.as_deref(),
    )
    .map_err(|e| e.to_string())
}

// ─── remove_recent_directory Command ──────────────────────────────────

#[tauri::command]
pub fn remove_recent_directory(
    state: State<'_, AppState>,
    path: String,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    WorkspaceRepo::delete_by_path(&conn, &path).map_err(|e| e.to_string())
}

// ─── 辅助函数 ─────────────────────────────────────────────────────────

fn extract_dir_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}

fn check_writable(dir: &Path) -> bool {
    let probe = dir.join(".misaka_write_test");
    if fs::write(&probe, b"").is_ok() {
        let _ = fs::remove_file(&probe);
        true
    } else {
        false
    }
}

fn count_top_level_entries(dir: &Path) -> u32 {
    fs::read_dir(dir)
        .map(|entries| entries.count() as u32)
        .unwrap_or(0)
}
