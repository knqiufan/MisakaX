# Claw 项目实施方案

> **项目代号：** Claw（基于 Misaka 项目的下一代桌面端 AI Agent 客户端）
> **文档版本：** v1.0
> **编制日期：** 2026-04-28
> **基于：** [技术选型 v2.1](./CLAW_ARCHITECTURE_SELECTION.md) + [调研报告 v1.0](./CLAW_TECH_SELECTION_REPORT.md)
> **开发模式：** 1 人 + Vibe Coding (AI 辅助开发)
> **预估总工期：** 18-22 周（约 4.5-5.5 个月）

---

## 目录

1. [实施概述](#1-实施概述)
2. [开发效率模型：Vibe Coding 因子](#2-开发效率模型vibe-coding-因子)
3. [Phase 0：环境搭建与项目初始化（第 1 周）](#3-phase-0环境搭建与项目初始化第-1-周)
4. [Phase 1：基础框架与核心 UI（第 2-4 周）](#4-phase-1基础框架与核心-ui第-2-4-周)
5. [Phase 2：对话系统与流式通信（第 5-7 周）](#5-phase-2对话系统与流式通信第-5-7-周)
6. [Phase 3：MCP 与会话管理（第 8-10 周）](#6-phase-3mcp-与会话管理第-8-10-周)
7. [Phase 4：Agent 编排与记忆系统（第 11-14 周）](#7-phase-4agent-编排与记忆系统第-11-14-周)
8. [Phase 5：Skills 与知识库（第 15-17 周）](#8-phase-5skills-与知识库第-15-17-周)
9. [Phase 6：高级功能与打磨（第 18-20 周）](#9-phase-6高级功能与打磨第-18-20-周)
10. [Phase B：Buddy 桌面伴侣（并行，第 8-20 周）](#10-phase-bbuddy-桌面伴侣并行第-8-20-周)
11. [风险清单与应对预案](#11-风险清单与应对预案)
12. [里程碑总览](#12-里程碑总览)

---

## 1. 实施概述

### 1.1 项目目标

从零构建一个跨平台（Windows/macOS/Linux）的桌面端 AI Agent 客户端，具备：

- 多模型 LLM 对话（流式）
- MCP 协议完整支持
- LangGraph Agent 编排 + SubAgent
- PowerMem 智能长期记忆
- 知识库 RAG（向量+全文混合搜索）
- Skills 系统（SKILL.md 标准）
- WASM 沙箱隔离
- Buddy 桌面伴侣（语音交互）
- 高性能 UI（React + shadcn/ui）

### 1.2 核心原则

| 原则 | 说明 |
|------|------|
| **垂直切片优先** | 每个 Phase 交付一个端到端可运行的功能切片，而非分层横切 |
| **Rust 先行** | 每个功能先实现 Rust 后端，再接 React 前端 |
| **可演示驱动** | 每个 Phase 结束时必须有可演示的产物 |
| **Sidecar 延后** | Python Sidecar 在 Phase 4 才引入，前三个阶段纯 Rust + React |
| **Buddy 并行** | Buddy 系统与主功能并行开发，不阻塞主线 |

### 1.3 技术栈速查

| 层 | 技术 |
|----|------|
| 桌面框架 | Tauri 2.x |
| 前端 | React 19 + TypeScript + Vite + Tailwind v4 + shadcn/ui |
| 状态管理 | Zustand + TanStack Query |
| AI UI | Vercel AI SDK |
| Rust 后端 | rusqlite + sqlite-vec + rmcp + rig-core + wasmtime + tokio |
| Python Sidecar | LangGraph + PowerMem + FastAPI + Nuitka |
| 数据库 | SQLite (WAL) + sqlite-vec (向量) + FTS5 (全文) |

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

从零创建 Claw 项目骨架，确保三端（Rust + React + Python）的开发环境完全就绪。

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

## 5. Phase 2：对话系统与流式通信（第 5-7 周）

### 5.1 目标

实现核心对话功能：用户发送消息 → Rig (Rust) 调用 LLM API → 流式 Token 返回 → React 实时渲染。这是整个应用的核心循环。

### 5.2 具体任务

| # | 任务 | 预估 | 说明 |
|---|------|------|------|
| 2.1 | Rig Provider 封装 (OpenAI/Anthropic/Gemini 统一接口) | 6h | Rust 端多模型支持 |
| 2.2 | Rust 流式 LLM 调用 + Tauri Event 推送 | 8h | `stream_token` 事件推送给前端 |
| 2.3 | `send_message` Command (保存消息 → 调用 LLM → 流式返回) | 6h | 核心对话链路 |
| 2.4 | ChatView 组件 (消息列表 + 输入框) | 6h | Vercel AI SDK 流式渲染 |
| 2.5 | MessageItem 组件 (Markdown 渲染 + 代码高亮 + 复制) | 6h | react-markdown + shiki |
| 2.6 | MessageInput 组件 (多行输入 + 快捷键 + 文件附件) | 4h | shadcn/ui Textarea |
| 2.7 | 会话列表侧边栏 (创建/切换/删除/搜索) | 5h | CRUD + Rust 后端 |
| 2.8 | 消息持久化 (SQLite messages 表读写) | 3h | 完整的消息存储 |
| 2.9 | Token 用量统计 (单消息 + 单会话累计) | 2h | Rig 返回的 usage 数据 |
| 2.10 | 思维链展示 (thinking blocks 折叠/展开) | 3h | Anthropic extended thinking |
| 2.11 | 停止生成 / 重新生成 | 2h | AbortController + 重试 |
| 2.12 | 多模态输入 (图片粘贴/拖拽附件) | 4h | Base64 编码发送 |

### 5.3 Phase 2 交付物

- **可完整对话的 AI 客户端**
- 支持 OpenAI / Anthropic / Gemini
- 流式输出 + Markdown 渲染 + 代码高亮
- 会话创建/切换/删除/搜索
- 思维链展示
- 图片输入

### 5.4 本阶段里程碑

> **M1: 核心可用** — Phase 2 完成后，Claw 已经是一个可日常使用的 AI 对话客户端。

---

## 6. Phase 3：MCP 与会话管理（第 8-10 周）

### 6.1 目标

实现 MCP Client（Rust 原生）+ 完整的会话管理增强。

### 6.2 具体任务

| # | 任务 | 预估 | 说明 |
|---|------|------|------|
| 3.1 | rmcp Client 集成 (stdio transport) | 8h | 通过子进程管理 MCP Server |
| 3.2 | rmcp HTTP/SSE transport | 4h | 网络类 MCP Server |
| 3.3 | MCP 配置加载 (mcp.json / settings 页面) | 4h | 多来源配置合并 |
| 3.4 | MCP Server 生命周期管理 (启动/停止/重启/健康检查) | 6h | 进程管理器 |
| 3.5 | Tool Call UI 展示 (工具名 + 参数 + 结果折叠) | 5h | React 组件 |
| 3.6 | Tool Call 权限审批流程 (approve/deny/always allow) | 4h | Human-in-the-loop |
| 3.7 | MCP 管理页面 (已连接 Servers + 可用工具列表) | 4h | Settings 子页面 |
| 3.8 | 会话分组/归档/置顶 | 3h | UI + Rust 后端 |
| 3.9 | 会话搜索 (FTS5 全文搜索消息内容) | 3h | 搜索消息内容 |
| 3.10 | 会话导入/导出 (JSON 格式) | 3h | 数据可移植 |

### 6.3 Phase 3 交付物

- **完整 MCP 支持**（连接 MCP Server、调用工具、展示结果）
- 权限审批流程
- 会话高级管理（分组、归档、搜索）

### 6.4 本阶段里程碑

> **M2: MCP 就绪** — Agent 可以通过 MCP 工具与外部世界交互。

---

## 7. Phase 4：Agent 编排与记忆系统（第 11-14 周）

### 7.1 目标

引入 Python Sidecar，实现 LangGraph Agent 编排 + PowerMem 智能记忆。这是 Claw 从"对话客户端"进化为"Agent 平台"的关键阶段。

### 7.2 具体任务

| # | 任务 | 预估 | 说明 |
|---|------|------|------|
| 4.1 | Python Sidecar 基础设施 (FastAPI + uvicorn + 启动脚本) | 4h | HTTP Server 骨架 |
| 4.2 | Rust Sidecar Manager (启动/停止/健康检查/自动重启) | 6h | 进程生命周期管理 |
| 4.3 | Rust ↔ Sidecar HTTP/WS 通信协议 | 4h | 请求/响应/流式 |
| 4.4 | LangGraph 基础图定义 (主 Agent StateGraph) | 8h | 核心 Agent 循环 |
| 4.5 | LangGraph 节点实现 (LLM 调用/工具调用/路由) | 6h | 节点逻辑 |
| 4.6 | LangGraph Checkpointer (langgraph-checkpoint-sqlite) | 3h | 状态持久化 |
| 4.7 | SubAgent 子图定义 (研究/编码/分析 子代理) | 6h | LangGraph 子图 |
| 4.8 | Command Router 路由判断逻辑 (简单→Rig / 复杂→Sidecar) | 4h | 双路径分流 |
| 4.9 | PowerMem 集成 (pip install powermem, .env 配置) | 3h | 记忆引擎初始化 |
| 4.10 | 记忆节点 (对话前检索 + 对话后提取存储) | 6h | LangGraph 记忆节点 |
| 4.11 | 记忆管理 UI (查看/搜索/删除记忆) | 4h | React 页面 |
| 4.12 | Human-in-the-loop (Agent 暂停 → 前端审批 → 继续) | 6h | LangGraph interrupt |
| 4.13 | Nuitka 打包脚本 (Python Sidecar → 可执行文件) | 4h | 分发准备 |

### 7.3 Phase 4 交付物

- **完整 Agent 编排系统**
- LangGraph 多步骤工作流 + SubAgent
- PowerMem 跨会话智能记忆
- Human-in-the-loop 审批
- Sidecar 打包为独立可执行文件

### 7.4 本阶段里程碑

> **M3: Agent 平台** — Claw 具备独立的 Agent 编排和记忆能力，不依赖任何外部 Agent SDK。

---

## 8. Phase 5：Skills 与知识库（第 15-17 周）

### 8.1 目标

实现 Skills 系统（SKILL.md 标准）和知识库 RAG。

### 8.2 具体任务

| # | 任务 | 预估 | 说明 |
|---|------|------|------|
| 5.1 | Skill 发现 (多目录扫描, walkdir + glob) | 3h | Rust 实现 |
| 5.2 | SKILL.md 解析 (YAML frontmatter + Markdown body) | 3h | serde_yaml |
| 5.3 | Skill 注册表 (DashMap + 缓存 + 去重) | 3h | 并发安全 |
| 5.4 | Skill 执行器 (Prompt 注入 + Fork 上下文) | 5h | Rust + Rig |
| 5.5 | Skill 热重载 (notify 文件监听) | 2h | 目录变化刷新 |
| 5.6 | Skills UI (浏览/搜索/启用/禁用) | 5h | React + shadcn |
| 5.7 | 文档导入 (PDF/MD/TXT 解析 + 纯文本提取) | 5h | Rust 文本提取 |
| 5.8 | 智能文档分块 (按语义边界切分 chunks) | 4h | 固定大小 + 重叠 |
| 5.9 | Embedding 生成 (Rig → OpenAI/本地模型) | 3h | 调用 Embedding API |
| 5.10 | 向量+全文索引写入 (sqlite-vec + FTS5) | 3h | 双索引并行写入 |
| 5.11 | 混合搜索 (sqlite-vec KNN + FTS5 + RRF 融合) | 4h | SQL CTE 实现 |
| 5.12 | RAG 对话集成 (检索→注入上下文→LLM 回答) | 4h | 知识增强对话 |
| 5.13 | 知识库管理 UI (导入/列表/删除/搜索测试) | 5h | React 页面 |

### 8.3 Phase 5 交付物

- **完整 Skills 系统**（发现、加载、执行、热重载、UI 管理）
- **知识库 RAG**（文档导入 → 分块 → 向量化 → 混合搜索 → 增强对话）

### 8.4 本阶段里程碑

> **M4: 知识增强** — Agent 可以基于用户导入的知识库进行精准问答。

---

## 9. Phase 6：高级功能与打磨（第 18-20 周）

### 9.1 目标

补全高级功能，全面打磨用户体验，准备发布。

### 9.2 具体任务

| # | 任务 | 预估 | 说明 |
|---|------|------|------|
| 6.1 | WASM 沙箱 (Wasmtime 集成 + 代码执行) | 8h | 安全执行环境 |
| 6.2 | Dashboard (会话统计/Token 图表/成本分析) | 6h | 数据可视化 |
| 6.3 | 文件浏览器 (树形目录 + 文件预览 + 代码高亮) | 6h | Monaco Editor |
| 6.4 | 任务管理 (任务列表 + 状态追踪) | 4h | 持久化任务 |
| 6.5 | 系统通知 (Agent 完成/错误/审批请求) | 2h | tauri-plugin-notification |
| 6.6 | 自动更新 (tauri-plugin-updater) | 3h | 增量更新 |
| 6.7 | 跨平台构建测试 (Windows + macOS + Linux) | 6h | CI/CD 配置 |
| 6.8 | 安装包构建 (MSI/DMG/AppImage) | 4h | Tauri 打包 |
| 6.9 | 性能优化 (React.memo/虚拟列表/懒加载) | 4h | 大会话性能 |
| 6.10 | 快捷键系统 (全局 + 页面级) | 3h | 键盘操作 |
| 6.11 | 启动向导 (首次运行引导配置) | 3h | 用户体验 |

### 9.3 Phase 6 交付物

- **可发布的完整桌面应用**
- WASM 代码沙箱
- Dashboard 数据分析
- 文件浏览器
- 自动更新
- 三平台安装包

### 9.4 本阶段里程碑

> **M5: 发布就绪 (v0.1.0)** — Claw 主体功能完整，可以面向早期用户发布。

---

## 10. Phase B：Buddy 桌面伴侣（并行，第 8-20 周）

### 10.1 并行开发策略

Buddy 系统与主应用共享 Rust Core 后端。从 Phase 4（Sidecar 引入后）开始，利用每周约 20% 的时间并行推进 Buddy。

### 10.2 开发阶段

| 子阶段 | 主线 Phase | 预估 | 任务 |
|--------|-----------|------|------|
| **B1: 透明浮窗** | Phase 4 | 6h | Tauri 多窗口 + transparent + alwaysOnTop + 拖拽 |
| **B2: 角色动画** | Phase 4 | 6h | Lottie 集成 + 待机/说话/思考 状态切换 |
| **B3: 文字对话** | Phase 4-5 | 8h | 迷你输入框 + LLM 对话 + 气泡回复 + 情绪驱动 |
| **B4: 语音输入** | Phase 5 | 10h | whisper-cpp-plus 集成 + 快捷键触发 + 实时转录 |
| **B5: 语音输出** | Phase 5-6 | 6h | piper-rs 集成 + 音频播放 + 口型状态切换 |
| **B6: 高级交互** | Phase 6 | 8h | MCP 工具调用 + 人设系统 + 多 Buddy 形象 |

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
| R2 | rmcp (MCP Rust SDK) 存在 Bug | 中 | 中 | 保留 TypeScript MCP SDK 作为降级方案；rmcp 有 Issue 可提 PR |
| R3 | Python Sidecar 打包体积过大 | 高 | 低 | Nuitka AOT 替代 PyInstaller；精简依赖树 |
| R4 | whisper-cpp-plus 跨平台编译问题 | 中 | 低 | Buddy 语音是 P1 功能，可推迟；先用全局快捷键+文字输入 |
| R5 | sqlite-vec 与 rusqlite 版本兼容 | 低 | 中 | 锁定已验证的版本组合 (rusqlite 0.34 + sqlite-vec 0.1) |
| R6 | Vercel AI SDK 与 Tauri IPC 集成 | 中 | 中 | 如不兼容，降级为手动 SSE 处理 + 自定义 React hooks |
| R7 | 个人精力不足 | 中 | 高 | 严格按 Phase 推进，保持 MVP 心态；Buddy 可完全推迟 |

---

## 12. 里程碑总览

```
Week  1  ──── Phase 0: 环境搭建 ──────────────────────────── ✓ 项目可运行
Week  2  ┐
Week  3  ├─── Phase 1: 基础框架 + UI ─────────────────────── ✓ 完整 UI 骨架
Week  4  ┘
Week  5  ┐
Week  6  ├─── Phase 2: 对话系统 ───────────────────── M1 ── ✓ 核心可用
Week  7  ┘
Week  8  ┐
Week  9  ├─── Phase 3: MCP + 会话管理 ────────────── M2 ── ✓ MCP 就绪
Week 10  ┘                                            │
Week 11  ┐                                      Phase B 并行启动
Week 12  ├─── Phase 4: Agent 编排 + 记忆 ─────── M3 ── ✓ Agent 平台
Week 13  │                                       │ B1: 浮窗
Week 14  ┘                                       │ B2: 动画
Week 15  ┐                                       │ B3: 文字对话
Week 16  ├─── Phase 5: Skills + 知识库 ──────── M4 ── ✓ 知识增强
Week 17  ┘                                       │ B4: 语音输入
Week 18  ┐                                       │ B5: 语音输出
Week 19  ├─── Phase 6: 高级功能 + 打磨 ─────── M5 ── ✓ 发布就绪
Week 20  ┘                                       │ B6: 高级交互
                                                  ↓
                                          Buddy v1.0 完成

═══════════════════════════════════════════════════════════
总周期: 20 周 (约 5 个月)
里程碑: M1(核心可用) → M2(MCP就绪) → M3(Agent平台) → M4(知识增强) → M5(发布就绪)
```

### 各里程碑版本号

| 里程碑 | 版本 | 核心能力 |
|--------|------|---------|
| M1 | v0.1.0-alpha | 多模型对话、流式输出、会话管理 |
| M2 | v0.2.0-alpha | + MCP 工具调用、权限审批 |
| M3 | v0.3.0-beta | + Agent 编排、SubAgent、智能记忆 |
| M4 | v0.4.0-beta | + Skills 系统、知识库 RAG |
| M5 | v0.5.0-rc | + 沙箱、Dashboard、自动更新、Buddy |

---

> **文档结束**
>
> 本实施方案基于 [CLAW_ARCHITECTURE_SELECTION.md v2.1](./CLAW_ARCHITECTURE_SELECTION.md) 的技术决策，
> 针对 1 人 + Vibe Coding 的开发模式进行了工期优化和阶段重组。
> 每个 Phase 交付可运行的功能切片，确保项目始终处于可演示状态。
