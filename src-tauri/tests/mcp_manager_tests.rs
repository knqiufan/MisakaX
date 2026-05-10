use std::collections::HashMap;

use misaka_x_lib::services::mcp::manager::McpManager;
use misaka_x_lib::services::mcp::types::{McpServerConfig, McpServerStatus, McpTransport};

fn make_config(id: &str, transport: McpTransport) -> McpServerConfig {
    McpServerConfig {
        id: id.to_string(),
        name: format!("Test {}", id),
        transport,
        auto_connect: true,
        env: HashMap::new(),
    }
}

fn make_stdio_config(id: &str) -> McpServerConfig {
    make_config(
        id,
        McpTransport::Stdio {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
        },
    )
}

// ─── 基础功能 ─────────────────────────────────────────────────────────

#[test]
fn test_new_manager_is_empty() {
    let manager = McpManager::new();
    assert_eq!(manager.connected_count(), 0);
    assert!(manager.list_servers().is_empty());
    assert!(manager.list_all_tools().is_empty());
}

#[test]
fn test_add_server_config() {
    let manager = McpManager::new();
    manager.add_server_config(make_stdio_config("srv-1"));

    let servers = manager.list_servers();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].id, "srv-1");
    assert_eq!(servers[0].status, McpServerStatus::Disconnected);
}

#[test]
fn test_add_multiple_configs() {
    let manager = McpManager::new();
    manager.add_server_config(make_stdio_config("srv-1"));
    manager.add_server_config(make_config(
        "srv-2",
        McpTransport::Http {
            url: "http://localhost:3000".to_string(),
            headers: HashMap::new(),
        },
    ));
    manager.add_server_config(make_config(
        "srv-3",
        McpTransport::Sse {
            url: "http://localhost:3001/sse".to_string(),
            headers: HashMap::new(),
        },
    ));

    assert_eq!(manager.list_servers().len(), 3);
}

#[test]
fn test_server_info() {
    let manager = McpManager::new();
    manager.add_server_config(make_stdio_config("srv-1"));

    let info = manager.server_info("srv-1");
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.id, "srv-1");
    assert_eq!(info.transport_type, "stdio");

    assert!(manager.server_info("nonexistent").is_none());
}

#[test]
fn test_is_connected_initial() {
    let manager = McpManager::new();
    manager.add_server_config(make_stdio_config("srv-1"));

    assert!(!manager.is_connected("srv-1"));
    assert!(!manager.is_connected("nonexistent"));
}

// ─── 重试计数 ─────────────────────────────────────────────────────────

#[test]
fn test_retry_count_operations() {
    let manager = McpManager::new();
    manager.add_server_config(make_stdio_config("srv-1"));

    assert_eq!(manager.retry_count("srv-1"), 0);

    manager.increment_retry("srv-1");
    assert_eq!(manager.retry_count("srv-1"), 1);

    manager.increment_retry("srv-1");
    manager.increment_retry("srv-1");
    assert_eq!(manager.retry_count("srv-1"), 3);

    manager.reset_retry("srv-1");
    assert_eq!(manager.retry_count("srv-1"), 0);
}

#[test]
fn test_retry_count_nonexistent() {
    let manager = McpManager::new();
    assert_eq!(manager.retry_count("nonexistent"), 0);
    manager.increment_retry("nonexistent");
}

// ─── find_server_for_tool ─────────────────────────────────────────────

#[test]
fn test_find_server_for_tool_no_servers() {
    let manager = McpManager::new();
    assert!(manager.find_server_for_tool("read_file").is_none());
}

// ─── remove_server ────────────────────────────────────────────────────

#[tokio::test]
async fn test_remove_server() {
    let manager = McpManager::new();
    manager.add_server_config(make_stdio_config("srv-1"));
    assert_eq!(manager.list_servers().len(), 1);

    manager.remove_server("srv-1").await.unwrap();
    assert!(manager.list_servers().is_empty());
}

// ─── 连接失败场景 ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_connect_invalid_command_sets_error_status() {
    let manager = McpManager::new();
    let config = make_config(
        "bad-srv",
        McpTransport::Stdio {
            command: "nonexistent_binary_that_does_not_exist_xyz_42".to_string(),
            args: vec![],
        },
    );

    let result = manager.connect(config).await;
    assert!(result.is_err());

    let info = manager.server_info("bad-srv").unwrap();
    match info.status {
        McpServerStatus::Error(_) => {}
        other => panic!("Expected Error status, got {:?}", other),
    }
}

#[tokio::test]
async fn test_disconnect_nonexistent_server_is_ok() {
    let manager = McpManager::new();
    let result = manager.disconnect("nonexistent").await;
    assert!(result.is_ok());
}

// ─── 健康检查 ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_health_check_disconnected_returns_false() {
    let manager = McpManager::new();
    manager.add_server_config(make_stdio_config("srv-1"));
    assert!(!manager.health_check("srv-1").await);
}

#[tokio::test]
async fn test_health_check_nonexistent_returns_false() {
    let manager = McpManager::new();
    assert!(!manager.health_check("nonexistent").await);
}

// ─── Default trait ────────────────────────────────────────────────────

#[test]
fn test_default_trait() {
    let manager = McpManager::default();
    assert!(manager.list_servers().is_empty());
}
