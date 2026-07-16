"""Provider-neutral helpers for extracting text / thinking from model chunks."""

from __future__ import annotations

from typing import Any


def extract_stream_deltas(chunk: Any) -> tuple[str, str]:
    """Return (token_text, thinking_text) from a LangChain chat model chunk.

    Deduplicates overlapping structured blocks and OpenAI-compat
    ``reasoning_content`` so the same reasoning is not shown twice.
    """
    if chunk is None:
        return "", ""

    content = getattr(chunk, "content", None)
    token_parts: list[str] = []
    thinking_parts: list[str] = []
    seen_thinking: set[str] = set()

    if isinstance(content, str):
        if content:
            token_parts.append(content)
    elif isinstance(content, list):
        for block in content:
            kind, text = _classify_block(block)
            if not text:
                continue
            if kind == "thinking":
                if text not in seen_thinking:
                    seen_thinking.add(text)
                    thinking_parts.append(text)
            else:
                token_parts.append(text)

    compat = _compat_reasoning(chunk)
    if compat and compat not in seen_thinking:
        thinking_parts.append(compat)

    return "".join(token_parts), "".join(thinking_parts)


def thinking_delta_from_cumulative(previous: str, incoming: str) -> str:
    """Return only the newly appended suffix when providers send cumulative text.

    Some OpenAI-compat endpoints re-send the full ``reasoning_content`` on every
    chunk. If ``incoming`` starts with ``previous``, emit only the suffix.
    Identical payloads yield an empty string (no duplicate frame).
    """
    if not incoming:
        return ""
    if not previous:
        return incoming
    if incoming == previous:
        return ""
    if incoming.startswith(previous):
        return incoming[len(previous) :]
    # Non-prefix case: treat as a fresh incremental delta.
    return incoming


def _classify_block(block: Any) -> tuple[str, str]:
    if isinstance(block, str):
        return "token", block
    if not isinstance(block, dict):
        text = getattr(block, "text", None) or getattr(block, "thinking", None)
        block_type = getattr(block, "type", None) or getattr(block, "block_type", None)
        if isinstance(text, str) and text:
            if _is_thinking_type(block_type) or hasattr(block, "thinking"):
                return "thinking", text
            return "token", text
        return "token", ""

    block_type = str(block.get("type") or block.get("block_type") or "")
    if _is_thinking_type(block_type):
        text = (
            block.get("thinking")
            or block.get("text")
            or block.get("reasoning")
            or block.get("content")
            or ""
        )
        return "thinking", str(text) if text else ""

    text = block.get("text") or block.get("content") or ""
    if isinstance(text, str) and text:
        return "token", text
    return "token", ""


def _is_thinking_type(block_type: Any) -> bool:
    if not block_type:
        return False
    lowered = str(block_type).lower()
    return lowered in {"thinking", "reasoning", "reasoning_content", "redacted_thinking"}


def _compat_reasoning(chunk: Any) -> str:
    """OpenAI-compatible additional_kwargs / response_metadata reasoning."""
    for attr in ("additional_kwargs", "response_metadata"):
        meta = getattr(chunk, attr, None)
        if not isinstance(meta, dict):
            continue
        for key in ("reasoning_content", "reasoning", "thinking"):
            value = meta.get(key)
            if isinstance(value, str) and value:
                return value
    return ""


def expand_tool_payload(
    name: str,
    raw_input: Any,
    run_id: str | None,
) -> dict[str, Any]:
    """Normalize tool_start payload; expand mcp_bridge args when present."""
    arguments: Any = raw_input if isinstance(raw_input, dict) else {"input": raw_input}
    server_id = "sidecar"
    tool_name = name or "unknown"

    if tool_name == "mcp_bridge" and isinstance(arguments, dict):
        server_id = str(arguments.get("server_id") or "mcp")
        tool_name = str(arguments.get("tool_name") or arguments.get("name") or "mcp_bridge")
        nested = arguments.get("arguments")
        if nested is not None:
            arguments = nested

    return {
        "id": run_id or "",
        "name": tool_name,
        "server_id": server_id,
        "input": arguments if isinstance(arguments, dict) else {"value": arguments},
    }


def should_emit_langgraph_event(event: dict[str, Any]) -> bool:
    """Skip nested/internal LangGraph frames that would duplicate tool/token SSE.

    Prefer root-level chat/tool events. Nested subagent or middleware echoes
    often re-emit the same ``on_tool_start`` with a different run_id.
    """
    tags = event.get("tags") or []
    if isinstance(tags, list):
        # LangGraph marks nested runs; skip common duplicate sources.
        skip = {"seq:parallel", "langsmith:hidden"}
        if any(t in skip for t in tags):
            return False

    # Events from deeply nested graphs (parent_ids length > 1) that are
    # chat-model streams can still be valuable; only filter tool events that
    # clearly belong to an internal "task" wrapper when name is task.
    kind = event.get("event")
    name = str(event.get("name") or "")
    if kind in {"on_tool_start", "on_tool_end", "on_tool_error"} and name == "task":
        # The outer task tool wraps subagent work; UI should show leaf tools.
        return False
    return True
