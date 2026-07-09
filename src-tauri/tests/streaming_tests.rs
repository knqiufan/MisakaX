use std::sync::atomic::Ordering;

use misaka_x_lib::services::llm::{
    StreamCompletePayload, StreamErrorPayload, StreamRegistry, StreamThinkingPayload,
    StreamTokenPayload, StreamToolCallPayload, StreamToolResultPayload, StreamUsage,
    TokenUsageInfo,
};

// ─── StreamRegistry tests ──────────────────────────────

#[test]
fn test_registry_register_and_unregister() {
    let registry = StreamRegistry::new();
    assert_eq!(registry.active_count(), 0);

    let flag = registry.register("session-1");
    assert!(registry.is_active("session-1"));
    assert_eq!(registry.active_count(), 1);
    assert!(!flag.load(Ordering::Relaxed));

    registry.unregister("session-1");
    assert!(!registry.is_active("session-1"));
    assert_eq!(registry.active_count(), 0);
}

#[test]
fn test_registry_abort() {
    let registry = StreamRegistry::new();
    let flag = registry.register("session-1");

    assert!(!flag.load(Ordering::Relaxed));
    let aborted = registry.abort("session-1");
    assert!(aborted);
    assert!(flag.load(Ordering::Relaxed));
}

#[test]
fn test_registry_abort_nonexistent() {
    let registry = StreamRegistry::new();
    let aborted = registry.abort("nonexistent");
    assert!(!aborted);
}

#[test]
fn test_registry_multiple_sessions() {
    let registry = StreamRegistry::new();
    let flag1 = registry.register("session-1");
    let flag2 = registry.register("session-2");
    assert_eq!(registry.active_count(), 2);

    registry.abort("session-1");
    assert!(flag1.load(Ordering::Relaxed));
    assert!(!flag2.load(Ordering::Relaxed));

    registry.unregister("session-1");
    assert_eq!(registry.active_count(), 1);
    assert!(registry.is_active("session-2"));
}

#[test]
fn test_registry_re_register_replaces_flag() {
    let registry = StreamRegistry::new();
    let old_flag = registry.register("session-1");
    registry.abort("session-1");
    assert!(old_flag.load(Ordering::Relaxed));

    let new_flag = registry.register("session-1");
    assert!(!new_flag.load(Ordering::Relaxed));
    assert_eq!(registry.active_count(), 1);
}

#[test]
fn test_registry_default() {
    let registry = StreamRegistry::default();
    assert_eq!(registry.active_count(), 0);
}

// ─── TokenUsageInfo tests ──────────────────────────────

#[test]
fn test_token_usage_info_serialize() {
    let usage = TokenUsageInfo {
        input_tokens: 100,
        output_tokens: 50,
        total_tokens: 150,
    };
    let json = serde_json::to_string(&usage).unwrap();
    assert!(json.contains("\"input_tokens\":100"));
    assert!(json.contains("\"output_tokens\":50"));
    assert!(json.contains("\"total_tokens\":150"));
}

#[test]
fn test_token_usage_from_stream_usage() {
    let stream_usage = StreamUsage {
        input_tokens: 42,
        output_tokens: 13,
        total_tokens: 55,
    };
    let info = TokenUsageInfo::from(stream_usage);
    assert_eq!(info.input_tokens, 42);
    assert_eq!(info.output_tokens, 13);
    assert_eq!(info.total_tokens, 55);
}

// ─── Event Payload serialization tests ─────────────────

#[test]
fn test_stream_token_payload_serialize() {
    let payload = StreamTokenPayload {
        session_id: "s1".to_string(),
        message_id: "m1".to_string(),
        delta: "Hello".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"session_id\":\"s1\""));
    assert!(json.contains("\"delta\":\"Hello\""));
}

#[test]
fn test_stream_thinking_payload_serialize() {
    let payload = StreamThinkingPayload {
        session_id: "s1".to_string(),
        message_id: "m1".to_string(),
        thinking_delta: "Let me think...".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"thinking_delta\":\"Let me think...\""));
}

#[test]
fn test_stream_complete_payload_serialize() {
    let payload = StreamCompletePayload {
        session_id: "s1".to_string(),
        message_id: "m1".to_string(),
        full_content: "Hello world".to_string(),
        full_thinking: "I thought about it".to_string(),
        usage: Some(TokenUsageInfo {
            input_tokens: 10,
            output_tokens: 5,
            total_tokens: 15,
        }),
        was_aborted: false,
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"full_content\":\"Hello world\""));
    assert!(json.contains("\"was_aborted\":false"));
    assert!(json.contains("\"input_tokens\":10"));
}

#[test]
fn test_stream_complete_payload_without_usage() {
    let payload = StreamCompletePayload {
        session_id: "s1".to_string(),
        message_id: "m1".to_string(),
        full_content: "".to_string(),
        full_thinking: "".to_string(),
        usage: None,
        was_aborted: true,
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"was_aborted\":true"));
    assert!(json.contains("\"usage\":null"));
}

#[test]
fn test_stream_error_payload_serialize() {
    let payload = StreamErrorPayload {
        session_id: "s1".to_string(),
        message_id: "m1".to_string(),
        error: "Connection refused".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"error\":\"Connection refused\""));
}

// ─── Tool Event Payload serialization tests ────────────
// 字段名必须与前端 use-stream-listener.ts 的解构完全一致。

#[test]
fn test_stream_tool_call_payload_serialize() {
    let payload = StreamToolCallPayload {
        session_id: "s1".to_string(),
        message_id: "m1".to_string(),
        tool_call_id: "tc1".to_string(),
        server_id: "srv".to_string(),
        server_name: "Filesystem".to_string(),
        tool_name: "read_file".to_string(),
        arguments: serde_json::json!({ "path": "package.json" }),
        status: "running".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"tool_call_id\":\"tc1\""));
    assert!(json.contains("\"server_id\":\"srv\""));
    assert!(json.contains("\"server_name\":\"Filesystem\""));
    assert!(json.contains("\"tool_name\":\"read_file\""));
    assert!(json.contains("\"arguments\":{\"path\":\"package.json\"}"));
    assert!(json.contains("\"status\":\"running\""));
}

#[test]
fn test_stream_tool_result_payload_serialize_complete() {
    let payload = StreamToolResultPayload {
        session_id: "s1".to_string(),
        message_id: "m1".to_string(),
        tool_call_id: "tc1".to_string(),
        result: Some(serde_json::json!({ "content": "ok" })),
        error: None,
        status: "complete".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"tool_call_id\":\"tc1\""));
    assert!(json.contains("\"result\":{\"content\":\"ok\"}"));
    assert!(json.contains("\"error\":null"));
    assert!(json.contains("\"status\":\"complete\""));
}

#[test]
fn test_stream_tool_result_payload_serialize_error() {
    let payload = StreamToolResultPayload {
        session_id: "s1".to_string(),
        message_id: "m1".to_string(),
        tool_call_id: "tc1".to_string(),
        result: None,
        error: Some("boom".to_string()),
        status: "error".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"result\":null"));
    assert!(json.contains("\"error\":\"boom\""));
    assert!(json.contains("\"status\":\"error\""));
}
