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
</p>

---

## 简介

MisakaX 是一款面向开发者的**桌面 AI Agent 客户端**：在本地运行、数据自持，同时支持主流云端与兼容接口的大模型。

| | |
|---|---|
| **平台** | Windows / macOS / Linux（Tauri 2） |
| **对话** | OpenAI · Anthropic · Gemini · OpenAI 兼容接口 |
| **本地能力** | 工作目录、资源管理器、Monaco 编辑、嵌入式终端 |
| **扩展** | MCP Client（stdio / HTTP）、Skills 仓库与安全扫描 |
| **存储** | SQLite（WAL）· FTS5 · sqlite-vec · 配置与密钥本地加密 |

## 功能亮点

- **多模型流式对话** — 停止 / 重生成、思维链展示、Token 统计、图片附件
- **会话管理** — 分组、置顶、归档、FTS5 全文搜索、导入导出
- **工作区** — 会话绑定工作目录、文件树、Monaco 多 Tab 编辑、只读 Git / 本地项目标识
- **嵌入式终端** — 基于 PTY 的 xterm 面板，窄 IPC、背压与进程树回收
- **MCP 工具** — 连接管理、对话内工具循环、权限审批（ask / approve / deny / always allow）
- **Skills** — 受管与 Codex / Claude / Cursor 来源统一管理；ZIP 隔离、离线静态扫描、审批 Gate
- **设置与安全** — Provider API Key AES-GCM 加密、主题（明/暗/系统）、中英文 i18n
- **Python Sidecar** — 自动预热、健康检查与 watchdog，承载 Agent 编排服务

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
│  FastAPI · Agent 编排                           │
└─────────────────────────────────────────────────┘
```

前端负责交互与渲染；Rust 负责本地数据、LLM/MCP、终端与安全边界；Python Sidecar 负责 Agent 编排与记忆等扩展能力。

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面壳 | Tauri 2.x |
| 前端 | React 19 · TypeScript · Vite 6 · Zustand · react-i18next |
| UI | Tailwind CSS v4 · shadcn/ui（New York / Zinc） |
| 后端 | Rust 2021 · tokio · rusqlite · rig-core · rmcp · portable-pty |
| 数据库 | SQLite（WAL）· FTS5 · sqlite-vec |
| Sidecar | Python 3.11 · FastAPI · uvicorn |

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

需要 Agent / 记忆扩展依赖时：

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

未安装 [cargo-nextest](https://nexte.st/) 时可用 `cargo test --all-features`。日常开发请依赖增量编译，**不要**在常规流程前执行 `cargo clean`。

## 项目结构

```
MisakaX/
├── src/                              # React 前端
│   ├── components/
│   │   ├── chat/                     # 对话、Composer、工作区、终端
│   │   ├── layout/                   # AppShell、侧边栏
│   │   ├── skills/                   # Skills 管理 UI
│   │   └── ui/                       # shadcn/ui 基础组件
│   ├── pages/                        # Chat、Settings 等页面
│   ├── stores/                       # Zustand 状态
│   ├── lib/ipc/                      # Tauri IPC 封装
│   ├── hooks/                        # 跨页面 hooks
│   └── locales/                      # i18n（zh-CN / en）
├── src-tauri/                        # Tauri + Rust 后端
│   ├── src/
│   │   ├── commands/                 # Tauri Commands（chat、session、mcp…）
│   │   ├── db/                       # SQLite 迁移与 repository
│   │   ├── services/
│   │   │   ├── llm/                  # 多 Provider 流式对话
│   │   │   ├── mcp/                  # MCP Client
│   │   │   ├── skills/               # Skills 安装、扫描、安全门
│   │   │   ├── workspace/            # 工作区与只读 Git 上下文
│   │   │   └── terminal/             # PTY 终端运行时
│   │   ├── config.rs                 # ~/.misakax 配置
│   │   ├── crypto.rs                 # API Key 加密
│   │   └── sidecar.rs                # Sidecar 生命周期
│   ├── capabilities/                 # Tauri ACL
│   └── tauri.conf.json
├── agent/                            # Python Sidecar
│   ├── app/
│   │   ├── main.py                   # FastAPI 入口
│   │   └── routers/                  # health / info / agent
│   ├── pyproject.toml
│   └── requirements.txt
└── docs/                             # 项目文档
```

## 运行时数据

默认目录：`~/.misakax/`

```
~/.misakax/
├── config.yaml      # 全局配置
├── data/misaka.db   # SQLite
├── mcp.json         # MCP Server 配置（可选）
├── skills/          # 受管 Skills
├── managed/skills/
├── plugins/
├── models/
└── logs/
```

## 贡献

1. Fork 本仓库并创建特性分支
2. 使用 [Conventional Commits](https://www.conventionalcommits.org/) 书写提交信息
3. 打开 Pull Request

## 许可证

本项目计划以 MIT License 开源。仓库根目录的 `LICENSE` 文件待补充；若 GitHub 仓库已设置 License 元数据，以该设置为准。
