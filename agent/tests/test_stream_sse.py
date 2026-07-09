"""Tests for SSE formatting helpers used by /agent/stream."""

import json

from app.routers.agent import _format_sse_event, _sse


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


def test_format_sse_event_tool_start():
    event = {
        "event": "on_tool_start",
        "name": "powermem_search",
        "data": {"input": {"query": "prefs"}},
    }
    result = _format_sse_event(event)
    assert result is not None
    assert "event: tool_start" in result
    assert "powermem_search" in result


def test_format_sse_event_tool_end():
    event = {
        "event": "on_tool_end",
        "name": "powermem_save",
        "data": {"output": "saved"},
    }
    result = _format_sse_event(event)
    assert result is not None
    assert "event: tool_end" in result
    assert "saved" in result


def test_format_sse_event_thinking_start():
    event = {"event": "on_chat_model_start", "data": {}}
    result = _format_sse_event(event)
    assert result is not None
    assert "event: thinking_start" in result


def test_format_sse_event_ignores_unknown():
    assert _format_sse_event({"event": "on_chain_start", "data": {}}) is None
