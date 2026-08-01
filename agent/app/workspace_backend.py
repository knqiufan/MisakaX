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
        safe = self._guard(file_path)
        self._ensure_writable(safe)
        return self._inner.write(safe, content)

    def edit(
        self,
        file_path: str,
        old_string: str,
        new_string: str,
        replace_all: bool = False,
    ) -> Any:
        safe = self._guard(file_path)
        self._ensure_writable(safe)
        return self._inner.edit(
            safe,
            old_string,
            new_string,
            replace_all=replace_all,
        )

    async def awrite(self, file_path: str, content: str) -> Any:
        safe = self._guard(file_path)
        self._ensure_writable(safe)
        return await self._inner.awrite(safe, content)

    async def aedit(
        self,
        file_path: str,
        old_string: str,
        new_string: str,
        replace_all: bool = False,
    ) -> Any:
        safe = self._guard(file_path)
        self._ensure_writable(safe)
        return await self._inner.aedit(
            safe,
            old_string,
            new_string,
            replace_all=replace_all,
        )

    def upload_files(self, files: list[tuple[str, bytes]]) -> Any:
        safe_files = [(self._guard(path), content) for path, content in files]
        for path, _content in safe_files:
            self._ensure_writable(path)
        return self._inner.upload_files(safe_files)

    async def aupload_files(self, files: list[tuple[str, bytes]]) -> Any:
        safe_files = [(self._guard(path), content) for path, content in files]
        for path, _content in safe_files:
            self._ensure_writable(path)
        return await self._inner.aupload_files(safe_files)

    @staticmethod
    def _ensure_writable(path: str) -> None:
        if path == SKILLS_PREFIX or path.startswith(f"{SKILLS_PREFIX}/"):
            raise ValueError("Activated Skills are mounted read-only")

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


class ActivatedSkillsIndexBackend:
    """Virtual read-only parent directory for independently routed Skills."""

    def __init__(self, slugs: list[str]):
        self._slugs = tuple(sorted(set(slugs)))

    def ls(self, path: str = "/") -> list[dict[str, Any]]:
        if path not in ("/", ""):
            return []
        return [
            {"path": f"/{slug}/", "is_dir": True, "size": 0, "modified_at": ""}
            for slug in self._slugs
        ]

    async def als(self, path: str = "/") -> list[dict[str, Any]]:
        return self.ls(path)

    def write(self, *_args, **_kwargs):
        raise ValueError("Activated Skills are mounted read-only")

    def edit(self, *_args, **_kwargs):
        raise ValueError("Activated Skills are mounted read-only")


def _mount_system_routes(
    routes: dict[str, Any],
    filesystem_cls: Any | None,
    selected_skill_dirs: dict[str, Path] | None,
    memories_dir: Path | None,
) -> None:
    if filesystem_cls is None:
        return
    if selected_skill_dirs:
        routes[f"{SKILLS_PREFIX}/"] = ActivatedSkillsIndexBackend(
            list(selected_skill_dirs)
        )
    for slug, directory in (selected_skill_dirs or {}).items():
        routes[f"{SKILLS_PREFIX}/{slug}/"] = filesystem_cls(
            root_dir=str(directory),
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
    selected_skill_dirs: dict[str, Path] | None = None,
    memories_dir: Path | None = None,
):
    """Bind project dir and only Rust-authorized Skill routes."""

    def factory(rt):
        routes: dict[str, Any] = {}
        _mount_system_routes(
            routes,
            filesystem_cls,
            selected_skill_dirs,
            memories_dir,
        )

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

        return WorkspacePathBackend(composite_cls(default=state_cls(rt), routes=routes))

    return factory


__all__ = [
    "MEMORY_FILE",
    "MEMORIES_PREFIX",
    "SKILLS_PREFIX",
    "WORKSPACE_PREFIX",
    "WorkspacePathBackend",
    "ActivatedSkillsIndexBackend",
    "coerce_file_data",
    "coerce_read_result",
    "make_workspace_backend_factory",
    "normalize_backend_path",
    "normalize_workspace_path",
]
