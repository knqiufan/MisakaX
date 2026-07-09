"""FastAPI dependency helpers for checkpointer and store."""

from __future__ import annotations

import logging
from functools import lru_cache
from typing import Any

from app.config import get_settings

logger = logging.getLogger(__name__)

_checkpointer: Any | None = None
_checkpointer_cm: Any | None = None
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
    """Return the process-wide AsyncSqliteSaver, or None if unavailable/uninitialized."""
    return _checkpointer


async def setup_checkpointer() -> None:
    """Enter AsyncSqliteSaver context, create schema, and enable WAL via setup()."""
    global _checkpointer, _checkpointer_cm
    if _checkpointer is not None:
        return

    settings = get_settings()
    settings.data_dir.mkdir(parents=True, exist_ok=True)

    try:
        from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver
    except ImportError:
        logger.warning("AsyncSqliteSaver unavailable; continuing without checkpointer")
        return

    conn_string = str(settings.checkpointer_db_path)
    cm = AsyncSqliteSaver.from_conn_string(conn_string)
    try:
        saver = await cm.__aenter__()
        await saver.setup()
    except Exception:
        logger.exception("Failed to initialize checkpointer")
        try:
            await cm.__aexit__(None, None, None)
        except Exception:
            logger.exception("Failed to clean up checkpointer after setup error")
        return

    _checkpointer_cm = cm
    _checkpointer = saver
    logger.info("Checkpointer ready at %s (WAL enabled via setup)", conn_string)


async def close_checkpointer() -> None:
    """Exit the AsyncSqliteSaver context manager and clear the singleton."""
    global _checkpointer, _checkpointer_cm
    cm = _checkpointer_cm
    _checkpointer = None
    _checkpointer_cm = None
    if cm is None:
        return
    try:
        await cm.__aexit__(None, None, None)
    except Exception:
        logger.exception("Failed to close checkpointer")
