"""Tests for the agent chat/stream placeholder endpoints."""

import pytest


@pytest.mark.asyncio
async def test_agent_chat_returns_501(client):
    payload = {
        "messages": [{"role": "user", "content": "hello"}],
    }
    response = await client.post("/agent/chat", json=payload)
    assert response.status_code == 501
    data = response.json()
    assert "not yet implemented" in data["detail"].lower()


@pytest.mark.asyncio
async def test_agent_stream_returns_501(client):
    payload = {
        "messages": [{"role": "user", "content": "hello"}],
    }
    response = await client.post("/agent/stream", json=payload)
    assert response.status_code == 501
    data = response.json()
    assert "not yet implemented" in data["detail"].lower()


@pytest.mark.asyncio
async def test_agent_chat_requires_messages(client):
    response = await client.post("/agent/chat", json={})
    assert response.status_code == 422


@pytest.mark.asyncio
async def test_agent_chat_validates_message_role(client):
    payload = {
        "messages": [{"role": "invalid_role", "content": "hello"}],
    }
    response = await client.post("/agent/chat", json=payload)
    assert response.status_code == 422


@pytest.mark.asyncio
async def test_agent_chat_accepts_full_request(client):
    payload = {
        "messages": [
            {"role": "system", "content": "You are helpful."},
            {"role": "user", "content": "Hello"},
        ],
        "config": {
            "model": "test-model",
            "temperature": 0.5,
            "max_tokens": 100,
            "stream": False,
        },
        "session_id": "test-session-123",
        "working_dir": "/tmp/test",
    }
    response = await client.post("/agent/chat", json=payload)
    assert response.status_code == 501
