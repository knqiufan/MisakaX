//! MCP Server 健康检查循环。
//!
//! 定期 ping 所有已连接的 Server，发现断连时自动重连（最多 3 次），
//! 状态变化通过 Tauri Event (`mcp:server_status`) 通知前端。

use std::sync::Arc;
use std::time::Duration;

use tauri::Emitter;

use crate::services::mcp::{McpManager, McpServerStatus};

const MCP_HEALTH_INTERVAL_SECS: u64 = 60;
const MCP_MAX_RETRIES: u32 = 3;

/// 启动后台健康检查循环。
pub fn start_health_loop(app: tauri::AppHandle, manager: Arc<McpManager>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(MCP_HEALTH_INTERVAL_SECS)).await;

            let servers = manager.list_servers();
            for info in servers {
                if info.status != McpServerStatus::Connected {
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
