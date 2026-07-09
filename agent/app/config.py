"""Sidecar configuration."""

from __future__ import annotations

from functools import lru_cache
from pathlib import Path

from pydantic import Field
from pydantic_settings import BaseSettings


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


settings = Settings()
