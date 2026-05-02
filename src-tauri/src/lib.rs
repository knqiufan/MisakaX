mod commands;
pub mod config;
pub mod db;

use config::AppConfig;
use std::sync::Mutex;

/// Application state shared across commands
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub config: Mutex<AppConfig>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Ensure directories exist early (before Tauri setup, in case setup fails)
    config::ensure_directories().expect("Failed to create config directories");

    // Load config early for validation
    let app_config = config::load_config().expect("Failed to load configuration");

    // Initialize database
    let db_path = config::db_path().expect("Failed to determine database path");
    let conn = db::init_database(&db_path).expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState {
            db: Mutex::new(conn),
            config: Mutex::new(app_config),
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::update_setting,
        ])
        .setup(|_app| {
            tracing::info!("MisakaX initialized successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running MisakaX");
}
