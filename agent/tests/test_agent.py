"""Tests for the agent chat/stream endpoints."""

from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from app.models import ChatRequest
from app.routers.agent import _thread_config
from app.agent import _artifact_hash


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
async def test_agent_chat_returns_assistant_response(client):
    last_msg = SimpleNamespace(content="Hello from agent", tool_calls=[])
    mock_agent = MagicMock()
    mock_agent.ainvoke = AsyncMock(return_value={"messages": [last_msg]})

    with (
        patch("app.routers.agent.build_agent", return_value=mock_agent),
        patch("app.routers.agent.get_checkpointer", return_value=None),
        patch("app.routers.agent.get_store", return_value=None),
    ):
        payload = {
            "messages": [{"role": "user", "content": "hello"}],
            "session_id": "sess-1",
        }
        response = await client.post("/agent/chat", json=payload)

    assert response.status_code == 200
    data = response.json()
    assert data["message"]["role"] == "assistant"
    assert data["message"]["content"] == "Hello from agent"
    mock_agent.ainvoke.assert_awaited_once()


@pytest.mark.asyncio
async def test_agent_chat_accepts_full_request(client):
    last_msg = SimpleNamespace(content="ok", tool_calls=[])
    mock_agent = MagicMock()
    mock_agent.ainvoke = AsyncMock(return_value={"messages": [last_msg]})

    with (
        patch("app.routers.agent.build_agent", return_value=mock_agent),
        patch("app.routers.agent.get_checkpointer", return_value=None),
        patch("app.routers.agent.get_store", return_value=None),
    ):
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

    assert response.status_code == 200
    assert response.json()["message"]["content"] == "ok"


@pytest.mark.asyncio
async def test_agent_chat_uses_generation_stamped_activation_view(client, tmp_path):
    skill_dir = tmp_path / "demo"
    skill_dir.mkdir()
    (skill_dir / "SKILL.md").write_text(
        "---\nname: demo\ndescription: Demo Skill\n---\n", encoding="utf-8"
    )
    last_msg = SimpleNamespace(content="ok", tool_calls=[])
    mock_agent = MagicMock()
    mock_agent.ainvoke = AsyncMock(return_value={"messages": [last_msg]})

    with (
        patch("app.routers.agent.build_agent", return_value=mock_agent) as build,
        patch(
            "app.routers.agent.get_settings",
            return_value=SimpleNamespace(
                skills_dir=tmp_path,
                agent_recursion_limit=100,
                agent_model="test-model",
            ),
        ),
        patch("app.routers.agent.get_checkpointer", return_value=None),
        patch("app.routers.agent.get_store", return_value=None),
    ):
        response = await client.post(
            "/agent/chat",
            json={
                "messages": [{"role": "user", "content": "hello"}],
                "selected_skill_ids": ["stable-demo-id"],
                "skill_activation": {
                    "generation": 7,
                    "skills": [
                        {
                            "skill_id": "stable-demo-id",
                            "slug": "demo",
                            "path": str(skill_dir),
                            "artifact_hash": _artifact_hash(skill_dir),
                        }
                    ],
                },
            },
        )

    assert response.status_code == 200
    assert [skill.skill_id for skill in build.call_args.kwargs["active_skills"]] == [
        "stable-demo-id"
    ]
    assert [skill.skill_id for skill in build.call_args.kwargs["selected_skills"]] == [
        "stable-demo-id"
    ]


@pytest.mark.asyncio
async def test_agent_chat_rejects_slug_only_skill_fallback(client):
    response = await client.post(
        "/agent/chat",
        json={
            "messages": [{"role": "user", "content": "hello"}],
            "selected_skill_ids": ["legacy-slug"],
        },
    )
    assert response.status_code == 400
    assert "activation view" in response.json()["detail"]


@pytest.mark.asyncio
async def test_agent_chat_rejects_changed_activation_artifact(client, tmp_path):
    skill_dir = tmp_path / "demo"
    skill_dir.mkdir()
    (skill_dir / "SKILL.md").write_text(
        "---\nname: demo\ndescription: Changed\n---\n", encoding="utf-8"
    )
    with patch(
        "app.routers.agent.get_settings",
        return_value=SimpleNamespace(
            skills_dir=tmp_path,
            agent_recursion_limit=100,
            agent_model="test-model",
        ),
    ):
        response = await client.post(
            "/agent/chat",
            json={
                "messages": [{"role": "user", "content": "hello"}],
                "skill_activation": {
                    "generation": 2,
                    "skills": [
                        {
                            "skill_id": "stable-demo-id",
                            "slug": "demo",
                            "path": str(skill_dir),
                            "artifact_hash": "stale-hash",
                        }
                    ],
                },
            },
        )
    assert response.status_code == 400
    assert "changed after Rust activation" in response.json()["detail"]


@pytest.mark.asyncio
async def test_agent_chat_returns_500_on_failure(client):
    mock_agent = MagicMock()
    mock_agent.ainvoke = AsyncMock(side_effect=RuntimeError("boom"))

    with (
        patch("app.routers.agent.build_agent", return_value=mock_agent),
        patch("app.routers.agent.get_checkpointer", return_value=None),
        patch("app.routers.agent.get_store", return_value=None),
    ):
        response = await client.post(
            "/agent/chat",
            json={"messages": [{"role": "user", "content": "hello"}]},
        )

    assert response.status_code == 500
    assert "boom" in response.json()["detail"]


def test_thread_config_raises_langgraph_recursion_limit():
    chat_req = ChatRequest(
        messages=[{"role": "user", "content": "hello"}],
        session_id="sess-1",
        agent_mode="chat",
    )
    research_req = ChatRequest(
        messages=[{"role": "user", "content": "hello"}],
        session_id="sess-1",
        agent_mode="research",
    )

    chat_config = _thread_config(chat_req)
    research_config = _thread_config(research_req)

    assert chat_config["configurable"]["thread_id"] == "sess-1"
    assert chat_config["recursion_limit"] == 50
    assert research_config["recursion_limit"] == 100


@pytest.mark.asyncio
async def test_agent_stream_emits_token_and_done(client):
    class Chunk:
        content = "Hi"

    async def fake_events(*_args, **_kwargs):
        yield {
            "event": "on_chat_model_stream",
            "data": {"chunk": Chunk()},
        }

    mock_agent = MagicMock()
    mock_agent.astream_events = fake_events

    with (
        patch("app.routers.agent.build_agent", return_value=mock_agent),
        patch("app.routers.agent.get_checkpointer", return_value=None),
        patch("app.routers.agent.get_store", return_value=None),
    ):
        response = await client.post(
            "/agent/stream",
            json={"messages": [{"role": "user", "content": "hello"}]},
        )

    assert response.status_code == 200
    body = response.text
    assert "event: token" in body
    assert "event: done" in body


@pytest.mark.asyncio
async def test_agent_stream_emits_error_event(client):
    async def failing_events(*_args, **_kwargs):
        raise RuntimeError("stream failed")
        yield  # pragma: no cover

    mock_agent = MagicMock()
    mock_agent.astream_events = failing_events

    with (
        patch("app.routers.agent.build_agent", return_value=mock_agent),
        patch("app.routers.agent.get_checkpointer", return_value=None),
        patch("app.routers.agent.get_store", return_value=None),
    ):
        response = await client.post(
            "/agent/stream",
            json={"messages": [{"role": "user", "content": "hello"}]},
        )

    assert response.status_code == 200
    assert "event: error" in response.text
    assert "stream failed" in response.text
