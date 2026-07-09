"""Tests for PowerMem engine bootstrap."""

from unittest.mock import MagicMock, patch

from app.config import get_settings
from app.memory import get_memory_engine


def test_get_memory_engine_disabled(monkeypatch):
    get_memory_engine.cache_clear()
    monkeypatch.setenv("MISAKA_POWERMEM_ENABLED", "false")
    get_settings.cache_clear()
    assert get_memory_engine() is None
    get_settings.cache_clear()
    get_memory_engine.cache_clear()


def test_get_memory_engine_import_error():
    get_memory_engine.cache_clear()
    with patch("app.memory._import_powermem", side_effect=ImportError("no powermem")):
        assert get_memory_engine() is None
    get_memory_engine.cache_clear()


def test_get_memory_engine_init_failure():
    get_memory_engine.cache_clear()
    fake_mod = MagicMock()
    fake_mod.Memory.side_effect = RuntimeError("boom")
    # Ensure create_memory path is not used as a fallback.
    del fake_mod.create_memory
    with patch("app.memory._import_powermem", return_value=fake_mod):
        assert get_memory_engine() is None
    get_memory_engine.cache_clear()
