//! Parse Python Sidecar SSE (`/agent/stream`) and map to existing Tauri stream events.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::StreamExt;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

use crate::services::llm::{
    emit_tool_call, emit_tool_result, StreamCompletePayload, StreamErrorPayload, StreamResult,
    StreamThinkingPayload, StreamTokenPayload, StreamToolCallPayload, StreamToolResultPayload,
    TokenUsageInfo,
};
use crate::services::ToolCallRecord;

const SIDECAR_SERVER_ID: &str = "sidecar";
const SIDECAR_SERVER_NAME: &str = "Sidecar";

#[derive(Debug, Clone, PartialEq)]
pub struct SseFrame {
    pub event: String,
    pub data: Value,
}

#[derive(Debug)]
pub enum MappedSidecarEvent {
    Token { delta: String },
    Thinking { delta: String },
    ToolCall(StreamToolCallPayload),
    ToolResult(StreamToolResultPayload),
    Done,
}

/// Outcome of a Sidecar SSE stream, including persistable tool records.
#[derive(Debug)]
pub struct SidecarStreamOutcome {
    pub result: StreamResult,
    pub tool_calls: Vec<ToolCallRecord>,
}

#[derive(Debug, Default)]
pub struct SidecarStreamAccumulator {
    content: String,
    thinking: String,
    usage: Option<TokenUsageInfo>,
    /// Open tool records keyed by stable SSE tool id (LangChain run_id).
    open_tools: HashMap<String, usize>,
    tool_calls: Vec<ToolCallRecord>,
}

impl SidecarStreamAccumulator {
    pub fn push_content(&mut self, delta: &str) {
        self.content.push_str(delta);
    }

    pub fn push_thinking(&mut self, delta: &str) {
        self.thinking.push_str(delta);
    }

    pub fn thinking_so_far(&self) -> &str {
        &self.thinking
    }

    /// Force-close still-running tools and return payloads that must be emitted
    /// to the UI before stream_complete / stream_error.
    pub fn settle_open_tools(&mut self, was_aborted: bool) -> Vec<StreamToolResultPayload> {
        let now = now_millis();
        let mut payloads = Vec::new();
        let status = if was_aborted { "aborted" } else { "error" };
        let default_error = if was_aborted {
            "Stream aborted before tool completed"
        } else {
            "Tool ended without a matching tool_end event"
        };

        for record in &mut self.tool_calls {
            if record.status != "running" {
                continue;
            }
            record.status = status.to_string();
            if record.error.is_none() {
                record.error = Some(default_error.to_string());
            }
            record.completed_at = Some(now);
            self.open_tools.remove(&record.id);
            payloads.push(StreamToolResultPayload {
                session_id: String::new(),
                message_id: String::new(),
                tool_call_id: record.id.clone(),
                result: None,
                error: record.error.clone(),
                status: status.to_string(),
            });
        }
        payloads
    }

    pub fn into_outcome(mut self, was_aborted: bool) -> SidecarStreamOutcome {
        let _ = self.settle_open_tools(was_aborted);
        SidecarStreamOutcome {
            result: StreamResult {
                content: self.content,
                thinking: self.thinking,
                usage: self.usage,
                was_aborted,
            },
            tool_calls: self.tool_calls,
        }
    }

    fn register_tool_start(
        &mut self,
        tool_id: &str,
        server_id: &str,
        server_name: &str,
        tool_name: &str,
        arguments: Value,
    ) -> String {
        let id = if tool_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            tool_id.to_string()
        };

        // Idempotent: duplicate tool_start with the same id must not create a
        // second UI/DB row (nested LangGraph echoes / Strict Mode double emit).
        if let Some(&index) = self.open_tools.get(&id) {
            let record = &mut self.tool_calls[index];
            record.server_id = server_id.to_string();
            record.server_name = server_name.to_string();
            record.tool_name = tool_name.to_string();
            record.arguments = arguments;
            record.status = "running".to_string();
            record.error = None;
            record.completed_at = None;
            return id;
        }
        if let Some(index) = self.tool_calls.iter().position(|r| r.id == id) {
            let record = &mut self.tool_calls[index];
            record.server_id = server_id.to_string();
            record.server_name = server_name.to_string();
            record.tool_name = tool_name.to_string();
            record.arguments = arguments;
            record.status = "running".to_string();
            record.error = None;
            record.completed_at = None;
            self.open_tools.insert(id.clone(), index);
            return id;
        }

        let record = ToolCallRecord {
            id: id.clone(),
            server_id: server_id.to_string(),
            server_name: server_name.to_string(),
            tool_name: tool_name.to_string(),
            arguments,
            result: None,
            status: "running".to_string(),
            error: None,
            started_at: Some(now_millis()),
            completed_at: None,
        };
        let index = self.tool_calls.len();
        self.tool_calls.push(record);
        self.open_tools.insert(id.clone(), index);
        id
    }

    fn resolve_tool_end(
        &mut self,
        tool_id: &str,
        tool_name: &str,
        result: Option<Value>,
        error: Option<String>,
        status: &str,
    ) -> String {
        let index = self
            .open_tools
            .remove(tool_id)
            .or_else(|| self.find_open_by_name(tool_name));

        if let Some(idx) = index {
            let record = &mut self.tool_calls[idx];
            record.result = result;
            record.error = error;
            record.status = status.to_string();
            record.completed_at = Some(now_millis());
            return record.id.clone();
        }

        let id = if tool_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            tool_id.to_string()
        };
        tracing::warn!(
            tool = %tool_name,
            id = %id,
            "tool_end without matching tool_start; recording fallback"
        );
        self.tool_calls.push(ToolCallRecord {
            id: id.clone(),
            server_id: SIDECAR_SERVER_ID.to_string(),
            server_name: SIDECAR_SERVER_NAME.to_string(),
            tool_name: tool_name.to_string(),
            arguments: json!({}),
            result,
            status: status.to_string(),
            error,
            started_at: Some(now_millis()),
            completed_at: Some(now_millis()),
        });
        id
    }

    fn find_open_by_name(&self, tool_name: &str) -> Option<usize> {
        self.tool_calls
            .iter()
            .enumerate()
            .rev()
            .find(|(_, r)| r.status == "running" && r.tool_name == tool_name)
            .map(|(i, _)| i)
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Parse a complete SSE document (or buffer containing complete frames).
pub fn parse_sse_frames(bytes: &[u8]) -> Result<Vec<SseFrame>, String> {
    let text = String::from_utf8_lossy(bytes);
    let mut frames = Vec::new();
    for block in text.split("\n\n") {
        if let Some(frame) = parse_sse_block(block)? {
            frames.push(frame);
        }
    }
    Ok(frames)
}

/// Parse a single SSE block (without trailing blank line).
pub fn parse_sse_block(block: &str) -> Result<Option<SseFrame>, String> {
    let trimmed = block.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let mut event = String::new();
    let mut data_lines: Vec<&str> = Vec::new();

    for line in trimmed.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start());
        }
    }

    if event.is_empty() {
        return Ok(None);
    }

    let data_raw = data_lines.join("\n");
    let data = if data_raw.is_empty() {
        Value::Object(Default::default())
    } else {
        serde_json::from_str(&data_raw).map_err(|e| format!("Invalid SSE JSON: {e}"))?
    };

    Ok(Some(SseFrame { event, data }))
}

/// Map one Python SSE event into a Tauri-facing event (or Done / ignore).
pub fn map_sidecar_event(
    event: &str,
    data: &Value,
    session_id: &str,
    message_id: &str,
    acc: &mut SidecarStreamAccumulator,
) -> Result<Option<MappedSidecarEvent>, String> {
    match event {
        "token" => {
            let delta = extract_token_content(data)?;
            if delta.is_empty() {
                return Ok(None);
            }
            acc.push_content(&delta);
            Ok(Some(MappedSidecarEvent::Token { delta }))
        }
        "thinking" => {
            let delta = extract_token_content(data)?;
            if delta.is_empty() {
                return Ok(None);
            }
            // Providers sometimes re-send cumulative reasoning; only append
            // the new suffix so stream_thinking and persisted thinking stay
            // free of duplicated paragraphs.
            let suffix = thinking_suffix_delta(acc.thinking_so_far(), &delta);
            if suffix.is_empty() {
                return Ok(None);
            }
            acc.push_thinking(&suffix);
            Ok(Some(MappedSidecarEvent::Thinking { delta: suffix }))
        }
        "tool_start" => Ok(Some(map_tool_start(data, session_id, message_id, acc))),
        "tool_end" => Ok(Some(map_tool_end(data, session_id, message_id, acc))),
        "done" => Ok(Some(MappedSidecarEvent::Done)),
        "error" => {
            let message = data
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Sidecar stream error")
                .to_string();
            Err(message)
        }
        "thinking_start" => Ok(None),
        _ => Ok(None),
    }
}

fn map_tool_start(
    data: &Value,
    session_id: &str,
    message_id: &str,
    acc: &mut SidecarStreamAccumulator,
) -> MappedSidecarEvent {
    let name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let tool_id = data
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let server_id = data
        .get("server_id")
        .and_then(|v| v.as_str())
        .unwrap_or(SIDECAR_SERVER_ID)
        .to_string();
    let server_name = if server_id == SIDECAR_SERVER_ID {
        SIDECAR_SERVER_NAME.to_string()
    } else {
        server_id.clone()
    };
    let arguments = data.get("input").cloned().unwrap_or_else(|| json!({}));
    let tool_call_id =
        acc.register_tool_start(&tool_id, &server_id, &server_name, &name, arguments.clone());

    MappedSidecarEvent::ToolCall(StreamToolCallPayload {
        session_id: session_id.to_string(),
        message_id: message_id.to_string(),
        tool_call_id,
        server_id,
        server_name,
        tool_name: name,
        arguments,
        status: "running".to_string(),
    })
}

fn map_tool_end(
    data: &Value,
    session_id: &str,
    message_id: &str,
    acc: &mut SidecarStreamAccumulator,
) -> MappedSidecarEvent {
    let name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let tool_id = data
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let error = data
        .get("error")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let status = data
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or(if error.is_some() { "error" } else { "complete" })
        .to_string();
    let output = data.get("output").map(|v| {
        if let Some(s) = v.as_str() {
            json!({ "content": s })
        } else {
            json!({ "content": v })
        }
    });
    let tool_call_id =
        acc.resolve_tool_end(&tool_id, &name, output.clone(), error.clone(), &status);

    MappedSidecarEvent::ToolResult(StreamToolResultPayload {
        session_id: session_id.to_string(),
        message_id: message_id.to_string(),
        tool_call_id,
        result: output,
        error,
        status,
    })
}

fn extract_token_content(data: &Value) -> Result<String, String> {
    match data.get("content") {
        Some(Value::String(s)) => Ok(s.clone()),
        Some(Value::Array(parts)) => {
            let mut out = String::new();
            for part in parts {
                if let Some(s) = part.as_str() {
                    out.push_str(s);
                } else if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                    out.push_str(text);
                }
            }
            Ok(out)
        }
        Some(other) => Ok(other.to_string()),
        None => Ok(String::new()),
    }
}

/// When `incoming` is a cumulative re-send of `previous`, return only the
/// newly appended suffix. Identical payloads yield an empty string.
fn thinking_suffix_delta(previous: &str, incoming: &str) -> String {
    if incoming.is_empty() {
        return String::new();
    }
    if previous.is_empty() {
        return incoming.to_string();
    }
    if incoming == previous {
        return String::new();
    }
    if let Some(suffix) = incoming.strip_prefix(previous) {
        return suffix.to_string();
    }
    incoming.to_string()
}

/// Consume a Sidecar HTTP SSE response body and emit Tauri stream events.
pub async fn consume_sidecar_stream(
    app: &AppHandle,
    response: reqwest::Response,
    session_id: &str,
    message_id: &str,
    abort_flag: Arc<AtomicBool>,
) -> Result<SidecarStreamOutcome, String> {
    let mut acc = SidecarStreamAccumulator::default();
    let mut leftover = Vec::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        if abort_flag.load(Ordering::Relaxed) {
            break;
        }

        let bytes = chunk.map_err(|e| format!("SSE read error: {e}"))?;
        let frames = append_sse_chunk(&bytes, &mut leftover)?;
        for frame in frames {
            match map_sidecar_event(&frame.event, &frame.data, session_id, message_id, &mut acc) {
                Ok(Some(MappedSidecarEvent::Token { delta })) => {
                    emit_token(app, session_id, message_id, &delta);
                }
                Ok(Some(MappedSidecarEvent::Thinking { delta })) => {
                    emit_thinking(app, session_id, message_id, &delta);
                }
                Ok(Some(MappedSidecarEvent::ToolCall(payload))) => {
                    emit_tool_call(app, &payload);
                }
                Ok(Some(MappedSidecarEvent::ToolResult(payload))) => {
                    emit_tool_result(app, &payload);
                }
                Ok(Some(MappedSidecarEvent::Done)) => {
                    return finalize_stream(app, session_id, message_id, acc, &abort_flag, None);
                }
                Ok(None) => {}
                Err(err) => {
                    return finalize_stream(
                        app,
                        session_id,
                        message_id,
                        acc,
                        &abort_flag,
                        Some(err),
                    );
                }
            }
        }
    }

    finalize_stream(app, session_id, message_id, acc, &abort_flag, None)
}

/// Append a byte chunk and return any complete SSE frames.
pub fn append_sse_chunk(chunk: &[u8], leftover: &mut Vec<u8>) -> Result<Vec<SseFrame>, String> {
    leftover.extend_from_slice(chunk);
    let text = String::from_utf8_lossy(leftover);
    let mut frames = Vec::new();
    let mut consumed = 0usize;
    let mut search_from = 0usize;

    while let Some(rel) = text[search_from..].find("\n\n") {
        let end = search_from + rel;
        let block = &text[consumed..end];
        if let Some(frame) = parse_sse_block(block)? {
            frames.push(frame);
        }
        consumed = end + 2;
        search_from = consumed;
    }

    *leftover = leftover[consumed..].to_vec();
    Ok(frames)
}

fn finalize_stream(
    app: &AppHandle,
    session_id: &str,
    message_id: &str,
    mut acc: SidecarStreamAccumulator,
    abort_flag: &Arc<AtomicBool>,
    stream_error: Option<String>,
) -> Result<SidecarStreamOutcome, String> {
    let was_aborted = abort_flag.load(Ordering::Relaxed) && stream_error.is_none();
    // Aborted streams mark leftovers aborted; errors/orphans become error.
    let pending = acc.settle_open_tools(was_aborted);
    for mut tool_payload in pending {
        tool_payload.session_id = session_id.to_string();
        tool_payload.message_id = message_id.to_string();
        emit_tool_result(app, &tool_payload);
    }

    let outcome = SidecarStreamOutcome {
        result: StreamResult {
            content: acc.content.clone(),
            thinking: acc.thinking.clone(),
            usage: acc.usage.clone(),
            was_aborted,
        },
        tool_calls: acc.tool_calls,
    };

    if let Some(err) = stream_error {
        emit_stream_error(app, session_id, message_id, &err);
        return Err(err);
    }

    let payload = StreamCompletePayload {
        session_id: session_id.to_string(),
        message_id: message_id.to_string(),
        full_content: outcome.result.content.clone(),
        full_thinking: outcome.result.thinking.clone(),
        usage: outcome.result.usage.clone(),
        was_aborted,
    };
    if let Err(e) = app.emit("stream_complete", &payload) {
        tracing::warn!(error = %e, "Failed to emit stream_complete");
    }
    Ok(outcome)
}

fn emit_token(app: &AppHandle, session_id: &str, message_id: &str, delta: &str) {
    let payload = StreamTokenPayload {
        session_id: session_id.to_string(),
        message_id: message_id.to_string(),
        delta: delta.to_string(),
    };
    if let Err(e) = app.emit("stream_token", &payload) {
        tracing::warn!(error = %e, "Failed to emit stream_token");
    }
}

fn emit_thinking(app: &AppHandle, session_id: &str, message_id: &str, delta: &str) {
    let payload = StreamThinkingPayload {
        session_id: session_id.to_string(),
        message_id: message_id.to_string(),
        thinking_delta: delta.to_string(),
    };
    if let Err(e) = app.emit("stream_thinking", &payload) {
        tracing::warn!(error = %e, "Failed to emit stream_thinking");
    }
}

fn emit_stream_error(app: &AppHandle, session_id: &str, message_id: &str, error: &str) {
    let payload = StreamErrorPayload {
        session_id: session_id.to_string(),
        message_id: message_id.to_string(),
        error: error.to_string(),
    };
    if let Err(e) = app.emit("stream_error", &payload) {
        tracing::warn!(error = %e, "Failed to emit stream_error");
    }
}
