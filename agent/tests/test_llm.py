"""Tests for provider-aware chat model resolution."""

from unittest.mock import MagicMock, patch

import pytest

from app.llm import resolve_chat_model, resolve_protocol
from app.models import ChatConfig


@pytest.mark.parametrize(
    ("provider", "api_compat", "model", "expected"),
    [
        ("openai", None, "gpt-4o", "openai"),
        ("anthropic", None, "claude-3", "anthropic"),
        ("google", None, "gemini-pro", "google"),
        ("custom", "openai", "my-model", "openai"),
        ("custom", "anthropic", "my-model", "anthropic"),
        ("deepseek", None, "deepseek-chat", "openai"),
        (None, None, "claude-sonnet-4", "anthropic"),
        (None, None, "gpt-4o", "openai"),
        (None, None, "gemini-2.0", "google"),
    ],
)
def test_resolve_protocol(provider, api_compat, model, expected):
    assert resolve_protocol(provider, api_compat, model) == expected


def test_resolve_chat_model_builds_openai_compat():
    cfg = ChatConfig(
        model="deepseek-chat",
        provider="custom",
        api_compat="openai",
        base_url="https://api.deepseek.com/v1",
        api_key="sk-test",
        temperature=0.2,
        max_tokens=128,
    )
    with patch("langchain_openai.ChatOpenAI") as chat_cls:
        chat_cls.return_value = MagicMock(name="openai-model")
        model = resolve_chat_model(cfg)

    assert model is chat_cls.return_value
    kwargs = chat_cls.call_args.kwargs
    assert kwargs["model"] == "deepseek-chat"
    assert kwargs["api_key"] == "sk-test"
    assert kwargs["base_url"] == "https://api.deepseek.com/v1"
    assert kwargs["temperature"] == 0.2
    assert kwargs["max_tokens"] == 128


def test_resolve_chat_model_builds_anthropic():
    cfg = ChatConfig(
        model="claude-sonnet-4-20250514",
        provider="anthropic",
        api_key="sk-ant-test",
    )
    with patch("langchain_anthropic.ChatAnthropic") as chat_cls:
        chat_cls.return_value = MagicMock(name="anthropic-model")
        model = resolve_chat_model(cfg)

    assert model is chat_cls.return_value
    assert chat_cls.call_args.kwargs["model"] == "claude-sonnet-4-20250514"
    assert chat_cls.call_args.kwargs["api_key"] == "sk-ant-test"


def test_resolve_chat_model_legacy_string_without_binding():
    cfg = ChatConfig(model="claude-sonnet-4-20250514")
    assert resolve_chat_model(cfg) == "claude-sonnet-4-20250514"


def test_resolve_chat_model_requires_api_key_when_provider_set(monkeypatch):
    # Isolate from developer shell / Sidecar-bridged provider env vars.
    for key in (
        "OPENAI_API_KEY",
        "MISAKA_OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "MISAKA_ANTHROPIC_API_KEY",
    ):
        monkeypatch.delenv(key, raising=False)
    cfg = ChatConfig(model="gpt-4o", provider="openai")
    with pytest.raises(RuntimeError, match="No API key"):
        resolve_chat_model(cfg)


def test_deepseek_thinking_disabled_extra_body():
    cfg = ChatConfig(
        model="deepseek-reasoner",
        provider="custom",
        vendor="deepseek",
        api_compat="openai",
        api_key="sk-test",
        thinking_mode="disabled",
    )
    with patch("langchain_openai.ChatOpenAI") as chat_cls:
        chat_cls.return_value = MagicMock()
        resolve_chat_model(cfg)
    assert chat_cls.call_args.kwargs["extra_body"] == {
        "thinking": {"type": "disabled"}
    }


def test_unknown_vendor_does_not_invent_thinking_fields():
    cfg = ChatConfig(
        model="gpt-4o",
        provider="openai",
        vendor="openai",
        api_key="sk-test",
        thinking_mode="disabled",
    )
    with patch("langchain_openai.ChatOpenAI") as chat_cls:
        chat_cls.return_value = MagicMock()
        resolve_chat_model(cfg)
    assert "extra_body" not in chat_cls.call_args.kwargs
    assert "thinking" not in chat_cls.call_args.kwargs


def test_gemini_flash_thinking_budget_zero():
    cfg = ChatConfig(
        model="gemini-2.5-flash",
        provider="google",
        api_key="sk-g",
        thinking_mode="disabled",
    )
    with patch("langchain_google_genai.ChatGoogleGenerativeAI") as chat_cls:
        chat_cls.return_value = MagicMock()
        resolve_chat_model(cfg)
    assert chat_cls.call_args.kwargs["thinking_budget"] == 0


def test_chat_config_defaults_thinking_fields():
    cfg = ChatConfig()
    assert cfg.thinking_enabled is None
    assert cfg.thinking_mode is None
    assert cfg.vendor is None
