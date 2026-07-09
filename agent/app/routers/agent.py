"""Agent chat endpoints — DeepAgents conversation entrypoints."""

from __future__ import annotations

import json
import logging
from collections.abc import AsyncGenerator
from typing import Any

from fastapi import APIRouter, HTTPException
from fastapi.responses import StreamingResponse

from app.agent import build_agent
from app.config import get_settings
from app.dependencies import get_checkpointer, get_store
from app.models import ChatMessage, ChatRequest, ChatResponse, MessageRole, TokenUsage, ToolCall

router = APIRouter(prefix="/agent", tags=["agent"])
logger = logging.getLogger(__name__)


@router.post("/chat", response_model=ChatResponse)
async def agent_chat(request: ChatRequest) -> ChatResponse:
    """Synchronous agent chat — wait for the full response."""
    try:
        agent = build_agent(
            session_id=request.session_id,
            working_dir=request.working_dir,
            checkpointer=get_checkpointer(),
            store=get_store(),
        )
        result = await agent.ainvoke(
            {"messages": _request_to_agent_messages(request)},
            config=_thread_config(request),
        )
        return _result_to_chat_response(result)
    except HTTPException:
        raise
    except Exception as exc:
        logger.exception("agent chat failed")
        raise HTTPException(status_code=500, detail=str(exc)) from exc


@router.post("/stream")
async def agent_stream(request: ChatRequest):
    """Streaming agent chat — SSE token/tool/done/error events."""
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
    try:
        agent = build_agent(
            session_id=request.session_id,
            working_dir=request.working_dir,
            checkpointer=get_checkpointer(),
            store=get_store(),
        )
        async for event in agent.astream_events(
            {"messages": _request_to_agent_messages(request)},
            config=_thread_config(request),
            version="v2",
        ):
            sse_data = _format_sse_event(event)
            if sse_data:
                yield sse_data
        yield _sse("done", {"finished": True})
    except Exception as exc:
        logger.exception("agent stream failed")
        yield _sse("error", {"message": str(exc)})


def _request_to_agent_messages(request: ChatRequest) -> list[dict[str, str]]:
    return [{"role": msg.role.value, "content": msg.content} for msg in request.messages]


def _thread_config(request: ChatRequest) -> dict[str, Any]:
    config: dict[str, Any] = {
        "configurable": {"thread_id": request.session_id or "default"},
    }
    if request.config.model:
        config["configurable"]["model"] = request.config.model
    return config


def _format_sse_event(event: dict[str, Any]) -> str | None:
    """Convert a LangGraph/deepagents event into an SSE payload."""
    kind = event.get("event")

    if kind == "on_chat_model_stream":
        chunk = event.get("data", {}).get("chunk")
        content = getattr(chunk, "content", None) if chunk is not None else None
        if content:
            return _sse("token", {"content": content})
        return None

    if kind == "on_tool_start":
        return _sse(
            "tool_start",
            {
                "name": event.get("name", ""),
                "input": event.get("data", {}).get("input", {}),
            },
        )

    if kind == "on_tool_end":
        output = event.get("data", {}).get("output", "")
        return _sse(
            "tool_end",
            {
                "name": event.get("name", ""),
                "output": str(output)[:2000],
            },
        )

    if kind == "on_chat_model_start":
        return _sse("thinking_start", {})

    return None


def _sse(event: str, data: dict[str, Any]) -> str:
    return f"event: {event}\ndata: {json.dumps(data, ensure_ascii=False)}\n\n"


def _result_to_chat_response(result: dict[str, Any]) -> ChatResponse:
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
        model=get_settings().agent_model,
    )


def _extract_usage(result: dict[str, Any]) -> TokenUsage:
    metadata = result.get("__metadata__", {}) or {}
    return TokenUsage(
        prompt_tokens=int(metadata.get("prompt_tokens", 0) or 0),
        completion_tokens=int(metadata.get("completion_tokens", 0) or 0),
        total_tokens=int(metadata.get("total_tokens", 0) or 0),
    )
