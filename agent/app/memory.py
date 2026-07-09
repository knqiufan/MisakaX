"""PowerMem engine bootstrap with safe lazy initialization."""

from __future__ import annotations

import logging
from functools import lru_cache
from typing import Any

from app.config import get_settings

logger = logging.getLogger(__name__)


def _import_powermem():
    """Import powermem lazily so missing installs degrade gracefully."""
    import powermem

    return powermem


@lru_cache
def get_memory_engine() -> Any | None:
    """Return a PowerMem engine singleton, or None when unavailable."""
    settings = get_settings()
    if not settings.powermem_enabled:
        logger.info("PowerMem disabled via settings")
        return None

    try:
        powermem = _import_powermem()
    except ImportError:
        logger.warning("powermem is not installed")
        return None

    try:
        settings.powermem_db_path.parent.mkdir(parents=True, exist_ok=True)
        # Support both factory-style and class-style APIs across powermem versions.
        if hasattr(powermem, "Memory"):
            engine = powermem.Memory(db_path=str(settings.powermem_db_path))
        elif hasattr(powermem, "create_memory"):
            engine = powermem.create_memory(db_path=str(settings.powermem_db_path))
        else:
            logger.warning("powermem API not recognized")
            return None
        return engine
    except Exception as exc:
        logger.warning("Failed to initialize PowerMem: %s", exc)
        return None
