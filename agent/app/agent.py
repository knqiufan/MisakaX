"""MisakaX DeepAgent assembly."""

from __future__ import annotations

import logging
import threading
from typing import Any, Literal

from app.config import get_settings
from app.prompts import (
    ANALYST_PROMPT,
    CODER_PROMPT,
    RESEARCHER_PROMPT,
    SYSTEM_PROMPT,
    build_system_prompt,
)
from app.workspace_backend import make_workspace_backend_factory

logger = logging.getLogger(__name__)

AgentMode = Literal["chat", "research"]

_HARNESS_LOCK = threading.Lock()
_PROVIDER_PROFILE_KEYS = (
    "openai",
    "anthropic",
    "google",
    "google_genai",
    "azure_openai",
    "bedrock",
)


def build_agent(
    session_id: str | None = None,
    working_dir: str | None = None,
    checkpointer=None,
    store=None,
    tools: list[Any] | None = None,
    include_subagents: bool | None = None,
    agent_mode: AgentMode = "chat",
    model: Any | None = None,
):
    """Create a DeepAgent graph for the given session context.

    ``agent_mode="chat"`` (default) disables the ``task`` tool and all sync
    subagents, including DeepAgents' auto-injected general-purpose agent.
    ``agent_mode="research"`` enables researcher/coder/analyst plus GP.
    """
    del session_id  # reserved for future per-session customization
    settings = get_settings()

    try:
        from deepagents import create_deep_agent
        from deepagents.backends import (
            CompositeBackend,
            LocalShellBackend,
            StateBackend,
        )
    except ImportError as exc:
        raise RuntimeError(
            "deepagents is not installed. Install with: pip install -e '.[agent]'"
        ) from exc

    mode = _resolve_agent_mode(agent_mode, include_subagents)
    validated = _resolve_working_dir(working_dir)
    agent_tools = tools if tools is not None else _safe_get_tools()
    subagents = _build_subagents(settings) if mode == "research" else []

    kwargs: dict[str, Any] = {
        "model": model if model is not None else settings.agent_model,
        "tools": agent_tools,
        "system_prompt": build_system_prompt(has_workspace=validated is not None),
        "subagents": subagents,
        "interrupt_on": settings.interrupt_config,
        "checkpointer": checkpointer,
        "store": store,
    }

    kwargs["skills"] = [str(settings.skills_dir)]
    kwargs["memory"] = [str(settings.memories_dir)]
    kwargs["backend"] = make_workspace_backend_factory(
        validated, LocalShellBackend, StateBackend, CompositeBackend
    )

    with _HARNESS_LOCK:
        _apply_mode_harness_profile(mode)
        return create_deep_agent(**kwargs)


def _resolve_agent_mode(
    agent_mode: AgentMode,
    include_subagents: bool | None,
) -> AgentMode:
    if include_subagents is None:
        return "research" if agent_mode == "research" else "chat"
    # Backward-compatible override used by older tests/callers.
    return "research" if include_subagents else "chat"


def _apply_mode_harness_profile(mode: AgentMode) -> None:
    """Register provider harness profiles so chat mode truly drops ``task``."""
    try:
        from deepagents import (
            GeneralPurposeSubagentProfile,
            HarnessProfileConfig,
            register_harness_profile,
        )
    except ImportError:
        logger.warning("Harness profile APIs unavailable; cannot disable general-purpose")
        return

    enabled = mode == "research"
    profile = HarnessProfileConfig(
        general_purpose_subagent=GeneralPurposeSubagentProfile(enabled=enabled)
    )
    for key in _PROVIDER_PROFILE_KEYS:
        register_harness_profile(key, profile)


def _resolve_working_dir(working_dir: str | None):
    if not working_dir:
        return None
    try:
        from app.utils import validate_working_dir

        return validate_working_dir(working_dir)
    except ValueError as exc:
        logger.warning("Ignoring invalid working_dir: %s", exc)
        return None
    except Exception as exc:  # pragma: no cover - defensive
        logger.warning("working_dir validation failed: %s", exc)
        return None


def _safe_get_tools() -> list[Any]:
    try:
        from app.tools import get_tools

        return get_tools()
    except Exception as exc:
        logger.warning("Custom tools unavailable: %s", exc)
        return []


def _build_subagents(settings) -> list[dict[str, Any]]:
    """Build SubAgent configs as plain dicts for deepagents compatibility."""
    del settings
    research_tools: list[Any] = []
    analyst_tools: list[Any] = []
    try:
        from app.tools import get_research_tools, powermem_search

        research_tools = get_research_tools()
        analyst_tools = [powermem_search]
    except Exception as exc:
        logger.warning("SubAgent tools unavailable: %s", exc)

    return [
        {
            "name": "researcher",
            "description": (
                "Deep research specialist for information gathering and synthesis"
            ),
            "system_prompt": RESEARCHER_PROMPT,
            "tools": research_tools,
        },
        {
            "name": "coder",
            "description": "Code generation, review, and refactoring specialist",
            "system_prompt": CODER_PROMPT,
            "tools": [],
        },
        {
            "name": "analyst",
            "description": "Problem decomposition and structured analysis specialist",
            "system_prompt": ANALYST_PROMPT,
            "tools": analyst_tools,
        },
    ]


# Re-export for tests that imported SYSTEM_PROMPT via this module historically.
__all__ = [
    "SYSTEM_PROMPT",
    "build_agent",
    "_build_subagents",
    "_resolve_working_dir",
    "_apply_mode_harness_profile",
]
