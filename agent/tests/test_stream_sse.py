"""Tests for SSE formatting helpers used by /agent/stream."""

import json

from app.routers.agent import _format_sse_event, _format_sse_events, _sse
from app.stream_content import expand_tool_payload, extract_stream_deltas


def test_sse_formats_event_and_data():
    payload = {"content": "hello"}
    result = _sse("token", payload)
    assert result.startswith("event: token\n")
    assert "data: " in result
    assert result.endswith("\n\n")
    data_line = result.split("\n")[1]
    assert json.loads(data_line.removeprefix("data: ")) == payload


def test_format_sse_event_token_chunk():
    class Chunk:
        content = "Hi"

    event = {
        "event": "on_chat_model_stream",
        "data": {"chunk": Chunk()},
    }
    result = _format_sse_event(event)
    assert result is not None
    assert "event: token" in result
    assert '"content": "Hi"' in result or '"content":"Hi"' in result


def test_format_sse_event_thinking_and_token():
    class Chunk:
        content = [{"type": "thinking", "thinking": "plan"}, {"type": "text", "text": "Hi"}]
        additional_kwargs = {}

    frames = _format_sse_events(
        {"event": "on_chat_model_stream", "data": {"chunk": Chunk()}}
    )
    assert len(frames) == 2
    assert "event: thinking" in frames[0]
    assert "event: token" in frames[1]


def test_format_sse_event_tool_start_uses_run_id():
    event = {
        "event": "on_tool_start",
        "name": "powermem_search",
        "run_id": "run-1",
        "data": {"input": {"query": "prefs"}},
    }
    result = _format_sse_event(event)
    assert result is not None
    assert "event: tool_start" in result
    assert "run-1" in result
    assert "powermem_search" in result


def test_format_sse_event_mcp_bridge_expanded():
    event = {
        "event": "on_tool_start",
        "name": "mcp_bridge",
        "run_id": "run-mcp",
        "data": {
            "input": {
                "server_id": "fs",
                "tool_name": "read_file",
                "arguments": {"path": "a.txt"},
            }
        },
    }
    result = _format_sse_event(event)
    assert result is not None
    data = json.loads(result.split("\n")[1].removeprefix("data: "))
    assert data["name"] == "read_file"
    assert data["server_id"] == "fs"
    assert data["input"]["path"] == "a.txt"


def test_format_sse_event_tool_end():
    event = {
        "event": "on_tool_end",
        "name": "powermem_save",
        "run_id": "run-2",
        "data": {"output": "saved"},
    }
    result = _format_sse_event(event)
    assert result is not None
    assert "event: tool_end" in result
    assert "saved" in result


def test_format_sse_event_tool_error():
    event = {
        "event": "on_tool_error",
        "name": "powermem_save",
        "run_id": "run-3",
        "data": {"error": "boom"},
    }
    result = _format_sse_event(event)
    assert result is not None
    assert "event: tool_end" in result
    assert "boom" in result
    assert "error" in result


def test_on_chat_model_start_no_longer_emits_thinking_start():
    assert _format_sse_event({"event": "on_chat_model_start", "data": {}}) is None


def test_format_sse_event_ignores_unknown():
    assert _format_sse_event({"event": "on_chain_start", "data": {}}) is None


def test_extract_stream_deltas_dedupes_compat_reasoning():
    class Chunk:
        content = [{"type": "thinking", "thinking": "same"}]
        additional_kwargs = {"reasoning_content": "same"}

    token, thinking = extract_stream_deltas(Chunk())
    assert token == ""
    assert thinking == "same"


def test_thinking_delta_from_cumulative_suffix_only():
    from app.stream_content import thinking_delta_from_cumulative

    assert thinking_delta_from_cumulative("", "ab") == "ab"
    assert thinking_delta_from_cumulative("ab", "abcd") == "cd"
    assert thinking_delta_from_cumulative("abcd", "abcd") == ""
    assert thinking_delta_from_cumulative("ab", "xy") == "xy"


def test_format_sse_skips_duplicate_tool_start_by_run_id():
    seen: set[str] = set()
    event = {
        "event": "on_tool_start",
        "name": "ls",
        "run_id": "run-dup",
        "data": {"input": {"path": "/"}},
    }
    first = _format_sse_events(event, emitted_tool_ids=seen)
    second = _format_sse_events(event, emitted_tool_ids=seen)
    assert len(first) == 1
    assert second == []


def test_format_sse_thinking_uses_cumulative_suffix():
    class Chunk:
        content = []
        additional_kwargs = {"reasoning_content": "hello world"}

    frames = _format_sse_events(
        {"event": "on_chat_model_stream", "data": {"chunk": Chunk()}},
        thinking_acc="hello ",
    )
    assert len(frames) == 1
    assert "event: thinking" in frames[0]
    assert "world" in frames[0]
    assert "hello world" not in frames[0].split("data: ", 1)[1]


def test_should_emit_skips_task_wrapper_tools():
    from app.stream_content import should_emit_langgraph_event

    assert should_emit_langgraph_event(
        {"event": "on_tool_start", "name": "task", "data": {}}
    ) is False
    assert should_emit_langgraph_event(
        {"event": "on_tool_start", "name": "ls", "data": {}}
    ) is True


def test_expand_tool_payload_powermem_stays_sidecar():
    payload = expand_tool_payload("powermem_search", {"query": "x"}, "rid")
    assert payload["server_id"] == "sidecar"
    assert payload["name"] == "powermem_search"
    assert payload["id"] == "rid"
