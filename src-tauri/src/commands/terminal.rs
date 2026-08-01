use std::sync::Arc;

use tauri::{State, WebviewWindow};

use crate::contracts::AppErrorPayload;
use crate::db::repository::SessionRepo;
use crate::services::terminal::{
    TerminalExitReason, TerminalKillReason, TerminalOwner, TerminalServiceError,
    TerminalSpawnRequest, TerminalState,
};
use crate::services::workspace::CanonicalWorkspace;
use crate::AppState;

#[tauri::command]
pub async fn terminal_spawn(
    window: WebviewWindow,
    state: State<'_, AppState>,
    chat_session_id: String,
    workspace_generation: u64,
    rows: u16,
    cols: u16,
    shell_profile: Option<String>,
) -> Result<TerminalState, AppErrorPayload> {
    require_feature(&state)?;
    let _binding_guard = state.workspace_terminal_guard.lock().await;
    let working_directory = session_workspace(&state, &chat_session_id)?;
    let workspace = CanonicalWorkspace::new(&working_directory)
        .map_err(|_| TerminalServiceError::SpawnFailed("workspace_missing").public_payload())?;
    let context = Arc::clone(&state.workspace_context)
        .get_context(&chat_session_id, workspace.clone(), false)
        .await;
    if context.generation != workspace_generation {
        return Err(TerminalServiceError::InvalidRequest("workspace_generation").public_payload());
    }
    // Re-read under the shared binding guard so a workspace update can neither
    // race validation nor redirect cwd between validation and PTY creation.
    let current = session_workspace(&state, &chat_session_id)?;
    let current = CanonicalWorkspace::new(current)
        .map_err(|_| TerminalServiceError::SpawnFailed("workspace_missing").public_payload())?;
    if current != workspace {
        return Err(TerminalServiceError::InvalidRequest("workspace_generation").public_payload());
    }

    Arc::clone(&state.terminal_manager)
        .spawn(TerminalSpawnRequest {
            owner: owner(&window, chat_session_id, workspace_generation),
            cwd: current.path().to_path_buf(),
            rows,
            cols,
            shell_profile,
        })
        .map_err(TerminalServiceError::public_payload)
}

#[tauri::command]
pub fn terminal_write(
    window: WebviewWindow,
    state: State<'_, AppState>,
    terminal_id: String,
    chat_session_id: String,
    workspace_generation: u64,
    data_base64: String,
) -> Result<(), AppErrorPayload> {
    require_feature(&state)?;
    state
        .terminal_manager
        .write_base64(
            &terminal_id,
            &owner(&window, chat_session_id, workspace_generation),
            &data_base64,
        )
        .map_err(TerminalServiceError::public_payload)
}

#[tauri::command]
pub fn terminal_resize(
    window: WebviewWindow,
    state: State<'_, AppState>,
    terminal_id: String,
    chat_session_id: String,
    workspace_generation: u64,
    rows: u16,
    cols: u16,
) -> Result<(), AppErrorPayload> {
    require_feature(&state)?;
    state
        .terminal_manager
        .resize(
            &terminal_id,
            &owner(&window, chat_session_id, workspace_generation),
            rows,
            cols,
        )
        .map_err(TerminalServiceError::public_payload)
}

#[tauri::command]
pub fn terminal_kill(
    window: WebviewWindow,
    state: State<'_, AppState>,
    terminal_id: String,
    chat_session_id: String,
    workspace_generation: u64,
    reason: TerminalKillReason,
) -> Result<(), AppErrorPayload> {
    require_feature(&state)?;
    state
        .terminal_manager
        .kill(
            &terminal_id,
            &owner(&window, chat_session_id, workspace_generation),
            TerminalExitReason::from(reason),
        )
        .map_err(TerminalServiceError::public_payload)
}

#[tauri::command]
pub fn terminal_get_state(
    window: WebviewWindow,
    state: State<'_, AppState>,
    terminal_id: String,
    chat_session_id: String,
    workspace_generation: u64,
) -> Result<TerminalState, AppErrorPayload> {
    require_feature(&state)?;
    state
        .terminal_manager
        .get_state(
            &terminal_id,
            &owner(&window, chat_session_id, workspace_generation),
        )
        .map_err(TerminalServiceError::public_payload)
}

fn owner(
    window: &WebviewWindow,
    chat_session_id: String,
    workspace_generation: u64,
) -> TerminalOwner {
    TerminalOwner {
        window_label: window.label().to_string(),
        chat_session_id,
        workspace_generation,
    }
}

fn session_workspace(
    state: &State<'_, AppState>,
    chat_session_id: &str,
) -> Result<String, AppErrorPayload> {
    let conn = state
        .db
        .lock()
        .map_err(|_| TerminalServiceError::Internal("database_lock").public_payload())?;
    SessionRepo::find_by_id(&conn, chat_session_id)
        .ok()
        .and_then(|session| session.working_directory)
        .filter(|path| !path.trim().is_empty())
        .ok_or_else(|| TerminalServiceError::SpawnFailed("workspace_missing").public_payload())
}

fn require_feature(state: &State<'_, AppState>) -> Result<(), AppErrorPayload> {
    if state.feature_flags.workspace_terminal {
        Ok(())
    } else {
        Err(TerminalServiceError::InvalidRequest("feature_disabled").public_payload())
    }
}
