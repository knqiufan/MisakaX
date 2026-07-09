"""Isolation probe: does stubbing click._winconsole avoid the 0xC0000409 crash?

Nuitka compiles click/_winconsole.py top-level ctypes (WINFUNCTYPE / windll)
which crashes onefile/standalone startup with STATUS_STACK_BUFFER_OVERRUN.
We pre-register a stub module so click's top-level
`from ._winconsole import _get_windows_console_stream` never touches the real
C-extension code. uvicorn only needs console I/O for interactive prompts,
which the sidecar never uses.
"""

import os
import sys
import types

_LOG = os.path.join(os.path.dirname(os.path.abspath(sys.argv[0])), "_probe_wc.log")


def _log(msg: str) -> None:
    with open(_LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")


def _install_winconsole_stub() -> None:
    if sys.platform != "win32":
        return
    stub = types.ModuleType("click._winconsole")

    def _get_windows_console_stream(f, encoding, errors):  # noqa: ANN001, ANN202
        return None

    stub._get_windows_console_stream = _get_windows_console_stream  # type: ignore[attr-defined]
    sys.modules["click._winconsole"] = stub


_log("START")
_install_winconsole_stub()
_log("STUB_INSTALLED")

import click  # noqa: E402

_log("CLICK_OK " + str(getattr(click, "__file__", "?")))

import uvicorn  # noqa: E402

_log("UVICORN_OK " + str(uvicorn.__version__))

print("PROBE_ALL_OK")
_log("ALL_OK")
