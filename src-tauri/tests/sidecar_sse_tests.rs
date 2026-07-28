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
    let outcome = acc.into_outcome(abort.load(Ordering::Relaxed));
    assert_eq!(outcome.result.content, "Hello");
    assert!(!outcome.result.was_aborted);
    assert!(outcome.result.usage.is_none());
    assert!(outcome.tool_calls.is_empty());
}

#[test]
fn map_thinking_event_accumulates() {
    let mut acc = SidecarStreamAccumulator::default();
    let mapped = map_sidecar_event("thinking", &json!({"content": "step1"}), "s", "m", &mut acc)
        .unwrap()
        .expect("thinking mapped");
    match mapped {
        MappedSidecarEvent::Thinking { delta } => assert_eq!(delta, "step1"),
        other => panic!("unexpected: {other:?}"),
    }
    let outcome = acc.into_outcome(false);
    assert_eq!(outcome.result.thinking, "step1");
}

#[test]
fn map_thinking_dedupes_cumulative_resends() {
    let mut acc = SidecarStreamAccumulator::default();
    map_sidecar_event("thinking", &json!({"content": "hello"}), "s", "m", &mut acc).unwrap();
    let second = map_sidecar_event(
        "thinking",
        &json!({"content": "hello world"}),
        "s",
        "m",
        &mut acc,
    )
    .unwrap()
    .expect("suffix thinking");
    match second {
        MappedSidecarEvent::Thinking { delta } => assert_eq!(delta, " world"),
        other => panic!("unexpected: {other:?}"),
    }
    let duplicate = map_sidecar_event(
        "thinking",
        &json!({"content": "hello world"}),
        "s",
        "m",
        &mut acc,
    )
    .unwrap();
    assert!(duplicate.is_none());
    assert_eq!(acc.thinking_so_far(), "hello world");
}

#[test]
fn map_duplicate_tool_start_same_id_is_idempotent() {
    let mut acc = SidecarStreamAccumulator::default();
    let first = map_sidecar_event(
        "tool_start",
        &json!({"id": "run-1", "name": "ls", "input": {"path": "/"}}),
        "s",
        "m",
        &mut acc,
    )
    .unwrap()
    .expect("first");
    let second = map_sidecar_event(
        "tool_start",
        &json!({"id": "run-1", "name": "ls", "input": {"path": "/"}}),
        "s",
        "m",
        &mut acc,
    )
    .unwrap()
    .expect("second upsert");
    let id1 = match first {
        MappedSidecarEvent::ToolCall(p) => p.tool_call_id,
        other => panic!("{other:?}"),
    };
    let id2 = match second {
        MappedSidecarEvent::ToolCall(p) => p.tool_call_id,
        other => panic!("{other:?}"),
    };
    assert_eq!(id1, id2);
    let outcome = acc.into_outcome(false);
    assert_eq!(outcome.tool_calls.len(), 1);
}

#[test]
fn map_tool_events_by_run_id_and_persist_records() {
    let mut acc = SidecarStreamAccumulator::default();
    let start_a = map_sidecar_event(
        "tool_start",
        &json!({"id": "run-a", "name": "search", "input": {"q": "1"}}),
        "s",
        "m",
        &mut acc,
    )
    .unwrap()
    .expect("start a");
    let start_b = map_sidecar_event(
        "tool_start",
        &json!({"id": "run-b", "name": "search", "input": {"q": "2"}}),
        "s",
        "m",
        &mut acc,
    )
    .unwrap()
    .expect("start b");
    let id_a = match start_a {
        MappedSidecarEvent::ToolCall(p) => p.tool_call_id,
        other => panic!("{other:?}"),
    };
    let id_b = match start_b {
        MappedSidecarEvent::ToolCall(p) => p.tool_call_id,
        other => panic!("{other:?}"),
    };
    assert_eq!(id_a, "run-a");
    assert_eq!(id_b, "run-b");

    map_sidecar_event(
        "tool_end",
        &json!({"id": "run-b", "name": "search", "output": "second", "status": "complete"}),
        "s",
        "m",
        &mut acc,
    )
    .unwrap();
    map_sidecar_event(
        "tool_end",
        &json!({"id": "run-a", "name": "search", "output": "first", "status": "complete"}),
        "s",
        "m",
        &mut acc,
    )
    .unwrap();

    let outcome = acc.into_outcome(false);
    assert_eq!(outcome.tool_calls.len(), 2);
    assert_eq!(outcome.tool_calls[0].id, "run-a");
    assert_eq!(
        outcome.tool_calls[0].result.as_ref().unwrap()["content"],
        "first"
    );
    assert_eq!(outcome.tool_calls[1].id, "run-b");
    assert_eq!(
        outcome.tool_calls[1].result.as_ref().unwrap()["content"],
        "second"
    );
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
