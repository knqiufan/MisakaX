use crate::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    serde_json::to_string(&*config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: Value,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;

    match key.as_str() {
        "theme" => config.theme = value.as_str().unwrap_or("system").to_string(),
        "language" => config.language = value.as_str().unwrap_or("en").to_string(),
        "accent_color" => config.accent_color = value.as_str().unwrap_or("#6366f1").to_string(),
        "default_model" => {
            config.default_model =
                value.as_str().unwrap_or("claude-sonnet-4-20250514").to_string()
        }
        "log_level" => config.log_level = value.as_str().unwrap_or("info").to_string(),
        _ => return Err(format!("Unknown setting key: {}", key)),
    }

    crate::config::save_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}
