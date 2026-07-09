"""Working directory safety helpers."""

from __future__ import annotations

from pathlib import Path

FORBIDDEN_DIRS = {
    "/",
    "/etc",
    "/usr",
    "/bin",
    "/sbin",
    "/var",
    "C:\\Windows",
    "C:\\Program Files",
    "C:\\Program Files (x86)",
}


def validate_working_dir(working_dir: str | None) -> Path | None:
    """Validate that working_dir exists, is a directory, and is not forbidden."""
    if not working_dir:
        return None

    path = Path(working_dir).expanduser().resolve()

    if not path.exists():
        raise ValueError(f"Working directory does not exist: {path}")
    if not path.is_dir():
        raise ValueError(f"Path is not a directory: {path}")

    forbidden_resolved: set[Path] = set()
    for item in FORBIDDEN_DIRS:
        candidate = Path(item)
        try:
            if candidate.exists():
                forbidden_resolved.add(candidate.resolve())
        except OSError:
            continue

    if path in forbidden_resolved or str(path) in FORBIDDEN_DIRS:
        raise ValueError(f"Forbidden directory: {path}")

    return path
