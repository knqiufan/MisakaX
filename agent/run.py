"""Standalone entry point for Nuitka builds.

When compiled with Nuitka (--standalone / --onefile), the app object must be
passed directly to Uvicorn instead of using the "app.main:app" string form,
because Nuitka cannot resolve module-string imports at runtime.
"""

import multiprocessing
import logging
import os
import sys
import types


def _is_frozen_runtime() -> bool:
    return getattr(sys, "frozen", False) or "__compiled__" in globals()


def _register_bundled_dll_dir() -> None:
    """Ensure the frozen binary's own directory wins the DLL search.

    On Anaconda + Windows, `_ssl.pyd` links against `libcrypto-3-x64.dll` /
    `libssl-3-x64.dll`. In a Nuitka onefile build these ship next to the
    executable (the extracted temp dir), but the OS may otherwise resolve a
    mismatched OpenSSL from `PATH`/System32 first, crashing the process with
    STATUS_STACK_BUFFER_OVERRUN (0xC0000409). Registering our own dir up front
    forces the bundled, ABI-matched DLLs to load. No-op for a normal
    interpreter run (not frozen).
    """
    if not _is_frozen_runtime():
        return
    exe_dir = os.path.dirname(os.path.abspath(sys.argv[0]))
    for candidate in {exe_dir, os.path.dirname(os.path.abspath(__file__))}:
        if candidate and os.path.isdir(candidate):
            try:
                os.add_dll_directory(candidate)
            except (OSError, AttributeError):
                pass


def _install_click_winconsole_stub() -> None:
    """Avoid importing Click's Windows console ctypes shim in frozen builds.

    Uvicorn imports Click for CLI helpers, but this sidecar never uses Click's
    interactive Windows console stream support. Under Nuitka + Conda on Windows,
    importing `click._winconsole` can terminate the process before Python raises
    an exception, so frozen builds provide the tiny API surface Click expects.
    """
    if sys.platform != "win32" or not _is_frozen_runtime():
        return

    stub = types.ModuleType("click._winconsole")

    def _get_windows_console_stream(_stream, _encoding, _errors):  # noqa: ANN001, ANN202
        return None

    stub._get_windows_console_stream = _get_windows_console_stream  # type: ignore[attr-defined]
    sys.modules["click._winconsole"] = stub


_register_bundled_dll_dir()
_install_click_winconsole_stub()

import uvicorn  # noqa: E402  (import after DLL dir registration)

from app.config import Settings, get_settings  # noqa: E402
from app.main import app  # noqa: E402


SIDECAR_HTTP_KEEP_ALIVE_SECONDS = 35


class SuccessfulHealthCheckAccessFilter(logging.Filter):
    """Suppress only successful local health checks from Uvicorn access logs."""

    def filter(self, record: logging.LogRecord) -> bool:
        args = record.args
        if not isinstance(args, tuple) or len(args) < 5:
            return True

        try:
            method = str(args[1]).upper()
            path = str(args[2]).split("?", maxsplit=1)[0]
            status_code = int(args[4])
        except (TypeError, ValueError):
            return True

        return not (method in {"GET", "HEAD"} and path == "/health" and 200 <= status_code < 300)


def configure_access_log_filter() -> None:
    """Attach the health-check filter once without disturbing other handlers."""
    access_logger = logging.getLogger("uvicorn.access")
    if not any(
        isinstance(log_filter, SuccessfulHealthCheckAccessFilter)
        for log_filter in access_logger.filters
    ):
        access_logger.addFilter(SuccessfulHealthCheckAccessFilter())


def build_uvicorn_config(settings: Settings) -> uvicorn.Config:
    """Build the common development and packaged Sidecar server configuration."""
    return uvicorn.Config(
        app,
        host=settings.host,
        port=settings.port,
        log_level=settings.log_level,
        timeout_keep_alive=SIDECAR_HTTP_KEEP_ALIVE_SECONDS,
        access_log=True,
    )


def main() -> None:
    multiprocessing.freeze_support()
    settings = get_settings()
    config = build_uvicorn_config(settings)
    configure_access_log_filter()
    uvicorn.Server(config).run()


if __name__ == "__main__":
    sys.exit(main() or 0)
