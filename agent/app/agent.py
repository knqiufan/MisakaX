"""MisakaX DeepAgent assembly."""

from __future__ import annotations

import logging
from typing import Any

from app.config import get_settings
from app.prompts import (
    ANALYST_PROMPT,
    CODER_PROMPT,
    RESEARCHER_PROMPT,
    SYSTEM_PROMPT,
)

logger = logging.getLogger(__name__)


def build_agent(
    session_id: str | None = None,
    working_dir: str | None = None,
    checkpointer=None,
    store=None,
    tools: list[Any] | None = None,
    include_subagents: bool = True,
):
    """Create a DeepAgent graph for the given session context."""
    del session_id  # reserved for future per-session customization
    settings = get_settings()

    try:
        from deepagents import create_deep_agent
        from deepagents.backends import CompositeBackend, FilesystemBackend, StateBackend
    except ImportError as exc:
        raise RuntimeError(
            "deepagents is not installed. Install with: pip install -e '.[agent]'"
        ) from exc

    routes: dict[str, object] = {}
    validated = _resolve_working_dir(working_dir)
    if validated is not None:
        routes["/workspace/"] = FilesystemBackend(root_dir=str(validated))

    agent_tools = tools if tools is not None else _safe_get_tools()
    subagents = _build_subagents(settings) if include_subagents else []

    kwargs: dict[str, Any] = {
        "model": settings.agent_model,
        "tools": agent_tools,
        "system_prompt": SYSTEM_PROMPT,
        "subagents": subagents,
        "interrupt_on": settings.interrupt_config,
        "checkpointer": checkpointer,
        "store": store,
    }

    kwargs["skills"] = [str(settings.skills_dir)]
    kwargs["memory"] = [str(settings.memories_dir)]
    kwargs["backend"] = lambda rt: CompositeBackend(
        default=StateBackend(rt),
        routes=routes,
    )
    return create_deep_agent(**kwargs)


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
