# MisakaX

<p align="center">
  <strong>现代化桌面 AI Agent 客户端</strong>
</p>

<p align="center">
  基于 Tauri 2.x + React 19 + Python Sidecar 构建的跨平台桌面 AI 助手
</p>

---

## 项目概况

MisakaX 是一个开源的桌面 AI Agent 客户端，采用三层架构设计：

- **Rust 后端** — 数据库、文件系统、MCP 协议、LLM 调用（当前对话走 Rig 过渡后端）
- **React 前端** — 会话管理、流式对话 UI、工作目录、Provider / MCP / Skills 设置
- **Python Sidecar** — Sidecar 预热与健康检查已就绪；DeepAgents + PowerMem 对话编排（Phase 4 规划中）

> **开发状态：** Phase 3 代码关门（~95%），**请先补齐最终 UI/实机复验记录再进入 Phase 4**。  
> 剩余任务见 [`PHASE_3_REMAINING_TODO.md`](docs/planning/PHASE_3_REMAINING_TODO.md)；总览见 [`DEVELOPMENT_STATUS.md`](docs/project/DEVELOPMENT_STATUS.md)。

## 已实现功能

### 基础设施（Phase 0）

- 跨平台桌面应用框架（Tauri 2.x，Windows / macOS / Linux）
- SQLite 数据库 Schema 迁移至 **v13**（WAL、FTS5、sqlite-vec、Skills 扫描与迁移状态）
- 配置管理（`~/.misakax/config.yaml`）
- Tauri 插件（fs、http、shell、dialog、clipboard、notification）

### UI 与设置（Phase 1）

- AppShell 主布局、侧边栏导航、Zustand 路由
- 设置页（通用 / 模型 / MCP / Skills / 外观 / 关于）
- Provider API Key CRUD + **加密存储**
- 主题切换（明 / 暗 / 跟随系统）、中 / 英文 i18n

### 对话与工作目录（Phase 2）

- 多 Provider LLM **流式对话**（Rig 过渡后端：OpenAI / Anthropic / Gemini / 兼容接口）
- 停止生成、重新生成、思维链展示、Token 统计、图片附件
- 会话管理（分组 / 置顶 / 归档 / 搜索 / 导入导出）
- **工作目录**绑定、资源管理器、Monaco 文件编辑
- Composer footer 只读 Git/本地项目标识（分支、detached HEAD、worktree/submodule；无 Git 写操作）

### MCP 与 Sidecar（Phase 3，主体完成）

- MCP Client（rmcp）：stdio / HTTP 连接、工具调用、权限审批
- Sidecar 自动预热、健康检查、前端状态指示
- Tool Call UI、MCP 设置页

### Skills 仓库与安全门

- Settings > Skills 统一管理受管、Codex、Claude 与 Cursor 来源；启用和消息选择使用 stable SkillId + activation generation
- 按需 summary、文件树和分段预览；普通详情不传输整份 `SKILL.md`
- ZIP quarantine、离线静态扫描、findings、审批/撤销、JSON/SARIF 导出和强制激活 Gate
- Schema v13 首次升级会禁用旧豁免 source、持久化显示批量扫描进度并支持失败重试；外部目录只读

### 尚未实现

- DeepAgents 全对话接管（Sidecar `/agent/*` 当前为 501 占位）
- PowerMem 长期记忆、知识库 RAG、Sandbox 隔离的可选 Skills 深度扫描器
- Dashboard、通知中心、Buddy 桌面伴侣

## 技术架构

```
┌────────────────────────────────────────────────┐
│  React 19 (TypeScript) + Tailwind CSS v4 +    │
│  shadcn/ui (New York / Zinc)                   │
│  → Vite 6 开发服务器 (:1420)                   │
├────────────────────────────────────────────────┤
│  Tauri 2.x (Rust)                              │
│  - SQLite (WAL) v13 + sqlite-vec + FTS5        │
│  - LLM: rig-core（过渡）+ rmcp（MCP）          │
│  - 配置 (~/.misakax/config.yaml)               │
├────────────────────────────────────────────────┤
│  Python Sidecar (:9527)                        │
│  - FastAPI: /health ✅  /agent/* Phase 4       │
│  - DeepAgents + PowerMem（规划中）             │
└────────────────────────────────────────────────┘
```

## 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 桌面框架 | Tauri 2.x | Rust 驱动，内存占用低 |
| 前端 | React 19 + TypeScript + Vite 6 | Zustand 状态、react-i18next |
| UI | Tailwind CSS v4 + shadcn/ui | 设计规范见 `docs/design/` |
| 后端 | Rust (tokio, rusqlite, rig-core, rmcp) | ~50+ Tauri Commands |
| 数据库 | SQLite + sqlite-vec + FTS5 | Schema v13 |
| 过渡对话 | rig-core 0.36 | Phase 4 后退役为降级路径 |
| Agent 编排 | DeepAgents（Phase 4） | 替换 Rig 直调 |
| 记忆引擎 | PowerMem（Phase 4） | Sidecar optional 依赖 |

## 快速开始

### 环境要求

| 工具 | 最低版本 | 用途 |
|------|---------|------|
| Rust | 1.95+ | Tauri 后端编译 |
| Node.js | 20+ | 前端开发 |
| Python | 3.11.x | Sidecar 运行 |
| npm | 10+ | 包管理 |

Windows 用户还需要 MSVC 构建工具、WebView2（Windows 11 已内置）。

### 安装依赖

```bash
npm install

cd agent
pip install -r requirements.txt
# Phase 4 开发时额外安装：
# pip install -e ".[agent,memory,dev]"
```

### 启动开发环境

```bash
# 完整桌面应用（推荐）
npm run tauri dev

# 仅前端（浏览器调试，无 Tauri IPC）
npm run dev

# 独立 Sidecar（可选；Tauri 默认 auto_start_sidecar 会自动预热）
cd agent
python run.py
```

首次使用：在 **设置 → 模型** 中添加 Provider API Key，然后在 **Chat** 新建会话即可对话。

### 构建与测试

```bash
npm run build          # 前端生产构建
npm test               # 前端 Vitest

cd src-tauri
cargo check            # Rust 编译检查（日常依赖增量编译，勿先 cargo clean）
cargo test --test crypto_tests   # 日常：按改动模块跑精准测试（映射表见优化指南 §4.2）
cargo nextest run --all-features --profile ci   # 提交前：全量测试（推荐，需 cargo install cargo-nextest）
cargo test             # 提交前：全量测试（未装 nextest 时的后备）
```

若 Rust 编译出现链接错误、metadata 异常或切分支后无法解释的失败，再在 `src-tauri/` 下按需执行 `cargo clean` 后重试。详见 [`docs/guides/rust-build-test-optimization.md`](docs/guides/rust-build-test-optimization.md)。

## 目录结构

```
misaka-x/
├── src/                           # React 前端
│   ├── components/
│   │   ├── chat/                  # 对话、会话、工作区、Composer
│   │   ├── layout/                # AppShell、Sidebar
│   │   └── ui/                    # shadcn/ui
│   ├── pages/                     # Chat、Settings、Knowledge 等
│   ├── stores/                    # Zustand（chat、settings、theme…）
│   ├── lib/ipc/                   # Tauri IPC 封装
│   └── locales/                   # i18n（zh-CN / en）
├── src-tauri/src/
│   ├── commands/                  # chat, session, mcp, settings…
│   ├── db/                        # migrations (v13), repository
│   ├── services/
│   │   ├── llm/                   # Rig Provider、流式、工厂
│   │   └── mcp/                   # rmcp Manager
│   ├── config.rs, crypto.rs, sidecar.rs
│   └── lib.rs                     # AppState + invoke_handler
├── agent/                         # Python Sidecar
│   └── app/
│       ├── main.py
│       └── routers/               # health, info, agent (501 占位)
├── docs/
│   ├── project/DEVELOPMENT_STATUS.md   # ← 开发进度与续做入口
│   ├── planning/                  # Phase 0–4 详细计划
│   ├── architecture/
│   └── design/                    # UI 规范
└── CLAUDE.md / AGENTS.md          # AI 辅助开发指引
```

完整模块说明见 [`docs/project/PROJECT_STRUCTURE.md`](docs/project/PROJECT_STRUCTURE.md)。

## 路线图

对齐 [`docs/planning/MISAKAX_IMPLEMENTATION_PLAN - Opus4.6.md`](docs/planning/MISAKAX_IMPLEMENTATION_PLAN%20-%20Opus4.6.md)：

| 阶段 | 内容 | 状态 |
|------|------|------|
| Phase 0 | 项目骨架、数据库、配置、UI 框架 | ✅ |
| Phase 1 | AppShell、设置、Provider、主题、i18n | ✅ |
| Phase 2 | Rig 过渡对话、流式渲染、工作目录 | ✅ |
| Phase 3 | Sidecar 预热、MCP、会话高级管理 | 🟡 **代码关门，待实机复验** |
| Phase 4 | DeepAgents 全对话迁移 + PowerMem | ⏸️ Phase 3 完成后 |
| Phase 5 | Skills + 知识库 RAG | 🟡 Skills 已交付；RAG 未开始 |
| Phase 6 | Dashboard、打包、跨平台发布 | 未开始 |

**续做指南：** [`PHASE_3_REMAINING_TODO.md`](docs/planning/PHASE_3_REMAINING_TODO.md)

## 运行时数据目录

```
~/.misakax/
├── config.yaml           # 全局配置（主题、语言、sidecar_port 等）
├── data/misaka.db        # SQLite（Schema v13）
├── mcp.json              # MCP Server 配置（可选）
├── skills/               # 受管 Skills（启用前必须通过扫描 Gate）
├── managed/skills/
├── plugins/
├── models/
└── logs/
```

## 贡献指南

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改（[Conventional Commits](https://www.conventionalcommits.org/)）
4. 推送到分支并创建 Pull Request

开发前请先阅读 [`DEVELOPMENT_STATUS.md`](docs/project/DEVELOPMENT_STATUS.md) 确认当前 Phase 与入口文件。

## 许可证

本项目采用 [MIT License](LICENSE) 开源。
