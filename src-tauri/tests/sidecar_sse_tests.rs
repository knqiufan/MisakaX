use misaka_x_lib::services::sidecar_sse::{
    append_sse_chunk, map_sidecar_event, parse_sse_frames, MappedSidecarEvent,
    SidecarStreamAccumulator,
};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[test]
fn parse_sse_frames_splits_complete_events() {
    let raw = "\
event: token\n\
data: {\"content\":\"Hi\"}\n\
\n\
event: done\n\
data: {\"finished\":true}\n\
\n";
    let frames = parse_sse_frames(raw.as_bytes()).unwrap();
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0].event, "token");
    assert_eq!(frames[0].data["content"], "Hi");
    assert_eq!(frames[1].event, "done");
    assert_eq!(frames[1].data["finished"], true);
}

#[test]
fn parse_sse_frames_handles_chunked_buffer_across_calls() {
    let mut leftover = Vec::new();
    let frames1 =
        append_sse_chunk(b"event: token\ndata: {\"content\":\"Hel", &mut leftover).unwrap();
    assert!(frames1.is_empty());

    let frames2 = append_sse_chunk(
        b"lo\"}\n\nevent: done\ndata: {\"finished\":true}\n\n",
        &mut leftover,
    )
    .unwrap();
    assert_eq!(frames2.len(), 2);
    assert_eq!(frames2[0].data["content"], "Hello");
    assert_eq!(frames2[1].event, "done");
    assert!(leftover.is_empty());
}

#[test]
fn map_token_event_to_delta() {
    let mapped = map_sidecar_event(
        "token",
        &json!({"content": "Hello"}),
        "session-1",
        "msg-1",
        &mut SidecarStreamAccumulator::default(),
    )
    .unwrap();
    match mapped {
        Some(MappedSidecarEvent::Token { delta }) => assert_eq!(delta, "Hello"),
        other => panic!("unexpected mapping: {other:?}"),
    }
}

#[test]
fn map_tool_start_and_end_share_tool_call_id() {
    let mut acc = SidecarStreamAccumulator::default();
    let start = map_sidecar_event(
        "tool_start",
        &json!({"name": "powermem_search", "input": {"query": "prefs"}}),
        "session-1",
        "msg-1",
        &mut acc,
    )
    .unwrap()
    .expect("tool_start mapped");

    let tool_call_id = match start {
        MappedSidecarEvent::ToolCall(payload) => {
            assert_eq!(payload.tool_name, "powermem_search");
            assert_eq!(payload.status, "running");
            assert_eq!(payload.server_id, "sidecar");
            assert_eq!(payload.arguments["query"], "prefs");
            payload.tool_call_id.clone()
        }
        other => panic!("unexpected: {other:?}"),
    };

    let end = map_sidecar_event(
        "tool_end",
        &json!({"name": "powermem_search", "output": "found"}),
        "session-1",
        "msg-1",
        &mut acc,
    )
    .unwrap()
    .expect("tool_end mapped");

    match end {
        MappedSidecarEvent::ToolResult(payload) => {
            assert_eq!(payload.tool_call_id, tool_call_id);
            assert_eq!(payload.status, "complete");
            assert_eq!(payload.result.as_ref().unwrap()["content"], "found");
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn map_error_event_returns_err() {
    let err = map_sidecar_event(
        "error",
        &json!({"message": "boom"}),
        "session-1",
        "msg-1",
        &mut SidecarStreamAccumulator::default(),
    )
    .unwrap_err();
    assert!(err.contains("boom"));
}

#[test]
fn map_done_event_returns_complete() {
    let mut acc = SidecarStreamAccumulator::default();
    acc.push_content("Hello");
    let abort = Arc::new(AtomicBool::new(false));
    let mapped = map_sidecar_event(
        "done",
        &json!({"finished": true}),
        "session-1",
        "msg-1",
        &mut acc,
    )
    .unwrap()
    .expect("done mapped");
    match mapped {
        MappedSidecarEvent::Done => {}
        other => panic!("unexpected: {other:?}"),
    }
    let result = acc.into_stream_result(abort.load(Ordering::Relaxed));
    assert_eq!(result.content, "Hello");
    assert!(!result.was_aborted);
    assert!(result.usage.is_none());
}

#[test]
fn map_unknown_and_thinking_start_are_ignored() {
    let mut acc = SidecarStreamAccumulator::default();
    assert!(
        map_sidecar_event("thinking_start", &json!({}), "s", "m", &mut acc)
            .unwrap()
            .is_none()
    );
    assert!(
        map_sidecar_event("interrupt", &json!({}), "s", "m", &mut acc)
            .unwrap()
            .is_none()
    );
}
