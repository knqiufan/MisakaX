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

# Full app (frontend + Rust backend)
npm run tauri dev    # Tauri dev mode

# Build
npm run build        # Frontend production build
npm run tauri build  # Full app build

# Python sidecar
cd agent
python -m uvicorn app.main:app --port 9527

# Rust checks (from src-tauri/; use incremental compile — do not cargo clean routinely)
cd src-tauri
cargo check          # Fast compile check
cargo test --test crypto_tests   # Targeted test during daily work (see mapping in docs/guides/rust-build-test-optimization.md §4.2)
cargo test --features test-private --test chat_commands_tests   # Commands tests that need test-private
cargo test           # Full suite before commit / in CI
cargo build          # Standalone Rust build when not using npm wrapper
```

**Cargo hygiene:** Do **not** run `cargo clean` before routine `cargo check`, `cargo test`, `cargo build`, `tauri dev`, or `tauri build` — incremental compile is intentional and much faster. Run `cargo clean` in `src-tauri/` only when troubleshooting: link errors, metadata mismatch, unexplained failures after a branch switch, or Rust toolchain / major dependency upgrades. See [`docs/guides/rust-build-test-optimization.md`](docs/guides/rust-build-test-optimization.md).

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
- **Frontend UI/UX**: Any time you author or refactor React UI or styling (`src/**/*.tsx`, shared CSS tokens, shell layout), **read and comply with** the project’s UI specs (they are complementary, not optional pick-one):
  - **Path-scoped reinforcement** (loads when editing matching files—reduces “forgot to load Codex” cases):
    - **Cursor**: `.cursor/rules/misaka-frontend-ui-specs.mdc` (`globs`: `src/**/*.tsx`, `src/**/*.css`, `src/*.css`)
    - **Codex**: `.Codex/rules/misaka-frontend-ui-specs.md` (`paths` frontmatter, same patterns)
  - **Global motion, color, and component tone** (desktop Agent style; no web-like bouncy/scaling): `docs/design/frontend-ui-guidelines.md`
  - **Main nav vs session column vs workspace chrome** (collapse semantics, session toolbar, workspace directory bar): `docs/design/shell-and-workspace-ui-spec.md`
  - **Buttons, menus, popovers, selects, dialogs, tooltips, toasts, and related controls**: `docs/design/button-menu-design-spec.md`
- **After UI work (mandatory doc sync)**: When the user asks to **change, adjust, polish, or refactor UI/UX** (layout, chrome, components, tokens, copy placement, interaction), do **not** stop at code only. When the implementation is done:
  1. **Extract reusable rules** — Summarize what future work must respect (semantics, sizing, tokens, i18n/a11y hooks, what *not* to do). Skip one-off bug narration; keep **normative, concise** bullets others can follow.
  2. **Patch the right spec(s)** — Update the smallest set of existing files under `docs/design/` (usually one of the three above). Add or revise sections so the new behavior is documented. If a rule clearly belongs in two docs, cross-link instead of duplicating long text.
  3. **Maintain doc hygiene** — Bump the **“最后审阅 / Last reviewed”** date in the edited spec’s header/metadata when present; keep headings and tables consistent with that file’s style.
  This is **automatic follow-through** for agents: the user should **not** need to ask for a separate “update the design doc” step after each UI task.
