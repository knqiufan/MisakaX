use crate::config::AppConfig;
use crate::AppState;
use std::collections::HashMap;
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
    value: serde_json::Value,
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

#[tauri::command]
pub fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn update_app_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    let mut current = state.config.lock().map_err(|e| e.to_string())?;
    *current = config;
    crate::config::save_config(&current).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_setting(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [&key],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = CURRENT_TIMESTAMP",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_all_settings(
    state: State<'_, AppState>,
) -> Result<HashMap<String, String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut map = HashMap::new();
    for row in rows {
        let (k, v) = row.map_err(|e| e.to_string())?;
        map.insert(k, v);
    }
    Ok(map)
}

#[derive(serde::Serialize)]
pub struct SystemInfo {
    pub app_version: String,
    pub os: String,
    pub arch: String,
    pub data_dir: String,
    pub db_size_bytes: u64,
}

#[tauri::command]
pub fn get_system_info() -> Result<SystemInfo, String> {
    let data_dir = crate::config::config_dir()
        .map(|d| d.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let db_path = crate::config::db_path()
        .map_err(|e| e.to_string())?;

    let db_size_bytes = std::fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(SystemInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        data_dir,
        db_size_bytes,
    })
}
