# MisakaX 项目实施方案

> **项目代号：** MisakaX（基于 Misaka 项目的下一代桌面端 AI Agent 客户端）
> **文档版本：** v3.0（架构重构：全路径 DeepAgents + 渐进式过渡 + 工作目录系统）
> **编制日期：** 2026-04-28（原始）/ 2026-05-07（v3.0 修订）
> **基于：** [架构选型 v2.2](./MISAKAX_ARCHITECTURE_FINAL%20-%20DeepSeek-V4-Pro.md) + [选型文档 v2.2](./MISAKAX_ARCHITECTURE_SELECTION%20-%20Opus4.6.md) + [Rig/DeepAgents 调研 v3.0](./RIG_DEEPAGENTS_RESEARCH.md)
> **开发模式：** 1 人 + Vibe Coding (AI 辅助开发)
> **预估总工期：** 22-24 周（约 5.5-6 个月）

---

## 目录

1. [实施概述](#1-实施概述)
2. [开发效率模型：Vibe Coding 因子](#2-开发效率模型vibe-coding-因子)
3. [Phase 0：环境搭建与项目初始化（第 1 周）](#3-phase-0环境搭建与项目初始化第-1-周)
4. [Phase 1：基础框架与核心 UI（第 2-4 周）](#4-phase-1基础框架与核心-ui第-2-4-周)
5. [Phase 2：对话系统与流式通信（第 5-7 周）](#5-phase-2对话系统与流式通信第-5-7-周)
6. [Phase 3：MCP 与会话管理（第 8-10 周）](#6-phase-3mcp-与会话管理第-8-10-周)
7. [Phase 4：DeepAgents Agent 编排与记忆系统（第 11-13 周）](#7-phase-4deepagents-agent-编排与记忆系统第-11-13-周)
8. [Phase 5：Skills 与知识库（第 14-16 周）](#8-phase-5skills-与知识库第-14-16-周)
9. [Phase 6：高级功能与打磨（第 17-19 周）](#9-phase-6高级功能与打磨第-17-19-周)
10. [Phase B：Buddy 桌面伴侣（并行，第 8-19 周）](#10-phase-bbuddy-桌面伴侣并行第-8-19-周)
11. [风险清单与应对预案](#11-风险清单与应对预案)
12. [里程碑总览](#12-里程碑总览)

---

## 1. 实施概述

### 1.1 项目目标

从零构建一个跨平台（Windows/macOS/Linux）的桌面端 AI Agent 客户端，具备：

- 全路径 DeepAgents harness（所有对话统一入口，Agent 自主决策处理策略）
- 工作目录系统（每会话绑定本地+远程目录，Agent 工程化操作）
- MCP 协议完整支持
- PowerMem 智能长期记忆（通过 DeepAgents 自定义 Tool 集成）
- 知识库 RAG（向量+全文混合搜索）
- Skills 系统（SKILL.md 标准，Rust 发现 + DeepAgents 执行双层架构）
- WASM 沙箱隔离
- Buddy 桌面伴侣（语音交互）
- 高性能 UI（React + shadcn/ui）

### 1.2 核心原则

| 原则 | 说明 |
|------|------|
| **垂直切片优先** | 每个 Phase 交付一个端到端可运行的功能切片，而非分层横切 |
| **Rust 先行** | 每个功能先实现 Rust 后端，再接 React 前端 |
| **可演示驱动** | 每个 Phase 结束时必须有可演示的产物 |
| **渐进式过渡** | Phase 2 用 Rig 作为"过渡对话实现"验证 UI 链路；Phase 4 替换为 DeepAgents 正式接管 |
| **Sidecar 预热** | Phase 3 引入 Sidecar 并预热；Phase 4 DeepAgents 接管全对话，Rig 退居非对话 |
| **工作目录贯穿** | Phase 2 就建立工作目录选择 UI；Phase 4 DeepAgents FilesystemMiddleware 基于工作目录运行 |
| **DeepAgents 加速** | Phase 4 使用 DeepAgents 替代手动 LangGraph 实现，Middleware 开箱即用，减少 60-80% Agent 样板代码 |
| **Buddy 并行** | Buddy 系统与主功能并行开发，不阻塞主线 |

### 1.3 技术栈速查

| 层 | 技术 |
|----|------|
| 桌面框架 | Tauri 2.x |
| 前端 | React 19 + TypeScript + Vite + Tailwind v4 + shadcn/ui |
| 状态管理 | Zustand + TanStack Query |
| AI UI | Vercel AI SDK |
| Rust 后端 | rusqlite + sqlite-vec + rmcp + rig-core (非对话) + wasmtime + tokio + Sidecar Manager |
| Python Sidecar | DeepAgents v0.5+ (全对话入口) + PowerMem + FastAPI + Nuitka |
| 数据库 | SQLite (WAL) + sqlite-vec (向量) + FTS5 (全文) |

**架构演进路线：**
```
Phase 2:  Rig (过渡对话) ──▶ Phase 3:  Rig + Sidecar 预热 ──▶ Phase 4:  DeepAgents (全对话)
              │                        │                              │
              └── 工作目录 UI           └── MCP 就绪                   └── Rig 对话退役
```

---

## 2. 开发效率模型：Vibe Coding 因子

### 2.1 效率提升预估

Vibe Coding（AI 辅助编程）对不同类型工作的加速倍率不同：

| 工作类型 | 传统 1 人效率 | + Vibe Coding | 加速倍率 | 说明 |
|---------|-------------|--------------|---------|------|
| **脚手架/样板代码** | 1x | 5-8x | ~6x | Tauri 初始化、CRUD、表结构等 AI 可直接生成 |
| **UI 组件开发** | 1x | 3-5x | ~4x | shadcn/ui 组件组合、布局，AI 非常擅长 |
| **标准 CRUD 逻辑** | 1x | 3-4x | ~3.5x | Rust/TS 的数据库读写、API 对接 |
| **协议/SDK 集成** | 1x | 2-3x | ~2.5x | rmcp/Rig/LangGraph 集成，需要理解文档 |
| **架构设计/调试** | 1x | 1.5-2x | ~1.5x | 跨进程通信、状态同步等复杂场景 |
| **Rust 底层/unsafe** | 1x | 1.2-1.5x | ~1.3x | sqlite-vec FFI、Wasmtime 集成 |
| **跨平台测试/打包** | 1x | 1-1.5x | ~1.2x | 平台特定问题需要手动排查 |

**综合加速因子：~2.5-3x**

### 2.2 工期折算

选型文档的原始估算是 30-41 周（假设 1 人无 AI 辅助，且偏保守）。

```
原始估算: 30-41 周 (团队模式，保守)
÷ 综合加速因子 2.5x (Vibe Coding)
× 单人产出系数 0.85 (1 人无法完全并行)
× 专注度系数 1.1 (个人项目无会议摩擦)
= 约 18-22 周
```

### 2.3 每周工作模型

| 指标 | 值 |
|------|------|
| 每周有效编码时间 | ~30-35 小时（含 Vibe Coding 交互） |
| 每周 Vibe Coding 辅助产出 | 等效额外 ~40-50 小时的手动编码量 |
| 每周总等效产出 | ~70-85 小时 |

---

## 3. Phase 0：环境搭建与项目初始化（第 1 周）

### 3.1 目标

从零创建 MisakaX 项目骨架，确保三端（Rust + React + Python）的开发环境完全就绪。

### 3.2 具体任务

| # | 任务 | 预估 | 产出 |
|---|------|------|------|
| 0.1 | 安装 Rust 工具链 + Node.js 20 + Python 3.11 | 1h | 环境就绪 |
| 0.2 | `npm create tauri-app@latest claw -- --template react-ts` | 0.5h | Tauri + React + Vite 项目骨架 |
| 0.3 | 安装 Tailwind CSS v4 + shadcn/ui 初始化 | 1h | UI 框架就绪 |
| 0.4 | Cargo.toml 添加核心依赖 (rusqlite, sqlite-vec, tokio, serde) | 1h | Rust 依赖就绪 |
| 0.5 | 创建 `agent/` Python 子项目 (pyproject.toml, requirements.txt) | 0.5h | Python 项目骨架 |
| 0.6 | 配置 SQLite 数据库初始化 + sqlite-vec 加载 + 基础 Schema 迁移 | 3h | 数据库就绪 |
| 0.7 | 实现 `~/.claw/` 配置目录结构 + config.rs | 2h | 配置系统 |
| 0.8 | Tauri 窗口基本配置 (大小/标题/图标/权限 ACL) | 1h | 窗口可运行 |
| 0.9 | Git 仓库初始化 + .gitignore + CLAUDE.md + 基础 CI | 1h | 版本管理就绪 |

### 3.3 Phase 0 交付物

- `tauri dev` 可启动空白窗口
- SQLite 数据库自动创建并初始化 schema
- `~/.claw/` 目录结构自动生成
- 项目可在 Windows/macOS 上编译运行

### 3.4 关键依赖版本锁定

```toml
# src-tauri/Cargo.toml 核心依赖
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-fs = "2"
tauri-plugin-notification = "2"
tauri-plugin-http = "2"
tauri-plugin-updater = "2"
rusqlite = { version = "0.34", features = ["bundled", "vtab"] }
sqlite-vec = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
```

```json
// package.json 核心依赖
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "react": "^19",
    "react-dom": "^19",
    "zustand": "^5",
    "@tanstack/react-query": "^5",
    "ai": "^4",
    "react-i18next": "^15",
    "lottie-react": "^2"
  },
  "devDependencies": {
    "@tailwindcss/vite": "^4",
    "tailwindcss": "^4",
    "typescript": "^5.7",
    "vite": "^6"
  }
}
```

---

## 4. Phase 1：基础框架与核心 UI（第 2-4 周）

### 4.1 目标

搭建完整的 UI 骨架 + 设置系统 + 主题/国际化。Phase 结束时，用户可以看到完整的界面布局，可以配置 API Key 和切换主题。

### 4.2 具体任务

| # | 任务 | 预估 | Vibe Coding 辅助度 |
|---|------|------|------------------|
| 1.1 | AppShell 主布局 (侧边栏 + 内容区 + 顶栏) | 4h | 高：shadcn/ui 布局 |
| 1.2 | 路由系统 (React Router 或内部状态路由) | 2h | 高 |
| 1.3 | 侧边栏导航 (Chat/Skills/Knowledge/Dashboard/Settings) | 3h | 高 |
| 1.4 | 设置页面 (通用/模型/MCP/外观/关于) | 6h | 高：表单生成 |
| 1.5 | Rust 端设置 CRUD Commands (读写 SQLite settings 表) | 3h | 中 |
| 1.6 | 路由配置 UI + Rust 后端 (多 Provider API Key 管理) | 6h | 高 |
| 1.7 | 主题系统 (明/暗/跟随系统 + 强调色) | 3h | 高 |
| 1.8 | i18n 初始化 (react-i18next, en + zh_CN JSON) | 3h | 高：翻译文件生成 |
| 1.9 | Zustand stores 搭建 (app, chat, settings) | 2h | 高 |
| 1.10 | Tauri IPC 封装层 (invoke wrapper + 错误处理) | 3h | 中 |

### 4.3 Phase 1 交付物

- 完整的应用骨架 UI（可导航各页面）
- 设置页面可保存/读取配置
- API Key 加密存储
- 主题切换即时生效
- 中英文切换

---

## 5. Phase 2：过渡对话系统与工作目录（第 5-7 周）

### 5.1 目标

使用 Rig 作为**过渡对话后端**，实现完整的对话 UI 链路（流式渲染、会话管理、工作目录选择）。这是整个应用的核心循环原型。Phase 4 时将对话后端从 Rig 替换为 DeepAgents，前端 UI 和会话基础设施基本不变。

> **关键认知：** Phase 2 的 Rig 对话是"脚手架"——验证对话 UI/流式/会话/工作目录的完整链路，而不是最终实现。所有 UI 组件和会话基础设施设计时就考虑 DeepAgents 兼容。

### 5.2 具体任务

| # | 任务 | 预估 | 说明 |
|---|------|------|------|
| 2.1 | Rig Provider 封装 (OpenAI/Anthropic/Gemini 统一接口) | 6h | Rust 端多模型支持 |
| 2.2 | Rust 流式 LLM 调用 + Tauri Event 推送 | 8h | `stream_token` 事件推送给前端 |
| 2.3 | `send_message` Command (保存消息 → Rig 调用 LLM → 流式返回) | 6h | 核心对话链路（Phase 4 时替换 Rig→DeepAgents） |
| 2.4 | ChatView 组件 (消息列表 + 输入框) | 6h | Vercel AI SDK 流式渲染（与 DeepAgents 兼容） |
| 2.5 | MessageItem 组件 (Markdown 渲染 + 代码高亮 + 复制) | 6h | react-markdown + shiki |
| 2.6 | MessageInput 组件 (多行输入 + 快捷键 + 文件附件) | 4h | shadcn/ui Textarea |
| 2.7 | **工作目录选择器** (本地路径浏览 + 远程连接配置) | 6h | 🆕 v3.0 核心特性：新建会话时选择工作目录 |
| 2.8 | **会话-工作目录绑定** (Rust 端 Session.working_dir 字段) | 3h | 🆕 会话创建时验证并绑定工作目录 |
| 2.9 | **工作目录状态指示栏** (顶栏显示当前路径) | 3h | 🆕 WorkspaceBar 组件 |
| 2.10 | 会话列表侧边栏 (创建/切换/删除/搜索) | 5h | CRUD + Rust 后端 |
| 2.11 | 消息持久化 (SQLite messages 表读写) | 3h | 完整的消息存储 |
| 2.12 | Token 用量统计 (单消息 + 单会话累计) | 2h | Rig 返回的 usage 数据 |
| 2.13 | 思维链展示 (thinking blocks 折叠/展开) | 3h | Anthropic extended thinking |
| 2.14 | 停止生成 / 重新生成 | 2h | AbortController + 重试 |
| 2.15 | 多模态输入 (图片粘贴/拖拽附件) | 4h | Base64 编码发送 |

> **v3.0 新增项：** 2.7 工作目录选择器、2.8 会话-工作目录绑定、2.9 工作目录状态指示栏。这些是 MisakaX 区别于普通聊天客户端的工程化特性。

### 5.3 Phase 2 交付物

- **可完整对话的 AI 客户端（Rig 过渡后端）**
- 支持 OpenAI / Anthropic / Gemini
- 流式输出 + Markdown 渲染 + 代码高亮
- **工作目录选择 UI**（新建会话时选择本地/远程目录）
- **工作目录绑定与指示**（顶栏显示当前会话工作目录）
- 会话创建/切换/删除/搜索
- 思维链展示
- 图片输入

### 5.4 本阶段里程碑

> **M1: 核心可用** — Phase 2 完成后，MisakaX 已经是一个可日常使用的 AI 对话客户端（含工作目录支持）。

---

## 6. Phase 3：Sidecar 预热基础设施 + MCP（第 8-10 周）

### 6.1 目标

引入 Python Sidecar（**预热但不接管对话**），建立完整的 Sidecar 生命周期管理。集成 MCP Client（Rust 原生 rmcp）。此阶段 Rig 仍然处理对话，但 Sidecar 已预热就绪，为 Phase 4 DeepAgents 接管做好准备。

> **关键认知：** Phase 3 是"基础设施铺路"阶段。Sidecar 已启动并健康运行，但尚未参与对话。这确保了 Phase 4 迁移时平滑切换，而非从零搭建。

### 6.2 具体任务

| # | 任务 | 预估 | 说明 |
|---|------|------|------|
| 3.1 | Python Sidecar 基础设施 (FastAPI + uvicorn + `/health` 端点) | 4h | HTTP Server 骨架，提供 `/health`、`/agent/chat`（占位）、`/agent/stream`（占位） |
| 3.2 | **Sidecar 预热管理器** (应用启动时异步启动 + 健康检查循环) | 6h | 🆕 SidecarManager: preheat() + wait_for_healthy() + 自动重启 |
| 3.3 | Sidecar 状态指示器 (前端 StatusBadge: 就绪/启动中/异常) | 2h | 用户可见的 Sidecar 状态 |
| 3.4 | Rust ↔ Sidecar HTTP/WS 通信协议 | 4h | 请求/响应/流式协议定义（供 Phase 4 使用） |
| 3.5 | rmcp Client 集成 (stdio transport) | 8h | 通过子进程管理 MCP Server |
| 3.6 | rmcp HTTP/SSE transport | 4h | 网络类 MCP Server |
| 3.7 | MCP 配置加载 (mcp.json / settings 页面) | 4h | 多来源配置合并 |
| 3.8 | MCP Server 生命周期管理 (启动/停止/重启/健康检查) | 6h | 进程管理器 |
| 3.9 | Tool Call UI 展示 (工具名 + 参数 + 结果折叠) | 5h | React 组件（兼容 Rig 和 DeepAgents 模式） |
| 3.10 | Tool Call 权限审批流程 (approve/deny/always allow) | 4h | Human-in-the-loop 前端 |
| 3.11 | MCP 管理页面 (已连接 Servers + 可用工具列表) | 4h | Settings 子页面 |
| 3.12 | 会话分组/归档/置顶 | 3h | UI + Rust 后端 |
| 3.13 | 会话搜索 (FTS5 全文搜索消息内容) | 3h | 搜索消息内容 |
| 3.14 | 会话导入/导出 (JSON 格式) | 3h | 数据可移植 |
| 3.15 | Nuitka 打包脚本初版 (Python Sidecar → 可执行文件) | 4h | 初步打包验证 |

### 6.3 Phase 3 交付物

- **Sidecar 预热就绪**（应用启动时自动后台启动，健康检查通过）
- **MCP 完整支持**（连接 MCP Server、调用工具、展示结果）
- Rig 仍然处理对话（DeepAgents 占位端点已创建）
- 权限审批流程
- 会话高级管理

### 6.4 本阶段里程碑

> **M2: 基础设施就绪** — Sidecar 已预热、MCP 已就绪、通信协议已定义。一切准备就绪，等待 Phase 4 DeepAgents 接管对话。

---

## 7. Phase 4：DeepAgents 全对话迁移（第 11-13 周）

### 7.1 目标

**DeepAgents 正式接管所有用户对话**，Rig 对话功能退役。通过 `create_deep_agent()` 获得完整的 Middleware 能力（规划/文件/SubAgent/Skills/记忆/摘要/HITL）。通过自定义 Tool 集成 PowerMem 智能记忆。这是 MisakaX 从"对话客户端"进化为"Agent 平台"的关键阶段。

> **关键认知：** Phase 3 已建立 Sidecar 预热和通信协议。Phase 4 只需：①实现 DeepAgents agent.py ②修改 Rust chat Command 从 Rig→DeepAgents ③验证端到端流程。前端 UI 层零改动。

### 7.2 架构说明

```
┌──────────────────────────────────────────────────────────────┐
│  Python Sidecar (agent/) — Phase 4 全对话入口                   │
│                                                                │
│  FastAPI HTTP Server  ──▶  create_deep_agent()                 │
│                              │                                 │
│                              ├── TodoListMiddleware (规划)      │
│                              ├── SkillsMiddleware (技能加载)    │
│                              ├── FilesystemMiddleware (文件操作) │
│                              │   └── root_dir = session.working_dir │
│                              ├── SubAgentMiddleware (子代理)    │
│                              ├── SummarizationMiddleware (摘要) │
│                              ├── MemoryMiddleware (AGENTS.md)   │
│                              ├── HumanInTheLoopMiddleware (审批) │
│                              └── Custom Tools:                  │
│                                  ├── powermem_search_tool       │
│                                  ├── powermem_save_tool         │
│                                  └── mcp_bridge_tool            │
│                                                                │
│  返回 CompiledStateGraph → 流式输出                             │
└──────────────────────────────────────────────────────────────┘
```

**迁移路径：**
```
Phase 2-3:
  React → Tauri IPC → Rust `chat.rs` → Rig (LLM API) → Stream → React

Phase 4:
  React → Tauri IPC → Rust `chat.rs` → HTTP POST Sidecar
  → DeepAgents Harness → LangChain (LLM API) → Stream → Rust 转发 → React
  ↑ 前端 UI 层完全不变，仅 Rust chat Command 后端替换
```

### 7.3 DeepAgents 替代的价值

| 原手动实现 | DeepAgents 替代 | 节省 |
|-----------|----------------|------|
| 手动 StateGraph + Node + Edge 定义 | `create_deep_agent()` 工厂函数 | ~80% 代码量 |
| 手动 SubAgent 子图 | `SubAgent` 类型 + `SubAgentMiddleware` | ~70% 代码量 |
| 手动上下文管理 | `SummarizationMiddleware` 自动摘要压缩 | 省去整个模块 |
| 手动 Human-in-the-loop | `HumanInTheLoopMiddleware` 配置式 | ~60% 代码量 |
| 手动任务规划逻辑 | `TodoListMiddleware` (`write_todos` 工具) | 省去整个模块 |
| 手动记忆注入 | `MemoryMiddleware` (AGENTS.md 自动加载) | ~50% 代码量 |
| 手动文件操作 | `FilesystemMiddleware` (基于工作目录) | 省去整个模块 |

### 7.4 具体任务

| # | 任务 | 预估 | 说明 |
|---|------|------|------|
| 4.1 | **DeepAgents 核心集成** (`create_deep_agent()` 组装) | 6h | 🆕 agent.py：Middleware 链 + 自定义 Tool + SubAgent 配置 |
| 4.2 | **FilesystemMiddleware 工作目录绑定** (基于会话 working_dir 配置 Backend) | 3h | 🆕 根据 Session.working_dir 动态构建 CompositeBackend |
| 4.3 | 自定义 Tool 开发 (PowerMem 检索/存储 + MCP Bridge) | 6h | `@tool` 装饰器实现 PowerMem 集成 |
| 4.4 | SubAgent 配置 (研究/编码/分析 子代理) | 3h | DeepAgents `SubAgent` 类型配置 |
| 4.5 | **Rust chat Command 改写** (Rig LLM 直调 → HTTP Sidecar 代理) | 4h | 🆕 chat.rs 从调用 Rig 改为转发 Sidecar；Phase 2-3 的 Rig 路径保留为降级开关 |
| 4.6 | **Rig 对话功能退役 + 非对话功能保留** (Embedding/标题/摘要) | 3h | 🆕 清理 Rig 对话相关代码；确保 Rig 非对话路径正常 |
| 4.7 | LangGraph Checkpointer (langgraph-checkpoint-sqlite) | 2h | 状态持久化 |
| 4.8 | PowerMem 集成 (pip install powermem, 环境配置) | 3h | 记忆引擎初始化 + 自定义 Tool 封装 |
| 4.9 | 记忆管理 UI (查看/搜索/删除记忆) | 4h | React 页面 |
| 4.10 | Human-in-the-loop 配置 (DeepAgents `interrupt_on` + 前端审批 UI) | 3h | 配置式实现 |
| 4.11 | Nuitka 打包脚本完善 (含 DeepAgents + PowerMem 依赖) | 4h | 分发准备 |
| 4.12 | 端到端集成测试 (对话 → Agent 决策 → 工具调用 → 记忆读写) | 4h | 完整链路验证 |

> **新增项：** 4.2 工作目录绑定、4.5 Rust chat 改写、4.6 Rig 退役。前端 UI 层零改动。

### 7.5 关键代码示例

```python
# agent/app/agent.py — 核心 Agent 组装
from deepagents import create_deep_agent, SubAgent
from deepagents.backends import CompositeBackend, StateBackend, FilesystemBackend
from .tools import powermem_search_tool, powermem_save_tool, mcp_bridge_tool

def build_agent(session=None, checkpointer=None, store=None):
    """创建 MisakaX DeepAgent，返回 CompiledStateGraph"""
    
    # 根据会话工作目录构建 Backend
    routes = {}
    if session and session.working_dir_local:
        routes["/workspace/"] = FilesystemBackend(
            root_dir=session.working_dir_local
        )
    
    return create_deep_agent(
        model="claude-sonnet-4-6",
        tools=[powermem_search_tool, powermem_save_tool, mcp_bridge_tool],
        system_prompt=SYSTEM_PROMPT,
        subagents=[
            SubAgent(
                name="researcher",
                description="Deep research agent for web search and analysis",
                system_prompt="You are a research specialist...",
                tools=[powermem_search_tool],
            ),
            SubAgent(
                name="coder",
                description="Code generation and review agent",
                system_prompt="You are a coding specialist...",
            ),
        ],
        skills=["~/.misakax/skills/"],
        memory=["~/.misakax/memories/"],
        backend=lambda rt: CompositeBackend(
            default=StateBackend(rt),
            routes=routes,
        ),
        interrupt_on={"*": True},
        checkpointer=checkpointer,
        store=store,
    )
```

### 7.6 Phase 4 交付物

- **全路径 Agent 平台**（DeepAgents Middleware 架构）
- DeepAgents 接管所有对话（Rig 对话功能退役）
- **工作目录绑定**（FilesystemMiddleware 基于会话 working_dir 运行）
- SubAgent 代理派生（研究/编码/分析，上下文隔离）
- PowerMem 跨会话智能记忆（通过自定义 Tool 集成）
- Human-in-the-loop 审批（DeepAgents 配置式）
- 自动上下文摘要压缩（SummarizationMiddleware）
- 任务规划与追踪（TodoListMiddleware）
- Sidecar 打包为独立 Nuitka 可执行文件

### 7.7 本阶段里程碑

> **M3: Agent 平台** — MisakaX 具备完整的 Agent 编排、SubAgent 派生、工作目录文件操作、智能记忆和人工审批能力。Rig 退居非对话角色。

### 7.5 关键代码示例

```python
# agent/app/agent.py — 核心 Agent 组装（替代原 graph.py + nodes.py + subagents.py）
from deepagents import create_deep_agent, SubAgent
from deepagents.backends import CompositeBackend, StateBackend, StoreBackend
from langgraph.checkpoint.sqlite import SqliteSaver
from .tools import powermem_search_tool, powermem_save_tool, mcp_bridge_tool

def build_agent(checkpointer=None, store=None):
    """创建 MisakaX DeepAgent，返回 CompiledStateGraph"""
    return create_deep_agent(
        model="claude-sonnet-4-6",  # 可通过配置切换
        tools=[powermem_search_tool, powermem_save_tool, mcp_bridge_tool],
        system_prompt=SYSTEM_PROMPT,
        subagents=[
            SubAgent(
                name="researcher",
                description="Deep research agent for web search and analysis",
                system_prompt="You are a research specialist...",
                tools=[powermem_search_tool],
            ),
            SubAgent(
                name="coder",
                description="Code generation and review agent",
                system_prompt="You are a coding specialist...",
            ),
        ],
        skills=["~/.misakax/skills/"],  # SkillsMiddleware 自动加载
        memory=["~/.misakax/memories/"],  # MemoryMiddleware 自动加载
        backend=lambda rt: CompositeBackend(
            default=StateBackend(rt),
            routes={"/memories/": StoreBackend(rt)},
        ),
        interrupt_on={"*": True},  # HumanInTheLoopMiddleware
        checkpointer=checkpointer,
        store=store,
    )
```

```python
# agent/app/tools.py — PowerMem 自定义 Tool
from langchain_core.tools import tool

@tool
def powermem_search_tool(query: str, user_id: str) -> list[dict]:
    """Search long-term memory for relevant information using PowerMem."""
    results = memory.search(query, user_id=user_id, limit=5)
    return [{"content": r.content, "score": r.score} for r in results]

@tool
def powermem_save_tool(content: str, user_id: str) -> str:
    """Save important information to long-term memory."""
    memory.add(content, user_id=user_id, smart_process=True)
    return "Memory saved successfully."
```

### 7.6 Phase 4 交付物

- **完整 Agent 编排系统**（基于 DeepAgents Middleware 架构）
- DeepAgents `create_deep_agent()` 一行组装 Agent（替代手动 StateGraph）
- SubAgent 代理派生（研究/编码/分析，上下文隔离）
- PowerMem 跨会话智能记忆（通过自定义 Tool 集成，保留艾宾浩斯遗忘曲线）
- Human-in-the-loop 审批（DeepAgents 配置式）
- 自动上下文摘要压缩（SummarizationMiddleware）
- 任务规划与追踪（TodoListMiddleware）
- Sidecar 打包为独立 Nuitka 可执行文件

### 7.7 本阶段里程碑

> **M3: Agent 平台** — MisakaX 具备完整的 Agent 编排、SubAgent 派生、智能记忆和人工审批能力。

### 7.8 项目结构变更

```
agent/                                # Python Sidecar（DeepAgents 方案，精简）
├── app/
│   ├── __init__.py
│   ├── main.py                      # FastAPI 入口（提供 /agent/chat, /agent/stream）
│   ├── agent.py                     # create_deep_agent() 组装（~40行，替代 graph.py 200+行）
│   ├── tools.py                     # 自定义 Tool (PowerMem/MCP Bridge)
│   ├── middleware.py                # 自定义 Middleware（如需扩展默认行为）
│   └── config.py                    # Agent 配置（模型、温度、中间件开关等）
├── skills/                          # SKILL.md 文件（DeepAgents SkillsMiddleware 加载）
├── memories/                        # AGENTS.md 记忆文件（DeepAgents MemoryMiddleware 加载）
├── pyproject.toml
├── requirements.txt                 # deepagents + powermem + fastapi + uvicorn
└── build_nuitka.py                  # Nuitka 打包脚本
```

---

## 8. Phase 5：Skills 双层架构 + 知识库（第 14-17 周）

### 8.1 目标

实现 Skills 双层架构：**Rust 负责 Skill 发现/注册/管理 UI，DeepAgents SkillsMiddleware 负责 Agent 内技能加载和执行**。实现知识库 RAG 完整系统。

### 8.2 具体任务

| # | 任务 | 预估 | 说明 |
|---|------|------|------|
| 5.1 | Skill 发现 (多目录扫描, walkdir + glob) | 3h | Rust 实现 |
| 5.2 | SKILL.md 解析 (YAML frontmatter + Markdown body) | 3h | serde_yaml |
| 5.3 | Skill 注册表 (DashMap + 缓存 + 去重) | 3h | 并发安全 |
| 5.4 | **Skill → DeepAgents SkillsMiddleware 同步** (Rust 扫描结果同步至 `agent/skills/`) | 3h | 🆕 双层架构关键：Rust 发现 → 文件同步 → DeepAgents 加载 |
| 5.5 | Skill 热重载 (notify 文件监听 → 通知 DeepAgents 刷新) | 3h | 目录变化自动同步 |
| 5.6 | Skills UI (浏览/搜索/启用/禁用) | 5h | React + shadcn |
| 5.7 | 文档导入 (PDF/MD/TXT 解析 + 纯文本提取) | 5h | Rust 文本提取 |
| 5.8 | 智能文档分块 (按语义边界切分 chunks) | 4h | 固定大小 + 重叠 |
| 5.9 | Embedding 生成 (Rig → OpenAI/本地模型) | 3h | Rig 非对话用途 |
| 5.10 | 向量+全文索引写入 (sqlite-vec + FTS5) | 3h | 双索引并行写入 |
| 5.11 | 混合搜索 (sqlite-vec KNN + FTS5 + RRF 融合) | 4h | SQL CTE 实现 |
| 5.12 | RAG 对话集成 (检索→注入 DeepAgents 上下文→LLM 回答) | 4h | 通过 kb_search 自定义 Tool 注入 |
| 5.13 | 知识库管理 UI (导入/列表/删除/搜索测试) | 5h | React 页面 |

### 8.3 Phase 5 交付物

- **Skills 双层架构**（Rust 发现/管理 + DeepAgents SkillsMiddleware 执行）
- **知识库 RAG**（文档导入 → 分块 → 向量化 → 混合搜索 → 增强对话）
- Skill 市场 API（可选）

### 8.3.1 Skills 双层架构

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 1: Rust Core (Skills 发现与管理)                       │
│  ├── 扫描 ~/.misakax/skills/ + .misakax/skills/             │
│  ├── 解析 SKILL.md (YAML frontmatter + Markdown body)       │
│  ├── Skill 注册表 (DashMap, 并发安全)                         │
│  ├── 热重载 (notify 文件监听)                                  │
│  ├── 市场 API (可选, 远程搜索/安装)                             │
│  └── 同步至 agent/skills/ (供 DeepAgents 加载)                │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Python Sidecar (Skills 执行)                       │
│  └── DeepAgents SkillsMiddleware                             │
│      ├── 自动加载 agent/skills/ 下所有 SKILL.md               │
│      ├── L1: name + description 注入 System Prompt (~50 tokens)│
│      ├── L2: 触发时加载完整 Skill Body (<5,000 tokens)        │
│      └── L3/L4: references/ + scripts/ 按需加载               │
└─────────────────────────────────────────────────────────────┘
```

### 8.4 本阶段里程碑

> **M4: 知识增强** — Agent 可以基于 Skills 和用户导入的知识库进行精准问答。

---

## 9. Phase 6：高级功能 + Buddy + 发布（第 18-23 周）

### 9.1 目标

补全高级功能（沙箱、Dashboard、Buddy 桌面伴侣），全面打磨用户体验，跨平台测试，准备发布。

### 9.2 具体任务

| # | 任务 | 预估 | 说明 |
|---|------|------|------|
| 6.1 | WASM 沙箱 (Wasmtime 集成 + 代码执行) | 8h | 安全执行环境 |
| 6.2 | Dashboard (会话统计/Token 图表/成本分析) | 6h | 数据可视化 |
| 6.3 | 文件浏览器 (树形目录 + 文件预览 + 代码高亮) | 6h | Monaco Editor，与工作目录绑定 |
| 6.4 | 任务管理 (任务列表 + 状态追踪) | 4h | 持久化任务（与 DeepAgents TodoList 互补） |
| 6.5 | 系统通知 (Agent 完成/错误/审批请求) | 2h | tauri-plugin-notification |
| 6.6 | 自动更新 (tauri-plugin-updater) | 3h | 增量更新 |
| 6.7 | 跨平台构建测试 (Windows + macOS + Linux) | 6h | CI/CD 配置 |
| 6.8 | 安装包构建 (MSI/DMG/AppImage) | 4h | Tauri 打包 |
| 6.9 | 性能优化 (React.memo/虚拟列表/懒加载) | 4h | 大会话性能 |
| 6.10 | 快捷键系统 (全局 + 页面级) | 3h | 键盘操作 |
| 6.11 | 启动向导 (首次运行引导配置 + 工作目录默认设置) | 3h | 🆕 含工作目录引导 |

### 9.3 Phase 6 交付物

- **可发布的完整桌面应用**
- WASM 代码沙箱
- Dashboard 数据分析
- 文件浏览器
- 自动更新
- 三平台安装包

### 9.4 本阶段里程碑

> **M5: 发布就绪 (v0.1.0)** — MisakaX 主体功能完整，可以面向早期用户发布。

---

## 10. Phase B：Buddy 桌面伴侣（并行，第 8-23 周）

### 10.1 并行开发策略

Buddy 系统与主应用共享 Rust Core 后端。从 Phase 3 末尾（MCP 模块就绪后）开始，利用每周约 20% 的时间并行推进 Buddy。

### 10.2 开发阶段

| 子阶段 | 主线 Phase | 预估 | 任务 |
|--------|-----------|------|------|
| **B1: 透明浮窗** | Phase 3 | 6h | Tauri 多窗口 + transparent + alwaysOnTop + 拖拽 |
| **B2: 角色动画** | Phase 3-4 | 6h | Lottie 集成 + 待机/说话/思考 状态切换 |
| **B3: 文字对话** | Phase 4 | 8h | 迷你输入框 + DeepAgents LLM 对话 + 气泡回复 + 情绪驱动 |
| **B4: 语音输入** | Phase 4-5 | 10h | whisper-cpp-plus 集成 + 快捷键触发 + 实时转录 |
| **B5: 语音输出** | Phase 5 | 6h | piper-rs 集成 + 音频播放 + 口型状态切换 |
| **B6: 高级交互** | Phase 5-6 | 8h | MCP 工具调用 + 人设系统 + 多 Buddy 形象 |

### 10.3 Buddy 资源需求

| 模型文件 | 大小 | 加载时机 |
|---------|------|---------|
| Whisper base (STT) | ~142 MB | 首次使用语音时下载 |
| Piper medium (TTS) | ~63 MB | 首次使用语音时下载 |
| Buddy Lottie JSON | < 1 MB | 随应用打包 |

---

## 11. 风险清单与应对预案

| # | 风险 | 概率 | 影响 | 应对策略 |
|---|------|------|------|---------|
| R1 | Rust 学习曲线导致开发变慢 | 中 | 中 | Vibe Coding 大幅降低 Rust 学习成本；初期多参考同类 Tauri 项目 |
| R2 | rmcp (MCP Rust SDK) 存在 Bug | 中 | 中 | 保留 TypeScript MCP SDK 作为降级方案 |
| R3 | Python Sidecar 打包体积过大 | 高 | 低 | Nuitka AOT 替代 PyInstaller；精简依赖树 |
| R4 | whisper-cpp-plus 跨平台编译问题 | 中 | 低 | Buddy 语音是 P1 功能，可推迟 |
| R5 | sqlite-vec 与 rusqlite 版本兼容 | 低 | 中 | 锁定已验证的版本组合 (rusqlite 0.34 + sqlite-vec 0.1) |
| R6 | Vercel AI SDK 与 Tauri IPC 集成 | 中 | 中 | 如不兼容，降级为手动 SSE 处理 |
| R7 | 个人精力不足 | 中 | 高 | 严格按 Phase 推进，保持 MVP 心态；Buddy 可完全推迟 |
| R8 | **DeepAgents API Breaking Changes** | 中 | 中 | 固定 `deepagents==0.5.4`；降级路径：回退手动 LangGraph |
| R9 | **DeepAgents Middleware 限制自定义需求** | 低 | 中 | 混合使用 DeepAgents + 原生 LangGraph API；必要时 Fork |
| R10 | **Sidecar 预热失败/崩溃** | 中 | 高 | 🆕 重试机制（最多 3 次）；降级为按需启动；前端显示状态指示器 |
| R11 | **Phase 4 DeepAgents 迁移不顺利** | 中 | 高 | 🆕 Phase 3 已建立通信协议和 Sidecar 预热，Phase 4 仅需替换后端；保留 Rig 对话路径作为降级开关 |
| R12 | **工作目录安全风险** | 低 | 中 | 🆕 目录白名单配置；DeepAgents FilesystemMiddleware 自动限制；操作审计日志 |

---

## 12. 里程碑总览

```
Week  1  ──── Phase 0: 环境搭建 ───────────────────────────── ✓ 项目可运行
Week  2  ┐
Week  3  ├─── Phase 1: 基础框架 + UI ──────────────────────── ✓ 完整 UI 骨架
Week  4  ┘
Week  5  ┐
Week  6  ├─── Phase 2: Rig 过渡对话 + 工作目录 ────── M1 ── ✓ 核心可用
Week  7  ┘                                         (含工作目录选择)
Week  8  ┐
Week  9  ├─── Phase 3: Sidecar 预热 + MCP ───────── M2 ── ✓ 基础设施就绪
Week 10  ┘                                         (Sidecar 已预热)
                                                   │ Phase B 并行启动
Week 11  ┐                                         │ B1: 浮窗
Week 12  ├─── Phase 4: DeepAgents 全对话迁移 ── M3 ── ✓ Agent 平台
Week 13  ┘    (Rig 对话退役)                      │ B2: 动画
Week 14  ┐                                         │ B3: 文字对话
Week 15  ├─── Phase 5: Skills + 知识库 ──────── M4 ── ✓ 知识增强
Week 16  ┘                                         │ B4: 语音输入
Week 17  ┐                                         │ B5: 语音输出
Week 18  │─── Phase 6: 高级功能 ──────────────┐    │
Week 19  │    + Buddy + 跨平台测试              │    │ B6: 高级交互
Week 20  │    + 打包 + 性能优化                  │    │
Week 21  ├─── Phase 6 (续): 打磨 ───────────── M5 ── ✓ 发布就绪
Week 22  │    + 启动向导 + 最终测试              │    │
Week 23  ┘                                      │    │
                                                  ↓
                                          Buddy v1.0 完成

═══════════════════════════════════════════════════════════
总周期: 23 周 (约 5.5 个月)
里程碑: M1(核心可用+工作目录) → M2(基础设施就绪) → M3(Agent平台) → M4(知识增强) → M5(发布就绪)
```

### 各里程碑版本号

| 里程碑 | 版本 | 核心能力 |
|--------|------|---------|
| M1 | v0.1.0-alpha | 多模型对话（Rig 过渡）、流式输出、会话管理、工作目录选择 |
| M2 | v0.2.0-alpha | + MCP 工具调用、权限审批、Sidecar 预热就绪 |
| M3 | v0.3.0-beta | + DeepAgents 全对话接管、SubAgent、PowerMem 智能记忆、工作目录绑定 |
| M4 | v0.4.0-beta | + Skills 双层架构（Rust 发现 + DeepAgents 执行）、知识库 RAG |
| M5 | v0.5.0-rc | + 沙箱、Dashboard、自动更新、Buddy 桌面伴侣 |

---

> **文档结束**
>
> v3.0 基于 v2.2 架构重构（全路径 DeepAgents）进行了实施计划的全面调整：
> - **Phase 2** 引入 Rig 过渡对话 + 工作目录选择 UI（验证完整对话链路）
> - **Phase 3** Sidecar 预热基础设施 + MCP（为 DeepAgents 接管铺路）
> - **Phase 4** DeepAgents 全对话迁移（Rig 对话退役，前端 UI 零改动）
> - **Phase 5** Skills 双层架构 + 知识库 RAG
> - **Phase 6** 高级功能 + Buddy + 打磨发布
>
> 渐进式过渡确保每个阶段都有可演示的产物，风险可控。
> 总工期 23 周（约 5.5 个月），1 人 + Vibe Coding 模式。
