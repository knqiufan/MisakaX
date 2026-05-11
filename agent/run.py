"""Standalone entry point for Nuitka builds.

When compiled with Nuitka (--standalone / --onefile), the app object must be
passed directly to uvicorn.run() instead of using the "app.main:app" string
form, because Nuitka cannot resolve module-string imports at runtime.
"""

import multiprocessing
import sys

import uvicorn

from app.config import settings
from app.main import app


def main() -> None:
    multiprocessing.freeze_support()
    uvicorn.run(
        app,
        host=settings.host,
        port=settings.port,
        log_level=settings.log_level,
    )


if __name__ == "__main__":
    sys.exit(main() or 0)
