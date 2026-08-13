"""Agent chat endpoints — DeepAgents conversation entrypoints."""

from __future__ import annotations

import json
import logging
import traceback
from collections.abc import AsyncGenerator
from typing import Any

from fastapi import APIRouter, HTTPException
from fastapi.responses import StreamingResponse

from app.agent import build_agent, resolve_selected_skills
from app.config import get_settings
from app.dependencies import get_checkpointer, get_store
from app.llm import resolve_chat_model
from app.models import (
    ChatMessage,
    ChatRequest,
    ChatResponse,
    MessageRole,
    TokenUsage,
    ToolCall,
)
from app.stream_content import (
    expand_tool_payload,
    extract_stream_deltas,
    should_emit_langgraph_event,
    thinking_delta_from_cumulative,
)
from app.usage import UsageCollector

router = APIRouter(prefix="/agent", tags=["agent"])
logger = logging.getLogger(__name__)


@router.post("/chat", response_model=ChatResponse)
async def agent_chat(request: ChatRequest) -> ChatResponse:
    """Synchronous agent chat — wait for the full response."""
    try:
        agent = _build_request_agent(request)
        result = await agent.ainvoke(
            {"messages": _request_to_agent_messages(request)},
            config=_thread_config(request),
        )
        return _result_to_chat_response(result, model=request.config.model)
    except HTTPException:
        raise
    except Exception as exc:
        logger.exception(
            "agent chat failed session_id=%s mode=%s",
            request.session_id,
            request.agent_mode,
        )
        raise HTTPException(status_code=500, detail=str(exc)) from exc


@router.post("/stream")
async def agent_stream(request: ChatRequest):
    """Streaming agent chat — SSE token/thinking/tool/done/error events."""
    return StreamingResponse(
        _stream_agent(request),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        },
    )


async def _stream_agent(request: ChatRequest) -> AsyncGenerator[str, None]:
    open_tools: dict[str, dict[str, Any]] = {}
    thinking_acc = ""
    emitted_tool_ids: set[str] = set()
    usage = UsageCollector(default_model=request.config.model)
    try:
        agent = _build_request_agent(request)
        async for event in agent.astream_events(
            {"messages": _request_to_agent_messages(request)},
            config=_thread_config(request),
            version="v2",
        ):
            usage.observe(event)
            if not should_emit_langgraph_event(event, agent_mode=request.agent_mode):
                continue
            for sse_data in _format_sse_events(
                event,
                thinking_acc=thinking_acc,
                emitted_tool_ids=emitted_tool_ids,
                open_tools=open_tools,
            ):
                if sse_data.startswith("event: thinking\n"):
                    data_line = sse_data.split("\n", 2)[1]
                    payload = json.loads(data_line.removeprefix("data: "))
                    thinking_acc += str(payload.get("content") or "")
                yield sse_data
        for frame in _close_open_tools(open_tools, reason="Stream ended without tool_end"):
            yield frame
        if usage_payload := usage.event_payload():
            yield _sse("usage", usage_payload)
        yield _sse("done", {"finished": True})
    except Exception as exc:
        tb = traceback.format_exc()
        logger.error(
            "agent stream failed session_id=%s mode=%s error=%s\n%s",
            request.session_id,
            request.agent_mode,
            exc,
            tb,
        )
        for frame in _close_open_tools(open_tools, reason=str(exc) or "stream error"):
            yield frame
        if usage_payload := usage.event_payload():
            yield _sse("usage", usage_payload)
        yield _sse("error", {"message": str(exc)})


def _build_request_agent(request: ChatRequest):
    """Assemble agent using the provider binding from the chat request."""
    model = resolve_chat_model(request.config)
    mode = "research" if request.agent_mode == "research" else "chat"
    settings = get_settings()
    try:
        if request.skill_activation is not None:
            active_skills = resolve_selected_skills(
                request.skill_activation.skills,
                settings.skills_dir,
            )
            by_id = {skill.skill_id: skill for skill in active_skills}
            if any(not skill_id for skill_id in request.selected_skill_ids):
                raise ValueError("Selected Skill ID cannot be empty")
            missing = [
                skill_id
                for skill_id in request.selected_skill_ids
                if skill_id not in by_id
            ]
            if missing:
                raise ValueError("Selected Skill is absent from the activation view")
            selected_skills = [by_id[skill_id] for skill_id in request.selected_skill_ids]
        elif request.selected_skills:
            # One-release compatibility path: an old client must still provide
            # explicit validated paths. Slugs alone are never expanded here.
            active_skills = resolve_selected_skills(
                request.selected_skills,
                settings.skills_dir,
            )
            selected_skills = active_skills
        elif request.selected_skill_ids:
            raise ValueError(
                "Skill IDs require a generation-stamped activation view"
            )
        else:
            active_skills = []
            selected_skills = []
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc
    return build_agent(
        session_id=request.session_id,
        working_dir=request.working_dir,
        checkpointer=get_checkpointer(),
        store=get_store(),
        model=model,
        agent_mode=mode,
        selected_skills=selected_skills,
        active_skills=active_skills,
    )


def _request_to_agent_messages(request: ChatRequest) -> list[dict[str, str]]:
    return [{"role": msg.role.value, "content": msg.content} for msg in request.messages]


def _thread_config(request: ChatRequest) -> dict[str, Any]:
    settings = get_settings()
    # Keep a finite recursion ceiling for research; chat stays leaner.
    limit = settings.agent_recursion_limit if request.agent_mode == "research" else 50
    config: dict[str, Any] = {
        "configurable": {"thread_id": request.session_id or "default"},
        "recursion_limit": limit,
    }
    if request.config.model:
        config["configurable"]["model"] = request.config.model
    return config


def _format_sse_events(
    event: dict[str, Any],
    *,
    thinking_acc: str = "",
    emitted_tool_ids: set[str] | None = None,
    open_tools: dict[str, dict[str, Any]] | None = None,
) -> list[str]:
    """Convert a LangGraph event into zero or more SSE payloads."""
    kind = event.get("event")
    run_id = event.get("run_id") or event.get("id")
    tool_ids = emitted_tool_ids if emitted_tool_ids is not None else set()
    open_map = open_tools if open_tools is not None else {}

    if kind == "on_chat_model_stream":
        chunk = event.get("data", {}).get("chunk")
        token, thinking = extract_stream_deltas(chunk)
        thinking = thinking_delta_from_cumulative(thinking_acc, thinking)
        frames: list[str] = []
        if thinking:
            frames.append(_sse("thinking", {"content": thinking}))
        if token:
            frames.append(_sse("token", {"content": token}))
        return frames

    if kind == "on_tool_start":
        payload = expand_tool_payload(
            name=str(event.get("name") or ""),
            raw_input=event.get("data", {}).get("input", {}),
            run_id=str(run_id) if run_id else None,
        )
        tool_id = str(payload.get("id") or "")
        if tool_id and tool_id in tool_ids:
            return []
        if tool_id:
            tool_ids.add(tool_id)
            open_map[tool_id] = {
                "name": payload.get("name") or "",
                "server_id": payload.get("server_id"),
            }
        return [_sse("tool_start", payload)]

    if kind == "on_tool_end":
        tool_id = str(run_id) if run_id else ""
        open_map.pop(tool_id, None)
        return [_format_tool_end(event, run_id, error=None)]

    if kind == "on_tool_error":
        tool_id = str(run_id) if run_id else ""
        open_map.pop(tool_id, None)
        err = event.get("data", {}).get("error") or event.get("data", {}).get("message")
        return [_format_tool_end(event, run_id, error=str(err or "tool error"))]

    return []


def _close_open_tools(
    open_tools: dict[str, dict[str, Any]],
    *,
    reason: str,
) -> list[str]:
    frames: list[str] = []
    for tool_id, meta in list(open_tools.items()):
        payload: dict[str, Any] = {
            "id": tool_id,
            "name": str(meta.get("name") or "unknown"),
            "output": "",
            "status": "error",
            "error": reason,
        }
        if meta.get("server_id"):
            payload["server_id"] = meta["server_id"]
        frames.append(_sse("tool_end", payload))
        open_tools.pop(tool_id, None)
    return frames


def _format_tool_end(event: dict[str, Any], run_id: Any, error: str | None) -> str:
    name = str(event.get("name") or "")
    output = event.get("data", {}).get("output", "")
    payload: dict[str, Any] = {
        "id": str(run_id) if run_id else "",
        "name": name,
        "output": str(output)[:2000] if output is not None else "",
        "status": "error" if error else "complete",
    }
    if name == "mcp_bridge":
        raw_input = event.get("data", {}).get("input")
        if isinstance(raw_input, dict):
            payload["name"] = str(
                raw_input.get("tool_name") or raw_input.get("name") or name
            )
            payload["server_id"] = str(raw_input.get("server_id") or "mcp")
    if error:
        payload["error"] = error
    return _sse("tool_end", payload)


def _format_sse_event(event: dict[str, Any]) -> str | None:
    """Backward-compatible single-frame helper for existing unit tests."""
    frames = _format_sse_events(event)
    return frames[0] if frames else None


def _sse(event: str, data: dict[str, Any]) -> str:
    return f"event: {event}\ndata: {json.dumps(data, ensure_ascii=False)}\n\n"


def _result_to_chat_response(
    result: dict[str, Any],
    model: str | None = None,
) -> ChatResponse:
    messages = result.get("messages") or []
    if not messages:
        raise RuntimeError("Agent returned no messages")

    last_msg = messages[-1]
    content = getattr(last_msg, "content", None)
    if content is None and isinstance(last_msg, dict):
        content = last_msg.get("content", "")
    content = content if isinstance(content, str) else str(content or "")

    raw_tool_calls = getattr(last_msg, "tool_calls", None)
    if raw_tool_calls is None and isinstance(last_msg, dict):
        raw_tool_calls = last_msg.get("tool_calls", [])
    tool_calls: list[ToolCall] = []
    for tc in raw_tool_calls or []:
        if isinstance(tc, dict):
            tool_calls.append(
                ToolCall(
                    id=str(tc.get("id", "")),
                    name=str(tc.get("name", "")),
                    arguments=tc.get("args") or tc.get("arguments") or {},
                )
            )

    return ChatResponse(
        message=ChatMessage(role=MessageRole.assistant, content=content),
        tool_calls=tool_calls,
        usage=_extract_usage(result),
        model=model or get_settings().agent_model,
    )


def _extract_usage(result: dict[str, Any]) -> TokenUsage:
    metadata = result.get("__metadata__", {}) or {}
    return TokenUsage(
        prompt_tokens=int(metadata.get("prompt_tokens", 0) or 0),
        completion_tokens=int(metadata.get("completion_tokens", 0) or 0),
        total_tokens=int(metadata.get("total_tokens", 0) or 0),
    )
