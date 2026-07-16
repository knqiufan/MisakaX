"""Workspace path validation and DeepAgents backend adapters."""

from __future__ import annotations

import logging
import re
from typing import Any

logger = logging.getLogger(__name__)

WORKSPACE_PREFIX = "/workspace"
_DRIVE_IN_WORKSPACE = re.compile(r"^/workspace([A-Za-z]:)")
_WINDOWS_ABS = re.compile(r"^[A-Za-z]:[\\/]")


def normalize_workspace_path(path: str | None) -> str:
    """Normalize and validate a virtual workspace path.

    Accepted forms:
    - ``/workspace``
    - ``/workspace/<relative>``
    - ``/`` or ``/<relative>`` (compat; rewritten to ``/workspace/...``)

    Rejects Windows absolute paths, ``/workspaceD:...`` concatenations, and ``..``.
    """
    if path is None or not str(path).strip():
        raise ValueError("Path is required; use /workspace/<relative-path>")

    raw = str(path).strip().replace("\\", "/")
    if _WINDOWS_ABS.match(raw) or raw.startswith("//"):
        raise ValueError(
            "Host absolute paths are not allowed. Use /workspace/<relative-path>."
        )
    if _DRIVE_IN_WORKSPACE.match(raw):
        raise ValueError(
            "Invalid path: do not concatenate /workspace with a Windows drive path "
            "(e.g. /workspaceD:\\...). Use /workspace/<relative-path>."
        )
    if not raw.startswith("/"):
        raw = f"/{raw}"

    if raw == "/workspace" or raw.startswith("/workspace/"):
        virtual = raw
    elif raw == "/":
        virtual = "/workspace"
    else:
        virtual = f"/workspace{raw}" if raw.startswith("/") else f"/workspace/{raw}"

    parts = [p for p in virtual.split("/") if p]
    if ".." in parts:
        raise ValueError("Path traversal ('..') is not allowed.")
    if not parts or parts[0] != "workspace":
        raise ValueError("Filesystem tools must use /workspace/<relative-path>.")

    return "/" + "/".join(parts)


def coerce_file_data(value: Any) -> Any:
    """Ensure FileData-like payloads never carry ``content=None``."""
    if value is None:
        return {"content": "", "encoding": "utf-8"}
    if isinstance(value, dict):
        if value.get("content") is None and "content" in value:
            fixed = dict(value)
            fixed["content"] = ""
            return fixed
        return value
    return value


def coerce_read_result(result: Any) -> Any:
    """Normalize backend read results so DeepAgents never sees content=None."""
    if result is None:
        return result
    file_data = getattr(result, "file_data", None)
    if file_data is not None:
        coerced = coerce_file_data(file_data)
        if coerced is not file_data:
            try:
                result.file_data = coerced
            except Exception:
                logger.debug("Could not assign coerced file_data on read result", exc_info=True)
        return result
    if isinstance(result, dict) and "file_data" in result:
        result = dict(result)
        result["file_data"] = coerce_file_data(result.get("file_data"))
        return result
    return coerce_file_data(result)


class WorkspacePathBackend:
    """Wrap a DeepAgents backend with /workspace rules and None-content defense."""

    def __init__(self, inner: Any):
        self._inner = inner

    def __getattr__(self, name: str) -> Any:
        return getattr(self._inner, name)

    def _guard(self, path: str | None) -> str:
        return normalize_workspace_path(path)

    def read(self, file_path: str, offset: int = 0, limit: int = 2000) -> Any:
        return coerce_read_result(
            self._inner.read(self._guard(file_path), offset=offset, limit=limit)
        )

    def write(self, file_path: str, content: str) -> Any:
        return self._inner.write(self._guard(file_path), content)

    def edit(
        self,
        file_path: str,
        old_string: str,
        new_string: str,
        replace_all: bool = False,
    ) -> Any:
        return self._inner.edit(
            self._guard(file_path),
            old_string,
            new_string,
            replace_all=replace_all,
        )

    def ls(self, path: str = "/workspace") -> Any:
        return self._inner.ls(self._guard(path))

    def glob(self, pattern: str, path: str = "/workspace") -> Any:
        return self._inner.glob(pattern, self._guard(path))

    def grep(
        self,
        pattern: str,
        path: str | None = "/workspace",
        glob: str | None = None,
    ) -> Any:
        safe = self._guard(path) if path else "/workspace"
        return self._inner.grep(pattern, path=safe, glob=glob)


def make_workspace_backend_factory(
    validated_root,
    local_shell_cls,
    state_cls,
    composite_cls,
):
    """Bind project dir under /workspace with LocalShellBackend + CompositeBackend."""

    def factory(rt):
        if validated_root is not None:
            shell = local_shell_cls(
                root_dir=str(validated_root),
                virtual_mode=True,
                inherit_env=True,
            )
            # Route /workspace/* onto the same shell-backed root. default=shell
            # keeps execute() cwd at the project directory.
            composite = composite_cls(
                default=shell,
                routes={f"{WORKSPACE_PREFIX}/": shell},
            )
            return WorkspacePathBackend(composite)
        return composite_cls(default=state_cls(rt), routes={})

    return factory


__all__ = [
    "WORKSPACE_PREFIX",
    "WorkspacePathBackend",
    "coerce_file_data",
    "coerce_read_result",
    "make_workspace_backend_factory",
    "normalize_workspace_path",
]
