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
        # Legacy path: let deepagents/init_chat_model infer from model string.
        logger.info("Using legacy model string for agent: %s", model_id)
        return model_id

    api_key = _resolve_api_key(protocol, config.api_key)
    if not api_key:
        raise RuntimeError(
            f"No API key available for protocol '{protocol}'. "
            "Configure the provider in settings and retry."
        )

    logger.info(
        "Resolving chat model protocol=%s model=%s base_url=%s",
        protocol,
        model_id,
        bool(config.base_url),
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
        # Custom / unknown provider defaults to OpenAI-compatible (same as Rust).
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


def _build_openai_model(model_id: str, api_key: str, config: ChatConfig) -> Any:
    from langchain_openai import ChatOpenAI

    kwargs = _common_model_kwargs(config)
    kwargs.update({"model": model_id, "api_key": api_key})
    if config.base_url:
        kwargs["base_url"] = config.base_url
    return ChatOpenAI(**kwargs)


def _build_anthropic_model(model_id: str, api_key: str, config: ChatConfig) -> Any:
    from langchain_anthropic import ChatAnthropic

    kwargs = _common_model_kwargs(config)
    kwargs.update({"model": model_id, "api_key": api_key})
    if config.base_url:
        kwargs["base_url"] = config.base_url
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
    return ChatGoogleGenerativeAI(**kwargs)
