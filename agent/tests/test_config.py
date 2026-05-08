"""Tests for the application configuration."""

from app.config import Settings


def test_default_settings():
    settings = Settings()
    assert settings.host == "127.0.0.1"
    assert settings.port == 9527
    assert settings.log_level == "info"
    assert settings.db_path == ""


def test_settings_env_prefix(monkeypatch):
    monkeypatch.setenv("MISAKA_PORT", "8080")
    monkeypatch.setenv("MISAKA_LOG_LEVEL", "debug")
    settings = Settings()
    assert settings.port == 8080
    assert settings.log_level == "debug"


def test_settings_custom_db_path(monkeypatch):
    monkeypatch.setenv("MISAKA_DB_PATH", "/tmp/test.db")
    settings = Settings()
    assert settings.db_path == "/tmp/test.db"
