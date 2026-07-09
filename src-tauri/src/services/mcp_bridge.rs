use std::sync::Arc;

use anyhow::Result;
use rmcp::model::CallToolResult;

use super::mcp::McpManager;

/// MCP 工具调用桥接器
///
/// 在 Rig 对话流中调用 MCP 工具的中间层。
/// 当 LLM 返回的响应中包含工具调用请求时，
/// 通过此模块将请求路由到对应的 MCP Server 并获取结果。
pub struct McpToolBridge {
    manager: Arc<McpManager>,
}

impl McpToolBridge {
    /// 注入到 system prompt 末尾的工具调用协议说明
    ///
    /// 与 `services/mcp/tool_loop.rs` 的解析器约定一致：模型需以**单个 JSON 对象**
    /// 发起一次工具调用（可选包裹在 ```json fenced block 中）。
    const TOOL_CALL_PROTOCOL: &'static str = "\
## Tool Call Protocol\n\
When you need to call a tool, respond with ONLY a single JSON object (optionally wrapped in a ```json code fence), and nothing else:\n\
```json\n\
{\"name\": \"<tool_name>\", \"arguments\": { /* schema-conforming args */ }}\n\
```\n\
Rules:\n\
- Call at most ONE tool per response.\n\
- Emit no prose before or after the JSON when calling a tool.\n\
- After you receive a message starting with \"[Tool Result for ...]\", use it to continue; either call another tool or give the final answer as plain text.\n\
- If no tool is needed, just answer normally in plain text.\n";

    pub fn new(manager: Arc<McpManager>) -> Self {
        Self { manager }
    }

    /// 收集所有已连接 MCP Server 的工具描述，用于注入系统 prompt
    ///
    /// 返回格式化的工具列表描述，LLM 可据此决定调用哪个工具。
    pub fn tool_descriptions(&self) -> Option<String> {
        let tools = self.manager.list_all_tools();
        if tools.is_empty() {
            return None;
        }

        let mut desc = String::from(
            "\n\n## Available Tools\n\
             You can call the following tools to help answer the user.\n\n",
        );

        for tool in &tools {
            desc.push_str(&format!("### {}\n", tool.name));
            if let Some(ref d) = tool.description {
                desc.push_str(&format!("{}\n", d));
            }
            desc.push_str(&format!(
                "Server: {}\nInput Schema: {}\n\n",
                tool.server_id,
                serde_json::to_string_pretty(&tool.input_schema)
                    .unwrap_or_else(|_| "{}".to_string())
            ));
        }

        desc.push_str(Self::TOOL_CALL_PROTOCOL);

        Some(desc)
    }

    /// 执行 MCP 工具调用
    ///
    /// 自动根据工具名查找对应 Server 并调用。
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult> {
        let server_id = self
            .manager
            .find_server_for_tool(tool_name)
            .ok_or_else(|| anyhow::anyhow!("No MCP Server found for tool '{}'", tool_name))?;

        self.manager
            .call_tool(&server_id, tool_name, arguments)
            .await
    }

    /// 提取工具调用结果为文本
    pub fn format_tool_result(result: &CallToolResult) -> String {
        let mut output = String::new();
        let is_error = result.is_error.unwrap_or(false);

        if is_error {
            output.push_str("[Tool Error] ");
        }

        for content in &result.content {
            match content.raw {
                rmcp::model::RawContent::Text(ref text) => {
                    output.push_str(&text.text);
                }
                rmcp::model::RawContent::Image(_) => {
                    output.push_str("[Image content]");
                }
                rmcp::model::RawContent::Audio(_) => {
                    output.push_str("[Audio content]");
                }
                rmcp::model::RawContent::Resource(_) => {
                    output.push_str("[Resource content]");
                }
                _ => {
                    output.push_str("[Unsupported content]");
                }
            }
        }

        output
    }

    /// 检查是否有可用的 MCP 工具
    pub fn has_tools(&self) -> bool {
        !self.manager.list_all_tools().is_empty()
    }
}
