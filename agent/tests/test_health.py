"""Tests for the health check endpoint."""

import pytest
from fastapi import FastAPI

from app.routers import health


@pytest.mark.asyncio
async def test_health_check_returns_ok(client):
    response = await client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "ok"
    assert data["service"] == "misaka-agent"


@pytest.mark.asyncio
async def test_health_check_has_enhanced_fields(client):
    response = await client.get("/health")
    data = response.json()
    assert "version" in data
    assert data["version"] == "0.1.0"
    assert "uptime_seconds" in data
    assert isinstance(data["uptime_seconds"], (int, float))
    assert "capabilities" in data
    assert isinstance(data["capabilities"], list)
    assert "health" in data["capabilities"]
    assert "info" in data["capabilities"]
    assert "agent_ready" in data
    if data["agent_ready"]:
        assert "agent_stream" in data["capabilities"]


def test_health_capabilities_are_cached(monkeypatch):
    application = FastAPI()
    calls = {"agent": 0, "deepagents": 0, "powermem": 0}

    def agent_ready() -> bool:
        calls["agent"] += 1
        return True

    def deepagents_available() -> bool:
        calls["deepagents"] += 1
        return True

    def powermem_available() -> bool:
        calls["powermem"] += 1
        return True

    monkeypatch.setattr(health, "_agent_module_ready", agent_ready)
    monkeypatch.setattr(health, "_deepagents_available", deepagents_available)
    monkeypatch.setattr(health, "_langgraph_available", lambda: False)
    monkeypatch.setattr(health, "_powermem_available", powermem_available)

    first = health.get_cached_health_capabilities(application)
    second = health.get_cached_health_capabilities(application)

    assert first is second
    assert first.agent_ready is True
    assert first.capabilities == ("health", "info", "agent", "agent_stream", "memory")
    assert calls == {"agent": 1, "deepagents": 1, "powermem": 1}


@pytest.mark.asyncio
async def test_health_check_is_get_only(client):
    response = await client.post("/health")
    assert response.status_code == 405


@pytest.mark.asyncio
async def test_openapi_schema_accessible(client):
    response = await client.get("/openapi.json")
    assert response.status_code == 200
    schema = response.json()
    assert schema["info"]["title"] == "MisakaX Agent"
    assert schema["info"]["version"] == "0.1.0"
