"""Pydantic v2 models for MisakaX Agent Sidecar API."""

from __future__ import annotations

from enum import Enum
from typing import Any

from pydantic import BaseModel, Field


# --------------------------------------------------------------------------- #
#  Chat / Agent models
# --------------------------------------------------------------------------- #

class MessageRole(str, Enum):
    system = "system"
    user = "user"
    assistant = "assistant"
    tool = "tool"


class ChatMessage(BaseModel):
    """A single message in a chat conversation."""

    role: MessageRole
    content: str
    name: str | None = None
    tool_call_id: str | None = None

    model_config = {"frozen": False, "extra": "ignore"}


class ToolCall(BaseModel):
    """Representation of a tool/function call requested by the model."""

    id: str
    name: str
    arguments: dict[str, Any] = Field(default_factory=dict)

    model_config = {"frozen": False, "extra": "ignore"}


class TokenUsage(BaseModel):
    """Token usage statistics for a single request."""

    prompt_tokens: int = 0
    completion_tokens: int = 0
    total_tokens: int = 0

    model_config = {"frozen": False, "extra": "ignore"}


class ChatConfig(BaseModel):
    """Configuration for a chat/agent request.

    Provider binding fields (`provider` / `api_compat` / `base_url` / `api_key`)
    are filled by Rust from the selected RouterConfig so Sidecar uses the same
    protocol as the settings UI (not a hardcoded Anthropic default).
    """

    model: str | None = None
    temperature: float = 0.7
    max_tokens: int | None = None
    stream: bool = False
    provider: str | None = None
    api_compat: str | None = None
    base_url: str | None = None
    api_key: str | None = None

    model_config = {"frozen": False, "extra": "ignore"}


class ChatRequest(BaseModel):
    """Request body for POST /agent/chat and /agent/stream."""

    messages: list[ChatMessage]
    config: ChatConfig = Field(default_factory=ChatConfig)
    session_id: str | None = None
    working_dir: str | None = None

    model_config = {"frozen": False, "extra": "ignore"}


class ChatResponse(BaseModel):
    """Response body for POST /agent/chat."""

    message: ChatMessage
    tool_calls: list[ToolCall] = Field(default_factory=list)
    usage: TokenUsage = Field(default_factory=TokenUsage)
    model: str | None = None

    model_config = {"frozen": False, "extra": "ignore"}


# --------------------------------------------------------------------------- #
#  Health / Info response models
# --------------------------------------------------------------------------- #

class HealthResponse(BaseModel):
    """Response body for GET /health."""

    status: str = "ok"
    service: str = "misaka-agent"
    version: str = "0.1.0"
    uptime_seconds: float = 0.0
    capabilities: list[str] = Field(default_factory=list)
    agent_ready: bool = False

    model_config = {"frozen": False, "extra": "ignore"}


class InfoResponse(BaseModel):
    """Response body for GET /info."""

    name: str = "misaka-agent"
    version: str = "0.1.0"
    python_version: str = ""
    langgraph_available: bool = False
    powermem_available: bool = False

    model_config = {"frozen": False, "extra": "ignore"}


# --------------------------------------------------------------------------- #
#  Memory models
# --------------------------------------------------------------------------- #

class MemoryItem(BaseModel):
    """A single normalized memory entry returned by the memory REST API."""

    id: str | None = None
    content: str = ""
    score: float | None = None
    created_at: str | None = None
    metadata: dict[str, Any] = Field(default_factory=dict)

    model_config = {"frozen": False, "extra": "ignore"}


class MemoryListResponse(BaseModel):
    """Paginated list of memory items."""

    items: list[MemoryItem]
    offset: int = 0
    limit: int = 20

    model_config = {"frozen": False, "extra": "ignore"}
