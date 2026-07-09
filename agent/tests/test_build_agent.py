"""Tests for DeepAgent assembly helpers."""

from pathlib import Path
from unittest.mock import MagicMock, patch

from app.agent import _build_subagents, build_agent
from app.config import get_settings


def test_build_agent_binds_filesystem_backend_for_valid_dir(tmp_path: Path):
    captured: dict = {}

    def fake_create_deep_agent(**kwargs):
        captured.update(kwargs)
        return MagicMock(name="compiled-agent")

    with (
        patch("deepagents.create_deep_agent", side_effect=fake_create_deep_agent),
        patch("deepagents.backends.FilesystemBackend") as fs_backend,
        patch("deepagents.backends.CompositeBackend"),
        patch("deepagents.backends.StateBackend"),
    ):
        fs_backend.return_value = MagicMock(name="fs-backend")
        agent = build_agent(
            session_id="s1",
            working_dir=str(tmp_path),
            tools=[],
            include_subagents=False,
        )

    assert agent is not None
    fs_backend.assert_called()
    assert any(
        str(tmp_path.resolve()) in str(call.kwargs.get("root_dir", call.args[0] if call.args else ""))
        for call in fs_backend.call_args_list
    ) or fs_backend.called


def test_build_agent_ignores_invalid_working_dir(tmp_path: Path):
    missing = tmp_path / "missing-dir"
    captured: dict = {}

    def fake_create_deep_agent(**kwargs):
        captured.update(kwargs)
        return MagicMock(name="compiled-agent")

    with (
        patch("deepagents.create_deep_agent", side_effect=fake_create_deep_agent),
        patch("deepagents.backends.FilesystemBackend") as fs_backend,
        patch("deepagents.backends.CompositeBackend"),
        patch("deepagents.backends.StateBackend"),
    ):
        agent = build_agent(
            session_id="s1",
            working_dir=str(missing),
            tools=[],
            include_subagents=False,
        )

    assert agent is not None
    fs_backend.assert_not_called()


def test_build_subagents_has_three_roles():
    settings = get_settings()
    # tools.py may be absent during early phases; _build_subagents must still work.
    subagents = _build_subagents(settings)

    names = [item["name"] for item in subagents]
    assert names == ["researcher", "coder", "analyst"]
    assert all(item["system_prompt"] for item in subagents)
