use crate::sidecar::SidecarStatus;
use crate::AppState;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_sidecar_status(state: State<'_, AppState>) -> SidecarStatus {
    state.sidecar.status()
}

#[tauri::command]
pub async fn restart_sidecar(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let sidecar = Arc::clone(&state.sidecar);
    sidecar.restart(app).await;
    Ok(())
}
