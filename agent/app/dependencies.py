"""FastAPI dependency helpers for checkpointer and store."""

from __future__ import annotations

import logging
from functools import lru_cache
from typing import Any

from app.config import get_settings

logger = logging.getLogger(__name__)

_checkpointer: Any | None = None
_store: Any | None = None


@lru_cache
def get_store():
    """Return a process-wide in-memory store singleton."""
    global _store
    if _store is not None:
        return _store

    try:
        from langgraph.store.memory import InMemoryStore
    except ImportError:
        logger.warning("langgraph store unavailable; continuing without store")
        return None

    _store = InMemoryStore()
    return _store


def get_checkpointer():
    """Return the AsyncSqliteSaver singleton, creating it lazily."""
    global _checkpointer
    if _checkpointer is not None:
        return _checkpointer

    settings = get_settings()
    settings.data_dir.mkdir(parents=True, exist_ok=True)

    try:
        from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver
    except ImportError:
        logger.warning("AsyncSqliteSaver unavailable; continuing without checkpointer")
        return None

    conn_string = str(settings.checkpointer_db_path)
    _checkpointer = AsyncSqliteSaver.from_conn_string(conn_string)
    return _checkpointer


async def setup_checkpointer() -> None:
    """Ensure checkpointer schema exists."""
    checkpointer = get_checkpointer()
    if checkpointer is None:
        return
    setup = getattr(checkpointer, "setup", None)
    if setup is None:
        return
    result = setup()
    if hasattr(result, "__await__"):
        await result


async def close_checkpointer() -> None:
    """Close checkpointer resources if present."""
    global _checkpointer
    checkpointer = _checkpointer
    if checkpointer is None:
        return

    for method_name in ("aclose", "close"):
        method = getattr(checkpointer, method_name, None)
        if method is None:
            continue
        result = method()
        if hasattr(result, "__await__"):
            await result
        break

    _checkpointer = None
