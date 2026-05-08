---
description:
alwaysApply: true
---

# MisakaX — Desktop AI Agent Client

Cross-platform desktop AI Agent client built with Tauri 2.x (Rust), React 19 (TypeScript), and Python Sidecar (LangGraph + PowerMem).

## Architecture

```
┌────────────────────────────────────────────────┐
│  React 19 (TypeScript) + Tailwind CSS v4 +    │
│  shadcn/ui (New York/Zinc)                     │
│  → Vite 6 dev server (:1420)                  │
├────────────────────────────────────────────────┤
│  Tauri 2.x (Rust)                              │
│  - SQLite (WAL) + sqlite-vec + FTS5           │
│  - Config management (~/.misakax/config.yaml)  │
│  - Plugin system (shell, fs, http, etc.)       │
├────────────────────────────────────────────────┤
│  Python Sidecar (:9527)                        │
│  - FastAPI + uvicorn                           │
│  - LangGraph Agent orchestration               │
│  - PowerMem memory engine                      │
└────────────────────────────────────────────────┘
```

## Documentation layout (`docs/`)

**Do not** add new documents directly under `docs/` root. Place files under the closest category so paths stay searchable and reviewable.

| Directory | Use for |
|-----------|---------|
| `docs/architecture/` | System architecture, tech selection, structural decisions |
| `docs/planning/` | Roadmaps, phased plans, implementation breakdowns |
| `docs/research/` | Spikes, comparisons, notes on external stacks or APIs |
| `docs/project/` | Repo structure, onboarding, project meta |
| `docs/design/` | UI/UX specs and visual design artifacts |
| `docs/guides/` | How-tos, learning notes, environment diagnostics |

When nothing fits, extend an existing branch (e.g. `guides/<topic>/`) rather than leaving loose files in `docs/`. Prefer stable filenames and a short header inside each doc (purpose, audience, last reviewed).

## Locked stack versions (reference before codegen)

Agents must align generated code APIs and imports with the versions declared in repo manifests—not assumed “latest”.

### Frontend (`package.json`)

| Area | Declared range / pins |
|------|------------------------|
| Runtime | `react` ^19, `react-dom` ^19 |
| Build | `vite` ^6, `typescript` ~5.7, `@vitejs/plugin-react` ^4 |
| Styling | `tailwindcss` ^4.2.x, `@tailwindcss/vite` ^4.2.x |
| Desktop bridge | `@tauri-apps/api` ^2, `@tauri-apps/cli` ^2, plugins under `@tauri-apps/plugin-*` ^2 |
| Tests | `vitest` ^4.1.x, `@testing-library/react` ^16.x, `jsdom` ^29.x |

### Rust (`src-tauri/Cargo.toml`)

| Area | Declared range / pins |
|------|------------------------|
| Edition | Rust 2021 (`edition = "2021"`) |
| Shell | `tauri` 2, `tauri-build` 2, plugins (`tauri-plugin-*`) 2 |
| SQLite | `rusqlite` 0.34 (`bundled`, `vtab`), `sqlite-vec` 0.1 |
| Async / HTTP | `tokio` 1.x, `reqwest` 0.12 |
| LLM (Phase 2) | `rig-core` 0.36 |

Other crates (`serde`, `tracing`, `chrono`, etc.) follow the versions pinned in `Cargo.toml`; extend code against those entries.

### Python (`agent/pyproject.toml`)

| Area | Declared range / pins |
|------|------------------------|
| Interpreter | `requires-python >= 3.11` (project targets 3.11.x in env docs) |
| Web stack | `fastapi >= 0.115`, `uvicorn[standard] >= 0.34`, `pydantic >= 2` |
| HTTP client | `httpx >= 0.28` |
| Optional agent | `langgraph >= 0.3`, LangChain provider packages `>= 0.3` |
| Optional memory | `powermem >= 1.1` |

If dependency APIs change across minors, prefer checking the manifest first, then official docs for **that** major/minor line.

## Project Structure

| Directory | Purpose |
|-----------|---------|
| `src/` | React frontend (Vite + TypeScript) |
| `src-tauri/src/` | Rust backend (Tauri 2.x) |
| `agent/` | Python Sidecar (FastAPI) |
| `docs/` | Categorized documentation (see table above) |

## Dev Commands

```bash
# Frontend only
npm run dev          # Vite dev server on :1420

# Full app (frontend + Rust backend): clean Rust artifacts first, then dev
cd src-tauri
cargo clean
cd ..
npm run tauri dev    # Tauri dev mode

# Build
npm run build        # Frontend production build

# Full app build: clean Rust artifacts first
cd src-tauri
cargo clean
cd ..
npm run tauri build  # Full app build

# Python sidecar
cd agent
python -m uvicorn app.main:app --port 9527

# Rust checks (always from src-tauri/): clean before any cargo workflow
cd src-tauri
cargo clean
cargo check          # Fast compile check

cd src-tauri
cargo clean
cargo test           # Run tests

cd src-tauri
cargo clean
cargo build          # Standalone Rust build when not using npm wrapper
```

**Cargo hygiene:** Before `cargo check`, `cargo test`, `cargo build`, or npm scripts that invoke the Tauri/Rust build (`tauri dev`, `tauri build`), run `cargo clean` inside `src-tauri/` so stale `target/` artifacts do not accumulate across iterations.

## Environment Setup

- **Rust**: 1.95.0+ (`x86_64-pc-windows-msvc` target)
- **Node**: 20+ (via nvm)
- **Python**: 3.11.11 (via conda env `misaka`)
- **C/C++ compiler**: MSVC (via Visual Studio Build Tools)

## Key Technical Decisions

- Tailwind CSS v4 (CSS-first, no config file)
- shadcn/ui New York style with Zinc base
- SQLite WAL mode for concurrent reads
- `bundled` SQLite via rusqlite (no system dependency)
- `x86_64-pc-windows-msvc` target (MSVC ABI)

## Code Style

- React 19 with function components and hooks
- Rust 2021 edition, standard module layout
- TypeScript strict mode
- No default exports (use named exports)
