"""Sidecar configuration."""

from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """Application settings."""

    host: str = "127.0.0.1"
    port: int = 9527
    log_level: str = "info"
    db_path: str = ""  # Will be set by Rust core via env var

    model_config = {"env_prefix": "MISAKA_"}


settings = Settings()
