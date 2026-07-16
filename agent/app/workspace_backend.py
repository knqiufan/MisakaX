"""Workspace path validation and DeepAgents backend adapters."""

from __future__ import annotations

import logging
import re
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)

WORKSPACE_PREFIX = "/workspace"
SKILLS_PREFIX = "/skills"
MEMORIES_PREFIX = "/memories"
MEMORY_FILE = f"{MEMORIES_PREFIX}/AGENTS.md"

_SYSTEM_PREFIXES = (SKILLS_PREFIX, MEMORIES_PREFIX)
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
    raw = _normalize_slashes(path)
    if _DRIVE_IN_WORKSPACE.match(raw):
        raise ValueError(
            "Invalid path: do not concatenate /workspace with a Windows drive path "
            "(e.g. /workspaceD:\\...). Use /workspace/<relative-path>."
        )

    if raw == "/workspace" or raw.startswith("/workspace/"):
        virtual = raw
    elif raw == "/":
        virtual = "/workspace"
    else:
        virtual = f"/workspace{raw}" if raw.startswith("/") else f"/workspace/{raw}"

    return _finalize_virtual_path(virtual, required_root="workspace")


def normalize_backend_path(path: str | None) -> str:
    """Normalize paths for workspace tools and system mounts.

    Allows ``/workspace``, ``/skills``, and ``/memories`` virtual trees.
    Rejects host absolute paths and ``..`` traversal.
    """
    raw = _normalize_slashes(path)
    for prefix in _SYSTEM_PREFIXES:
        if raw == prefix or raw.startswith(f"{prefix}/"):
            return _finalize_virtual_path(raw, required_root=prefix.lstrip("/"))
    return normalize_workspace_path(path)


def _normalize_slashes(path: str | None) -> str:
    if path is None or not str(path).strip():
        raise ValueError("Path is required; use /workspace/<relative-path>")
    raw = str(path).strip().replace("\\", "/")
    if _WINDOWS_ABS.match(raw) or raw.startswith("//"):
        raise ValueError(
            "Host absolute paths are not allowed. Use /workspace/<relative-path>."
        )
    if not raw.startswith("/"):
        raw = f"/{raw}"
    return raw


def _finalize_virtual_path(virtual: str, *, required_root: str) -> str:
    parts = [p for p in virtual.split("/") if p]
    if ".." in parts:
        raise ValueError("Path traversal ('..') is not allowed.")
    if not parts or parts[0] != required_root:
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
        return normalize_backend_path(path)

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

    async def als(self, path: str = "/workspace") -> Any:
        return await self._inner.als(self._guard(path))

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

    def download_files(self, paths: list[str]) -> Any:
        return self._inner.download_files([self._guard(p) for p in paths])

    async def adownload_files(self, paths: list[str]) -> Any:
        return await self._inner.adownload_files([self._guard(p) for p in paths])


def _mount_system_routes(
    routes: dict[str, Any],
    filesystem_cls: Any | None,
    skills_dir: Path | None,
    memories_dir: Path | None,
) -> None:
    if filesystem_cls is None:
        return
    if skills_dir is not None:
        root = Path(skills_dir)
        root.mkdir(parents=True, exist_ok=True)
        routes[f"{SKILLS_PREFIX}/"] = filesystem_cls(
            root_dir=str(root),
            virtual_mode=True,
        )
    if memories_dir is not None:
        root = Path(memories_dir)
        root.mkdir(parents=True, exist_ok=True)
        routes[f"{MEMORIES_PREFIX}/"] = filesystem_cls(
            root_dir=str(root),
            virtual_mode=True,
        )


def make_workspace_backend_factory(
    validated_root,
    local_shell_cls,
    state_cls,
    composite_cls,
    *,
    filesystem_cls=None,
    skills_dir: Path | None = None,
    memories_dir: Path | None = None,
):
    """Bind project dir under /workspace; mount global skills/memories separately."""

    def factory(rt):
        routes: dict[str, Any] = {}
        _mount_system_routes(routes, filesystem_cls, skills_dir, memories_dir)

        if validated_root is not None:
            shell = local_shell_cls(
                root_dir=str(validated_root),
                virtual_mode=True,
                inherit_env=True,
            )
            # Route /workspace/* onto the same shell-backed root. default=shell
            # keeps execute() cwd at the project directory.
            routes[f"{WORKSPACE_PREFIX}/"] = shell
            composite = composite_cls(default=shell, routes=routes)
            return WorkspacePathBackend(composite)

        return composite_cls(default=state_cls(rt), routes=routes)

    return factory


__all__ = [
    "MEMORY_FILE",
    "MEMORIES_PREFIX",
    "SKILLS_PREFIX",
    "WORKSPACE_PREFIX",
    "WorkspacePathBackend",
    "coerce_file_data",
    "coerce_read_result",
    "make_workspace_backend_factory",
    "normalize_backend_path",
    "normalize_workspace_path",
]
