//! Rig 对话内 MCP 工具执行循环
//!
//! 在一次 assistant 回复中驱动「流式回复 → 解析工具调用 → 审批 → 执行 → emit
//! → 回灌 history」的多轮循环，直到模型给出纯文本回答或达到最大轮数。
//!
//! 设计约束（Phase 3）：继续 Rig 直调；工具调用协议为 system prompt 中约定的
//! 单个 JSON 对象（见 `mcp_bridge::McpToolBridge::TOOL_CALL_PROTOCOL`）。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::db::models::{Message, Session};
use crate::services::llm::backend::MessageAttachment;
use crate::services::llm::{
    emit_tool_call, emit_tool_result, RigBackend, StreamCompletePayload, StreamResult,
    StreamToolCallPayload, StreamToolResultPayload, TokenUsageInfo,
};
use crate::services::mcp_bridge::McpToolBridge;
use crate::services::ToolCallRecord;

use super::approval::ensure_tool_allowed;
use super::manager::McpManager;

/// 对话内工具循环的最大轮数（含最终纯文本回答轮）
pub const MAX_TOOL_ROUNDS: usize = 5;

/// 从模型输出中解析出的工具调用请求
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedToolCall {
    pub name: String,
    pub arguments: Value,
}

/// 工具循环的最终产出
pub struct ToolLoopOutcome {
    pub result: StreamResult,
    pub tool_calls: Vec<ToolCallRecord>,
}

/// 单次工具执行的上下文（减少方法参数）
struct ToolCtx {
    tool_call_id: String,
    server_id: String,
    server_name: String,
    tool_name: String,
    arguments: Value,
    started_at: i64,
}

impl ToolCtx {
    fn new(call: &ParsedToolCall, server_id: String, manager: &McpManager) -> Self {
        let server_name = manager
            .server_info(&server_id)
            .map(|info| info.name)
            .unwrap_or_else(|| server_id.clone());
        Self {
            tool_call_id: uuid::Uuid::new_v4().to_string(),
            server_id,
            server_name,
            tool_name: call.name.clone(),
            arguments: call.arguments.clone(),
            started_at: now_ms(),
        }
    }

    fn orphan(call: &ParsedToolCall) -> Self {
        Self {
            tool_call_id: uuid::Uuid::new_v4().to_string(),
            server_id: String::new(),
            server_name: String::new(),
            tool_name: call.name.clone(),
            arguments: call.arguments.clone(),
            started_at: now_ms(),
        }
    }
}

/// MCP 对话工具循环执行器（借用 command 层的运行时依赖）
pub struct McpToolLoop<'a> {
    app: &'a AppHandle,
    db: &'a Mutex<Connection>,
    manager: &'a Arc<McpManager>,
    backend: &'a RigBackend,
    session: &'a Session,
    model_id: &'a str,
    abort_flag: Arc<AtomicBool>,
    message_id: &'a str,
    max_rounds: usize,
}

impl<'a> McpToolLoop<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app: &'a AppHandle,
        db: &'a Mutex<Connection>,
        manager: &'a Arc<McpManager>,
        backend: &'a RigBackend,
        session: &'a Session,
        model_id: &'a str,
        abort_flag: Arc<AtomicBool>,
        message_id: &'a str,
        max_rounds: usize,
    ) -> Self {
        Self {
            app,
            db,
            manager,
            backend,
            session,
            model_id,
            abort_flag,
            message_id,
            max_rounds,
        }
    }

    /// 执行工具循环，返回最终可见文本与工具调用记录
    pub async fn run(
        self,
        history: Vec<Message>,
        user_content: String,
        attachments: Option<Vec<MessageAttachment>>,
    ) -> Result<ToolLoopOutcome, String> {
        let bridge = McpToolBridge::new(Arc::clone(self.manager));
        let mut records: Vec<ToolCallRecord> = Vec::new();
        let mut working_history = history;
        let mut current_user = user_content;
        let mut current_attachments = attachments;

        let mut visible = String::new();
        let mut thinking = String::new();
        let mut usage: Option<TokenUsageInfo> = None;
        let mut was_aborted = false;

        for _round in 0..self.max_rounds {
            if self.abort_flag.load(Ordering::Relaxed) {
                was_aborted = true;
                break;
            }

            let result = self
                .backend
                .stream_round(
                    self.app,
                    self.session,
                    &working_history,
                    &current_user,
                    &current_attachments,
                    self.model_id,
                    Arc::clone(&self.abort_flag),
                    self.message_id,
                )
                .await
                .map_err(|e| e.to_string())?;

            was_aborted = result.was_aborted;
            if !result.thinking.is_empty() {
                thinking = result.thinking.clone();
            }
            usage = merge_token_usage(usage, result.usage.clone());

            let Some(call) = parse_tool_call_from_content(&result.content) else {
                visible = result.content;
                break;
            };

            visible = strip_tool_call_json(&result.content);
            if was_aborted {
                break;
            }

            working_history.push(assistant_history_message(&self.session.id, &result.content));

            let (record, feedback) = self.execute_one_tool(&bridge, call).await;
            records.push(record);
            current_user = feedback;
            current_attachments = None;

            if self.abort_flag.load(Ordering::Relaxed) {
                was_aborted = true;
                break;
            }
        }

        self.emit_complete(&visible, &thinking, usage.clone(), was_aborted);

        Ok(ToolLoopOutcome {
            result: StreamResult {
                content: visible,
                thinking,
                usage,
                was_aborted,
            },
            tool_calls: records,
        })
    }

    /// 执行单个工具调用：emit 事件 → 审批 → 调用 → 记录结果
    async fn execute_one_tool(
        &self,
        bridge: &McpToolBridge,
        call: ParsedToolCall,
    ) -> (ToolCallRecord, String) {
        let Some(server_id) = self.manager.find_server_for_tool(&call.name) else {
            let ctx = ToolCtx::orphan(&call);
            let err = format!("No connected MCP server provides tool '{}'", call.name);
            self.emit_running(&ctx);
            let record = self.settle(&ctx, "error", None, Some(err));
            return (
                record,
                tool_feedback(
                    &call.name,
                    "Error: no connected MCP server provides this tool.",
                ),
            );
        };

        let ctx = ToolCtx::new(&call, server_id, self.manager);
        self.emit_running(&ctx);

        if let Some(denied) = self.check_approval(&ctx).await {
            return denied;
        }

        match bridge
            .call_tool(&ctx.tool_name, ctx.arguments.clone())
            .await
        {
            Ok(res) => {
                let formatted = McpToolBridge::format_tool_result(&res);
                let result_json = serde_json::to_value(&res).ok();
                let record = self.settle(&ctx, "complete", result_json, None);
                (record, tool_feedback(&ctx.tool_name, &formatted))
            }
            Err(e) => {
                let msg = e.to_string();
                let record = self.settle(&ctx, "error", None, Some(msg.clone()));
                (
                    record,
                    tool_feedback(&ctx.tool_name, &format!("Error: {msg}")),
                )
            }
        }
    }

    /// 审批判定；被拒/出错时返回已 settle 的记录与回灌文本，允许时返回 None
    async fn check_approval(&self, ctx: &ToolCtx) -> Option<(ToolCallRecord, String)> {
        match ensure_tool_allowed(
            self.app,
            self.db,
            self.manager,
            &ctx.server_id,
            &ctx.tool_name,
            &ctx.arguments,
        )
        .await
        {
            Ok(true) => None,
            Ok(false) => {
                let record = self.settle(ctx, "error", None, Some("Tool call denied".to_string()));
                Some((
                    record,
                    tool_feedback(&ctx.tool_name, "Error: the user denied this tool call."),
                ))
            }
            Err(e) => {
                let record = self.settle(ctx, "error", None, Some(e.clone()));
                Some((
                    record,
                    tool_feedback(&ctx.tool_name, &format!("Error: {e}")),
                ))
            }
        }
    }

    fn emit_running(&self, ctx: &ToolCtx) {
        emit_tool_call(
            self.app,
            &StreamToolCallPayload {
                session_id: self.session.id.clone(),
                message_id: self.message_id.to_string(),
                tool_call_id: ctx.tool_call_id.clone(),
                server_id: ctx.server_id.clone(),
                server_name: ctx.server_name.clone(),
                tool_name: ctx.tool_name.clone(),
                arguments: ctx.arguments.clone(),
                status: "running".to_string(),
            },
        );
    }

    /// 构建记录并 emit `stream:tool_result`
    fn settle(
        &self,
        ctx: &ToolCtx,
        status: &str,
        result: Option<Value>,
        error: Option<String>,
    ) -> ToolCallRecord {
        emit_tool_result(
            self.app,
            &StreamToolResultPayload {
                session_id: self.session.id.clone(),
                message_id: self.message_id.to_string(),
                tool_call_id: ctx.tool_call_id.clone(),
                result: result.clone(),
                error: error.clone(),
                status: status.to_string(),
            },
        );

        ToolCallRecord {
            id: ctx.tool_call_id.clone(),
            server_id: ctx.server_id.clone(),
            server_name: ctx.server_name.clone(),
            tool_name: ctx.tool_name.clone(),
            arguments: ctx.arguments.clone(),
            result,
            status: status.to_string(),
            error,
            started_at: Some(ctx.started_at),
            completed_at: Some(now_ms()),
        }
    }

    /// 循环结束后统一 emit 一次 `stream_complete`
    fn emit_complete(
        &self,
        content: &str,
        thinking: &str,
        usage: Option<TokenUsageInfo>,
        was_aborted: bool,
    ) {
        let payload = StreamCompletePayload {
            session_id: self.session.id.clone(),
            message_id: self.message_id.to_string(),
            full_content: content.to_string(),
            full_thinking: thinking.to_string(),
            usage,
            was_aborted,
        };
        if let Err(e) = self.app.emit("stream_complete", &payload) {
            tracing::warn!(error = %e, "Failed to emit stream_complete from tool loop");
        }
    }
}

// ─── 纯函数：工具调用解析 ─────────────────────────────────────────────

/// 从模型输出中解析工具调用
///
/// 依次尝试：最后一个 fenced code block → 全文整段 JSON → 首个 `{` 到末个 `}` 子串。
/// 任一候选反序列化为含 `name`/`tool_name`/`tool` 的对象即视为工具调用。
pub fn parse_tool_call_from_content(content: &str) -> Option<ParsedToolCall> {
    json_candidates(content)
        .into_iter()
        .filter_map(|candidate| serde_json::from_str::<Value>(&candidate).ok())
        .find_map(|value| build_parsed_call(&value))
}

/// 去掉工具调用 JSON 后的可见文本
///
/// 工具调用轮通常整段都是 JSON（可选 fenced），返回空串；最终答复轮无工具 JSON，原样返回。
pub fn strip_tool_call_json(content: &str) -> String {
    if let Some(block) = extract_last_fenced_block(content) {
        if is_tool_call_json(&block) {
            return remove_last_fenced_region(content).trim().to_string();
        }
    }

    let trimmed = content.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') && is_tool_call_json(trimmed) {
        return String::new();
    }

    content.trim().to_string()
}

fn is_tool_call_json(candidate: &str) -> bool {
    serde_json::from_str::<Value>(candidate)
        .ok()
        .and_then(|value| build_parsed_call(&value))
        .is_some()
}

fn build_parsed_call(value: &Value) -> Option<ParsedToolCall> {
    let obj = value.as_object()?;
    let name = obj
        .get("name")
        .or_else(|| obj.get("tool_name"))
        .or_else(|| obj.get("tool"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())?
        .to_string();

    let arguments = obj
        .get("arguments")
        .or_else(|| obj.get("args"))
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

    Some(ParsedToolCall { name, arguments })
}

fn json_candidates(content: &str) -> Vec<String> {
    let mut candidates = Vec::new();

    if let Some(block) = extract_last_fenced_block(content) {
        candidates.push(block);
    }

    let trimmed = content.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        candidates.push(trimmed.to_string());
    }

    if let (Some(start), Some(end)) = (content.find('{'), content.rfind('}')) {
        if end > start {
            candidates.push(content[start..=end].to_string());
        }
    }

    candidates
}

/// 提取最后一个 ``` fenced 代码块的内部内容（去掉可选语言标签行）
fn extract_last_fenced_block(content: &str) -> Option<String> {
    let fences: Vec<usize> = content.match_indices("```").map(|(i, _)| i).collect();
    if fences.len() < 2 {
        return None;
    }
    let open = fences[fences.len() - 2] + 3;
    let close = fences[fences.len() - 1];
    if open > close {
        return None;
    }
    Some(
        strip_fence_language(&content[open..close])
            .trim()
            .to_string(),
    )
}

/// 移除最后一个 fenced 代码块（含围栏），保留前后正文
fn remove_last_fenced_region(content: &str) -> String {
    let fences: Vec<usize> = content.match_indices("```").map(|(i, _)| i).collect();
    if fences.len() < 2 {
        return content.to_string();
    }
    let open = fences[fences.len() - 2];
    let close = fences[fences.len() - 1] + 3;

    let mut result = String::from(&content[..open]);
    if close < content.len() {
        result.push_str(&content[close..]);
    }
    result
}

/// 去掉 fenced block 首行的语言标签（如 `json`）
fn strip_fence_language(raw: &str) -> &str {
    match raw.split_once('\n') {
        Some((first, rest)) => {
            let token = first.trim();
            if !token.is_empty() && token.chars().all(|c| c.is_ascii_alphanumeric()) {
                rest
            } else {
                raw
            }
        }
        None => raw,
    }
}

// ─── 内部辅助 ─────────────────────────────────────────────────────────

fn tool_feedback(tool_name: &str, body: &str) -> String {
    format!("[Tool Result for {tool_name}]\n{body}")
}

fn assistant_history_message(session_id: &str, content: &str) -> Message {
    Message {
        id: String::new(),
        session_id: session_id.to_string(),
        role: "assistant".to_string(),
        content: content.to_string(),
        token_usage: None,
        model: None,
        thinking_content: None,
        attachments: None,
        status: "complete".to_string(),
        tool_calls: None,
        blocks: Vec::new(),
        created_at: String::new(),
    }
}

pub fn merge_token_usage(
    acc: Option<TokenUsageInfo>,
    next: Option<TokenUsageInfo>,
) -> Option<TokenUsageInfo> {
    match (acc, next) {
        (None, next) => next,
        (acc, None) => acc,
        (Some(a), Some(b)) => Some(TokenUsageInfo {
            input_tokens: a.input_tokens + b.input_tokens,
            output_tokens: a.output_tokens + b.output_tokens,
            total_tokens: a.total_tokens + b.total_tokens,
        }),
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
