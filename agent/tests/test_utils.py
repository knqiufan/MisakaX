"""Tests for working directory validation."""

from pathlib import Path

import pytest

from app.utils import FORBIDDEN_DIRS, validate_working_dir


def test_validate_working_dir_none():
    assert validate_working_dir(None) is None
    assert validate_working_dir("") is None


def test_validate_working_dir_missing(tmp_path: Path):
    missing = tmp_path / "does-not-exist"
    with pytest.raises(ValueError, match="does not exist"):
        validate_working_dir(str(missing))


def test_validate_working_dir_file(tmp_path: Path):
    file_path = tmp_path / "note.txt"
    file_path.write_text("x", encoding="utf-8")
    with pytest.raises(ValueError, match="not a directory"):
        validate_working_dir(str(file_path))


def test_validate_working_dir_valid(tmp_path: Path):
    result = validate_working_dir(str(tmp_path))
    assert result == tmp_path.resolve()


def test_validate_working_dir_forbidden_windows():
    # Use a known forbidden entry from the set; resolve may normalize casing.
    forbidden = next(iter(FORBIDDEN_DIRS))
    # Only assert when the path exists on this machine; otherwise skip.
    path = Path(forbidden)
    if not path.exists() or not path.is_dir():
        pytest.skip(f"Forbidden path not present: {forbidden}")
    with pytest.raises(ValueError, match="Forbidden directory"):
        validate_working_dir(str(path))
