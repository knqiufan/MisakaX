"""Tests for agent prompt constants."""

from app.prompts import (
    ANALYST_PROMPT,
    CODER_PROMPT,
    RESEARCHER_PROMPT,
    SYSTEM_PROMPT,
)


def _approx_token_count(text: str) -> int:
    # Rough English/CJK-agnostic estimate: ~4 chars per token.
    return max(1, len(text) // 4)


def test_system_prompt_non_empty():
    assert SYSTEM_PROMPT.strip()
    assert "MisakaX" in SYSTEM_PROMPT


def test_subagent_prompts_exist_and_are_bounded():
    for name, prompt in (
        ("researcher", RESEARCHER_PROMPT),
        ("coder", CODER_PROMPT),
        ("analyst", ANALYST_PROMPT),
    ):
        assert prompt.strip(), f"{name} prompt empty"
        assert _approx_token_count(prompt) <= 500, f"{name} prompt too long"
