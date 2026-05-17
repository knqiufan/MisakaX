use std::collections::HashMap;

use anyhow::{Context, Result};
use dashmap::DashMap;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::service::ServiceExt;
use rmcp::transport::child_process::TokioChildProcess;
use rmcp::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
};

use super::types::{
    McpClient, McpServerConfig, McpServerHandle, McpServerInfo, McpServerStatus, McpToolInfo,
    McpTransport,
};

/// MCP Server 管理器
///
/// 管理所有 MCP Server 连接的生命周期：连接、断开、工具列表、工具调用。
/// 内部使用 `DashMap` 实现无锁并发安全。
pub struct McpManager {
    servers: DashMap<String, McpServerHandle>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            servers: DashMap::new(),
        }
    }

    /// 连接到一个 MCP Server
    pub async fn connect(&self, config: McpServerConfig) -> Result<()> {
        let server_id = config.id.clone();

        self.servers
            .insert(server_id.clone(), McpServerHandle::new(config.clone()));

        self.update_status(&server_id, McpServerStatus::Connecting);

        match self.do_connect(&config).await {
            Ok((client, tools)) => {
                if let Some(mut handle) = self.servers.get_mut(&server_id) {
                    handle.client = Some(client);
                    handle.tools = tools;
                    handle.status = McpServerStatus::Connected;
                    handle.retry_count = 0;

                    tracing::info!(
                        server_id = %server_id,
                        tools_count = handle.tools.len(),
                        "MCP Server connected"
                    );
                }
                Ok(())
            }
            Err(e) => {
                let err_msg = e.to_string();
                self.update_status(&server_id, McpServerStatus::Error(err_msg.clone()));
                tracing::error!(server_id = %server_id, error = %err_msg, "MCP connect failed");
                Err(e)
            }
        }
    }

    /// 断开指定 Server 的连接
    pub async fn disconnect(&self, server_id: &str) -> Result<()> {
        if let Some(mut handle) = self.servers.get_mut(server_id) {
            if let Some(mut client) = handle.client.take() {
                if let Err(e) = client.close().await {
                    tracing::warn!(
                        server_id = %server_id,
                        error = ?e,
                        "Error closing MCP client"
                    );
                }
            }
            handle.tools.clear();
            handle.status = McpServerStatus::Disconnected;
            tracing::info!(server_id = %server_id, "MCP Server disconnected");
        }
        Ok(())
    }

    /// 重新连接指定 Server
    pub async fn reconnect(&self, server_id: &str) -> Result<()> {
        let config = self
            .servers
            .get(server_id)
            .map(|h| h.config.clone())
            .context("Server not found")?;

        self.disconnect(server_id).await?;
        self.connect(config).await
    }

    /// 调用指定 Server 的工具
    pub async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult> {
        let peer = {
            let handle = self
                .servers
                .get(server_id)
                .context("MCP Server not found")?;

            if handle.status != McpServerStatus::Connected {
                anyhow::bail!(
                    "MCP Server '{}' is not connected (status: {:?})",
                    server_id,
                    handle.status
                );
            }

            let client = handle.client.as_ref().context("Client not initialized")?;
            client.peer().clone()
        };

        let arguments = match arguments {
            serde_json::Value::Object(map) => map,
            serde_json::Value::Null => serde_json::Map::new(),
            other => {
                let mut map = serde_json::Map::new();
                map.insert("input".to_string(), other);
                map
            }
        };

        let params = CallToolRequestParams::new(tool_name.to_string()).with_arguments(arguments);

        let result = peer
            .call_tool(params)
            .await
            .map_err(|e| anyhow::anyhow!("MCP tool call failed: {e}"))?;

        tracing::info!(
            server_id = %server_id,
            tool = %tool_name,
            is_error = result.is_error.unwrap_or(false),
            "MCP tool call completed"
        );

        Ok(result)
    }

    /// 获取所有已连接 Server 的工具列表
    pub fn list_all_tools(&self) -> Vec<McpToolInfo> {
        self.servers
            .iter()
            .filter(|entry| entry.status == McpServerStatus::Connected)
            .flat_map(|entry| entry.to_tool_infos())
            .collect()
    }

    /// 获取指定 Server 的信息
    pub fn server_info(&self, server_id: &str) -> Option<McpServerInfo> {
        self.servers.get(server_id).map(|h| h.to_info())
    }

    /// 列出所有 Server
    pub fn list_servers(&self) -> Vec<McpServerInfo> {
        self.servers.iter().map(|entry| entry.to_info()).collect()
    }

    /// 移除一个 Server 配置（先断开再移除）
    pub async fn remove_server(&self, server_id: &str) -> Result<()> {
        self.disconnect(server_id).await?;
        self.servers.remove(server_id);
        Ok(())
    }

    /// 添加 Server 配置（不立即连接）
    pub fn add_server_config(&self, config: McpServerConfig) {
        let handle = McpServerHandle::new(config);
        self.servers.insert(handle.config.id.clone(), handle);
    }

    /// 检查指定 Server 是否存活
    pub fn is_connected(&self, server_id: &str) -> bool {
        self.servers
            .get(server_id)
            .is_some_and(|h| h.status == McpServerStatus::Connected)
    }

    /// 通过工具名查找所属的 Server ID
    pub fn find_server_for_tool(&self, tool_name: &str) -> Option<String> {
        for entry in self.servers.iter() {
            if entry.status == McpServerStatus::Connected
                && entry.tools.iter().any(|t| t.name.as_ref() == tool_name)
            {
                return Some(entry.config.id.clone());
            }
        }
        None
    }

    /// 获取连接的 Server 数量
    pub fn connected_count(&self) -> usize {
        self.servers
            .iter()
            .filter(|e| e.status == McpServerStatus::Connected)
            .count()
    }

    /// 获取指定 Server 的重试次数
    pub fn retry_count(&self, server_id: &str) -> u32 {
        self.servers.get(server_id).map_or(0, |h| h.retry_count)
    }

    /// 递增重试计数
    pub fn increment_retry(&self, server_id: &str) {
        if let Some(mut handle) = self.servers.get_mut(server_id) {
            handle.retry_count += 1;
        }
    }

    /// 重置重试计数
    pub fn reset_retry(&self, server_id: &str) {
        if let Some(mut handle) = self.servers.get_mut(server_id) {
            handle.retry_count = 0;
        }
    }

    /// 检查 Server 连接的健康状态
    ///
    /// 通过调用 `list_tools` 验证连接活跃性。
    pub async fn health_check(&self, server_id: &str) -> bool {
        let peer = {
            let handle = match self.servers.get(server_id) {
                Some(h) if h.status == McpServerStatus::Connected => h,
                _ => return false,
            };
            match handle.client.as_ref() {
                Some(c) => c.peer().clone(),
                None => return false,
            }
        };

        match peer.list_all_tools().await {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!(server_id = %server_id, error = ?e, "MCP health check failed");
                false
            }
        }
    }
}

// ─── 私有方法 ─────────────────────────────────────────────────────────

impl McpManager {
    /// 根据 transport 类型执行实际连接
    async fn do_connect(
        &self,
        config: &McpServerConfig,
    ) -> Result<(McpClient, Vec<rmcp::model::Tool>)> {
        match &config.transport {
            McpTransport::Stdio { command, args } => {
                self.connect_stdio(command, args, &config.env).await
            }
            McpTransport::Http { url, headers } => self.connect_http(url, headers).await,
            McpTransport::Sse { url, headers } => self.connect_http(url, headers).await,
        }
    }

    /// Stdio transport：通过子进程连接
    async fn connect_stdio(
        &self,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<(McpClient, Vec<rmcp::model::Tool>)> {
        let mut cmd = tokio::process::Command::new(command);
        cmd.args(args);

        for (key, value) in env {
            cmd.env(key, value);
        }

        let transport =
            TokioChildProcess::new(cmd).context("Failed to spawn MCP server process")?;

        let client = ()
            .serve(transport)
            .await
            .map_err(|e| anyhow::anyhow!("MCP client initialization failed: {e}"))?;

        let tools = client
            .peer()
            .list_all_tools()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list MCP tools: {e}"))?;

        Ok((client, tools))
    }

    /// HTTP/SSE transport：通过 Streamable HTTP 连接
    async fn connect_http(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<(McpClient, Vec<rmcp::model::Tool>)> {
        let mut transport_config = StreamableHttpClientTransportConfig::with_uri(url);

        if !headers.is_empty() {
            let mut header_map = std::collections::HashMap::new();
            for (key, value) in headers {
                let name =
                    http::HeaderName::from_bytes(key.as_bytes()).context("Invalid header name")?;
                let val = http::HeaderValue::from_str(value).context("Invalid header value")?;
                header_map.insert(name, val);
            }
            transport_config = transport_config.custom_headers(header_map);
        }

        let transport = StreamableHttpClientTransport::from_config(transport_config);

        let client = ()
            .serve(transport)
            .await
            .map_err(|e| anyhow::anyhow!("MCP HTTP client initialization failed: {e}"))?;

        let tools = client
            .peer()
            .list_all_tools()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list MCP tools: {e}"))?;

        Ok((client, tools))
    }

    fn update_status(&self, server_id: &str, status: McpServerStatus) {
        if let Some(mut handle) = self.servers.get_mut(server_id) {
            handle.status = status;
        }
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}
