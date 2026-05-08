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

## Project Structure

| Directory | Purpose |
|-----------|---------|
| `src/` | React frontend (Vite + TypeScript) |
| `src-tauri/src/` | Rust backend (Tauri 2.x) |
| `agent/` | Python Sidecar (FastAPI) |
| `docs/` | Documentation (`architecture/`, `planning/`, `research/`, `project/`, `design/`, `guides/`) |

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
cd agent && python -m uvicorn app.main:app --port 9527

# Rust checks
cargo check          # Fast compile check (from src-tauri/)
cargo test           # Run tests
```

## Environment Setup

- **Rust**: 1.95.0+ (`x86_64-pc-windows-gnu` target)
- **Node**: 20+ (via nvm)
- **Python**: 3.11.11 (via conda env `misaka`)
- **C compiler**: MinGW-w64 GCC 15.2.0 (via MSYS2 at `D:\soft\msys64\mingw64\bin`, configured in `.cargo/config.toml`)
- **Linker**: Rust self-contained `ld.exe` (binutils 2.42+)

## Key Technical Decisions

- Tailwind CSS v4 (CSS-first, no config file)
- shadcn/ui New York style with Zinc base
- SQLite WAL mode for concurrent reads
- `bundled` SQLite via rusqlite (no system dependency)
- `x86_64-pc-windows-gnu` target (GNU ABI, not MSVC)

## Code Style

- React 19 with function components and hooks
- Rust 2021 edition, standard module layout
- TypeScript strict mode
- No default exports (use named exports)
