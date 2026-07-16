"""Resolve LangChain chat models from Sidecar ChatConfig (provider-aware)."""

from __future__ import annotations

import logging
import os
from typing import Any

from app.config import get_settings
from app.models import ChatConfig

logger = logging.getLogger(__name__)


def resolve_chat_model(config: ChatConfig) -> Any:
    """Build a chat model for DeepAgents from per-request ChatConfig.

    Prefers explicit provider/api_compat from Rust RouterConfig. Falls back to
    settings.agent_model string when no provider binding is present.
    """
    model_id = (config.model or "").strip() or get_settings().agent_model
    protocol = resolve_protocol(config.provider, config.api_compat, model_id)

    if not config.provider and not config.api_compat and not config.api_key:
        logger.info("Using legacy model string for agent: %s", model_id)
        return model_id

    api_key = _resolve_api_key(protocol, config.api_key)
    if not api_key:
        raise RuntimeError(
            f"No API key available for protocol '{protocol}'. "
            "Configure the provider in settings and retry."
        )

    logger.info(
        "Resolving chat model protocol=%s vendor=%s model=%s thinking_mode=%s",
        protocol,
        config.vendor,
        model_id,
        config.thinking_mode,
    )

    if protocol == "openai":
        return _build_openai_model(model_id, api_key, config)
    if protocol == "anthropic":
        return _build_anthropic_model(model_id, api_key, config)
    if protocol == "google":
        return _build_google_model(model_id, api_key, config)

    raise RuntimeError(
        f"Unsupported chat protocol '{protocol}'. Expected openai, anthropic, or google."
    )


def resolve_protocol(
    provider: str | None,
    api_compat: str | None,
    model_id: str,
) -> str:
    """Mirror Rust ProviderFactory protocol selection."""
    provider_l = (provider or "").strip().lower()
    compat_l = (api_compat or "").strip().lower()

    if provider_l in {"openai", "anthropic", "google"}:
        return provider_l
    if compat_l in {"openai", "anthropic"}:
        return compat_l
    if provider_l:
        return compat_l or "openai"

    model_l = model_id.lower()
    if model_l.startswith("claude") or model_l.startswith("anthropic"):
        return "anthropic"
    if model_l.startswith("gemini") or model_l.startswith("google"):
        return "google"
    return "openai"


def _resolve_api_key(protocol: str, request_key: str | None) -> str | None:
    if request_key and request_key.strip():
        return request_key.strip()
    env_names = {
        "openai": ("OPENAI_API_KEY", "MISAKA_OPENAI_API_KEY"),
        "anthropic": ("ANTHROPIC_API_KEY", "MISAKA_ANTHROPIC_API_KEY"),
        "google": ("GOOGLE_API_KEY", "MISAKA_GOOGLE_API_KEY"),
    }
    for name in env_names.get(protocol, ()):
        value = os.environ.get(name)
        if value and value.strip():
            return value.strip()
    return None


def _common_model_kwargs(config: ChatConfig) -> dict[str, Any]:
    kwargs: dict[str, Any] = {"temperature": config.temperature}
    if config.max_tokens is not None:
        kwargs["max_tokens"] = config.max_tokens
    return kwargs


def _vendor_of(config: ChatConfig) -> str:
    return (config.vendor or config.provider or "").strip().lower()


def _thinking_mode(config: ChatConfig) -> str | None:
    if config.thinking_mode in {"enabled", "disabled"}:
        return config.thinking_mode
    if config.thinking_enabled is True:
        return "enabled"
    if config.thinking_enabled is False:
        return "disabled"
    return None


def _apply_deepseek_thinking(kwargs: dict[str, Any], mode: str | None) -> None:
    if mode not in {"enabled", "disabled"}:
        return
    extra = dict(kwargs.get("extra_body") or {})
    extra["thinking"] = {"type": mode}
    kwargs["extra_body"] = extra


def _apply_gemini_thinking(kwargs: dict[str, Any], model_id: str, mode: str | None) -> None:
    model_l = model_id.lower()
    is_flash_budget = (
        ("gemini-2.5-flash" in model_l or "gemini-2.0-flash-lite" in model_l)
        and "pro" not in model_l
    )
    if not is_flash_budget:
        # Pro / Gemini 3: do not invent private fields; Rust should switch models.
        return
    if mode == "disabled":
        kwargs["thinking_budget"] = 0


def _apply_anthropic_thinking(kwargs: dict[str, Any], model_id: str, mode: str | None) -> None:
    model_l = model_id.lower()
    # Only catalog-verified Claude 3.x IDs accept a simple disabled flag.
    if not ("claude-3-7-sonnet" in model_l or "claude-3-5-sonnet" in model_l):
        return
    if mode == "disabled":
        kwargs["thinking"] = {"type": "disabled"}


def _build_openai_model(model_id: str, api_key: str, config: ChatConfig) -> Any:
    from langchain_openai import ChatOpenAI

    kwargs = _common_model_kwargs(config)
    kwargs.update({"model": model_id, "api_key": api_key})
    if config.base_url:
        kwargs["base_url"] = config.base_url

    vendor = _vendor_of(config)
    mode = _thinking_mode(config)
    if vendor == "deepseek" or "deepseek" in model_id.lower():
        _apply_deepseek_thinking(kwargs, mode)
    # OpenAI reasoning series: never invent private thinking fields.

    return ChatOpenAI(**kwargs)


def _build_anthropic_model(model_id: str, api_key: str, config: ChatConfig) -> Any:
    from langchain_anthropic import ChatAnthropic

    kwargs = _common_model_kwargs(config)
    kwargs.update({"model": model_id, "api_key": api_key})
    if config.base_url:
        kwargs["base_url"] = config.base_url
    _apply_anthropic_thinking(kwargs, model_id, _thinking_mode(config))
    return ChatAnthropic(**kwargs)


def _build_google_model(model_id: str, api_key: str, config: ChatConfig) -> Any:
    try:
        from langchain_google_genai import ChatGoogleGenerativeAI
    except ImportError as exc:
        raise RuntimeError(
            "Google/Gemini support requires langchain-google-genai. "
            "Install it or switch to an OpenAI/Anthropic-compatible provider."
        ) from exc

    kwargs = _common_model_kwargs(config)
    kwargs.update({"model": model_id, "google_api_key": api_key})
    _apply_gemini_thinking(kwargs, model_id, _thinking_mode(config))
    return ChatGoogleGenerativeAI(**kwargs)
