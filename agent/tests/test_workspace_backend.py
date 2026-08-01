"""Tests for /workspace path rules and content coercion."""

from pathlib import Path

import pytest

from app.workspace_backend import (
    MEMORIES_PREFIX,
    SKILLS_PREFIX,
    coerce_file_data,
    make_workspace_backend_factory,
    normalize_backend_path,
    normalize_workspace_path,
)


def test_normalize_accepts_workspace_paths():
    assert normalize_workspace_path("/workspace") == "/workspace"
    assert normalize_workspace_path("/workspace/README.md") == "/workspace/README.md"
    assert normalize_workspace_path("/src/main.rs") == "/workspace/src/main.rs"
    assert normalize_workspace_path("/") == "/workspace"


def test_normalize_backend_accepts_system_mounts():
    assert normalize_backend_path("/skills") == "/skills"
    assert normalize_backend_path("/skills/demo/SKILL.md") == "/skills/demo/SKILL.md"
    assert normalize_backend_path("/memories/AGENTS.md") == "/memories/AGENTS.md"
    assert normalize_backend_path("/workspace/src") == "/workspace/src"


def test_normalize_rejects_host_and_concat_paths():
    with pytest.raises(ValueError, match="Host absolute"):
        normalize_workspace_path(r"D:\learn\GitHub\powermem\README_CN.md")
    with pytest.raises(ValueError, match="concatenate"):
        normalize_workspace_path(r"/workspaceD:\learn\GitHub\powermem\README_CN.md")
    with pytest.raises(ValueError, match="traversal"):
        normalize_workspace_path("/workspace/../secret")
    with pytest.raises(ValueError, match="Host absolute"):
        normalize_backend_path(r"C:\Users\Administrator\.misakax\skills")


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


def test_factory_mounts_only_activated_skills_and_memories(tmp_path: Path):
    project = tmp_path / "project"
    external_skill = tmp_path / "external" / "demo"
    memories = tmp_path / "memories"
    project.mkdir()
    external_skill.mkdir(parents=True)

    class FakeLocal:
        def __init__(self, **kwargs):
            self.kwargs = kwargs

    class FakeFs:
        def __init__(self, **kwargs):
            self.kwargs = kwargs

    class FakeComposite:
        def __init__(self, default, routes=None, **_kwargs):
            self.default = default
            self.routes = routes or {}

    factory = make_workspace_backend_factory(
        project,
        FakeLocal,
        lambda _rt: object(),
        FakeComposite,
        filesystem_cls=FakeFs,
        selected_skill_dirs={"demo": external_skill},
        memories_dir=memories,
    )
    backend = factory(object())
    routes = backend._inner.routes
    assert f"{SKILLS_PREFIX}/" in routes
    assert not hasattr(routes[f"{SKILLS_PREFIX}/"], "kwargs")
    assert f"{SKILLS_PREFIX}/demo/" in routes
    assert f"{MEMORIES_PREFIX}/" in routes
    assert "/workspace/" in routes
    assert memories.is_dir()
    assert Path(routes[f"{SKILLS_PREFIX}/demo/"].kwargs["root_dir"]) == external_skill
    assert routes[f"{SKILLS_PREFIX}/demo/"].kwargs["virtual_mode"] is True


def test_disabled_skills_are_not_exposed_by_a_global_skills_mount(tmp_path: Path):
    """Only the activation view may own /skills routes."""

    class FakeFs:
        def __init__(self, **kwargs):
            self.kwargs = kwargs

    class FakeComposite:
        def __init__(self, default, routes=None, **_kwargs):
            self.default = default
            self.routes = routes or {}

    factory = make_workspace_backend_factory(
        None,
        object,
        lambda _rt: object(),
        FakeComposite,
        filesystem_cls=FakeFs,
    )

    routes = factory(object())._inner.routes
    assert f"{SKILLS_PREFIX}/" not in routes


def test_skills_route_outside_workspace_root(tmp_path: Path):
    """Regression: global skills must not be resolved under the project root."""
    pytest.importorskip("deepagents")
    from deepagents.backends import CompositeBackend, FilesystemBackend, LocalShellBackend

    project = tmp_path / "project"
    skills = tmp_path / "global-skills"
    project.mkdir()
    (project / "README.md").write_text("project", encoding="utf-8")
    skill_dir = skills / "demo"
    skill_dir.mkdir(parents=True)
    (skill_dir / "SKILL.md").write_text("# demo\n", encoding="utf-8")

    factory = make_workspace_backend_factory(
        project,
        LocalShellBackend,
        object,
        CompositeBackend,
        filesystem_cls=FilesystemBackend,
        selected_skill_dirs={"demo": skill_dir},
        memories_dir=tmp_path / "memories",
    )
    backend = factory(object())
    listed = backend.ls("/skills/")
    entries = listed.entries if hasattr(listed, "entries") else listed
    paths = {item.get("path") for item in (entries or [])}
    assert any(path and "demo" in path for path in paths)


def test_selected_external_skill_route_reads_from_its_original_directory(tmp_path: Path):
    """A selected compatibility Skill must mount ahead of the managed root."""
    pytest.importorskip("deepagents")
    from deepagents.backends import CompositeBackend, FilesystemBackend, LocalShellBackend

    project = tmp_path / "project"
    external_skill = tmp_path / "external" / "demo"
    project.mkdir()
    external_skill.mkdir(parents=True)
    (external_skill / "SKILL.md").write_text("# external demo\n", encoding="utf-8")

    factory = make_workspace_backend_factory(
        project,
        LocalShellBackend,
        object,
        CompositeBackend,
        filesystem_cls=FilesystemBackend,
        selected_skill_dirs={"demo": external_skill},
        memories_dir=tmp_path / "memories",
    )
    backend = factory(object())
    result = backend.read("/skills/demo/SKILL.md")
    file_data = result.file_data if hasattr(result, "file_data") else result["file_data"]
    assert file_data["content"] == "# external demo\n"


def test_activated_skills_are_read_only(tmp_path: Path):
    class FakeInner:
        def write(self, path, content):
            return (path, content)

        def edit(self, path, *args, **kwargs):
            return (path, args, kwargs)

        def upload_files(self, files):
            return files

    from app.workspace_backend import WorkspacePathBackend

    backend = WorkspacePathBackend(FakeInner())
    with pytest.raises(ValueError, match="read-only"):
        backend.write("/skills/demo/SKILL.md", "changed")
    with pytest.raises(ValueError, match="read-only"):
        backend.edit("/skills/demo/SKILL.md", "a", "b")
    with pytest.raises(ValueError, match="read-only"):
        backend.upload_files([("/skills/demo/new.txt", b"content")])
    assert backend.write("/workspace/note.txt", "ok") == (
        "/workspace/note.txt",
        "ok",
    )
