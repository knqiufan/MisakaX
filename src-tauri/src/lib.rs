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
#[cfg(not(feature = "test-private"))]
mod sidecar_ownership;
#[cfg(feature = "test-private")]
pub mod sidecar_ownership;
pub mod tray;

use config::AppConfig;
use db::repository::SessionRepo;
use services::llm::StreamRegistry;
use services::mcp::McpManager;
use services::sidecar_client::SidecarClient;
use sidecar::SidecarManager;
use std::sync::{Arc, Mutex};
use tauri::{Manager, RunEvent};

/// Application state shared across commands
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub config: Mutex<AppConfig>,
    pub sidecar: Arc<SidecarManager>,
    pub sidecar_client: SidecarClient,
    /// 流式会话注册表 — 跟踪活跃流并支持 abort（停止生成）。
    /// DashMap 内部实现并发安全，无需外层 Mutex。
    pub stream_registry: StreamRegistry,
    /// MCP Server 管理器 — 管理所有 MCP Server 连接的生命周期
    pub mcp_manager: Arc<McpManager>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    config::ensure_directories().expect("Failed to create config directories");
    let app_config = config::load_config().expect("Failed to load configuration");

    let db_path = config::db_path().expect("Failed to determine database path");
    let conn = db::init_database(&db_path).expect("Failed to initialize database");

    let agent_dir = sidecar::resolve_agent_working_dir();
    tracing::info!(
        path = %agent_dir.display(),
        "Python sidecar working directory"
    );

    let sidecar_port = app_config.sidecar_port;
    let mcp_bridge_port = app_config.mcp_bridge_port;
    let auto_start = app_config.auto_start_sidecar;

    let sidecar_api_key_env = {
        use crate::db::repository::RouterConfigRepo;
        use crate::sidecar::{build_sidecar_api_key_env, collect_sidecar_api_key_sources};
        match RouterConfigRepo::list_all(&conn) {
            Ok(configs) => {
                let sources = collect_sidecar_api_key_sources(&configs);
                build_sidecar_api_key_env(&sources)
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to load router configs for sidecar API keys");
                std::collections::HashMap::new()
            }
        }
    };

    let sidecar = Arc::new(SidecarManager::with_options(
        agent_dir,
        sidecar_port,
        mcp_bridge_port,
        sidecar_api_key_env,
    ));

    let sidecar_client = SidecarClient::new(sidecar_port);

    let mcp_manager = Arc::new(McpManager::new());

    let mcp_configs = {
        let config_root = config::config_dir().unwrap_or_default();
        services::mcp::McpConfigLoader::load_all(&config_root, &conn).unwrap_or_default()
    };

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
            mcp_manager: Arc::clone(&mcp_manager),
        })
        .manage(tray::TrayState::default())
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::settings::get_app_config,
            commands::settings::update_app_config,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_all_settings,
            commands::settings::get_system_info,
            commands::tray::update_tray_context,
            commands::tray::resolve_close_request,
            commands::router_configs::list_router_configs,
            commands::router_configs::create_router_config,
            commands::router_configs::create_router_config_with_models,
            commands::router_configs::update_router_config,
            commands::router_configs::delete_router_config,
            commands::router_configs::reveal_router_api_key,
            commands::router_configs::test_router_connection,
            commands::models::list_available_models,
            commands::models::list_custom_models,
            commands::models::add_custom_model,
            commands::models::replace_custom_models,
            commands::models::delete_custom_model,
            commands::models::fetch_provider_models,
            commands::models::test_model,
            commands::chat::send_message,
            commands::chat::stop_generation,
            commands::chat::regenerate_message,
            commands::chat::generate_session_title,
            commands::chat::get_messages,
            commands::fs_explorer::fs_list_dir,
            commands::fs_explorer::fs_read_text_file,
            commands::fs_explorer::fs_write_text_file,
            commands::fs_explorer::fs_reveal_in_explorer,
            commands::workspace::browse_directory,
            commands::workspace::validate_directory,
            commands::workspace::get_recent_directories,
            commands::workspace::record_directory_usage,
            commands::workspace::remove_recent_directory,
            commands::workspace::list_workspace_preferences,
            commands::workspace::update_workspace_preference,
            commands::session::create_session,
            commands::session::list_sessions,
            commands::session::update_session,
            commands::session::delete_session,
            commands::session::search_sessions,
            commands::session::update_session_working_dir,
            commands::session::get_session,
            commands::session::pin_session,
            commands::session::archive_session,
            commands::session::set_session_group,
            commands::session::list_session_groups,
            commands::session::search_messages,
            commands::session::export_sessions,
            commands::session::import_sessions,
            commands::session::backfill_session_workspaces,
            commands::sidecar::get_sidecar_status,
            commands::sidecar::restart_sidecar,
            commands::mcp::mcp_list_servers,
            commands::mcp::mcp_connect_server,
            commands::mcp::mcp_disconnect_server,
            commands::mcp::mcp_restart_server,
            commands::mcp::mcp_list_tools,
            commands::mcp::mcp_call_tool,
            commands::mcp::mcp_add_server_config,
            commands::mcp::mcp_remove_server_config,
            commands::mcp::mcp_approve_tool_call,
            commands::mcp::mcp_deny_tool_call,
            commands::mcp::mcp_list_permissions,
            commands::mcp::mcp_reset_permission,
            commands::skills::skills_list_installed,
            commands::skills::skills_get_detail,
            commands::skills::skills_inspect_archive,
            commands::skills::skills_install_archive,
            commands::skills::skills_search_remote,
            commands::skills::skills_get_remote_detail,
            commands::skills::skills_install_remote,
            commands::skills::skills_import_modelscope,
            commands::skills::skills_export_installed,
            commands::skills::skills_download_remote,
            commands::skills::skills_set_enabled,
            commands::skills::skills_uninstall,
        ])
        .on_window_event(tray::handle_window_event)
        .setup(move |app| {
            tracing::info!("MisakaX initialized successfully");

            tray::setup(app.handle())?;

            // Backfill sessions that still lack a working directory.
            {
                let state = app.state::<AppState>();
                if let Ok(default_dir) = crate::config::default_workspace_dir() {
                    let path = default_dir.to_string_lossy().to_string();
                    let _ = std::fs::create_dir_all(&default_dir);
                    if let Ok(conn) = state.db.lock() {
                        match SessionRepo::backfill_null_workspaces(&conn, &path) {
                            Ok(n) if n > 0 => {
                                tracing::info!(count = n, "Backfilled session workspaces");
                            }
                            Err(e) => {
                                tracing::warn!(error = %e, "Session workspace backfill failed")
                            }
                            _ => {}
                        }
                    }
                }
            }

            // MCP HTTP Bridge must bind before Sidecar becomes Ready.
            let mcp_bridge_port = {
                let cfg = app.state::<AppState>();
                cfg.config.lock().map(|c| c.mcp_bridge_port).unwrap_or(9528)
            };
            let bridge_app = app.handle().clone();
            let sidecar_for_bridge_err = Arc::clone(&sidecar);
            tauri::async_runtime::spawn(async move {
                if let Err(e) = services::mcp_http_bridge::serve(bridge_app, mcp_bridge_port).await
                {
                    sidecar_for_bridge_err.set_mcp_bridge_ready(false);
                    tracing::error!(error = %e, "MCP HTTP bridge stopped");
                }
            });

            if auto_start {
                let app_handle = app.handle().clone();
                let sidecar_for_preheat = Arc::clone(&sidecar);
                // Give the bridge a brief head start, then preheat Sidecar.
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    sidecar_for_preheat.preheat(app_handle);
                });

                let sidecar_for_check = Arc::clone(&sidecar);
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(12)).await;
                    if !sidecar_for_check.is_ready_for_chat() {
                        tracing::warn!(
                            status = ?sidecar_for_check.status(),
                            bridge_ready = sidecar_for_check.mcp_bridge_is_ready(),
                            "Managed Sidecar is not ready after startup window"
                        );
                        return;
                    }
                    let client = SidecarClient::new(sidecar_port);
                    match client.health().await {
                        Ok(resp) => {
                            tracing::info!(
                                "Managed Sidecar health OK (v{}, uptime={:.1}s)",
                                resp.version,
                                resp.uptime_seconds,
                            );
                        }
                        Err(e) => {
                            tracing::warn!("Managed Sidecar health check failed: {}", e);
                        }
                    }
                });
            } else {
                tracing::info!(
                    "Python Sidecar auto-start disabled (set auto_start_sidecar: true in config)"
                );
            }

            // MCP: 异步连接 auto_connect Server
            let mcp_for_connect = Arc::clone(&mcp_manager);
            let mcp_configs_for_setup = mcp_configs.clone();
            let app_handle_for_mcp = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let auto_configs: Vec<_> = mcp_configs_for_setup
                    .into_iter()
                    .filter(|c| c.auto_connect)
                    .collect();

                if auto_configs.is_empty() {
                    tracing::info!("No MCP servers configured for auto-connect");
                    return;
                }

                tracing::info!(count = auto_configs.len(), "Auto-connecting MCP servers");

                for config in auto_configs {
                    let server_name = config.name.clone();
                    if let Err(e) = mcp_for_connect.connect(config).await {
                        tracing::warn!(
                            server = %server_name,
                            error = %e,
                            "Failed to auto-connect MCP server"
                        );
                    }
                }

                // 启动健康检查循环
                services::mcp::start_health_loop(app_handle_for_mcp, mcp_for_connect);
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building MisakaX")
        .run(|app_handle, event| {
            if matches!(event, RunEvent::Exit | RunEvent::ExitRequested { .. }) {
                let state = app_handle.state::<AppState>();
                state.sidecar.shutdown();
            }
        });
}
