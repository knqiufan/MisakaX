"""Nuitka build script for MisakaX Agent Sidecar.

Usage:
    cd agent
    pip install nuitka   # one-time dev dependency
    python build_nuitka.py [--clean]

Produces:  misaka-agent.exe  (Windows) / misaka-agent  (Linux/macOS)
"""

from __future__ import annotations

import argparse
import os
import platform
import shutil
import subprocess
import sys
import time
from pathlib import Path

AGENT_DIR = Path(__file__).resolve().parent
DIST_DIR = AGENT_DIR / "dist"

OUTPUT_NAME = "misaka-agent"


OPENSSL_DLL_NAMES = ["libcrypto-3-x64.dll", "libssl-3-x64.dll"]


def _sync_anaconda_openssl_dlls() -> list[Path]:
    """Ensure Anaconda's OpenSSL DLLs sit next to `_ssl.pyd` before building.

    Anaconda ships `libcrypto-3-x64.dll` / `libssl-3-x64.dll` under
    `<prefix>/Library/bin`, but `_ssl.pyd` lives in `<prefix>/DLLs`. When they are
    apart, Nuitka's dependency scan may pick a mismatched OpenSSL from the system
    `PATH`, and the frozen binary then crashes at startup (ImportError, or worse
    STATUS_STACK_BUFFER_OVERRUN 0xC0000409). Copying the correct DLLs beside
    `_ssl.pyd` is the Anaconda-recommended fix and is idempotent.

    Returns the list of DLL paths now present in `<prefix>/DLLs` (empty on
    non-Anaconda layouts, where standard CPython bundles SSL correctly).
    """
    prefix = Path(sys.prefix)
    src_dir = prefix / "Library" / "bin"
    dst_dir = prefix / "DLLs"
    if not src_dir.is_dir() or not dst_dir.is_dir():
        return []

    synced: list[Path] = []
    for name in OPENSSL_DLL_NAMES:
        src = src_dir / name
        dst = dst_dir / name
        if not src.exists():
            continue
        if not dst.exists():
            print(f"Syncing OpenSSL DLL for build: {name} -> {dst_dir}")
            shutil.copy2(src, dst)
        synced.append(dst)
    return synced


def _build_env_with_conda_dlls() -> dict[str, str]:
    """Return an env for the Nuitka subprocess with the active interpreter's
    Conda DLL directories prepended to PATH.

    Nuitka locates Anaconda's shared libraries (e.g. `libcrypto-3-x64.dll`,
    which `_ssl.pyd` links against) by scanning Conda prefixes present on
    `PATH`. When the build is launched via an absolute interpreter path without
    `conda activate`, `<prefix>/Library/bin` is missing from `PATH`, so Nuitka
    cannot find those DLLs and the frozen binary crashes at startup
    (STATUS_STACK_BUFFER_OVERRUN 0xC0000409). Prepending them fixes discovery
    without requiring the caller to activate the environment first.
    """
    env = os.environ.copy()
    prefix = Path(sys.prefix)
    dll_dirs = [
        prefix,
        prefix / "Library" / "bin",
        prefix / "Library" / "mingw-w64" / "bin",
        prefix / "Library" / "usr" / "bin",
        prefix / "Scripts",
        prefix / "bin",
        prefix / "DLLs",
    ]
    existing = [str(d) for d in dll_dirs if d.is_dir()]
    if existing:
        env["PATH"] = os.pathsep.join([*existing, env.get("PATH", "")])
    return env


def _openssl_dll_args() -> list[str]:
    """Force-bundle Anaconda's OpenSSL DLLs into the frozen root.

    Nuitka's automatic dependency scan does not reliably collect these on all
    Anaconda layouts, so we bundle them explicitly next to the executable.
    Combined with `run.py`'s `os.add_dll_directory`, this guarantees `_ssl`
    loads the ABI-matched OpenSSL rather than a stray system copy.
    """
    src_dir = Path(sys.prefix) / "Library" / "bin"
    args: list[str] = []
    for name in OPENSSL_DLL_NAMES:
        dll = src_dir / name
        if dll.exists():
            args.append(f"--include-data-files={dll}={name}")
    return args


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
        "--include-module=ssl",
        "--include-module=_ssl",
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
        "--include-package=langchain_openai",
        "--include-package=langchain_anthropic",
        "--include-package=langchain_google_genai",
        "--nofollow-import-to=click._winconsole",
        "--assume-yes-for-downloads",
        "--no-deployment-flag=self-execution",
    ]
    args.extend(_openssl_dll_args())
    args.append("run.py")
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

    # Anaconda-safety: copy OpenSSL DLLs beside `_ssl.pyd` and expose the Conda
    # DLL dirs on PATH so Nuitka's dependency scan bundles the correct versions.
    _sync_anaconda_openssl_dlls()
    build_env = _build_env_with_conda_dlls()
    nuitka_args = _build_nuitka_args()

    print("Building MisakaX Agent Sidecar with Nuitka ...")
    print(f"  Working dir : {AGENT_DIR}")
    print(f"  Python      : {sys.executable}")
    print(f"  Command     : {' '.join(nuitka_args)}\n")

    t0 = time.monotonic()
    result = subprocess.run(nuitka_args, cwd=str(AGENT_DIR), env=build_env)
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
