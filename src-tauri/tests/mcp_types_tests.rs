use misaka_x_lib::services::mcp::types::{
    McpServerConfig, McpServerHandle, McpServerStatus, McpToolInfo, McpTransport,
};
use std::collections::HashMap;

// ─── McpTransport 序列化/反序列化 ─────────────────────────────────────

#[test]
fn test_stdio_transport_serde() {
    let transport = McpTransport::Stdio {
        command: "npx".to_string(),
        args: vec![
            "-y".to_string(),
            "@modelcontextprotocol/server-filesystem".to_string(),
        ],
    };

    let json = serde_json::to_string(&transport).unwrap();
    let parsed: McpTransport = serde_json::from_str(&json).unwrap();

    match parsed {
        McpTransport::Stdio { command, args } => {
            assert_eq!(command, "npx");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Stdio transport"),
    }
}

#[test]
fn test_http_transport_serde() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());

    let transport = McpTransport::Http {
        url: "http://localhost:3000/mcp".to_string(),
        headers,
    };

    let json = serde_json::to_string(&transport).unwrap();
    let parsed: McpTransport = serde_json::from_str(&json).unwrap();

    match parsed {
        McpTransport::Http { url, headers } => {
            assert_eq!(url, "http://localhost:3000/mcp");
            assert_eq!(headers.get("Authorization").unwrap(), "Bearer token123");
        }
        _ => panic!("Expected Http transport"),
    }
}

#[test]
fn test_sse_transport_serde() {
    let transport = McpTransport::Sse {
        url: "http://localhost:3000/sse".to_string(),
        headers: HashMap::new(),
    };

    let json = serde_json::to_string(&transport).unwrap();
    assert!(json.contains("\"type\":\"sse\""));
}

// ─── McpServerConfig ──────────────────────────────────────────────────

#[test]
fn test_server_config_defaults() {
    let json = r#"{
        "id": "test-server",
        "name": "Test Server",
        "transport": {
            "type": "stdio",
            "command": "echo",
            "args": []
        }
    }"#;

    let config: McpServerConfig = serde_json::from_str(json).unwrap();
    assert!(config.auto_connect, "auto_connect should default to true");
    assert!(config.env.is_empty());
}

#[test]
fn test_server_config_full() {
    let config = McpServerConfig {
        id: "fs-server".to_string(),
        name: "Filesystem".to_string(),
        transport: McpTransport::Stdio {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "server-filesystem".to_string()],
        },
        auto_connect: false,
        env: {
            let mut m = HashMap::new();
            m.insert("HOME".to_string(), "/tmp".to_string());
            m
        },
    };

    let json = serde_json::to_string_pretty(&config).unwrap();
    let parsed: McpServerConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.id, "fs-server");
    assert!(!parsed.auto_connect);
    assert_eq!(parsed.env.get("HOME").unwrap(), "/tmp");
}

// ─── McpServerStatus ──────────────────────────────────────────────────

#[test]
fn test_status_serde() {
    let statuses = vec![
        (McpServerStatus::Disconnected, "\"disconnected\""),
        (McpServerStatus::Connecting, "\"connecting\""),
        (McpServerStatus::Connected, "\"connected\""),
    ];

    for (status, expected) in statuses {
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, expected);
    }

    let err = McpServerStatus::Error("timeout".to_string());
    let json = serde_json::to_string(&err).unwrap();
    assert!(json.contains("timeout"));
}

#[test]
fn test_status_equality() {
    assert_eq!(McpServerStatus::Connected, McpServerStatus::Connected);
    assert_ne!(McpServerStatus::Connected, McpServerStatus::Disconnected);
    assert_ne!(
        McpServerStatus::Error("a".to_string()),
        McpServerStatus::Error("b".to_string())
    );
}

// ─── McpServerHandle ──────────────────────────────────────────────────

#[test]
fn test_handle_initial_state() {
    let config = McpServerConfig {
        id: "test".to_string(),
        name: "Test".to_string(),
        transport: McpTransport::Stdio {
            command: "echo".to_string(),
            args: vec![],
        },
        auto_connect: true,
        env: HashMap::new(),
    };

    let handle = McpServerHandle::new(config);
    assert!(handle.client.is_none());
    assert!(handle.tools.is_empty());
    assert_eq!(handle.status, McpServerStatus::Disconnected);
    assert_eq!(handle.retry_count, 0);
}

#[test]
fn test_handle_to_info() {
    let config = McpServerConfig {
        id: "http-test".to_string(),
        name: "HTTP Test".to_string(),
        transport: McpTransport::Http {
            url: "http://localhost:3000".to_string(),
            headers: HashMap::new(),
        },
        auto_connect: false,
        env: HashMap::new(),
    };

    let handle = McpServerHandle::new(config);
    let info = handle.to_info();

    assert_eq!(info.id, "http-test");
    assert_eq!(info.name, "HTTP Test");
    assert_eq!(info.transport_type, "http");
    assert_eq!(info.tools_count, 0);
    assert!(!info.auto_connect);
}

#[test]
fn test_handle_to_info_sse_type() {
    let config = McpServerConfig {
        id: "sse-test".to_string(),
        name: "SSE Test".to_string(),
        transport: McpTransport::Sse {
            url: "http://localhost:3000/sse".to_string(),
            headers: HashMap::new(),
        },
        auto_connect: true,
        env: HashMap::new(),
    };

    let handle = McpServerHandle::new(config);
    let info = handle.to_info();
    assert_eq!(info.transport_type, "sse");
}

// ─── McpToolInfo ──────────────────────────────────────────────────────

#[test]
fn test_tool_info_serialization() {
    let tool = McpToolInfo {
        server_id: "srv-1".to_string(),
        name: "read_file".to_string(),
        description: Some("Read a file from disk".to_string()),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" }
            },
            "required": ["path"]
        }),
    };

    let json = serde_json::to_string(&tool).unwrap();
    let parsed: McpToolInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.server_id, "srv-1");
    assert_eq!(parsed.name, "read_file");
    assert!(parsed.description.is_some());
}
