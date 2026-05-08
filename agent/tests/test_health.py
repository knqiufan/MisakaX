"""Tests for the health check endpoint."""

import pytest


@pytest.mark.asyncio
async def test_health_check_returns_ok(client):
    response = await client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "ok"
    assert data["service"] == "misaka-agent"


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
