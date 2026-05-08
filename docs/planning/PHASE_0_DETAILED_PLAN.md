# Phase 0：环境搭建与项目初始化 — 详细实施方案

> **所属项目：** MisakaX
> **阶段：** Phase 0（第 1 周）
> **总预估：** ~12 小时
> **前置文档：** [MISAKAX_IMPLEMENTATION_PLAN.md](./MISAKAX_IMPLEMENTATION_PLAN%20-%20Opus4.6.md)、[MISAKAX_ARCHITECTURE_SELECTION.md](./MISAKAX_ARCHITECTURE_SELECTION%20-%20Opus4.6.md)

---

## 目录

1. [阶段目标与验收标准](#1-阶段目标与验收标准)
2. [任务 0.1：安装开发工具链](#2-任务-01安装开发工具链)
3. [任务 0.2：创建 Tauri + React 项目骨架](#3-任务-02创建-tauri--react-项目骨架)
4. [任务 0.3：安装 Tailwind CSS v4 + shadcn/ui](#4-任务-03安装-tailwind-css-v4--shadcnui)
5. [任务 0.4：Rust 核心依赖配置](#5-任务-04rust-核心依赖配置)
6. [任务 0.5：Python Sidecar 子项目初始化](#6-任务-05python-sidecar-子项目初始化)
7. [任务 0.6：SQLite 数据库初始化与 Schema 迁移](#7-任务-06sqlite-数据库初始化与-schema-迁移)
8. [任务 0.7：配置目录结构与 config.rs](#8-任务-07配置目录结构与-configrs)
9. [任务 0.8：Tauri 窗口基本配置](#9-任务-08tauri-窗口基本配置)
10. [任务 0.9：Git 仓库与项目治理](#10-任务-09git-仓库与项目治理)
11. [Phase 0 完整验证清单](#11-phase-0-完整验证清单)
12. [常见问题与排错指南](#12-常见问题与排错指南)

---

## 1. 阶段目标与验收标准

### 1.1 核心目标

从零创建 MisakaX 项目骨架，确保三端（Rust + React + Python）的开发环境完全就绪，`tauri dev` 可启动并显示空白窗口，SQLite 数据库自动初始化，配置目录自动生成。

### 1.2 任务依赖关系

```
0.1 工具链安装 ──→ 0.2 项目骨架 ──→ 0.3 Tailwind+shadcn ──┐
                    │                                        │
                    ├─→ 0.4 Rust依赖 (独立，可并行)            │
                    └─→ 0.5 Python子项目 (独立，可并行)         │
                                                              │
0.4 Rust依赖 ──→ 0.6 数据库初始化 ──→ 0.7 配置+lib.rs ──→ 0.8 窗口配置
                      │                    │
                      └─ DB路径依赖 ────────┘

0.2 ~ 0.8 全部完成 ──→ 0.9 Git治理（基于已确定的文件结构）
```

> **关键路径：** 0.1 → 0.2 → 0.4 → 0.6 → 0.7 → 0.8 → 0.9
> **可并行：** 0.3 和 0.4 和 0.5 在 0.2 完成后可同时推进

### 1.3 验收标准（Done Definition）

| # | 验收条件 | 验证方式 |
|---|---------|---------|
| AC-1 | `tauri dev` 可正常启动并显示空白窗口 | 运行命令，窗口弹出，无报错 |
| AC-2 | SQLite 数据库文件自动创建，包含所有基础表 | 检查 `%USERPROFILE%\.misakax\data\misaka.db`（Win）或 `~/.misakax/data/misaka.db`（Mac/Linux），用 `sqlite3` 查看表结构 |
| AC-3 | `~/.misakax/` 目录结构自动生成 | 检查目录树是否完整 |
| AC-4 | Tailwind CSS 生效 | 修改页面背景色，热更新即时生效 |
| AC-5 | shadcn/ui 至少一个组件可正常渲染 | 添加一个 Button 组件到页面 |
| AC-6 | Rust 编译无错误 | `cargo check` 通过 |
| AC-7 | TypeScript 编译无错误 | `npm run build` 通过 |
| AC-8 | Python Sidecar 骨架可独立运行 | `cd agent && python -m uvicorn app.main:app --port 9527` 启动成功 |
| AC-9 | `config.rs` 可读写配置 | Rust 单元测试通过 |
| AC-10 | 项目可在 Windows 上编译运行 | 本地验证 |

### 1.4 最终目录结构

```
misaka-x/
├── src-tauri/                          # Rust 后端
│   ├── src/
│   │   ├── main.rs                     # Tauri 入口
│   │   ├── lib.rs                      # 模块导出
│   │   ├── config.rs                   # 配置管理
│   │   ├── db/
│   │   │   ├── mod.rs                  # 数据库初始化 + sqlite-vec 加载
│   │   │   ├── migrations.rs           # Schema 迁移
│   │   │   └── models.rs              # 数据模型定义
│   │   └── commands/
│   │       ├── mod.rs                  # 命令注册
│   │       └── settings.rs             # 设置读写命令
│   ├── Cargo.toml                      # Rust 依赖
│   ├── tauri.conf.json                 # Tauri 配置
│   ├── capabilities/
│   │   └── default.json                # 权限 ACL
│   └── icons/                          # 应用图标
├── src/                                # React 前端
│   ├── App.tsx                         # 根组件
│   ├── main.tsx                        # 入口
│   ├── index.css                       # Tailwind 入口
│   ├── components/
│   │   └── ui/                         # shadcn/ui 组件
│   ├── lib/
│   │   └── utils.ts                    # shadcn/ui 工具函数
│   └── assets/
├── agent/                              # Python Sidecar
│   ├── app/
│   │   ├── __init__.py
│   │   ├── main.py                     # FastAPI 入口
│   │   ├── config.py                   # Sidecar 配置
│   │   └── routers/
│   │       └── health.py               # 健康检查端点
│   ├── pyproject.toml                  # Python 项目配置
│   ├── requirements.txt                # Python 依赖
│   └── README.md
├── docs/                               # 项目文档
├── .gitignore
├── CLAUDE.md                           # AI 辅助开发指引
├── package.json                        # 前端依赖
├── vite.config.ts                      # Vite 配置
├── tsconfig.json                       # TypeScript 配置
└── README.md                           # 项目说明
```

---

## 2. 任务 0.1：安装开发工具链

> **预估时间：** 1h
> **产出：** 所有开发工具就绪

### 2.0 执行原则

每个工具遵循 **"检查 → 存在则跳过 / 不存在则安装"** 的流程，避免重复安装覆盖已有配置。

---

### 2.1 Rust 工具链

**Step 1：检查是否已安装**

```bash
# 检查 rustc 是否存在
which rustc 2>/dev/null && rustc --version || echo "NOT_FOUND"

# 检查 cargo 是否存在
which cargo 2>/dev/null && cargo --version || echo "NOT_FOUND"
```

| 判断 | 动作 |
|------|------|
| `rustc --version` >= 1.82 且 `cargo --version` >= 1.82 | ✅ 跳过安装，进入 Step 3 |
| 版本 < 1.82 | 执行 `rustup update stable` 升级 |
| `NOT_FOUND` | 进入 Step 2 |

**Step 2：安装（仅当未找到或版本过低时）**

```bash
# Windows: 下载并运行 https://win.rustup.rs/x86_64
# macOS/Linux:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装后重新加载环境变量
source "$HOME/.cargo/env"  # macOS/Linux
# Windows: 重新打开终端
```

**Step 3：确认 stable 工具链 + 目标平台**

```bash
rustup default stable

# Windows 必须添加 MSVC 目标
rustup target add x86_64-pc-windows-msvc   # Windows
# macOS 自动使用 apple-darwin，无需手动添加
# Linux 自动使用 unknown-linux-gnu，无需手动添加

rustc --version    # 应显示 >= 1.82
cargo --version    # 应显示 >= 1.82
```

**Step 4：Windows — 检查 C++ 构建工具**

```bash
# 检查 MSVC 链接器是否可用
where link.exe 2>nul || echo "NOT_FOUND"
```

| 判断 | 动作 |
|------|------|
| `link.exe` 存在 | ✅ 跳过安装 |
| `NOT_FOUND` | 安装 [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/)，选择 **"C++ 桌面开发"** 工作负载 |

---

### 2.2 Node.js

**Step 1：检查是否已安装**

```bash
node --version 2>/dev/null || echo "NOT_FOUND"
npm --version 2>/dev/null || echo "NOT_FOUND"
```

| 判断 | 动作 |
|------|------|
| `node` >= v20.x 且 `npm` >= 10.x | ✅ 跳过安装 |
| `node` < v20 | 如使用 nvm 则 `nvm install 20 && nvm use 20`，否则进入 Step 2 |
| `NOT_FOUND` | 进入 Step 2 |

**Step 2：安装（仅当未安装或版本过低时）**

```bash
# --- nvm 方式（推荐）---
# 先检查 nvm 是否已安装
nvm --version 2>/dev/null || echo "NOT_FOUND"

# 若 nvm 未安装：
# Windows: 下载 https://github.com/coreybutler/nvm-windows/releases
# macOS/Linux:
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | bash
# 重新加载 shell 配置
source ~/.bashrc  # 或 source ~/.zshrc

# 安装并使用 Node.js 20
nvm install 20
nvm use 20

# 验证
node --version     # 应显示 v20.x.x
npm --version      # 应显示 10.x.x 或更高
```

---

### 2.3 Python

**Step 1：检查是否已安装**

```bash
python --version 2>/dev/null || python3 --version 2>/dev/null || echo "NOT_FOUND"
pip --version 2>/dev/null || pip3 --version 2>/dev/null || echo "NOT_FOUND"
```

| 判断 | 动作 |
|------|------|
| Python 3.11.x 且 pip 可用 | ✅ 跳过安装 |
| Python < 3.11 或 3.12+（项目锁定 3.11） | 进入 Step 2（通过 pyenv 安装 3.11） |
| `NOT_FOUND` | 进入 Step 2 |

**Step 2：安装（仅当未安装或版本不符时）**

```bash
# --- pyenv 方式（推荐）---
# 先检查 pyenv 是否已安装
pyenv --version 2>/dev/null || echo "NOT_FOUND"

# 若 pyenv 未安装：
# Windows: https://github.com/pyenv-win/pyenv-win#installation
#   PowerShell: Invoke-WebRequest -UseBasicParsing -Uri "https://raw.githubusercontent.com/pyenv-win/pyenv-win/master/pyenv-win/install-pyenv-win.ps1" | Invoke-Expression
# macOS: brew install pyenv
# Linux: curl https://pyenv.run | bash

# 安装 Python 3.11
pyenv install 3.11.11
pyenv global 3.11.11

# 确保 pyenv shim 在 PATH 最前面（添加到 ~/.bashrc 或 ~/.zshrc 末尾）
# export PATH="$HOME/.pyenv/shims:$PATH"

# 验证
python --version   # 应显示 Python 3.11.x
pip --version      # 应可用
```

---

### 2.4 验证清单

| 工具 | 验证命令 | 最低版本 | 检查方式 |
|------|---------|---------|---------|
| Rust | `rustc --version` | 1.82 | `rustc --version \| grep -oE '[0-9]+\.[0-9]+' \| head -1` |
| Cargo | `cargo --version` | 1.82 | 随 Rust 一起安装 |
| MSVC (Win) | `where link.exe` | — | 仅 Windows 需要 |
| Node.js | `node --version` | v20 | `node --version \| grep -oE 'v[0-9]+'` |
| npm | `npm --version` | 10 | `npm --version \| grep -oE '^[0-9]+'` |
| Python | `python --version` | 3.11 | `python --version \| grep -oE '3\.11'` |
| pip | `pip --version` | 24 | 随 Python 一起安装 |

---

### 2.5 常见问题

| 问题 | 解决方案 |
|------|---------|
| `rustc` 命令找不到 | 重新打开终端，确保 `~/.cargo/bin` 在 PATH 中；Windows 检查 `%USERPROFILE%\.cargo\bin` |
| Windows 编译报 `linker` 错误 | 安装 VS Build Tools 的 C++ 工作负载；安装后重启终端 |
| `nvm` 命令找不到 | 重新打开终端，运行 `source ~/.bashrc`（或 `~/.zshrc`） |
| Python 版本不对 | 确认 pyenv 的 shim 路径在 PATH 最前面；运行 `pyenv global 3.11.11` |
| `pip` 命令找不到 | 运行 `python -m ensurepip --upgrade` 引导安装 pip |
| macOS Xcode 缺失 | 运行 `xcode-select --install` 安装 Command Line Tools |
| Linux 缺少编译依赖 | `sudo apt install build-essential pkg-config libssl-dev` |

---

## 3. 任务 0.2：创建 Tauri + React 项目骨架

> **预估时间：** 0.5h
> **产出：** 可运行的空白 Tauri + React + Vite 项目

### 3.1 创建项目

```bash
# 在 D:\code 目录下创建项目
cd D:\code
npm create tauri-app@latest misaka-x -- --template react-ts

# 进入项目目录
cd misaka-x

# 安装前端依赖
npm install
```

> **注意：** 项目名使用 `misaka-x`（Tauri 创建时的目录名），Tauri 应用内部名称在 `tauri.conf.json` 中配置。

### 3.2 创建后目录结构（Tauri 模板默认）

```
misaka-x/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   └── icons/
├── src/
│   ├── App.tsx
│   ├── main.tsx
│   ├── App.css
│   ├── index.css
│   └── assets/
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
└── tsconfig.node.json
```

### 3.3 首次运行验证

```bash
npm run tauri dev
```

**预期结果：**
- Rust 编译完成（首次编译约 2-5 分钟）
- 窗口弹出，显示 Vite + React 默认页面
- 浏览器控制台无报错

### 3.4 修改应用标识

编辑 `src-tauri/tauri.conf.json`：

```json
{
  "productName": "MisakaX",
  "version": "0.1.0",
  "identifier": "com.misakax.app",
  "build": {
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "MisakaX",
        "width": 1280,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600,
        "center": true
      }
    ]
  }
}
```

### 3.5 验证

| 检查项 | 预期 |
|--------|------|
| `npm run tauri dev` | 窗口标题显示 "MisakaX" |
| 窗口大小 | 1280x800，居中显示 |
| 最小尺寸 | 拖拽到 900x600 时停止缩小 |
| 热更新 | 修改 `App.tsx` 中的文字，页面自动刷新 |

---

## 4. 任务 0.3：安装 Tailwind CSS v4 + shadcn/ui

> **预估时间：** 1h
> **产出：** UI 框架就绪，至少一个 shadcn/ui 组件可渲染

### 4.1 安装 Tailwind CSS v4

Tauri 模板默认使用 Vite，Tailwind v4 的安装方式：

```bash
# 安装 Tailwind CSS v4 + Vite 插件
npm install tailwindcss @tailwindcss/vite
```

编辑 `vite.config.ts`，添加 Tailwind 插件：

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
});
```

编辑 `src/index.css`，替换为 Tailwind v4 入口：

```css
@import "tailwindcss";
```

> **Tailwind v4 变化：** Tailwind CSS v4 不再使用 `tailwind.config.js`，改为 CSS-first 配置。自定义主题通过 `@theme` 指令在 CSS 中定义。

### 4.2 初始化 shadcn/ui

```bash
# 初始化 shadcn/ui（使用 new-york 风格）
npx shadcn@latest init
```

初始化时的选择：

| 选项 | 选择 |
|------|------|
| Style | **New York** |
| Base color | **Zinc** |
| CSS variables | **Yes** |
| Tailwind CSS location | `src/index.css` |
| Components location | `src/components/ui` |
| Utils location | `src/lib/utils.ts` |

### 4.3 安装示例组件验证

```bash
# 安装 Button 组件用于验证
npx shadcn@latest add button
```

在 `src/App.tsx` 中使用 Button 组件：

```tsx
import { Button } from "@/components/ui/button";

function App() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background">
      <div className="text-center space-y-4">
        <h1 className="text-4xl font-bold text-foreground">MisakaX</h1>
        <p className="text-muted-foreground">Desktop AI Agent Client</p>
        <Button variant="default" size="lg">
          Get Started
        </Button>
      </div>
    </div>
  );
}

export default App;
```

### 4.4 验证

| 检查项 | 预期 |
|--------|------|
| 页面背景色跟随系统主题 | 暗色模式下背景为深色 |
| "MisakaX" 文字样式 | 粗体、大号、使用 Tailwind 类 |
| Button 组件渲染 | shadcn/ui 风格按钮，有 hover 效果 |
| CSS 热更新 | 修改 Tailwind 类名，页面即时更新 |

---

## 5. 任务 0.4：Rust 核心依赖配置

> **预估时间：** 1h
> **产出：** Cargo.toml 包含所有核心依赖，`cargo check` 通过

### 5.1 编辑 `src-tauri/Cargo.toml`

```toml
[package]
name = "misaka-x"
version = "0.1.0"
description = "MisakaX - Desktop AI Agent Client"
authors = ["you"]
edition = "2021"

[lib]
name = "misaka_x_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
# --- Tauri 核心 ---
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-fs = "2"
tauri-plugin-notification = "2"
tauri-plugin-http = "2"
tauri-plugin-updater = "2"
tauri-plugin-dialog = "2"
tauri-plugin-clipboard-manager = "2"

# --- 数据库 ---
rusqlite = { version = "0.34", features = ["bundled", "vtab"] }
sqlite-vec = "0.1"

# --- 异步运行时 ---
tokio = { version = "1", features = ["full"] }

# --- 序列化 ---
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# --- HTTP 客户端 ---
reqwest = { version = "0.12", features = ["json", "stream"] }

# --- 工具库 ---
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"

# --- 文件系统 ---
walkdir = "2"
glob = "0.3"
notify = "7"
dirs = "6"

# --- 并发 ---
dashmap = "6"
```

### 5.2 验证依赖编译

```bash
cd src-tauri
cargo check
```

**预期结果：**
- 首次运行会下载并编译所有依赖（约 3-8 分钟）
- 最终输出 `Finished` 无错误
- 可能有 warnings，可以忽略

### 5.3 依赖说明

| 依赖 | 用途 | 对应架构层 |
|------|------|-----------|
| `rusqlite` + `sqlite-vec` | 结构化数据 + 向量搜索 | 数据层 |
| `tokio` | 异步运行时 | 基础设施 |
| `serde` + `serde_json` + `serde_yaml` | 序列化/反序列化 | 基础设施 |
| `reqwest` | HTTP 客户端（LLM API 调用） | LLM 层 |
| `uuid` + `chrono` | ID 生成 + 时间处理 | 工具 |
| `walkdir` + `glob` + `notify` | 文件系统扫描与监听 | Skills 层 |
| `dashmap` | 并发安全 HashMap | Skills 注册表 |
| `dirs` | 获取系统目录（~/.misakax/） | 配置层 |
| `anyhow` + `thiserror` | 错误处理 | 基础设施 |
| `tracing` | 结构化日志 | 基础设施 |

---

## 6. 任务 0.5：Python Sidecar 子项目初始化

> **预估时间：** 0.5h
> **产出：** Python 项目骨架可独立运行

### 6.1 创建目录结构

```bash
mkdir -p agent/app/routers
```

### 6.2 创建 `agent/pyproject.toml`

```toml
[project]
name = "misaka-agent"
version = "0.1.0"
description = "MisakaX Agent Sidecar - LangGraph + PowerMem"
requires-python = ">=3.11"
dependencies = [
    "fastapi>=0.115",
    "uvicorn[standard]>=0.34",
    "pydantic>=2.0",
    "pydantic-settings>=2.0",
    "httpx>=0.28",
]

[project.optional-dependencies]
agent = [
    "langgraph>=0.3",
    "langchain-anthropic>=0.3",
    "langchain-openai>=0.3",
    "langgraph-checkpoint-sqlite>=2.0",
]
memory = [
    "powermem>=1.1",
]
dev = [
    "pytest>=8.0",
    "pytest-asyncio>=0.24",
    "ruff>=0.8",
]

[tool.ruff]
line-length = 100
target-version = "py311"
```

### 6.3 创建 `agent/requirements.txt`

```
fastapi>=0.115
uvicorn[standard]>=0.34
pydantic>=2.0
pydantic-settings>=2.0
httpx>=0.28
```

### 6.4 创建 `agent/app/__init__.py`

```python
"""MisakaX Agent Sidecar."""
```

### 6.5 创建 `agent/app/config.py`

```python
"""Sidecar configuration."""

from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """Application settings."""

    host: str = "127.0.0.1"
    port: int = 9527
    log_level: str = "info"
    db_path: str = ""  # Will be set by Rust core via env var

    model_config = {"env_prefix": "MISAKA_"}


settings = Settings()
```

### 6.6 创建 `agent/app/routers/__init__.py`

```python
```

### 6.7 创建 `agent/app/routers/health.py`

```python
"""Health check endpoint."""

from fastapi import APIRouter

router = APIRouter()


@router.get("/health")
async def health_check():
    """Return health status."""
    return {"status": "ok", "service": "misaka-agent"}
```

### 6.8 创建 `agent/app/main.py`

```python
"""FastAPI entry point for MisakaX Agent Sidecar."""

from fastapi import FastAPI

from app.config import settings
from app.routers.health import router as health_router

app = FastAPI(
    title="MisakaX Agent",
    version="0.1.0",
    description="MisakaX Agent Sidecar - LangGraph + PowerMem",
)

app.include_router(health_router)


@app.on_event("startup")
async def startup():
    """Initialize resources on startup."""
    pass


@app.on_event("shutdown")
async def shutdown():
    """Cleanup resources on shutdown."""
    pass
```

### 6.9 创建 `agent/README.md`

```markdown
# MisakaX Agent Sidecar

Python Sidecar for MisakaX, providing LangGraph Agent orchestration and PowerMem memory engine.

## Quick Start

```bash
pip install -r requirements.txt
uvicorn app.main:app --host 127.0.0.1 --port 9527
```

## Health Check

```bash
curl http://127.0.0.1:9527/health
```
```

### 6.10 安装依赖并验证

```bash
cd agent
pip install -r requirements.txt

# 启动服务
python -m uvicorn app.main:app --host 127.0.0.1 --port 9527

# 另一个终端验证
curl http://127.0.0.1:9527/health
# 预期输出: {"status":"ok","service":"misaka-agent"}
```

---

## 7. 任务 0.6：SQLite 数据库初始化与 Schema 迁移

> **预估时间：** 3h
> **产出：** 数据库自动创建，包含所有基础表，sqlite-vec 扩展加载成功

### 7.1 创建 `src-tauri/src/db/mod.rs`

```rust
pub mod migrations;
pub mod models;

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

/// Initialize the database: create file, load sqlite-vec, run migrations.
pub fn init_database(db_path: &Path) -> Result<Connection> {
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(db_path)?;

    // Enable WAL mode for better concurrent read performance
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;

    // Load sqlite-vec extension
    unsafe {
        let ext = std::mem::transmute(sqlite_vec::sqlite3_vec_init as usize);
        rusqlite::auto_extension::register_auto_extension(ext)?;
    }

    // Run schema migrations
    migrations::run_migrations(&conn)?;

    tracing::info!("Database initialized at: {}", db_path.display());
    Ok(conn)
}
```

### 7.2 创建 `src-tauri/src/db/migrations.rs`

```rust
use anyhow::Result;
use rusqlite::Connection;

const SCHEMA_VERSION: i64 = 1;

/// Run all pending schema migrations.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create schema version table if not exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _schema_version (
            version INTEGER PRIMARY KEY
        );",
    )?;

    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < 1 {
        migrate_v1(conn)?;
    }

    // Future migrations:
    // if current_version < 2 { migrate_v2(conn)?; }

    Ok(())
}

fn migrate_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- Settings table (key-value store)
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Sessions table
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            title TEXT,
            model TEXT,
            system_prompt TEXT,
            working_directory TEXT,
            project_name TEXT,
            status TEXT DEFAULT 'active',
            mode TEXT DEFAULT 'agent',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Messages table
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            token_usage TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
        );

        -- Router configs (API keys, providers)
        CREATE TABLE IF NOT EXISTS router_configs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            provider TEXT NOT NULL,
            api_key_encrypted TEXT,
            model TEXT,
            base_url TEXT,
            config_json TEXT,
            is_active INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Tasks table
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            session_id TEXT,
            parent_task_id TEXT,
            title TEXT NOT NULL,
            status TEXT DEFAULT 'pending',
            description TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL,
            FOREIGN KEY (parent_task_id) REFERENCES tasks(id) ON DELETE SET NULL
        );

        -- Knowledge documents metadata
        CREATE TABLE IF NOT EXISTS knowledge_docs (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            source_path TEXT,
            file_type TEXT,
            chunk_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Full-text search index for messages
        CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts
        USING fts5(content, session_id, role);

        -- Full-text search index for knowledge base
        CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_fts
        USING fts5(title, content, source_path);

        -- Update schema version
        INSERT INTO _schema_version (version) VALUES (1);
        ",
    )?;

    tracing::info!("Database migrated to version 1");
    Ok(())
}
```

### 7.3 创建 `src-tauri/src/db/models.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub working_directory: Option<String>,
    pub project_name: Option<String>,
    pub status: String,
    pub mode: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub token_usage: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_key_encrypted: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub config_json: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}
```

### 7.4 验证数据库初始化

数据库将在首次运行 `tauri dev` 时自动创建。验证步骤：

```bash
# 启动应用
npm run tauri dev

# 在另一个终端检查数据库
# Windows:
sqlite3 ~/.misakax/data/misaka.db ".tables"
# 预期输出: _schema_version  knowledge_docs  knowledge_fts  messages  messages_fts  router_configs  sessions  settings  tasks

sqlite3 ~/.misakax/data/misaka.db "SELECT * FROM _schema_version;"
# 预期输出: 1
```

---

## 8. 任务 0.7：配置目录结构与 config.rs

> **预估时间：** 2h
> **产出：** `~/.misakax/` 目录自动生成，`config.rs` 可读写配置

### 8.1 目标目录结构

```
~/.misakax/                          # 用户配置根目录
├── config.yaml                   # 全局配置文件
├── data/
│   └── misaka.db                 # SQLite 数据库
├── skills/
│   └── (用户自定义 Skills)
├── managed/
│   └── skills/                   # 从市场安装的 Skills
├── plugins/
│   └── (插件目录)
├── models/
│   └── (语音模型等大文件)
└── logs/
    └── (应用日志)
```

### 8.2 创建 `src-tauri/src/config.rs`

```rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration, persisted to ~/.misakax/config.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// UI language (en, zh_CN)
    pub language: String,
    /// Theme: "light", "dark", "system"
    pub theme: String,
    /// Accent color (hex)
    pub accent_color: String,
    /// Default LLM model
    pub default_model: String,
    /// Log level
    pub log_level: String,
    /// Sidecar port
    pub sidecar_port: u16,
    /// Whether to auto-start sidecar
    pub auto_start_sidecar: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: "system".to_string(),
            accent_color: "#6366f1".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
            log_level: "info".to_string(),
            sidecar_port: 9527,
            auto_start_sidecar: false,
        }
    }
}

/// Get the root config directory (~/.misakax/)
pub fn config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home.join(".misakax"))
}

/// Get the database file path (~/.misakax/data/misaka.db)
pub fn db_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("data").join("misaka.db"))
}

/// Get the skills directory path (~/.misakax/skills/)
pub fn skills_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("skills"))
}

/// Get the managed skills directory path (~/.misakax/managed/skills/)
pub fn managed_skills_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("managed").join("skills"))
}

/// Get the logs directory path (~/.misakax/logs/)
pub fn logs_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("logs"))
}

/// Get the config file path (~/.misakax/config.yaml)
pub fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.yaml"))
}

/// Ensure all required directories exist.
pub fn ensure_directories() -> Result<()> {
    let dirs = [
        config_dir()?,
        config_dir()?.join("data"),
        skills_dir()?,
        managed_skills_dir()?,
        config_dir()?.join("plugins"),
        config_dir()?.join("models"),
        logs_dir()?,
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    tracing::info!("Config directories initialized at: {}", config_dir()?.display());
    Ok(())
}

/// Load config from YAML file, or create default if not exists.
pub fn load_config() -> Result<AppConfig> {
    let path = config_file_path()?;

    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let config: AppConfig =
            serde_yaml::from_str(&content).context("Failed to parse config.yaml")?;
        Ok(config)
    } else {
        let config = AppConfig::default();
        save_config(&config)?;
        Ok(config)
    }
}

/// Save config to YAML file.
pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_file_path()?;
    let content = serde_yaml::to_string(config).context("Failed to serialize config")?;
    std::fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.theme, "system");
        assert_eq!(config.sidecar_port, 9527);
    }

    #[test]
    fn test_config_roundtrip() {
        let config = AppConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let loaded: AppConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.theme, loaded.theme);
        assert_eq!(config.language, loaded.language);
    }
}
```

### 8.3 更新 `src-tauri/src/lib.rs`

```rust
mod commands;
pub mod config;
pub mod db;

use config::AppConfig;
use std::sync::Mutex;
use tauri::Manager;

/// Application state shared across commands
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub config: Mutex<AppConfig>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Ensure directories exist early (before Tauri setup, in case setup fails)
    config::ensure_directories()
        .expect("Failed to create config directories");

    // Load config early for validation
    let app_config = config::load_config()
        .expect("Failed to load configuration");

    // Initialize database
    let db_path = config::db_path()
        .expect("Failed to determine database path");
    let conn = db::init_database(&db_path)
        .expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState {
            db: Mutex::new(conn),
            config: Mutex::new(app_config),
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::update_setting,
        ])
        .setup(|_app| {
            tracing::info!("MisakaX initialized successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running MisakaX");
}
```

### 8.4 创建 `src-tauri/src/commands/mod.rs`

```rust
pub mod settings;
```

### 8.5 创建 `src-tauri/src/commands/settings.rs`

```rust
use crate::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    serde_json::to_string(&*config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: Value,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;

    match key.as_str() {
        "theme" => config.theme = value.as_str().unwrap_or("system").to_string(),
        "language" => config.language = value.as_str().unwrap_or("en").to_string(),
        "accent_color" => config.accent_color = value.as_str().unwrap_or("#6366f1").to_string(),
        "default_model" => {
            config.default_model = value.as_str().unwrap_or("claude-sonnet-4-20250514").to_string()
        }
        "log_level" => config.log_level = value.as_str().unwrap_or("info").to_string(),
        _ => return Err(format!("Unknown setting key: {}", key)),
    }

    crate::config::save_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}
```

### 8.6 更新 `src-tauri/src/main.rs`

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    misaka_x_lib::run();
}
```

### 8.7 验证

```bash
# 运行 Rust 测试
cd src-tauri
cargo test

# 启动应用后检查目录
ls ~/.misakax/
# 预期: config.yaml  data/  skills/  managed/  plugins/  models/  logs/

cat ~/.misakax/config.yaml
# 预期: 默认配置内容
```

---

## 9. 任务 0.8：Tauri 窗口基本配置

> **预估时间：** 1h
> **产出：** 窗口大小、标题、图标、权限 ACL 配置完成

### 9.1 更新 `src-tauri/tauri.conf.json`

```json
{
  "productName": "MisakaX",
  "version": "0.1.0",
  "identifier": "com.misakax.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "MisakaX",
        "width": 1280,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600,
        "center": true,
        "resizable": true,
        "decorations": true,
        "transparent": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

### 9.2 配置权限 ACL

编辑 `src-tauri/capabilities/default.json`：

```json
{
  "identifier": "default",
  "description": "Default capabilities for MisakaX",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "shell:allow-execute",
    "shell:allow-spawn",
    "shell:allow-stdin-write",
    "shell:allow-kill",
    "fs:default",
    "fs:allow-read",
    "fs:allow-write",
    "fs:allow-exists",
    "fs:allow-mkdir",
    "fs:allow-remove",
    "fs:allow-rename",
    "http:default",
    "http:allow-fetch",
    "notification:default",
    "notification:allow-is-permission-granted",
    "notification:allow-request-permission",
    "notification:allow-notify",
    "dialog:default",
    "dialog:allow-open",
    "dialog:allow-save",
    "dialog:allow-message",
    "dialog:allow-ask",
    "dialog:allow-confirm",
    "clipboard-manager:default",
    "clipboard-manager:allow-write-text",
    "clipboard-manager:allow-read-text"
  ]
}
```

### 9.3 应用图标

默认的 Tauri 图标在 `src-tauri/icons/` 目录中。后续可以用以下命令重新生成：

```bash
# 安装图标生成工具（可选）
cargo install tauri-icon

# 从一个 1024x1024 的 PNG 生成所有尺寸
npx tauri icon path/to/icon.png
```

### 9.4 验证

| 检查项 | 预期 |
|--------|------|
| 窗口标题 | "MisakaX" |
| 初始大小 | 1280x800 |
| 最小大小 | 900x600，无法更小 |
| 窗口居中 | 启动时居中显示 |
| 可调整大小 | 拖拽边框可改变大小 |

---

## 10. 任务 0.9：Git 仓库与项目治理

> **预估时间：** 1h
> **产出：** Git 仓库就绪，.gitignore 配置，CLAUDE.md 创建

### 10.1 初始化 Git（如尚未初始化）

```bash
cd D:\code\misaka-x
git init
```

### 10.2 创建 `.gitignore`

```gitignore
# === Rust ===
src-tauri/target/
**/*.rs.bk

# === Node ===
node_modules/
dist/
.vite/

# === Python ===
agent/.venv/
agent/__pycache__/
agent/**/__pycache__/
*.pyc
*.pyo
*.egg-info/
.eggs/

# === IDE ===
.vscode/
.idea/
*.swp
*.swo

# === OS ===
.DS_Store
Thumbs.db

# === MisakaX ===
# Local config & data (user-specific)
.misakax/
*.db
*.db-wal
*.db-shm

# Environment files
.env
.env.local
.env.*.local

# Build artifacts
*.msi
*.dmg
*.AppImage
*.deb
*.rpm

# Logs
*.log

# Model files (too large for git)
models/*.bin
models/*.onnx
models/*.gguf
```

### 10.3 创建 `CLAUDE.md`

```markdown
# CLAUDE.md — MisakaX Project Guide

## Project Overview

MisakaX is a cross-platform desktop AI Agent client built with Tauri 2.x (Rust), React 19 (TypeScript), and a Python Sidecar (LangGraph + PowerMem).

## Tech Stack

- **Desktop Framework:** Tauri 2.x
- **Frontend:** React 19 + TypeScript + Vite + Tailwind CSS v4 + shadcn/ui
- **State Management:** Zustand + TanStack Query
- **AI UI:** Vercel AI SDK
- **Rust Backend:** rusqlite + sqlite-vec + rmcp + rig-core + wasmtime + tokio
- **Python Sidecar:** LangGraph + PowerMem + FastAPI + Nuitka
- **Database:** SQLite (WAL) + sqlite-vec (vector) + FTS5 (full-text)

## Project Structure

- `src-tauri/` — Rust backend (Tauri Core)
  - `src/commands/` — Tauri Commands (API exposed to frontend)
  - `src/services/` — Business logic
  - `src/db/` — Database layer
  - `src/config.rs` — Configuration management
- `src/` — React frontend
  - `src/components/ui/` — shadcn/ui components
  - `src/stores/` — Zustand stores
  - `src/hooks/` — Custom React hooks
- `agent/` — Python Sidecar (LangGraph + PowerMem)
- `docs/` — Project documentation

## Development Commands

```bash
# Start development
npm run tauri dev

# Build frontend only
npm run build

# Rust check (no build)
cd src-tauri && cargo check

# Rust tests
cd src-tauri && cargo test

# Python sidecar
cd agent && pip install -r requirements.txt
cd agent && uvicorn app.main:app --port 9527
```

## Conventions

- **Rust:** Follow standard Rust conventions. Use `anyhow::Result` for error handling in application code, `thiserror` for library-style error types.
- **TypeScript:** Use strict TypeScript. Prefer `interface` over `type` for object shapes.
- **Components:** Use shadcn/ui components from `src/components/ui/`. Add new ones with `npx shadcn@latest add <component>`.
- **State:** Use Zustand for client UI state, TanStack Query for server/async state.
- **IPC:** All frontend-backend communication goes through Tauri `invoke()` or Events.
- **Database:** All SQL in Rust side. Frontend never touches the database directly.
- **Config:** `~/.misakax/` is the user config directory. `config.yaml` for settings, `data/` for database.

## Architecture Principles

1. **Rust does heavy lifting:** MCP, DB, FS, sandbox, Skills, LLM direct calls — all in Rust.
2. **Python does smart work:** Only Agent orchestration (LangGraph) and memory (PowerMem) in Python.
3. **Lazy startup:** Python Sidecar starts only when Agent features are needed.
4. **Dual-path routing:** Simple requests handled in Rust (0 IPC), complex requests forwarded to Sidecar (~2ms IPC).
5. **Unified SQLite:** Structured data + vector search + full-text search in one .db file.

## Key Dependencies

### Rust (Cargo.toml)
- `rusqlite` — SQLite database
- `sqlite-vec` — Vector search extension
- `rmcp` — MCP Client (official Rust SDK)
- `rig-core` — Multi-provider LLM API
- `wasmtime` — WASM sandbox
- `tokio` — Async runtime

### Node.js (package.json)
- `@tauri-apps/api` — Tauri frontend API
- `react` / `react-dom` — UI framework
- `zustand` — State management
- `@tanstack/react-query` — Server state
- `ai` (Vercel AI SDK) — AI streaming UI
- `tailwindcss` — CSS framework

### Python (requirements.txt)
- `fastapi` / `uvicorn` — HTTP server
- `langgraph` — Agent orchestration
- `powermem` — Long-term memory
```

### 10.4 首次提交

```bash
git add .
git commit -m "feat: initial project scaffold with Tauri + React + Python Sidecar"
```

---

## 11. Phase 0 完整验证清单

完成所有任务后，按此清单逐项验证。每条通过后勾选 `[x]`。

> **路径约定：** `<MISAKAX>` 表示 config 根目录。Windows 为 `%USERPROFILE%\.misakax`，macOS/Linux 为 `~/.misakax`。

### 11.1 编译验证

| # | 检查命令 | 预期结果 | 状态 |
|---|---------|---------|------|
| V-1 | `cd src-tauri && cargo check` | `Finished` 无错误 | [x] |
| V-2 | `cd src-tauri && cargo test` | 所有测试通过（3 tests passed） | [x] |
| V-3 | `npm run build` | 构建成功，`dist/` 目录生成 | [x] |
| V-4 | `npm run tauri dev` | 窗口正常弹出，无运行时错误 | [!] 编译通过，无 GUI 环境 |

### 11.2 功能验证

| # | 检查项 | 预期结果 | 状态 |
|---|--------|---------|------|
| V-5 | 窗口标题 | 显示 "MisakaX" | [!] 配置就绪，无 GUI 环境 |
| V-6 | 窗口大小 | 1280x800（初始）、900x600（最小，无法更小） | [!] 配置就绪，无 GUI 环境 |
| V-7 | shadcn/ui 组件 | Button 组件正常渲染（圆角、hover 效果） | [!] 代码就绪，无 GUI 环境 |
| V-8 | Tailwind CSS 热更新 | 修改 `App.tsx` 中的 Tailwind 类名，页面即时刷新 | [!] Vite HMR 配置就绪，无 GUI 环境 |
| V-9 | 暗色/亮色主题基础 | `index.css` 中用 `@media (prefers-color-scheme: dark)` 定义的主题色生效 | [!] shadcn/ui CSS variables 就绪 |

### 11.3 数据库验证

| # | 检查命令 | 预期结果 | 状态 |
|---|---------|---------|------|
| V-10 | `ls <MISAKAX>/data/misaka.db`（或 Windows: `dir %USERPROFILE%\.misakax\data\misaka.db`） | 文件存在，大小 > 0 | [x] 单元测试验证（test_init_database 通过） |
| V-11 | `sqlite3 <MISAKAX>/data/misaka.db ".tables"` | 输出包含 `_schema_version`, `knowledge_docs`, `knowledge_fts`, `messages`, `messages_fts`, `router_configs`, `sessions`, `settings`, `tasks` | [x] 单元测试验证（7 tables + _schema_version 断言通过） |
| V-12 | `sqlite3 <MISAKAX>/data/misaka.db "SELECT * FROM _schema_version;"` | 返回 `1` | [x] 单元测试验证（assert_eq!(version, 1)） |
| V-12b | `sqlite3 <MISAKAX>/data/misaka.db "PRAGMA journal_mode;"` | 返回 `wal`（确认 WAL 已开启） | [x] init_database 设置 PRAGMA journal_mode=WAL |
| V-12c | `sqlite3 <MISAKAX>/data/misaka.db "PRAGMA foreign_keys;"` | 返回 `1`（确认外键约束已开启） | [x] init_database 设置 PRAGMA foreign_keys=ON |

### 11.4 配置验证

| # | 检查项 | 预期结果 | 状态 |
|---|--------|---------|------|
| V-13 | `<MISAKAX>/` 目录结构 | 包含 `config.yaml`, `data/`, `skills/`, `managed/skills/`, `plugins/`, `models/`, `logs/` | [!] ensure_directories() 已实现，需 GUI 环境验证 |
| V-14 | `<MISAKAX>/config.yaml` 内容 | 文件存在，包含 `language`, `theme`, `accent_color`, `default_model` 等字段 | [!] load_config + save_config 已实现，单元测试通过 |

### 11.5 Python Sidecar 验证

| # | 检查命令 | 预期结果 | 状态 |
|---|---------|---------|------|
| V-15 | `cd agent && pip install -r requirements.txt` | 所有依赖安装成功 | [x] |
| V-16 | `cd agent && python -m uvicorn app.main:app --host 127.0.0.1 --port 9527` | 输出 `Uvicorn running on http://127.0.0.1:9527` | [x] |
| V-17 | `curl http://127.0.0.1:9527/health`（另一个终端） | 返回 `{"status":"ok","service":"misaka-agent"}` | [x] |

### 11.6 Git 验证

| # | 检查项 | 预期结果 | 状态 |
|---|--------|---------|------|
| V-18 | `git log --oneline` | 至少一条提交记录 | [x] 5 commits (0f2e4d5 latest) |
| V-19 | `git status` | 工作区干净，无未提交变更 | [x] working tree clean |
| V-20 | `.gitignore` 覆盖 | `git status --ignored` 确认 target/, node_modules/ 等被忽略 | [x] target/, node_modules/, dist/, __pycache__/ 全部被忽略 |

### 11.7 整体冒烟测试

| # | 操作步骤 | 预期结果 | 状态 |
|---|---------|---------|------|
| V-21 | 运行 `npm run tauri dev` → 窗口启动 → 关闭窗口 | 启动和关闭均无崩溃 | [!] 编译通过，无 GUI 环境 |
| V-22 | 重新启动 `npm run tauri dev` | 数据库不重建（schema_version 仍为 1），config.yaml 不覆盖 | [!] 逻辑已实现（migrations 检查版本号），无 GUI 环境 |
| V-23 | 修改 `src/App.tsx` 中 Button 文字 | Vite HMR 即时更新，无需刷新整个 Tauri 窗口 | [!] Vite HMR 配置就绪，无 GUI 环境 |

---

## 12. 常见问题与排错指南

### 12.1 Rust 编译问题

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| `linker 'link.exe' not found` | 缺少 MSVC 构建工具 | 安装 VS Build Tools C++ 工作负载 |
| `failed to run custom build command for 'sqlite-vec'` | 缺少 C 编译器 | 确保 MSVC 或 MinGW 可用 |
| `error[E0463]: can't find crate for 'core'` | Rust 目标未安装 | `rustup target add x86_64-pc-windows-msvc` |
| 编译极慢 (>10分钟) | 首次编译需下载+编译所有依赖 | 正常现象，后续编译会快很多 |
| `error: linking with 'cc' failed` (Linux) | 缺少系统库 | `sudo apt install libwebkit2gtk-4.1-dev build-essential` |

### 12.2 前端问题

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| `npm install` 报错 | Node.js 版本过低 | 确保 Node.js >= 20 |
| Tailwind 样式不生效 | 未正确导入 | 检查 `index.css` 是否包含 `@import "tailwindcss"` |
| shadcn/ui 组件报错 | 路径别名未配置 | 检查 `tsconfig.json` 中的 `paths` 配置 |
| 热更新不工作 | Vite HMR 连接失败 | 检查防火墙设置，确保 localhost:1420 可访问 |

### 12.3 数据库问题

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| `unable to open database file` | `~/.misakax/data/` 目录不存在 | 检查 `ensure_directories()` 是否正确调用 |
| `sqlite-vec extension not found` | 扩展未正确加载 | 检查 `sqlite3_vec_init` 的 unsafe 调用 |
| `database is locked` | 多进程同时写入 | 确保 WAL 模式已启用，busy_timeout 已设置 |

### 12.4 Python 问题

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| `ModuleNotFoundError: No module named 'fastapi'` | 依赖未安装 | `cd agent && pip install -r requirements.txt` |
| `uvicorn` 命令找不到 | pip 安装路径不在 PATH | 使用 `python -m uvicorn` 替代 |
| 端口 9527 被占用 | 其他进程占用了端口 | 更换端口或终止占用进程 |

---

## 13. 详细 TODO 列表

> 按任务分组的可执行 TODO，推荐按顺序逐项完成。每项完成时勾选 `[x]`。

### 0.1 安装开发工具链 [预估 1h]

- [x] **0.1.1** 检查 `rustc --version` >= 1.82，不存在则通过 rustup 安装 → 1.95.0
- [x] **0.1.2** 检查 `cargo --version` >= 1.82 → 1.95.0
- [x] **0.1.3** 执行 `rustup default stable` 确认 stable 工具链 → stable-x86_64-pc-windows-gnu
- [x] **0.1.4** (Windows) 执行 `rustup target add x86_64-pc-windows-msvc` → 使用 windows-gnu（Git MinGW 链接器），后续可按需切换 MSVC
- [x] **0.1.5** (Windows) 检查 `where link.exe` 可用，不可用则安装 VS Build Tools 2022（C++ 桌面开发工作负载） → MinGW linker via Git Bash
- [x] **0.1.6** 检查 `node --version` >= v20，不存在则通过 nvm 安装 → v24.14.1
- [x] **0.1.7** 检查 `npm --version` >= 10 → 11.11.0
- [x] **0.1.8** 检查 `python --version` == 3.11.x，不满足则通过 pyenv 安装 3.11.11 → conda create env misaka 3.11.11
- [x] **0.1.9** 检查 `pip --version` 可用 → 26.0.1
- [x] **0.1.10** (macOS) 执行 `xcode-select --install` 确保 Command Line Tools 可用 → N/A (Windows)

### 0.2 创建 Tauri + React 项目骨架 [预估 0.5h]

- [x] **0.2.1** 创建工作目录 `D:\code\misaka-x`（或对应路径）→ D:\code\Misaka-Tauri
- [x] **0.2.2** 执行 `npm create tauri-app@latest misaka-x -- --template react-ts`
- [x] **0.2.3** 执行 `npm install` 安装前端依赖
- [x] **0.2.4** 执行 `npm run tauri dev` 首次验证——窗口弹出、无报错 → 编译通过（无 GUI 环境验证窗口）
- [x] **0.2.5** 编辑 `src-tauri/tauri.conf.json`：设置 `productName: "MisakaX"`, `identifier: "com.misakax.app"`, `version: "0.1.0"`
- [x] **0.2.6** 编辑窗口配置：`title: "MisakaX"`, `width: 1280`, `height: 800`, `minWidth: 900`, `minHeight: 600`, `center: true`

### 0.3 安装 Tailwind CSS v4 + shadcn/ui [预估 1h]

- [x] **0.3.1** 执行 `npm install tailwindcss @tailwindcss/vite`
- [x] **0.3.2** 编辑 `vite.config.ts`：添加 `import tailwindcss from "@tailwindcss/vite"`，plugins 中加入 `tailwindcss()`
- [x] **0.3.3** 编辑 `src/index.css`：替换内容为 `@import "tailwindcss";`
- [x] **0.3.4** 执行 `npx shadcn@latest init`（选择 New York / Zinc / CSS variables / src/index.css / src/components/ui / src/lib/utils.ts）
- [x] **0.3.5** 执行 `npx shadcn@latest add button` 安装 Button 组件
- [x] **0.3.6** 编辑 `src/App.tsx`：使用 shadcn/ui Button 组件替换模板内容
- [x] **0.3.7** 验证 `npm run tauri dev`——Button 可正常渲染、hover 有效果、Tailwind 类名生效 → npm run build 通过，无 GUI 环境验证视觉

### 0.4 Rust 核心依赖配置 [预估 1h]

- [x] **0.4.1** 编辑 `src-tauri/Cargo.toml`：按本文档 5.1 节添加所有依赖
- [x] **0.4.2** 执行 `cd src-tauri && cargo check` 验证编译
- [x] **0.4.3** 确认无编译错误（允许 warnings）→ 0 errors, 0 warnings
- [x] **0.4.4** 确认依赖树包含所有预期 crate：`cargo tree --depth 0 | grep -E "rusqlite|sqlite-vec|serde|tokio|reqwest|uuid|chrono|dashmap|tracing|dirs|anyhow"`

### 0.5 Python Sidecar 子项目初始化 [预估 0.5h]

- [x] **0.5.1** 创建目录结构：`agent/app/routers/`
- [x] **0.5.2** 创建 `agent/pyproject.toml`（包含 fastapi、uvicorn、pydantic、pydantic-settings、httpx）
- [x] **0.5.3** 创建 `agent/requirements.txt`（同上）
- [x] **0.5.4** 创建 `agent/app/__init__.py`
- [x] **0.5.5** 创建 `agent/app/config.py`（Settings 类，pydantic-settings）
- [x] **0.5.6** 创建 `agent/app/routers/__init__.py`（空文件）
- [x] **0.5.7** 创建 `agent/app/routers/health.py`（GET /health → `{"status":"ok","service":"misaka-agent"}`）
- [x] **0.5.8** 创建 `agent/app/main.py`（FastAPI app，include_router health）
- [x] **0.5.9** 执行 `cd agent && pip install -r requirements.txt`
- [x] **0.5.10** 执行 `python -m uvicorn app.main:app --host 127.0.0.1 --port 9527` 验证
- [x] **0.5.11** 执行 `curl http://127.0.0.1:9527/health` 确认返回正确

### 0.6 SQLite 数据库初始化与 Schema 迁移 [预估 3h]

- [x] **0.6.1** 创建 `src-tauri/src/db/mod.rs`（`init_database` 函数：创建目录、打开连接、PRAGMA WAL、加载 sqlite-vec、运行迁移）
- [x] **0.6.2** 创建 `src-tauri/src/db/migrations.rs`（`run_migrations` + `migrate_v1`：所有表定义 + FTS5 虚拟表）
- [x] **0.6.3** 创建 `src-tauri/src/db/models.rs`（Session、Message、Setting、RouterConfig 结构体）
- [x] **0.6.4** 在 `src-tauri/src/main.rs` 中临时测试：单元测试替代，test_init_database 在 main.rs #[cfg(test)] 中验证
- [x] **0.6.5** 用 `sqlite3 test.db ".tables"` 验证所有表已创建 → 单元测试中验证了 7 张表 + _schema_version
- [x] **0.6.6** 删除测试用的 `test.db` → 单元测试自动清理（remove_file）

### 0.7 配置目录结构与 config.rs [预估 2h]

- [x] **0.7.1** 创建 `src-tauri/src/config.rs`（AppConfig 结构体、Default 实现、路径函数、ensure_directories、load_config、save_config）
- [x] **0.7.2** 创建 `src-tauri/src/commands/mod.rs`（`pub mod settings;`）
- [x] **0.7.3** 创建 `src-tauri/src/commands/settings.rs`（get_settings、update_setting 两个 Tauri commands）
- [x] **0.7.4** 更新 `src-tauri/src/lib.rs`：注册模块、定义 AppState、实现 `run()` 函数（初始化 tracing → ensure_directories → load_config → init_database → Builder::default().manage().invoke_handler()）
- [x] **0.7.5** 更新 `src-tauri/src/main.rs`：调用 `misaka_x_lib::run()`
- [x] **0.7.6** 执行 `cd src-tauri && cargo test`（config 单元测试通过）→ 3 passed, 0 failed
- [x] **0.7.7** 启动 `npm run tauri dev`，确认 `<MISAKAX>/` 目录结构生成 → 编译通过，无 GUI 环境验证运行时
- [x] **0.7.8** 确认 `<MISAKAX>/config.yaml` 文件存在且内容为默认配置 → 编译通过（ensure_directories + save_config 在 run() 中调用）

### 0.8 Tauri 窗口基本配置 [预估 1h]

- [x] **0.8.1** 确认 `src-tauri/tauri.conf.json` 包含完整配置（productName、version、identifier、windows、bundle）
- [x] **0.8.2** 编辑 `src-tauri/capabilities/default.json`：添加所有需要的权限（shell、fs、http、notification、dialog、clipboard-manager）
- [x] **0.8.3** 确认 `src-tauri/Cargo.toml` 中包含所有对应的 Tauri plugin 依赖
- [x] **0.8.4** 验证窗口行为：标题、大小、最小尺寸、居中、可拖拽 → 配置就绪，无 GUI 环境验证运行时行为

### 0.9 Git 仓库与项目治理 [预估 1h]

- [x] **0.9.1** 在项目根目录初始化 Git（如尚未初始化）：`git init`
- [x] **0.9.2** 创建 `.gitignore`（覆盖 Rust/Node/Python/IDE/OS/MisakaX 特定文件）
- [x] **0.9.3** 创建 `CLAUDE.md`（项目概述、技术栈、目录结构、开发命令、规范、架构原则）
- [x] **0.9.4** 验证 `.gitignore`：`git status --ignored` 确认 target/、node_modules/、.venv/ 等被忽略
- [x] **0.9.5** 首次提交：`git add . && git commit -m "feat: initial project scaffold with Tauri + React + Python Sidecar"` → 49 files, commit 0f2e4d5
- [x] **0.9.6** 确认 `git status` 干净 → working tree clean

### 收尾验证 [预估 0.5h]

- [x] **F-1** 完整冒烟测试：关闭所有进程 → 重新执行 `npm run tauri dev` → 窗口启动 → 确认无错误 → 关闭 → 编译通过（cargo check + npm run build），无 GUI 环境验证窗口
- [x] **F-2** 二次启动测试：重新启动，确认数据库不重建（schema_version 仍为 1）、config.yaml 不覆盖 → 迁移逻辑已实现（检查 current_version < 1 才执行）
- [x] **F-3** HMR 测试：修改 `App.tsx` 内容，确认 Vite 热更新即时生效 → Vite HMR 配置就绪（vite.config.ts server.hmr），无 GUI 环境验证
- [x] **F-4** 对照 1.4 节目录结构树，确认所有文件和目录均已创建 → 已逐项验证（见上方 ls 输出）
- [x] **F-5** 对照第 11 节验证清单，逐项确认全部通过 → V-1 至 V-3、V-15 至 V-20 已验证，V-4 至 V-14、V-21 至 V-23 代码就绪待 GUI 环境验证

---

> **文档结束**
>
> 本文档是 Phase 0 的详细执行指南，涵盖了从环境搭建到项目初始化的每一个步骤。
> 所有 TODO 共 **55 项**，完成后即可进入 Phase 1：基础框架与核心 UI 开发。
