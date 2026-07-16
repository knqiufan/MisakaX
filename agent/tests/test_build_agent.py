"""Tests for DeepAgent assembly helpers."""

from pathlib import Path
from unittest.mock import MagicMock, patch

from app.agent import _build_subagents, build_agent
from app.config import get_settings
from app.prompts import build_system_prompt


def test_build_agent_binds_local_shell_as_default_for_valid_dir(tmp_path: Path):
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
        patch("deepagents.backends.CompositeBackend"),
        patch("deepagents.backends.StateBackend"),
    ):
        local_shell.return_value = MagicMock(name="local-shell")
        agent = build_agent(
            session_id="s1",
            working_dir=str(tmp_path),
            tools=[],
            include_subagents=False,
        )

    assert agent is not None
    assert backend_factory is not None
    backend = backend_factory(MagicMock(name="runtime"))
    assert backend is local_shell.return_value
    local_shell.assert_called_once()
    call_kwargs = local_shell.call_args.kwargs
    assert Path(call_kwargs["root_dir"]).resolve() == tmp_path.resolve()
    assert call_kwargs["virtual_mode"] is True
    assert call_kwargs["inherit_env"] is True
    assert str(tmp_path.resolve()) in captured["system_prompt"]
    assert "/workspace" not in captured["system_prompt"] or "Do NOT invent a `/workspace`" in captured["system_prompt"]


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
        patch("deepagents.backends.CompositeBackend") as composite,
        patch("deepagents.backends.StateBackend") as state_backend,
    ):
        composite.return_value = MagicMock(name="composite")
        state_backend.return_value = MagicMock(name="state")
        agent = build_agent(
            session_id="s1",
            working_dir=None,
            tools=[],
            include_subagents=False,
        )

    assert agent is not None
    assert backend_factory is not None
    backend = backend_factory(MagicMock(name="runtime"))
    assert backend is composite.return_value
    local_shell.assert_not_called()
    state_backend.assert_called_once()
    assert captured["system_prompt"] == build_system_prompt(None)


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
        patch("deepagents.backends.CompositeBackend") as composite,
        patch("deepagents.backends.StateBackend") as state_backend,
    ):
        composite.return_value = MagicMock(name="composite")
        agent = build_agent(
            session_id="s1",
            working_dir=str(missing),
            tools=[],
            include_subagents=False,
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
        patch("deepagents.backends.CompositeBackend"),
        patch("deepagents.backends.StateBackend"),
    ):
        build_agent(
            session_id="s1",
            tools=[],
            include_subagents=False,
            model=fake_model,
        )

    assert captured["model"] is fake_model


def test_build_system_prompt_includes_working_dir_rules(tmp_path: Path):
    prompt = build_system_prompt(str(tmp_path))
    assert str(tmp_path) in prompt
    assert "Do NOT invent a `/workspace` prefix" in prompt


def test_build_subagents_has_three_roles():
    settings = get_settings()
    # tools.py may be absent during early phases; _build_subagents must still work.
    subagents = _build_subagents(settings)

    names = [item["name"] for item in subagents]
    assert names == ["researcher", "coder", "analyst"]
    assert all(item["system_prompt"] for item in subagents)
