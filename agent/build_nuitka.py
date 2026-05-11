"""Nuitka build script for MisakaX Agent Sidecar.

Usage:
    cd agent
    pip install nuitka   # one-time dev dependency
    python build_nuitka.py [--clean]

Produces:  misaka-agent.exe  (Windows) / misaka-agent  (Linux/macOS)
"""

from __future__ import annotations

import argparse
import platform
import shutil
import subprocess
import sys
import time
from pathlib import Path

AGENT_DIR = Path(__file__).resolve().parent
DIST_DIR = AGENT_DIR / "dist"

OUTPUT_NAME = "misaka-agent"


def _build_nuitka_args() -> list[str]:
    args = [
        sys.executable,
        "-m",
        "nuitka",
        "--standalone",
        "--onefile",
        f"--output-filename={OUTPUT_NAME}",
        f"--output-dir={DIST_DIR}",
        "--include-package=app",
        "--include-module=uvicorn.workers",
        "--include-module=uvicorn.lifespan.on",
        "--include-module=uvicorn.protocols.http.auto",
        "--include-module=uvicorn.protocols.http.h11_impl",
        "--include-module=uvicorn.protocols.http.httptools_impl",
        "--include-module=uvicorn.protocols.websockets.auto",
        "--include-module=uvicorn.protocols.websockets.wsproto_impl",
        "--include-module=uvicorn.logging",
        "--include-module=uvicorn.loops.auto",
        "--include-package=pydantic",
        "--include-package=pydantic_core",
        "--include-package=pydantic_settings",
        "--include-package=fastapi",
        "--include-package=starlette",
        "--include-package=anyio",
        "--include-package=httptools",
        "--include-package=multiprocessing",
        "--assume-yes-for-downloads",
        "--no-deployment-flag=self-execution",
        "run.py",
    ]
    return args


def _clean_dist() -> None:
    if DIST_DIR.exists():
        print(f"Cleaning {DIST_DIR} ...")
        shutil.rmtree(DIST_DIR)
    DIST_DIR.mkdir(parents=True, exist_ok=True)


def _find_artifact() -> Path | None:
    suffix = ".exe" if platform.system() == "Windows" else ""
    candidate = DIST_DIR / f"{OUTPUT_NAME}{suffix}"
    if candidate.exists():
        return candidate
    for f in DIST_DIR.rglob(f"{OUTPUT_NAME}*"):
        if f.is_file():
            return f
    return None


def _report_artifact(path: Path) -> None:
    size_mb = path.stat().st_size / (1024 * 1024)
    print(f"\n{'=' * 60}")
    print(f"  Build successful!")
    print(f"  Artifact : {path}")
    print(f"  Size     : {size_mb:.1f} MB")
    print(f"  Platform : {platform.system()} {platform.machine()}")
    print(f"{'=' * 60}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Build MisakaX Agent with Nuitka")
    parser.add_argument("--clean", action="store_true", help="Remove dist/ before building")
    args = parser.parse_args()

    if args.clean:
        _clean_dist()
    elif not DIST_DIR.exists():
        DIST_DIR.mkdir(parents=True, exist_ok=True)

    nuitka_args = _build_nuitka_args()

    print("Building MisakaX Agent Sidecar with Nuitka ...")
    print(f"  Working dir : {AGENT_DIR}")
    print(f"  Python      : {sys.executable}")
    print(f"  Command     : {' '.join(nuitka_args)}\n")

    t0 = time.monotonic()
    result = subprocess.run(nuitka_args, cwd=str(AGENT_DIR))
    elapsed = time.monotonic() - t0

    if result.returncode != 0:
        print(f"\n[ERROR] Nuitka build failed (exit code {result.returncode})")
        return result.returncode

    print(f"\nBuild completed in {elapsed:.1f}s")

    artifact = _find_artifact()
    if artifact:
        _report_artifact(artifact)
    else:
        print("[WARN] Build exited 0 but artifact not found in dist/")

    return 0


if __name__ == "__main__":
    sys.exit(main())
