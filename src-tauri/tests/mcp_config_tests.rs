use std::path::PathBuf;

use misaka_x_lib::services::mcp::config::McpConfigLoader;

#[test]
fn test_load_from_nonexistent_file() {
    let path = PathBuf::from("nonexistent_mcp.json");
    let configs = McpConfigLoader::load_from_file(&path).unwrap();
    assert!(configs.is_empty());
}

#[test]
fn test_load_empty_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp.json");
    std::fs::write(&path, r#"{"mcpServers":{}}"#).unwrap();

    let configs = McpConfigLoader::load_from_file(&path).unwrap();
    assert!(configs.is_empty());
}

#[test]
fn test_load_stdio_server_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp.json");
    std::fs::write(
        &path,
        r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
                    "env": {"HOME": "/tmp"}
                }
            }
        }"#,
    )
    .unwrap();

    let configs = McpConfigLoader::load_from_file(&path).unwrap();
    assert_eq!(configs.len(), 1);

    let config = &configs[0];
    assert_eq!(config.id, "filesystem");
    assert_eq!(config.name, "filesystem");
    assert!(config.auto_connect);

    match &config.transport {
        misaka_x_lib::services::mcp::McpTransport::Stdio { command, args } => {
            assert_eq!(command, "npx");
            assert_eq!(args.len(), 3);
        }
        _ => panic!("Expected Stdio transport"),
    }

    assert_eq!(config.env.get("HOME").unwrap(), "/tmp");
}

#[test]
fn test_load_http_server_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp.json");
    std::fs::write(
        &path,
        r#"{
            "mcpServers": {
                "remote-server": {
                    "url": "https://mcp.example.com/v1",
                    "headers": {"Authorization": "Bearer sk-123"},
                    "transportType": "http",
                    "autoConnect": false
                }
            }
        }"#,
    )
    .unwrap();

    let configs = McpConfigLoader::load_from_file(&path).unwrap();
    assert_eq!(configs.len(), 1);

    let config = &configs[0];
    assert_eq!(config.id, "remote-server");
    assert!(!config.auto_connect);

    match &config.transport {
        misaka_x_lib::services::mcp::McpTransport::Http { url, headers } => {
            assert_eq!(url, "https://mcp.example.com/v1");
            assert_eq!(headers.get("Authorization").unwrap(), "Bearer sk-123");
        }
        _ => panic!("Expected Http transport"),
    }
}

#[test]
fn test_load_sse_server_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp.json");
    std::fs::write(
        &path,
        r#"{
            "mcpServers": {
                "sse-server": {
                    "url": "https://mcp.example.com/sse",
                    "transportType": "sse"
                }
            }
        }"#,
    )
    .unwrap();

    let configs = McpConfigLoader::load_from_file(&path).unwrap();
    assert_eq!(configs.len(), 1);

    match &configs[0].transport {
        misaka_x_lib::services::mcp::McpTransport::Sse { url, .. } => {
            assert_eq!(url, "https://mcp.example.com/sse");
        }
        _ => panic!("Expected Sse transport"),
    }
}

#[test]
fn test_load_multiple_servers() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp.json");
    std::fs::write(
        &path,
        r#"{
            "mcpServers": {
                "server-a": {"command": "echo", "args": ["hello"]},
                "server-b": {"url": "http://localhost:3001"},
                "server-c": {"url": "http://localhost:3002", "transportType": "sse"}
            }
        }"#,
    )
    .unwrap();

    let configs = McpConfigLoader::load_from_file(&path).unwrap();
    assert_eq!(configs.len(), 3);
}

#[test]
fn test_ensure_default_config_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let mcp_path = dir.path().join("mcp.json");

    assert!(!mcp_path.exists());
    McpConfigLoader::ensure_default_config(dir.path()).unwrap();
    assert!(mcp_path.exists());

    let content = std::fs::read_to_string(&mcp_path).unwrap();
    assert!(content.contains("mcpServers"));
}

#[test]
fn test_ensure_default_config_does_not_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let mcp_path = dir.path().join("mcp.json");

    std::fs::write(&mcp_path, r#"{"mcpServers":{"my-server":{"command":"test"}}}"#).unwrap();
    McpConfigLoader::ensure_default_config(dir.path()).unwrap();

    let content = std::fs::read_to_string(&mcp_path).unwrap();
    assert!(
        content.contains("my-server"),
        "Should not overwrite existing file"
    );
}

#[test]
fn test_load_from_db() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    misaka_x_lib::db::migrations::run_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO mcp_servers (id, name, transport_json, auto_connect, env_json)
         VALUES ('db-srv', 'DB Server', '{\"type\":\"stdio\",\"command\":\"echo\",\"args\":[]}', 1, NULL)",
        [],
    )
    .unwrap();

    let configs = McpConfigLoader::load_from_db(&conn).unwrap();
    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].id, "db-srv");
}

#[test]
fn test_load_all_file_takes_priority() {
    let dir = tempfile::tempdir().unwrap();
    let mcp_path = dir.path().join("mcp.json");

    std::fs::write(
        &mcp_path,
        r#"{"mcpServers":{"shared-id":{"command":"file-version","args":[]}}}"#,
    )
    .unwrap();

    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    misaka_x_lib::db::migrations::run_migrations(&conn).unwrap();
    conn.execute(
        "INSERT INTO mcp_servers (id, name, transport_json, auto_connect)
         VALUES ('shared-id', 'DB Version', '{\"type\":\"stdio\",\"command\":\"db-version\",\"args\":[]}', 1)",
        [],
    )
    .unwrap();

    let configs = McpConfigLoader::load_all(dir.path(), &conn).unwrap();
    assert_eq!(configs.len(), 1, "Duplicate ID should be merged");

    match &configs[0].transport {
        misaka_x_lib::services::mcp::McpTransport::Stdio { command, .. } => {
            assert_eq!(command, "file-version", "File config should take priority");
        }
        _ => panic!("Expected Stdio"),
    }
}
