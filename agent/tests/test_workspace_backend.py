"""Tests for /workspace path rules and content coercion."""

from pathlib import Path

import pytest

from app.workspace_backend import (
    coerce_file_data,
    make_workspace_backend_factory,
    normalize_workspace_path,
)


def test_normalize_accepts_workspace_paths():
    assert normalize_workspace_path("/workspace") == "/workspace"
    assert normalize_workspace_path("/workspace/README.md") == "/workspace/README.md"
    assert normalize_workspace_path("/src/main.rs") == "/workspace/src/main.rs"
    assert normalize_workspace_path("/") == "/workspace"


def test_normalize_rejects_host_and_concat_paths():
    with pytest.raises(ValueError, match="Host absolute"):
        normalize_workspace_path(r"D:\learn\GitHub\powermem\README_CN.md")
    with pytest.raises(ValueError, match="concatenate"):
        normalize_workspace_path(r"/workspaceD:\learn\GitHub\powermem\README_CN.md")
    with pytest.raises(ValueError, match="traversal"):
        normalize_workspace_path("/workspace/../secret")


def test_coerce_file_data_none_content():
    assert coerce_file_data(None) == {"content": "", "encoding": "utf-8"}
    assert coerce_file_data({"content": None, "encoding": "utf-8"})["content"] == ""


def test_workspace_backend_reads_via_workspace_prefix(tmp_path: Path):
    (tmp_path / "README.md").write_text("hello", encoding="utf-8")

    class FakeLocal:
        def __init__(self, **kwargs):
            self.kwargs = kwargs
            self.root = Path(kwargs["root_dir"])

        def read(self, file_path: str, offset: int = 0, limit: int = 2000):
            rel = file_path.lstrip("/")
            if rel.startswith("workspace/"):
                rel = rel[len("workspace/") :]
            text = (self.root / rel).read_text(encoding="utf-8")
            return type("R", (), {"error": None, "file_data": {"content": text}})()

    class FakeComposite:
        def __init__(self, default, routes=None, **_kwargs):
            self.default = default
            self.routes = routes or {}

        def read(self, file_path: str, offset: int = 0, limit: int = 2000):
            return self.default.read(file_path, offset=offset, limit=limit)

    factory = make_workspace_backend_factory(
        tmp_path, FakeLocal, object, FakeComposite
    )
    backend = factory(object())
    result = backend.read("/workspace/README.md")
    assert result.file_data["content"] == "hello"

    with pytest.raises(ValueError, match="concatenate"):
        backend.read(r"/workspaceD:\x\README.md")
