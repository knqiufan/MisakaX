"""MisakaX custom tools for PowerMem and MCP bridge."""

from __future__ import annotations

from typing import Any

from langchain_core.tools import tool

from app.config import get_settings


@tool
def powermem_search(query: str, limit: int = 5) -> list[dict[str, Any]]:
    """Search long-term memory for information relevant to the query."""
    from app.memory import get_memory_engine

    engine = get_memory_engine()
    if engine is None:
        return [{"content": "Memory system is not available.", "score": 0.0}]

    results = engine.search(query, limit=limit)
    output: list[dict[str, Any]] = []
    for item in results or []:
        content = getattr(item, "content", None)
        if content is None and isinstance(item, dict):
            content = item.get("content", "")
        score = getattr(item, "score", None)
        if score is None and isinstance(item, dict):
            score = item.get("score", 0.0)
        created_at = getattr(item, "created_at", None)
        if created_at is None and isinstance(item, dict):
            created_at = item.get("created_at")
        output.append(
            {
                "content": str(content or ""),
                "score": float(score or 0.0),
                "created_at": str(created_at) if created_at is not None else None,
            }
        )
    return output


@tool
def powermem_save(content: str, importance: str = "normal") -> str:
    """Save important information to long-term memory for future recall."""
    from app.memory import get_memory_engine

    engine = get_memory_engine()
    if engine is None:
        return "Memory system is not available. Information was not saved."

    engine.add(content, metadata={"importance": importance}, smart_process=True)
    preview = content[:50]
    return f"Memory saved successfully. Content: {preview}..."


@tool
async def mcp_bridge(
    server_id: str,
    tool_name: str,
    arguments: dict[str, Any] | None = None,
) -> str:
    """Call an MCP tool managed by the Rust MCP HTTP bridge."""
    import httpx

    settings = get_settings()
    url = f"{settings.mcp_bridge_url.rstrip('/')}/mcp/call_tool"

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                url,
                json={
                    "server_id": server_id,
                    "tool_name": tool_name,
                    "arguments": arguments or {},
                },
            )
    except httpx.TimeoutException:
        return "MCP call failed: request timed out"
    except httpx.HTTPError as exc:
        return f"MCP call failed: {exc}"

    if response.status_code != 200:
        return f"MCP call failed: {response.text}"

    payload = response.json()
    result = payload.get("result")
    if result is None:
        return "No result returned."
    return str(result)


def get_tools() -> list:
    """Return all custom tools for the main agent."""
    return [powermem_search, powermem_save, mcp_bridge]


def get_research_tools() -> list:
    """Return research-oriented tools for the researcher subagent."""
    return [powermem_search]
