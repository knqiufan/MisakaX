//! Parse Python Sidecar SSE (`/agent/stream`) and map to existing Tauri stream events.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::StreamExt;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

use crate::services::llm::{
    emit_tool_call, emit_tool_result, StreamCompletePayload, StreamErrorPayload, StreamResult,
    StreamTokenPayload, StreamToolCallPayload, StreamToolResultPayload, TokenUsageInfo,
};

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
    ToolCall(StreamToolCallPayload),
    ToolResult(StreamToolResultPayload),
    Done,
}

#[derive(Debug, Default)]
pub struct SidecarStreamAccumulator {
    content: String,
    thinking: String,
    usage: Option<TokenUsageInfo>,
    /// Stack of open tool_call_ids keyed by tool name (LIFO for same-name tools).
    open_tools: HashMap<String, Vec<String>>,
}

impl SidecarStreamAccumulator {
    pub fn push_content(&mut self, delta: &str) {
        self.content.push_str(delta);
    }

    pub fn into_stream_result(self, was_aborted: bool) -> StreamResult {
        StreamResult {
            content: self.content,
            thinking: self.thinking,
            usage: self.usage,
            was_aborted,
        }
    }

    fn register_tool_start(&mut self, name: &str) -> String {
        let tool_call_id = uuid::Uuid::new_v4().to_string();
        self.open_tools
            .entry(name.to_string())
            .or_default()
            .push(tool_call_id.clone());
        tool_call_id
    }

    fn resolve_tool_end(&mut self, name: &str) -> String {
        if let Some(stack) = self.open_tools.get_mut(name) {
            if let Some(id) = stack.pop() {
                if stack.is_empty() {
                    self.open_tools.remove(name);
                }
                return id;
            }
        }
        tracing::warn!(tool = %name, "tool_end without matching tool_start; generating fallback id");
        uuid::Uuid::new_v4().to_string()
    }
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
        "tool_start" => {
            let name = data
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let arguments = data.get("input").cloned().unwrap_or_else(|| json!({}));
            let tool_call_id = acc.register_tool_start(&name);
            Ok(Some(MappedSidecarEvent::ToolCall(StreamToolCallPayload {
                session_id: session_id.to_string(),
                message_id: message_id.to_string(),
                tool_call_id,
                server_id: SIDECAR_SERVER_ID.to_string(),
                server_name: SIDECAR_SERVER_NAME.to_string(),
                tool_name: name,
                arguments,
                status: "running".to_string(),
            })))
        }
        "tool_end" => {
            let name = data
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let output = data
                .get("output")
                .map(|v| {
                    if let Some(s) = v.as_str() {
                        s.to_string()
                    } else {
                        v.to_string()
                    }
                })
                .unwrap_or_default();
            let tool_call_id = acc.resolve_tool_end(&name);
            Ok(Some(MappedSidecarEvent::ToolResult(
                StreamToolResultPayload {
                    session_id: session_id.to_string(),
                    message_id: message_id.to_string(),
                    tool_call_id,
                    result: Some(json!({ "content": output })),
                    error: None,
                    status: "complete".to_string(),
                },
            )))
        }
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

/// Consume a Sidecar HTTP SSE response body and emit Tauri stream events.
pub async fn consume_sidecar_stream(
    app: &AppHandle,
    response: reqwest::Response,
    session_id: &str,
    message_id: &str,
    abort_flag: Arc<AtomicBool>,
) -> Result<StreamResult, String> {
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
                Ok(Some(MappedSidecarEvent::ToolCall(payload))) => {
                    emit_tool_call(app, &payload);
                }
                Ok(Some(MappedSidecarEvent::ToolResult(payload))) => {
                    emit_tool_result(app, &payload);
                }
                Ok(Some(MappedSidecarEvent::Done)) => {
                    return finalize_stream(app, session_id, message_id, acc, &abort_flag);
                }
                Ok(None) => {}
                Err(err) => {
                    emit_stream_error(app, session_id, message_id, &err);
                    return Err(err);
                }
            }
        }
    }

    finalize_stream(app, session_id, message_id, acc, &abort_flag)
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
    acc: SidecarStreamAccumulator,
    abort_flag: &Arc<AtomicBool>,
) -> Result<StreamResult, String> {
    let was_aborted = abort_flag.load(Ordering::Relaxed);
    let result = acc.into_stream_result(was_aborted);
    let payload = StreamCompletePayload {
        session_id: session_id.to_string(),
        message_id: message_id.to_string(),
        full_content: result.content.clone(),
        full_thinking: result.thinking.clone(),
        usage: result.usage.clone(),
        was_aborted,
    };
    if let Err(e) = app.emit("stream_complete", &payload) {
        tracing::warn!(error = %e, "Failed to emit stream_complete");
    }
    Ok(result)
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
