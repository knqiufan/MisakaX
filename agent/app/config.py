"""Sidecar configuration."""

from __future__ import annotations

import logging
import os
from functools import lru_cache
from pathlib import Path

from pydantic import Field
from pydantic_settings import BaseSettings

logger = logging.getLogger(__name__)


def _default_interrupt_config() -> dict[str, bool]:
    return {
        "file_write": True,
        "file_delete": True,
        "shell_execute": True,
    }


class Settings(BaseSettings):
    """Application settings loaded from MISAKA_* environment variables."""

    host: str = "127.0.0.1"
    port: int = 9527
    log_level: str = "info"
    debug: bool = False
    db_path: str = ""

    agent_model: str = "claude-sonnet-4-20250514"
    temperature: float = 0.7
    max_tokens: int | None = None
    # LangGraph defaults to 25 graph steps, which is too low for tool-heavy
    # DeepAgent turns and subagent delegation. Keep a finite upper bound.
    agent_recursion_limit: int = Field(default=100, ge=25, le=500)

    anthropic_api_key: str | None = None
    openai_api_key: str | None = None

    skills_dir: Path = Field(default_factory=lambda: Path.home() / ".misakax" / "skills")
    memories_dir: Path = Field(default_factory=lambda: Path.home() / ".misakax" / "memories")
    data_dir: Path = Field(default_factory=lambda: Path.home() / ".misakax" / "data")

    powermem_enabled: bool = True
    powermem_db_path: Path = Field(
        default_factory=lambda: Path.home() / ".misakax" / "data" / "powermem.db"
    )
    checkpointer_db_path: Path = Field(
        default_factory=lambda: Path.home() / ".misakax" / "data" / "checkpoints.db"
    )

    interrupt_config: dict[str, bool] = Field(default_factory=_default_interrupt_config)
    mcp_bridge_url: str = "http://127.0.0.1:9528"

    model_config = {"env_prefix": "MISAKA_", "env_file": ".env", "extra": "ignore"}


@lru_cache
def get_settings() -> Settings:
    """Return a process-wide Settings singleton."""
    return Settings()


def bridge_provider_api_keys(settings_obj: Settings | None = None) -> None:
    """Bridge MISAKA_* keys into standard provider env vars used by LangChain.

    Does not overwrite keys that are already present in the process environment.
    Never logs key values.
    """
    cfg = settings_obj or get_settings()
    bridged: list[str] = []
    if cfg.anthropic_api_key and not os.environ.get("ANTHROPIC_API_KEY"):
        os.environ["ANTHROPIC_API_KEY"] = cfg.anthropic_api_key
        bridged.append("ANTHROPIC_API_KEY")
    if cfg.openai_api_key and not os.environ.get("OPENAI_API_KEY"):
        os.environ["OPENAI_API_KEY"] = cfg.openai_api_key
        bridged.append("OPENAI_API_KEY")
    if bridged:
        logger.info("Bridged provider API keys into process env: %s", ", ".join(bridged))
