use std::collections::HashMap;

use rmcp::model::Tool;
use rmcp::service::{RoleClient, RunningService};
use serde::{Deserialize, Serialize};

// ─── MCP Server 配置 ──────────────────────────────────────────────────

/// MCP Server 传输层配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpTransport {
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
    },
    Http {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
    Sse {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
}

/// 单个 MCP Server 的完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub transport: McpTransport,
    #[serde(default = "default_true")]
    pub auto_connect: bool,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

// ─── MCP Server 运行时状态 ────────────────────────────────────────────

/// MCP Server 连接状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum McpServerStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// 工具信息（前端展示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    pub server_id: String,
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

impl McpToolInfo {
    pub fn from_tool(server_id: &str, tool: &Tool) -> Self {
        let input_schema = serde_json::to_value(&tool.input_schema)
            .unwrap_or(serde_json::Value::Object(Default::default()));

        Self {
            server_id: server_id.to_string(),
            name: tool.name.to_string(),
            description: tool.description.clone().map(|s| s.to_string()),
            input_schema,
        }
    }
}

/// Server 概览信息（前端展示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub id: String,
    pub name: String,
    pub transport_type: String,
    pub status: McpServerStatus,
    pub tools_count: usize,
    pub auto_connect: bool,
}

// ─── Server 运行时句柄（内部） ────────────────────────────────────────

/// rmcp client 的类型别名
///
/// `RunningService<RoleClient, _>` 的第二个泛型 `S` 是 `Service<RoleClient>` 实现，
/// rmcp 默认客户端使用 `()` 作为无状态 service handler。
pub type McpClient = RunningService<RoleClient, ()>;

/// 服务端运行时句柄，存储在 DashMap 中
pub struct McpServerHandle {
    pub config: McpServerConfig,
    pub client: Option<McpClient>,
    pub tools: Vec<Tool>,
    pub status: McpServerStatus,
    pub retry_count: u32,
}

impl McpServerHandle {
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            client: None,
            tools: Vec::new(),
            status: McpServerStatus::Disconnected,
            retry_count: 0,
        }
    }

    pub fn to_info(&self) -> McpServerInfo {
        let transport_type = match &self.config.transport {
            McpTransport::Stdio { .. } => "stdio",
            McpTransport::Http { .. } => "http",
            McpTransport::Sse { .. } => "sse",
        };

        McpServerInfo {
            id: self.config.id.clone(),
            name: self.config.name.clone(),
            transport_type: transport_type.to_string(),
            status: self.status.clone(),
            tools_count: self.tools.len(),
            auto_connect: self.config.auto_connect,
        }
    }

    pub fn to_tool_infos(&self) -> Vec<McpToolInfo> {
        self.tools
            .iter()
            .map(|t| McpToolInfo::from_tool(&self.config.id, t))
            .collect()
    }
}
