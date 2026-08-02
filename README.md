<p align="center">
  <img src="src/assets/brand/misakax-logo.svg" alt="MisakaX" width="96" height="96" />
</p>

<h1 align="center">MisakaX</h1>

<p align="center">
  <strong>跨平台桌面 AI Agent 客户端</strong>
</p>

<p align="center">
  基于 Tauri 2 · React 19 · Python Sidecar<br />
  多模型流式对话 · 工作区与终端 · MCP 工具 · Skills 安全门
</p>

<p align="center">
  <a href="#快速开始"><img src="https://img.shields.io/badge/get_started-quick-0ea5e9?style=flat-square" alt="Get Started" /></a>
  <a href="#技术栈"><img src="https://img.shields.io/badge/Tauri-2.x-FFC131?style=flat-square&logo=tauri&logoColor=white" alt="Tauri" /></a>
  <a href="#技术栈"><img src="https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=black" alt="React" /></a>
  <a href="#技术栈"><img src="https://img.shields.io/badge/Rust-2021-DEA584?style=flat-square&logo=rust&logoColor=black" alt="Rust" /></a>
  <a href="#技术栈"><img src="https://img.shields.io/badge/Python-3.11-3776AB?style=flat-square&logo=python&logoColor=white" alt="Python" /></a>
  <img src="https://img.shields.io/badge/status-active_development-yellow?style=flat-square" alt="Status" />
</p>

---

## 简介

MisakaX 是一款面向开发者的**桌面 AI Agent 客户端**：在本地运行、数据自持，同时支持主流云端与兼容接口的大模型。

当前已可用作多 Provider 流式对话客户端，并具备工作区、嵌入式终端、MCP 工具调用与 Skills 管理能力。Agent 编排（DeepAgents）与长期记忆（PowerMem）处于下一阶段规划中。

| | |
|---|---|
| **平台** | Windows / macOS / Linux（Tauri 2） |
| **对话** | OpenAI · Anthropic · Gemini · OpenAI 兼容接口 |
| **本地能力** | 工作目录、资源管理器、Monaco 编辑、嵌入式终端 |
| **扩展** | MCP Client（stdio / HTTP）、Skills 仓库与安全扫描 |
| **存储** | SQLite（WAL）· FTS5 · sqlite-vec · 配置与密钥本地加密 |

开发进度与续做入口见 [`docs/project/DEVELOPMENT_STATUS.md`](docs/project/DEVELOPMENT_STATUS.md)。

## 功能亮点

- **多模型流式对话** — 停止 / 重生成、思维链展示、Token 统计、图片附件
- **会话管理** — 分组、置顶、归档、FTS5 全文搜索、导入导出
- **工作区** — 会话绑定工作目录、文件树、Monaco 多 Tab 编辑、只读 Git / 本地项目标识
- **嵌入式终端** — 基于 PTY 的 xterm 面板，窄 IPC、背压与进程树回收（Windows 已深度验证）
- **MCP 工具** — 连接管理、对话内工具循环、权限审批（ask / approve / deny / always allow）
- **Skills** — 受管与 Codex / Claude / Cursor 来源统一管理；ZIP 隔离、离线静态扫描、审批 Gate
- **设置与安全** — Provider API Key AES-GCM 加密、主题（明/暗/系统）、中英文 i18n
- **Python Sidecar** — 自动预热、健康检查与 watchdog；对话编排端点待 Phase 4 接入

## 架构

```
┌─────────────────────────────────────────────────┐
│  React 19 + TypeScript + Tailwind CSS v4        │
│  shadcn/ui · Zustand · Vite 6 (:1420)           │
├─────────────────────────────────────────────────┤
│  Tauri 2.x (Rust)                               │
│  SQLite · LLM (rig-core) · MCP (rmcp) · PTY     │
│  Config (~/.misakax/config.yaml)                │
├─────────────────────────────────────────────────┤
│  Python Sidecar (:9527)                         │
│  FastAPI · /health ✅ · Agent 编排（规划中）      │
└─────────────────────────────────────────────────┘
```

前端负责交互与渲染；Rust 负责本地数据、LLM/MCP、终端与安全边界；Python Sidecar 预留给 DeepAgents + PowerMem 的 Agent 编排层。

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面壳 | Tauri 2.x |
| 前端 | React 19 · TypeScript · Vite 6 · Zustand · react-i18next |
| UI | Tailwind CSS v4 · shadcn/ui（New York / Zinc） |
| 后端 | Rust 2021 · tokio · rusqlite · rig-core · rmcp · portable-pty |
| 数据库 | SQLite（WAL）· FTS5 · sqlite-vec · Schema v13 |
| Sidecar | Python 3.11 · FastAPI · uvicorn（DeepAgents / PowerMem optional） |

## 快速开始

### 环境要求

| 工具 | 版本 | 用途 |
|------|------|------|
| Rust | 1.95+ | Tauri / 后端编译 |
| Node.js | 20+ | 前端与工具链 |
| Python | 3.11.x | Sidecar |
| npm | 10+ | 包管理 |

Windows 另需 MSVC（Visual Studio Build Tools）与 WebView2（Windows 11 通常已内置）。

### 安装

```bash
npm install

cd agent
pip install -r requirements.txt
```

Agent / 记忆可选依赖（Phase 4）：

```bash
pip install -e ".[agent,memory,dev]"
```

### 开发运行

```bash
# 完整桌面应用（推荐）
npm run tauri dev

# 仅前端（浏览器调试，无 Tauri IPC）
npm run dev

# 独立启动 Sidecar（可选；默认可由应用自动预热）
cd agent
python run.py
```

首次使用：打开 **设置 → 模型**，添加 Provider API Key，再在 Chat 新建会话即可对话。

### 构建与测试

```bash
npm run build
npm test

cd src-tauri
cargo check
cargo nextest run --all-features --profile ci
```

未安装 [cargo-nextest](https://nexte.st/) 时可用 `cargo test --all-features`。日常开发请依赖增量编译，**不要**在常规流程前执行 `cargo clean`。详见 [`docs/guides/rust-build-test-optimization.md`](docs/guides/rust-build-test-optimization.md)。

## 项目结构

```
MisakaX/
├── src/                 # React 前端（chat / layout / settings / stores）
├── src-tauri/           # Tauri + Rust 后端（commands / db / services）
├── agent/               # Python Sidecar（FastAPI）
├── docs/
│   ├── architecture/    # 架构与选型
│   ├── planning/        # 阶段计划与执行指南
│   ├── design/          # UI / UX 规范
│   ├── project/         # 进度与结构说明
│   ├── research/        # 调研笔记
│   └── guides/          # 开发与环境指南
├── AGENTS.md            # AI 辅助开发约定
└── CLAUDE.md
```

完整模块说明：[项目结构](docs/project/PROJECT_STRUCTURE.md)

## 路线图

| 阶段 | 内容 | 状态 |
|------|------|------|
| Phase 0–2 | 骨架、设置、流式对话、工作目录 | ✅ |
| Phase 3 | Sidecar 预热、MCP、会话高级能力 | 🟡 代码关门，待实机复验 |
| Phase 4 | DeepAgents 对话编排 + PowerMem | ⏸️ |
| Phase 5 | Skills（已交付）+ 知识库 RAG | 🟡 / ⬜ |
| Phase 6 | Dashboard、打包与跨平台发布 | ⬜ |

专项进度（Skills / Terminal / Sandbox）：[`WORKSPACE_SECURITY_SKILLS_PROGRESS.md`](docs/project/WORKSPACE_SECURITY_SKILLS_PROGRESS.md)

## 文档

| 文档 | 说明 |
|------|------|
| [开发状态](docs/project/DEVELOPMENT_STATUS.md) | 当前进度与续做入口 |
| [总执行指南](docs/planning/MASTER_IMPLEMENTATION_EXECUTION_GUIDE.md) | AI Coding / 协作执行规范 |
| [UI 指南](docs/design/frontend-ui-guidelines.md) | 桌面 Agent 风格与动效 |
| [Shell / Workspace 规范](docs/design/shell-and-workspace-ui-spec.md) | 导航与工作区 chrome |
| [按钮与菜单规范](docs/design/button-menu-design-spec.md) | 控件与浮层 |

## 运行时数据

默认目录：`~/.misakax/`

```
~/.misakax/
├── config.yaml      # 全局配置
├── data/misaka.db   # SQLite（Schema v13）
├── mcp.json         # MCP Server 配置（可选）
├── skills/          # 受管 Skills
├── managed/skills/
├── plugins/
├── models/
└── logs/
```

## 贡献

1. Fork 本仓库并创建特性分支
2. 提交前阅读 [开发状态](docs/project/DEVELOPMENT_STATUS.md)，确认当前 Phase 与入口文件
3. 使用 [Conventional Commits](https://www.conventionalcommits.org/) 书写提交信息
4. 打开 Pull Request

UI 改动请同步遵循 `docs/design/` 下规范；Rust 改动优先跑对应精准测试，提交前跑全量套件。

## 许可证

本项目计划以 MIT License 开源。仓库根目录的 `LICENSE` 文件待补充；若 GitHub 仓库已设置 License 元数据，以该设置为准。
