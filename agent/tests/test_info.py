"""Tests for the info endpoint."""

import pytest


@pytest.mark.asyncio
async def test_info_returns_metadata(client):
    response = await client.get("/info")
    assert response.status_code == 200
    data = response.json()
    assert data["name"] == "misaka-agent"
    assert data["version"] == "0.1.0"
    assert "python_version" in data
    assert len(data["python_version"]) > 0
    assert isinstance(data["langgraph_available"], bool)
    assert isinstance(data["powermem_available"], bool)


@pytest.mark.asyncio
async def test_info_is_get_only(client):
    response = await client.post("/info")
    assert response.status_code == 405
