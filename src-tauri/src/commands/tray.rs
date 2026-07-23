use serde::Deserialize;
use tauri::{AppHandle, State};

use crate::{config::CloseBehavior, tray, AppState};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseRequestAction {
    MinimizeToTray,
    Quit,
}

#[tauri::command]
pub fn update_tray_context(
    app: AppHandle,
    state: State<'_, AppState>,
    working_directory: Option<String>,
    project_name: Option<String>,
    workspace_kind: Option<String>,
) -> Result<(), String> {
    let language = state
        .config
        .lock()
        .map_err(|error| error.to_string())?
        .language
        .clone();
    tray::update_context(
        &app,
        working_directory,
        project_name,
        workspace_kind,
        &language,
    )
}

#[tauri::command]
pub fn resolve_close_request(
    app: AppHandle,
    state: State<'_, AppState>,
    action: CloseRequestAction,
    remember: bool,
) -> Result<(), String> {
    let close_behavior = match action {
        CloseRequestAction::MinimizeToTray => CloseBehavior::MinimizeToTray,
        CloseRequestAction::Quit => CloseBehavior::Quit,
    };

    if remember {
        let updated_config = {
            let mut config = state.config.lock().map_err(|error| error.to_string())?;
            config.close_behavior = close_behavior;
            crate::config::save_config(&config).map_err(|error| error.to_string())?;
            config.clone()
        };
        tray::refresh_labels(&app, &updated_config);
    }

    match action {
        CloseRequestAction::MinimizeToTray => tray::hide_main_window(&app),
        CloseRequestAction::Quit => {
            app.exit(0);
            Ok(())
        }
    }
}
