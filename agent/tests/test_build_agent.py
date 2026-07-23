"""Tests for DeepAgent assembly helpers."""

from pathlib import Path
from unittest.mock import MagicMock, patch

from app.agent import (
    SelectedSkill,
    _apply_mode_harness_profile,
    _build_subagents,
    build_agent,
    resolve_selected_skills,
)
from app.config import get_settings
from app.prompts import build_system_prompt
from app.workspace_backend import MEMORY_FILE, SKILLS_PREFIX, WorkspacePathBackend


def test_build_agent_mounts_workspace_for_valid_dir(tmp_path: Path):
    captured: dict = {}
    backend_factory = None

    def fake_create_deep_agent(**kwargs):
        nonlocal backend_factory
        captured.update(kwargs)
        backend_factory = kwargs.get("backend")
        return MagicMock(name="compiled-agent")

    with (
        patch("deepagents.create_deep_agent", side_effect=fake_create_deep_agent),
        patch("deepagents.backends.LocalShellBackend") as local_shell,
        patch("deepagents.backends.FilesystemBackend") as filesystem,
        patch("deepagents.backends.CompositeBackend") as composite,
        patch("deepagents.backends.StateBackend"),
        patch("app.agent._apply_mode_harness_profile") as apply_profile,
    ):
        shell = MagicMock(name="local-shell")
        local_shell.return_value = shell
        filesystem.side_effect = lambda **kwargs: MagicMock(name="fs", kwargs=kwargs)
        composite.return_value = MagicMock(name="composite")
        agent = build_agent(
            session_id="s1",
            working_dir=str(tmp_path),
            tools=[],
            agent_mode="chat",
        )

    assert agent is not None
    assert backend_factory is not None
    backend = backend_factory(MagicMock(name="runtime"))
    assert isinstance(backend, WorkspacePathBackend)
    local_shell.assert_called_once()
    call_kwargs = local_shell.call_args.kwargs
    assert Path(call_kwargs["root_dir"]).resolve() == tmp_path.resolve()
    assert call_kwargs["virtual_mode"] is True
    composite.assert_called_once()
    routes = composite.call_args.kwargs.get("routes") or composite.call_args.args[1]
    assert "/workspace/" in routes
    assert f"{SKILLS_PREFIX}/" in routes
    assert "/memories/" in routes
    assert captured["skills"] == [f"{SKILLS_PREFIX}/"]
    assert captured["memory"] == [MEMORY_FILE]
    assert captured["subagents"] == []
    assert "/workspace" in captured["system_prompt"]
    assert str(tmp_path.resolve()) not in captured["system_prompt"]
    apply_profile.assert_called_once_with("chat")


def test_build_agent_research_mode_includes_subagents(tmp_path: Path):
    captured: dict = {}

    def fake_create_deep_agent(**kwargs):
        captured.update(kwargs)
        return MagicMock(name="compiled-agent")

    with (
        patch("deepagents.create_deep_agent", side_effect=fake_create_deep_agent),
        patch("deepagents.backends.LocalShellBackend"),
        patch("deepagents.backends.FilesystemBackend"),
        patch("deepagents.backends.CompositeBackend"),
        patch("deepagents.backends.StateBackend"),
        patch("app.agent._apply_mode_harness_profile") as apply_profile,
    ):
        build_agent(
            session_id="s1",
            working_dir=str(tmp_path),
            tools=[],
            agent_mode="research",
        )

    names = [item["name"] for item in captured["subagents"]]
    assert names == ["researcher", "coder", "analyst"]
    apply_profile.assert_called_once_with("research")


def test_build_agent_uses_state_backend_without_working_dir():
    captured: dict = {}
    backend_factory = None

    def fake_create_deep_agent(**kwargs):
        nonlocal backend_factory
        captured.update(kwargs)
        backend_factory = kwargs.get("backend")
        return MagicMock(name="compiled-agent")

    with (
        patch("deepagents.create_deep_agent", side_effect=fake_create_deep_agent),
        patch("deepagents.backends.LocalShellBackend") as local_shell,
        patch("deepagents.backends.FilesystemBackend"),
        patch("deepagents.backends.CompositeBackend") as composite,
        patch("deepagents.backends.StateBackend") as state_backend,
        patch("app.agent._apply_mode_harness_profile"),
    ):
        composite.return_value = MagicMock(name="composite")
        state_backend.return_value = MagicMock(name="state")
        agent = build_agent(
            session_id="s1",
            working_dir=None,
            tools=[],
            agent_mode="chat",
        )

    assert agent is not None
    assert backend_factory is not None
    backend = backend_factory(MagicMock(name="runtime"))
    assert backend is composite.return_value
    local_shell.assert_not_called()
    state_backend.assert_called_once()
    routes = composite.call_args.kwargs.get("routes") or {}
    assert f"{SKILLS_PREFIX}/" in routes
    assert captured["skills"] == [f"{SKILLS_PREFIX}/"]
    assert captured["system_prompt"] == build_system_prompt(has_workspace=False)


def test_build_agent_ignores_invalid_working_dir(tmp_path: Path):
    missing = tmp_path / "missing-dir"
    backend_factory = None

    def fake_create_deep_agent(**kwargs):
        nonlocal backend_factory
        backend_factory = kwargs.get("backend")
        return MagicMock(name="compiled-agent")

    with (
        patch("deepagents.create_deep_agent", side_effect=fake_create_deep_agent),
        patch("deepagents.backends.LocalShellBackend") as local_shell,
        patch("deepagents.backends.FilesystemBackend"),
        patch("deepagents.backends.CompositeBackend") as composite,
        patch("deepagents.backends.StateBackend") as state_backend,
        patch("app.agent._apply_mode_harness_profile"),
    ):
        composite.return_value = MagicMock(name="composite")
        agent = build_agent(
            session_id="s1",
            working_dir=str(missing),
            tools=[],
            agent_mode="chat",
        )

    assert agent is not None
    backend_factory(MagicMock(name="runtime"))
    local_shell.assert_not_called()
    state_backend.assert_called_once()


def test_build_agent_uses_explicit_model_override():
    captured: dict = {}
    fake_model = MagicMock(name="openai-chat-model")

    def fake_create_deep_agent(**kwargs):
        captured.update(kwargs)
        return MagicMock(name="compiled-agent")

    with (
        patch("deepagents.create_deep_agent", side_effect=fake_create_deep_agent),
        patch("deepagents.backends.FilesystemBackend"),
        patch("deepagents.backends.CompositeBackend"),
        patch("deepagents.backends.StateBackend"),
        patch("app.agent._apply_mode_harness_profile"),
    ):
        build_agent(
            session_id="s1",
            tools=[],
            agent_mode="chat",
            model=fake_model,
        )

    assert captured["model"] is fake_model


def test_build_system_prompt_uses_workspace_not_host_path(tmp_path: Path):
    prompt = build_system_prompt(has_workspace=True, working_dir=str(tmp_path))
    assert "/workspace" in prompt
    assert str(tmp_path) not in prompt
    assert "Do NOT invent" not in prompt


def test_build_subagents_has_three_roles():
    settings = get_settings()
    subagents = _build_subagents(settings)

    names = [item["name"] for item in subagents]
    assert names == ["researcher", "coder", "analyst"]
    assert all(item["system_prompt"] for item in subagents)


def test_apply_mode_harness_profile_registers_providers():
    registered: dict = {}

    def fake_register(key, profile):
        registered[key] = profile

    with (
        patch("deepagents.register_harness_profile", side_effect=fake_register),
        patch("deepagents.GeneralPurposeSubagentProfile") as gp_cls,
        patch("deepagents.HarnessProfileConfig") as cfg_cls,
    ):
        gp_cls.side_effect = lambda **kwargs: kwargs
        cfg_cls.side_effect = lambda **kwargs: kwargs
        _apply_mode_harness_profile("chat")

    assert "openai" in registered
    assert "anthropic" in registered


def test_selected_skills_are_validated_and_added_to_prompt(tmp_path: Path):
    skill_dir = tmp_path / "code-review"
    skill_dir.mkdir()
    (skill_dir / "SKILL.md").write_text(
        "---\nname: code-review\ndescription: Review changes safely\n---\n",
        encoding="utf-8",
    )

    selected = resolve_selected_skills(["code-review"], tmp_path)
    prompt = build_system_prompt(selected_skills=selected)

    assert selected == [SelectedSkill("code-review", "Review changes safely")]
    assert "/skills/<slug>/SKILL.md" in prompt
    assert "`code-review`: Review changes safely" in prompt


def test_selected_skills_reject_missing_or_invalid_slugs(tmp_path: Path):
    try:
        resolve_selected_skills(["../outside"], tmp_path)
    except ValueError as exc:
        assert "Invalid selected Skill" in str(exc)
    else:
        raise AssertionError("Expected invalid Skill slug to fail")

    try:
        resolve_selected_skills(["missing-skill"], tmp_path)
    except ValueError as exc:
        assert "not installed" in str(exc)
    else:
        raise AssertionError("Expected missing Skill to fail")
