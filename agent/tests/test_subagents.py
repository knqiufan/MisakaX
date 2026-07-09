"""Tests for SubAgent configuration."""

from unittest.mock import patch

from app.agent import _build_subagents
from app.config import get_settings


def test_build_subagents_binds_expected_tools():
    settings = get_settings()
    fake_research = ["research-tool"]
    fake_search = "powermem_search_tool"

    with patch("app.tools.get_research_tools", return_value=fake_research):
        with patch("app.tools.powermem_search", fake_search):
            subagents = _build_subagents(settings)

    by_name = {item["name"]: item for item in subagents}
    assert set(by_name) == {"researcher", "coder", "analyst"}

    assert by_name["researcher"]["tools"] == fake_research
    assert by_name["coder"]["tools"] == []
    assert by_name["analyst"]["tools"] == [fake_search]
    assert all(item["system_prompt"].strip() for item in subagents)
    assert all(item["description"].strip() for item in subagents)
