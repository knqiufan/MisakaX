#[cfg(not(feature = "test-private"))]
mod commands;
#[cfg(feature = "test-private")]
pub mod commands;
pub mod config;
pub mod crypto;
pub mod db;
pub mod services;
#[cfg(not(feature = "test-private"))]
mod sidecar;
#[cfg(feature = "test-private")]
pub mod sidecar;

use config::AppConfig;
use services::llm::StreamRegistry;
use services::sidecar_client::SidecarClient;
use sidecar::SidecarManager;
use std::sync::{Arc, Mutex};

/// Application state shared across commands
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub config: Mutex<AppConfig>,
    pub sidecar: Arc<SidecarManager>,
    pub sidecar_client: SidecarClient,
    /// 流式会话注册表 — 跟踪活跃流并支持 abort（停止生成）。
    /// DashMap 内部实现并发安全，无需外层 Mutex。
    pub stream_registry: StreamRegistry,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    config::ensure_directories().expect("Failed to create config directories");
    let app_config = config::load_config().expect("Failed to load configuration");

    let db_path = config::db_path().expect("Failed to determine database path");
    let conn = db::init_database(&db_path).expect("Failed to initialize database");

    let agent_dir = std::env::current_dir()
        .map(|d| d.join("agent"))
        .unwrap_or_else(|_| std::path::PathBuf::from("agent"));

    let sidecar_port = app_config.sidecar_port;
    let auto_start = app_config.auto_start_sidecar;

    let sidecar = Arc::new(SidecarManager::new(
        agent_dir.to_str().unwrap_or("agent").to_string(),
        sidecar_port,
    ));

    let sidecar_client = SidecarClient::new(sidecar_port);

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
            sidecar: Arc::clone(&sidecar),
            sidecar_client,
            stream_registry: StreamRegistry::new(),
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::settings::get_app_config,
            commands::settings::update_app_config,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_all_settings,
            commands::settings::get_system_info,
            commands::router_configs::list_router_configs,
            commands::router_configs::create_router_config,
            commands::router_configs::update_router_config,
            commands::router_configs::delete_router_config,
            commands::router_configs::test_router_connection,
            commands::models::list_available_models,
            commands::models::add_custom_model,
            commands::models::delete_custom_model,
            commands::chat::send_message,
            commands::chat::stop_generation,
            commands::chat::regenerate_message,
            commands::chat::get_messages,
            commands::workspace::browse_directory,
            commands::workspace::validate_directory,
            commands::workspace::get_recent_directories,
            commands::workspace::record_directory_usage,
            commands::workspace::remove_recent_directory,
            commands::session::create_session,
            commands::session::list_sessions,
            commands::session::update_session,
            commands::session::delete_session,
            commands::session::search_sessions,
            commands::session::update_session_working_dir,
            commands::session::get_session,
            commands::sidecar::get_sidecar_status,
            commands::sidecar::restart_sidecar,
        ])
        .setup(move |app| {
            tracing::info!("MisakaX initialized successfully");

            if auto_start {
                let app_handle = app.handle().clone();
                sidecar.preheat(app_handle.clone());

                let sidecar_port_for_check = sidecar_port;
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(12)).await;
                    let client = SidecarClient::new(sidecar_port_for_check);
                    match client.health().await {
                        Ok(resp) => {
                            tracing::info!(
                                "Sidecar health check via SidecarClient: OK (v{}, uptime={:.1}s)",
                                resp.version,
                                resp.uptime_seconds,
                            );
                        }
                        Err(e) => {
                            tracing::warn!("Sidecar health check via SidecarClient failed: {}", e);
                        }
                    }
                });
            } else {
                tracing::info!(
                    "Python Sidecar auto-start disabled (set auto_start_sidecar: true in config)"
                );
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running MisakaX");
}
