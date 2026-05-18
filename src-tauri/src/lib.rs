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
use services::mcp::McpManager;
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
    let auto_start = app_config.auto_start_sidecar;

    let sidecar = Arc::new(SidecarManager::new(agent_dir, sidecar_port));

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
            commands::chat::get_messages,
            commands::fs_explorer::fs_list_dir,
            commands::fs_explorer::fs_read_text_file,
            commands::fs_explorer::fs_write_text_file,
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
            commands::session::pin_session,
            commands::session::archive_session,
            commands::session::set_session_group,
            commands::session::list_session_groups,
            commands::session::search_messages,
            commands::session::export_sessions,
            commands::session::import_sessions,
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
                start_mcp_health_loop(app_handle_for_mcp, mcp_for_connect);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running MisakaX");
}

const MCP_HEALTH_INTERVAL_SECS: u64 = 60;
const MCP_MAX_RETRIES: u32 = 3;

/// MCP Server 健康检查循环
///
/// 定期 ping 所有已连接的 Server，发现断连时自动重连（最多 3 次），
/// 状态变化通过 Tauri Event (`mcp:server_status`) 通知前端。
fn start_mcp_health_loop(app: tauri::AppHandle, manager: Arc<McpManager>) {
    use tauri::Emitter;

    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(MCP_HEALTH_INTERVAL_SECS)).await;

            let servers = manager.list_servers();
            for info in servers {
                if info.status != services::mcp::McpServerStatus::Connected {
                    continue;
                }

                let alive = manager.health_check(&info.id).await;
                if alive {
                    manager.reset_retry(&info.id);
                    continue;
                }

                tracing::warn!(
                    server_id = %info.id,
                    "MCP Server health check failed, attempting reconnect"
                );

                let retries = manager.retry_count(&info.id);
                if retries >= MCP_MAX_RETRIES {
                    tracing::error!(
                        server_id = %info.id,
                        retries = retries,
                        "MCP Server exceeded max retries, giving up"
                    );
                    continue;
                }

                manager.increment_retry(&info.id);

                if let Err(e) = manager.reconnect(&info.id).await {
                    tracing::warn!(
                        server_id = %info.id,
                        error = %e,
                        "MCP reconnect attempt failed"
                    );
                }

                if let Some(updated) = manager.server_info(&info.id) {
                    let _ = app.emit("mcp:server_status", &updated);
                }
            }
        }
    });
}
