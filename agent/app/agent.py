"""MisakaX DeepAgent assembly."""

from __future__ import annotations

import logging
import re
import threading
import hashlib
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal

from app.config import get_settings
from app.prompts import (
    ANALYST_PROMPT,
    CODER_PROMPT,
    RESEARCHER_PROMPT,
    SYSTEM_PROMPT,
    build_system_prompt,
)
from app.workspace_backend import (
    MEMORY_FILE,
    SKILLS_PREFIX,
    make_workspace_backend_factory,
)
from app.models import SelectedSkillMount

logger = logging.getLogger(__name__)

AgentMode = Literal["chat", "research"]
_SKILL_SLUG_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")

_HARNESS_LOCK = threading.Lock()
_PROVIDER_PROFILE_KEYS = (
    "openai",
    "anthropic",
    "google",
    "google_genai",
    "azure_openai",
    "bedrock",
)


@dataclass(frozen=True)
class SelectedSkill:
    skill_id: str
    slug: str
    description: str
    directory: Path
    artifact_hash: str


def build_agent(
    session_id: str | None = None,
    working_dir: str | None = None,
    checkpointer=None,
    store=None,
    tools: list[Any] | None = None,
    include_subagents: bool | None = None,
    agent_mode: AgentMode = "chat",
    model: Any | None = None,
    selected_skills: list[SelectedSkill] | None = None,
    active_skills: list[SelectedSkill] | None = None,
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
            FilesystemBackend,
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
        "system_prompt": build_system_prompt(
            has_workspace=validated is not None,
            selected_skills=selected_skills or [],
        ),
        "subagents": subagents,
        "interrupt_on": settings.interrupt_config,
        "checkpointer": checkpointer,
        "store": store,
    }

    # DeepAgents skills/memory paths are backend-virtual, not host absolute paths.
    # Host dirs are mounted at /skills and /memories via CompositeBackend routes.
    kwargs["skills"] = [f"{SKILLS_PREFIX}/"] if active_skills else []
    kwargs["memory"] = [MEMORY_FILE]
    kwargs["backend"] = make_workspace_backend_factory(
        validated,
        LocalShellBackend,
        StateBackend,
        CompositeBackend,
        filesystem_cls=FilesystemBackend,
        selected_skill_dirs={skill.slug: skill.directory for skill in active_skills or []},
        memories_dir=settings.memories_dir,
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


def resolve_selected_skills(
    selected_skill_mounts: list[SelectedSkillMount],
    skills_dir: Path,
) -> list[SelectedSkill]:
    """Validate Rust-provided mounts before exposing them to the Agent."""
    if len(selected_skill_mounts) > 20:
        raise ValueError("At most 20 Skills can be selected for one request")
    resolved: list[SelectedSkill] = []
    seen: set[str] = set()
    allowed_roots = _allowed_skill_roots(skills_dir)
    for mount in selected_skill_mounts:
        slug = mount.slug
        if slug in seen:
            continue
        seen.add(slug)
        _validate_skill_slug(slug)
        directory = _validate_selected_skill_directory(mount.path, allowed_roots)
        markdown = _read_selected_skill_markdown(directory)
        name = _frontmatter_value(markdown, "name")
        if name != slug:
            raise ValueError(f"Selected Skill '{slug}' has an invalid manifest")
        description = _frontmatter_value(markdown, "description")
        if not description:
            raise ValueError(f"Selected Skill '{slug}' has no description")
        resolved.append(
            SelectedSkill(
                skill_id=mount.skill_id,
                slug=slug,
                description=description,
                directory=directory,
                artifact_hash=mount.artifact_hash,
            )
        )
        if mount.artifact_hash and _artifact_hash(directory) != mount.artifact_hash:
            raise ValueError(f"Selected Skill '{slug}' changed after Rust activation")
    return resolved


def _artifact_hash(root: Path) -> str:
    """Mirror Rust's deterministic S1 artifact hash before a directory is mounted."""
    digest = hashlib.sha256()
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        digest.update(path.relative_to(root).as_posix().encode())
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def _validate_skill_slug(slug: str) -> None:
    if not _SKILL_SLUG_RE.fullmatch(slug):
        raise ValueError(f"Invalid selected Skill identifier: {slug!r}")


def _allowed_skill_roots(skills_dir: Path) -> tuple[Path, ...]:
    home = Path.home()
    return (
        skills_dir.resolve(),
        (home / ".codex" / "skills").resolve(),
        (home / ".claude" / "skills").resolve(),
        (home / ".cursor" / "skills").resolve(),
    )


def _validate_selected_skill_directory(raw: str, allowed_roots: tuple[Path, ...]) -> Path:
    try:
        directory = Path(raw).resolve(strict=True)
    except (OSError, RuntimeError) as exc:
        raise ValueError("Selected Skill is not installed") from exc
    if not directory.is_dir() or not any(
        directory == root or root in directory.parents for root in allowed_roots
    ):
        raise ValueError("Selected Skill is outside an allowed Skills directory")
    return directory


def _read_selected_skill_markdown(directory: Path) -> str:
    candidate = directory / "SKILL.md"
    if not candidate.is_file():
        raise ValueError("Selected Skill is not installed")
    return candidate.read_text(encoding="utf-8")


def _frontmatter_value(markdown: str, key: str) -> str | None:
    if not markdown.startswith("---"):
        return None
    end = markdown.find("\n---", 3)
    if end < 0:
        return None
    for line in markdown[3:end].splitlines():
        name, separator, value = line.partition(":")
        if separator and name.strip() == key:
            return value.strip().strip("\"'")
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
    "resolve_selected_skills",
    "_build_subagents",
    "_resolve_working_dir",
    "_apply_mode_harness_profile",
]
