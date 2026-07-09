"""Tests for custom agent tools."""

from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from app.tools import get_research_tools, get_tools, mcp_bridge, powermem_save, powermem_search


def test_get_tools_contains_core_tools():
    tools = get_tools()
    names = {getattr(tool, "name", None) or getattr(tool, "__name__", "") for tool in tools}
    assert "powermem_search" in names
    assert "powermem_save" in names
    assert "mcp_bridge" in names


def test_get_research_tools_is_subset():
    names = {
        getattr(tool, "name", None) or getattr(tool, "__name__", "")
        for tool in get_research_tools()
    }
    assert names == {"powermem_search"}


def test_powermem_search_when_engine_missing():
    with patch("app.memory.get_memory_engine", return_value=None):
        result = powermem_search.invoke({"query": "prefs", "limit": 3})
    assert isinstance(result, list)
    assert "not available" in result[0]["content"].lower()


def test_powermem_search_returns_engine_results():
    engine = MagicMock()
    engine.search.return_value = [
        SimpleNamespace(content="liked dark mode", score=0.9, created_at="2026-01-01"),
    ]
    with patch("app.memory.get_memory_engine", return_value=engine):
        result = powermem_search.invoke({"query": "prefs", "limit": 1})
    assert result[0]["content"] == "liked dark mode"
    assert result[0]["score"] == 0.9


def test_powermem_save_when_engine_missing():
    with patch("app.memory.get_memory_engine", return_value=None):
        result = powermem_save.invoke({"content": "remember this", "importance": "high"})
    assert "not available" in result.lower()


def test_powermem_save_success():
    engine = MagicMock()
    with patch("app.memory.get_memory_engine", return_value=engine):
        result = powermem_save.invoke({"content": "remember this", "importance": "high"})
    engine.add.assert_called_once()
    assert "saved" in result.lower()


@pytest.mark.asyncio
async def test_mcp_bridge_success():
    response = MagicMock()
    response.status_code = 200
    response.json.return_value = {"result": "ok-from-mcp"}
    response.text = "ok"

    mock_client = AsyncMock()
    mock_client.__aenter__.return_value = mock_client
    mock_client.post = AsyncMock(return_value=response)

    with patch("httpx.AsyncClient", return_value=mock_client):
        result = await mcp_bridge.ainvoke(
            {
                "server_id": "fs",
                "tool_name": "read_file",
                "arguments": {"path": "a.txt"},
            }
        )
    assert result == "ok-from-mcp"


@pytest.mark.asyncio
async def test_mcp_bridge_http_error():
    response = MagicMock()
    response.status_code = 500
    response.text = "boom"
    response.json.return_value = {"error": "boom"}

    mock_client = AsyncMock()
    mock_client.__aenter__.return_value = mock_client
    mock_client.post = AsyncMock(return_value=response)

    with patch("httpx.AsyncClient", return_value=mock_client):
        result = await mcp_bridge.ainvoke(
            {"server_id": "fs", "tool_name": "read_file", "arguments": {}}
        )
    assert "failed" in result.lower()


@pytest.mark.asyncio
async def test_mcp_bridge_missing_result_field():
    response = MagicMock()
    response.status_code = 200
    response.json.return_value = {}
    response.text = "{}"

    mock_client = AsyncMock()
    mock_client.__aenter__.return_value = mock_client
    mock_client.post = AsyncMock(return_value=response)

    with patch("httpx.AsyncClient", return_value=mock_client):
        result = await mcp_bridge.ainvoke(
            {"server_id": "fs", "tool_name": "read_file", "arguments": {}}
        )
    assert "no result" in result.lower()
