"""Tests for the application configuration."""

from pathlib import Path

from app.config import Settings, get_settings


def test_default_settings():
    settings = Settings()
    assert settings.host == "127.0.0.1"
    assert settings.port == 9527
    assert settings.log_level == "info"
    assert settings.db_path == ""
    assert settings.agent_model == "claude-sonnet-4-20250514"
    assert settings.temperature == 0.7
    assert settings.max_tokens is None
    assert settings.agent_recursion_limit == 100
    assert settings.powermem_enabled is True
    assert settings.mcp_bridge_url == "http://127.0.0.1:9528"


def test_default_path_fields():
    settings = Settings()
    home = Path.home()
    assert settings.data_dir == home / ".misakax" / "data"
    assert settings.skills_dir == home / ".misakax" / "skills"
    assert settings.memories_dir == home / ".misakax" / "memories"
    assert settings.powermem_db_path == home / ".misakax" / "data" / "powermem.db"
    assert settings.checkpointer_db_path == home / ".misakax" / "data" / "checkpoints.db"


def test_default_interrupt_config():
    settings = Settings()
    assert settings.interrupt_config["file_write"] is True
    assert settings.interrupt_config["file_delete"] is True
    assert settings.interrupt_config["shell_execute"] is True


def test_settings_env_prefix(monkeypatch):
    monkeypatch.setenv("MISAKA_PORT", "8080")
    monkeypatch.setenv("MISAKA_LOG_LEVEL", "debug")
    monkeypatch.setenv("MISAKA_AGENT_MODEL", "gpt-4o")
    monkeypatch.setenv("MISAKA_AGENT_RECURSION_LIMIT", "150")
    monkeypatch.setenv("MISAKA_MCP_BRIDGE_URL", "http://127.0.0.1:9999")
    settings = Settings()
    assert settings.port == 8080
    assert settings.log_level == "debug"
    assert settings.agent_model == "gpt-4o"
    assert settings.agent_recursion_limit == 150
    assert settings.mcp_bridge_url == "http://127.0.0.1:9999"


def test_settings_custom_db_path(monkeypatch):
    monkeypatch.setenv("MISAKA_DB_PATH", "/tmp/test.db")
    settings = Settings()
    assert settings.db_path == "/tmp/test.db"


def test_get_settings_returns_singleton():
    get_settings.cache_clear()
    first = get_settings()
    second = get_settings()
    assert first is second
    get_settings.cache_clear()


def test_bridge_provider_api_keys(monkeypatch):
    from app.config import Settings, bridge_provider_api_keys

    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)
    monkeypatch.delenv("OPENAI_API_KEY", raising=False)
    cfg = Settings(
        anthropic_api_key="sk-ant-test",
        openai_api_key="sk-openai-test",
    )
    bridge_provider_api_keys(cfg)
    assert __import__("os").environ["ANTHROPIC_API_KEY"] == "sk-ant-test"
    assert __import__("os").environ["OPENAI_API_KEY"] == "sk-openai-test"

    # Does not overwrite existing values.
    monkeypatch.setenv("ANTHROPIC_API_KEY", "keep-me")
    bridge_provider_api_keys(cfg)
    assert __import__("os").environ["ANTHROPIC_API_KEY"] == "keep-me"
