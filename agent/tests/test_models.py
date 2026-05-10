"""Tests for Pydantic v2 models."""

import pytest
from pydantic import ValidationError

from app.models import (
    ChatConfig,
    ChatMessage,
    ChatRequest,
    ChatResponse,
    HealthResponse,
    InfoResponse,
    MessageRole,
    TokenUsage,
    ToolCall,
)


class TestChatMessage:
    def test_valid_message(self):
        msg = ChatMessage(role=MessageRole.user, content="hello")
        assert msg.role == MessageRole.user
        assert msg.content == "hello"
        assert msg.name is None

    def test_all_roles(self):
        for role in ["system", "user", "assistant", "tool"]:
            msg = ChatMessage(role=role, content="test")
            assert msg.role == role

    def test_invalid_role_rejected(self):
        with pytest.raises(ValidationError):
            ChatMessage(role="invalid", content="test")

    def test_extra_fields_ignored(self):
        msg = ChatMessage(role="user", content="hi", unknown_field="x")
        assert not hasattr(msg, "unknown_field")


class TestChatConfig:
    def test_defaults(self):
        cfg = ChatConfig()
        assert cfg.model is None
        assert cfg.temperature == 0.7
        assert cfg.max_tokens is None
        assert cfg.stream is False

    def test_custom_values(self):
        cfg = ChatConfig(model="gpt-4", temperature=0.2, max_tokens=500, stream=True)
        assert cfg.model == "gpt-4"
        assert cfg.temperature == 0.2


class TestChatRequest:
    def test_minimal_request(self):
        req = ChatRequest(
            messages=[ChatMessage(role="user", content="hello")]
        )
        assert len(req.messages) == 1
        assert req.config.temperature == 0.7
        assert req.session_id is None

    def test_full_request(self):
        req = ChatRequest(
            messages=[
                ChatMessage(role="system", content="be helpful"),
                ChatMessage(role="user", content="hi"),
            ],
            config=ChatConfig(model="claude-3", temperature=0.5),
            session_id="sess-1",
            working_dir="/tmp",
        )
        assert req.session_id == "sess-1"
        assert req.config.model == "claude-3"


class TestToolCall:
    def test_basic(self):
        tc = ToolCall(id="tc-1", name="search", arguments={"q": "hello"})
        assert tc.id == "tc-1"
        assert tc.arguments["q"] == "hello"


class TestTokenUsage:
    def test_defaults(self):
        usage = TokenUsage()
        assert usage.prompt_tokens == 0
        assert usage.total_tokens == 0


class TestChatResponse:
    def test_minimal(self):
        resp = ChatResponse(
            message=ChatMessage(role="assistant", content="hi")
        )
        assert resp.message.content == "hi"
        assert resp.tool_calls == []
        assert resp.usage.total_tokens == 0


class TestHealthResponse:
    def test_defaults(self):
        hr = HealthResponse()
        assert hr.status == "ok"
        assert hr.version == "0.1.0"
        assert hr.agent_ready is False


class TestInfoResponse:
    def test_defaults(self):
        ir = InfoResponse()
        assert ir.name == "misaka-agent"
        assert ir.langgraph_available is False
