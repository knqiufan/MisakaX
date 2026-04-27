# Claw 桌面端技术选型调研报告

> **项目代号：** Claw（基于 Misaka 项目的下一代桌面端 AI Agent 客户端）
> **调研日期：** 2026-04-21
> **文档版本：** v1.0
> **目标：** 基于 Tauri 2.x 重写桌面端应用，脱离 claude-agent-sdk 依赖，手动实现 MCP、Skills、沙箱、SubAgent、记忆等全部核心功能

---

## 目录

1. [项目背景与需求分析](#1-项目背景与需求分析)
2. [当前 Misaka 项目功能清单](#2-当前-misaka-项目功能清单)
3. [Tauri 2.x 核心架构分析](#3-tauri-2x-核心架构分析)
4. [前端 UI 框架选型](#4-前端-ui-框架选型)
5. [AI Agent 编排框架调研](#5-ai-agent-编排框架调研)
6. [Rust 生态 AI Agent 框架](#6-rust-生态-ai-agent-框架)
7. [核心功能实现方案](#7-核心功能实现方案)
8. [状态管理方案](#8-状态管理方案)
9. [数据存储方案](#9-数据存储方案)
10. [综合技术栈推荐](#10-综合技术栈推荐)
11. [架构设计方案](#11-架构设计方案)
12. [风险评估与应对策略](#12-风险评估与应对策略)
13. [结论与最终推荐](#13-结论与最终推荐)
14. [Buddy 桌面伴侣系统深度调研](#14-buddy-桌面伴侣系统深度调研)

---

## 1. 项目背景与需求分析

### 1.1 项目动机

当前 Misaka 项目基于 Python + Flet（Flutter UI 框架）构建，依赖 `claude-agent-sdk` 和 `@anthropic-ai/claude-code` CLI 与 Claude 交互。虽然功能已覆盖对话、会话管理、MCP 服务、Skills 管理等核心场景，但存在以下局限：

| 问题 | 说明 |
|------|------|
| **性能瓶颈** | Python + Flet 在复杂 UI 场景下性能有限，启动慢、内存占用高 |
| **SDK 依赖** | 强依赖 claude-agent-sdk，无法灵活扩展至其他 LLM 提供商 |
| **功能受限** | SubAgent、沙箱隔离、持久化记忆等功能未能独立实现 |
| **分发困难** | PyInstaller 打包体积大，跨平台兼容性差 |
| **架构耦合** | UI 层和业务逻辑耦合度较高，不易维护 |

### 1.2 核心需求矩阵

| 需求类别 | 具体要求 | 优先级 |
|---------|---------|--------|
| **多模型支持** | 不依赖单一 SDK，支持 OpenAI/Anthropic/Gemini/本地模型 | P0 |
| **MCP 协议** | 完整实现 MCP Client，支持 stdio/HTTP/SSE 传输 | P0 |
| **Agent 编排** | 支持多步骤工作流、条件分支、并行执行 | P0 |
| **SubAgent** | 支持子代理派生、任务委托、独立上下文 | P0 |
| **沙箱隔离** | 安全执行用户/Agent 生成的代码 | P1 |
| **持久化记忆** | 跨会话记忆检索、向量化存储 | P0 |
| **Skills 系统** | 可发现、安装、管理、执行的技能模块 | P1 |
| **流式对话** | 支持实时 token 流式输出和思维链展示 | P0 |
| **高性能 UI** | Material Design 3 风格，流畅的交互体验 | P0 |
| **跨平台** | Windows / macOS / Linux 全平台支持 | P0 |
| **小体积分发** | 安装包尽可能小 | P1 |

---

## 2. 当前 Misaka 项目功能清单

以下是 Claw 必须覆盖（甚至超越）的功能矩阵：

| 功能模块 | Misaka 现有能力 | Claw 目标增强 |
|---------|----------------|-------------|
| **对话系统** | 流式文本/思维链、多模态输入（图片）、Agent/Plan/Ask 三种模式 | + 支持更多模型提供商，+ 结构化输出 |
| **会话管理** | CRUD、搜索、分组、归档、SDK 会话恢复、CLI 导入 | + 分支对话、+ 会话模板 |
| **MCP** | 从配置文件加载、stdio/HTTP/SSE 传输、生命周期管理 | + 独立 MCP Client 实现、+ 动态发现 |
| **Skills** | 全局/项目/已安装/插件路径扫描、Skyll 市场搜索安装 | + 本地 Skills 沙箱执行、+ Skills API |
| **路由配置** | 多 Router 配置、API Key 管理、Agent Team 实验性支持 | + 智能路由、+ 负载均衡 |
| **环境诊断** | CLI/API 健康检查、版本检测、启动向导 | + 自动修复 |
| **Dashboard** | 会话/消息统计、Token 图表、技能统计 | + 实时监控、+ 成本分析 |
| **任务管理** | DB 持久化任务列表 | + 任务依赖图、+ 进度追踪 |
| **文件浏览** | 树形目录、文件预览、路径安全 | + 代码高亮、+ Diff 视图 |
| **权限系统** | SDK can_use_tool 回调、自动允许、跳过权限 | + 细粒度 ACL、+ 审计日志 |
| **国际化** | en/zh-CN/zh-TW | + 更多语言、+ 动态加载 |
| **主题系统** | 明/暗/跟随系统、强调色 | + 自定义主题、+ 主题市场 |
| **沙箱** | 仅文件路径安全检查 | + WASM 沙箱、+ 资源限制 |
| **SubAgent** | 依赖 SDK 实验性 Agent Teams | + 独立 SubAgent 调度器 |
| **记忆** | 仅 slash 命令 /memory（CLAUDE.md 风格） | + 向量化持久记忆、+ 跨会话检索 |

---

## 3. Tauri 2.x 核心架构分析

### 3.1 架构概述

```
┌─────────────────────────────────────────────────┐
│              Web Frontend (WebView)              │
│         React/Vue/Svelte + TypeScript            │
├─────────────────────────────────────────────────┤
│             Tauri IPC (Message Passing)           │
├─────────────────────────────────────────────────┤
│              Rust Core Process                    │
│     ┌──────────┬──────────┬───────────┐          │
│     │  Plugins  │ Commands │  Sidecar   │         │
│     └──────────┴──────────┴───────────┘          │
├─────────────────────────────────────────────────┤
│           OS Native APIs / File System            │
└─────────────────────────────────────────────────┘
```

### 3.2 Tauri 2.x vs Electron 对比（2026 年数据）

| 指标 | Tauri 2.x | Electron 34.x | 差异 |
|------|-----------|---------------|------|
| Hello World 包体积 | 3.2 MB | 85 MB | Tauri 小 96% |
| 复杂应用包体积 | 8.6 MB | 244 MB | Tauri 小 96% |
| 冷启动时间 | 380 ms | 1,420 ms | Tauri 快 3.7x |
| 空闲内存 | 42 MB | 168 MB | Tauri 少 75% |
| IPC 延迟 | 0.12 ms | 0.45 ms | Tauri 快 3.75x |
| 文件读取 (100MB) | 85 ms | 142 ms | Tauri 快 40% |
| 后端语言 | Rust | Node.js | Rust 性能更优 |
| 渲染引擎 | 系统 WebView | 自带 Chromium | Tauri 更轻量 |
| 安全模型 | 基于许可的 ACL | 完整 Node.js 访问 | Tauri 更安全 |
| 插件生态 | 成长中 (~100s) | 成熟 (~1000s) | Electron 更丰富 |
| 构建速度 | 48s (初次) | 22s (初次) | Electron 更快 |

### 3.3 Tauri Sidecar 模式

Tauri 的 **Sidecar** 模式是实现多语言后端的关键架构。它允许将任何可执行文件（Go、Python、Node.js 等编译产物）与 Tauri 应用一起打包分发：

```
Tauri App
├── WebView (前端 UI)
├── Rust Core (主进程，负责系统调用、IPC)
└── Sidecar (Python/Go/Node 可执行文件)
    ├── AI Agent 逻辑（如 LangGraph）
    ├── MCP Server 管理
    └── 复杂业务逻辑
```

配置方式：
```json
{
  "bundle": {
    "externalBin": ["binaries/agent-server"]
  }
}
```

Rust 端启动 Sidecar：
```rust
use tauri_plugin_shell::ShellExt;
let sidecar = app.shell().sidecar("agent-server").unwrap();
let (mut rx, child) = sidecar.spawn().expect("Failed to spawn sidecar");
```

### 3.4 Tauri 关键插件

| 插件 | 功能 | 对 Claw 的价值 |
|------|------|---------------|
| `tauri-plugin-fs` | 文件系统访问 | 文件浏览、项目扫描 |
| `tauri-plugin-shell` | 进程管理、Sidecar | 启动 Agent 后端、MCP Server |
| `tauri-plugin-http` | HTTP 客户端 | LLM API 调用 |
| `tauri-plugin-websocket` | WebSocket | 流式通信 |
| `tauri-plugin-store` | 持久化键值存储 | 设置存储 |
| `tauri-plugin-notification` | 系统通知 | Agent 完成通知 |
| `tauri-plugin-clipboard` | 剪贴板 | 复制代码 |
| `tauri-plugin-dialog` | 原生对话框 | 文件选择 |
| `tauri-plugin-updater` | 自动更新 | 应用更新 |
| `tauri-plugin-sql` | SQL 数据库 | SQLite 存储 |

---

## 4. 前端 UI 框架选型

### 4.1 候选框架对比

| 指标 | React 19 | Vue 4 | Svelte 5 | SolidJS |
|------|----------|-------|----------|---------|
| **性能（1K 行创建）** | 28.4 ops/s | 31.2 ops/s | 39.5 ops/s | 42.8 ops/s |
| **启动时间** | 52ms | 45ms | 32ms | 28ms |
| **Bundle 大小** | ~45KB | ~16KB | ~5KB (无运行时) | ~7KB |
| **内存占用** | 较高 (VDOM) | 中等 (VDOM) | 低 (无 VDOM) | 最低 (无 VDOM) |
| **Lighthouse 分数** | 92 | 94 | 96 | 98 |
| **npm 周下载量** | ~190M | ~18M | ~2.7M | ~1.5M |
| **生态成熟度** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **组件库数量** | 极丰富 | 丰富 | 有限 | 较少 |
| **TypeScript 支持** | 优秀 | 优秀 | 优秀 | 优秀 |
| **学习曲线** | 中等 | 低 | 低 | 中等 |
| **社区活跃度** | 最高 | 高 | 中等 | 中等 |
| **Tauri 模板** | ✅ 官方 | ✅ 官方 | ✅ 官方 | ✅ 官方 |

### 4.2 UI 组件库对比

#### React 生态

| 组件库 | Bundle Size | 风格 | 特点 | 推荐度 |
|--------|-------------|------|------|--------|
| **shadcn/ui** | 0KB (源码拷贝) | 现代极简 | Radix + Tailwind，完全可控，2026 年 React 标配 | ⭐⭐⭐⭐⭐ |
| **Nocta UI** | 0KB (源码拷贝) | 暗色现代 | Ariakit + Tailwind，无包锁定 | ⭐⭐⭐⭐ |
| **NexUI** | 0KB (源码拷贝) | shadcn 风格 | 100+ 组件，shadcn/ui 的增强版 | ⭐⭐⭐⭐ |
| **Ninna UI** | 按需安装 | Chakra 风格 | Radix + Tailwind v4，69 组件 | ⭐⭐⭐⭐ |
| **Ant Design** | 较大 | 企业级 | 功能齐全但体积大 | ⭐⭐⭐ |

#### Svelte 生态

| 组件库 | 特点 | 推荐度 |
|--------|------|--------|
| **Skeleton UI** | Svelte 原生，Tailwind 集成 | ⭐⭐⭐⭐ |
| **shadcn-svelte** | shadcn/ui 的 Svelte 移植 | ⭐⭐⭐⭐ |
| **Melt UI** | 无样式 headless 组件 | ⭐⭐⭐ |

### 4.3 前端框架推荐结论

#### 🏆 主推荐：React 19 + TypeScript

**理由：**
1. **生态最完善**：190M+ 周下载量，最大的组件库和工具链生态
2. **shadcn/ui 生态**：2026 年事实标准的 UI 组件库，完美匹配桌面端应用需求
3. **人才储备**：最大的开发者社区，招聘/协作最容易
4. **AI Agent UI 参考实现最多**：ChatML、Graphone 等同类产品均使用 React
5. **与 Tauri 配合成熟**：大量生产案例验证

#### 🥈 备选方案：Svelte 5 + TypeScript

**理由：**
1. 性能更优，无 VDOM 开销
2. 编译时优化，bundle 更小
3. Graphone（Tauri 2.x AI Agent 工作台）即采用 Svelte 5
4. 但组件生态较小，某些复杂 UI 需求可能需要自行实现

---

## 5. AI Agent 编排框架调研

### 5.1 主流框架全景对比

| 框架 | 语言 | GitHub Stars | 架构模式 | MCP 支持 | 多 Agent | 记忆 | 学习曲线 | 生产就绪度 |
|------|------|-------------|---------|---------|---------|------|---------|-----------|
| **LangGraph** | Python/JS | 25K | 图状态机 | 适配器 | 图节点 | Checkpointer + Store | 陡峭 | ⭐⭐⭐⭐⭐ |
| **CrewAI** | Python | 46K | 角色团队 | 原生 (v1.10) | Sequential/Parallel/Hierarchical | 内建短期+长期 | 低 | ⭐⭐⭐⭐ |
| **AutoGen** | Python/.NET | 36K | 对话式 | 部分 | GroupChat | 对话历史 | 中 | ⭐⭐⭐ |
| **OpenAI Agents SDK** | Python | 19K | 原语式 | 原生 | Handoffs | Sessions | 低 | ⭐⭐⭐ |
| **Semantic Kernel** | C#/Python/Java | 28K | 插件式 | 原生 | Graph-based | 内建 | 中 | ⭐⭐⭐⭐ |
| **Mastra** | TypeScript | 22K | 工作流 | 原生 | Workflows | 内建 | 低 | ⭐⭐⭐⭐ |
| **Vercel AI SDK** | TypeScript | N/A (20M+ npm/mo) | 流式 UI | 原生 | 手动 | 手动 | 低 | ⭐⭐⭐⭐ |
| **Google ADK** | Python | 17K | 工作流 Agent | 社区 | Workflow Agents | Session + Memory Bank | 中 | ⭐⭐⭐ |
| **Pydantic AI** | Python | 16K | 类型安全 | 社区 | 手动 | 手动 | 低 | ⭐⭐⭐ |
| **Deep Agents** | Python | 新 (2026.3) | 分层规划 | 继承 LG | Sub-Agent 委托 | 时序记忆 | 陡峭 | ⭐⭐⭐ |

### 5.2 LangGraph 深度分析

#### 核心架构
```
StateGraph → Nodes (计算步骤) → Edges (控制流) → 共享 State 对象
```

#### 关键能力

| 能力 | 描述 | Claw 价值 |
|------|------|----------|
| **持久化执行** | 通过 Checkpointer 实现崩溃恢复、长时间运行 | Agent 任务不中断 |
| **Human-in-the-loop** | 在任意节点暂停等待人工审批 | 权限审批、敏感操作确认 |
| **短期记忆 (Checkpointer)** | 线程级状态快照，每个 super-step 保存 | 对话连续性 |
| **长期记忆 (Store)** | 跨线程 JSON 文档存储，命名空间隔离 | 用户偏好、事实记忆 |
| **子图 (Subgraph)** | 嵌套图执行，独立状态和检查点 | SubAgent 实现 |
| **流式输出** | 类型安全的 StreamPart 输出 (v2) | 实时 UI 渲染 |
| **时间旅行** | 回溯到任意检查点重放 | 调试和回滚 |

#### 记忆系统架构

| 记忆类型 | 作用域 | 持久性 | 用途 | 生产存储 |
|---------|--------|--------|------|---------|
| **Checkpointer（短期）** | 单线程 | 线程生命周期 | 对话连续性 | SqliteSaver / AsyncPostgresSaver |
| **Store（长期）** | 跨线程 | 无期限 | 用户偏好、历史事实 | PostgresStore |

#### 与 Tauri 的集成方案

```
┌─────────────────────────────────────────┐
│           Tauri WebView (React)          │
│            ↕ Tauri IPC                   │
├─────────────────────────────────────────┤
│           Tauri Rust Core                │
│    ┌──────────────────────────────┐      │
│    │   Sidecar Manager            │      │
│    │   (启动/停止/健康检查)          │      │
│    └──────────┬───────────────────┘      │
│               ↕ HTTP/WebSocket           │
│    ┌──────────────────────────────┐      │
│    │   Python Sidecar (FastAPI)    │      │
│    │   ├── LangGraph Agent 编排    │      │
│    │   ├── MCP Client             │      │
│    │   ├── Memory Store           │      │
│    │   └── Tool Executor          │      │
│    └──────────────────────────────┘      │
└─────────────────────────────────────────┘
```

**Python Sidecar 打包方式：**
- 使用 **PyInstaller** 或 **Nuitka** 将 Python + LangGraph 编译为独立可执行文件
- 通过 `tauri.conf.json` 的 `externalBin` 字段注册
- Tauri 启动时自动启动 Sidecar，关闭时自动清理

### 5.3 TypeScript 原生方案分析

如果希望避免 Python Sidecar，可以在 Tauri 前端侧使用 TypeScript 原生框架：

| 方案 | 框架 | 优势 | 劣势 |
|------|------|------|------|
| **Vercel AI SDK** | TypeScript | 与 React 完美集成、原生流式 UI、MCP 原生支持 | 多 Agent 编排需手动实现 |
| **Mastra** | TypeScript | 内建工作流、记忆、MCP 原生支持 | 相对年轻 |
| **LangGraph.js** | TypeScript | 与 Python 版 API 一致 | 功能略滞后于 Python 版 |

### 5.4 Agent 编排框架推荐结论

#### 🏆 方案 A（推荐）：LangGraph (Python Sidecar) + Vercel AI SDK (前端)

```
React UI ←→ Vercel AI SDK (流式渲染) ←→ Tauri IPC ←→ Rust Core ←→ Python Sidecar (LangGraph)
```

**理由：**
1. **LangGraph** 是 2026 年最成熟的 Agent 编排框架，生产就绪度最高
2. 内建完整的记忆系统（Checkpointer + Store）
3. 子图机制天然支持 SubAgent
4. Human-in-the-loop 直接对应权限审批流程
5. Deep Agents 扩展提供更高级的规划和委托能力
6. **Vercel AI SDK** 负责前端流式渲染，与 React 完美集成

**风险：**
- Python Sidecar 增加打包体积（预计 +50-80MB）
- 需管理两个进程的生命周期
- 跨进程通信引入少量延迟

#### 🥈 方案 B：纯 TypeScript 方案 (Mastra / LangGraph.js)

```
React UI ←→ Mastra/LangGraph.js ←→ Tauri IPC ←→ Rust Core
```

**理由：**
1. 无需 Python Sidecar，包体积更小
2. 统一 TypeScript 技术栈
3. MCP 原生支持

**风险：**
- TypeScript Agent 框架的记忆和编排能力不如 Python 生态成熟
- 复杂 Agent 工作流实现难度更高

#### 🥉 方案 C：Rust 原生方案 (Rig)

```
React UI ←→ Tauri IPC ←→ Rust Core (Rig Agent 编排)
```

**理由：**
1. 最小包体积，最佳性能
2. 与 Tauri Rust 后端无缝集成

**风险：**
- Rust AI Agent 生态远不如 Python/TypeScript 成熟
- Rig 的记忆/编排能力有限
- 开发效率较低

---

## 6. Rust 生态 AI Agent 框架

### 6.1 Rig

| 指标 | 详情 |
|------|------|
| **GitHub Stars** | 6,700+ |
| **许可证** | MIT |
| **最新版本** | rig-core (2026-04 活跃更新) |
| **贡献者** | 180+ |
| **下载量** | 541K+ |

**核心特性：**
- 20+ LLM 提供商统一接口（OpenAI、Anthropic、Gemini、Cohere、AWS Bedrock 等）
- 10+ 向量存储集成（MongoDB、Qdrant、LanceDB、PostgreSQL、Neo4j 等）
- Agent + Tool 调用工作流
- RAG Pipeline 支持
- 流式输出
- **完整 WASM 兼容**（核心库）
- 类型安全

**适合 Claw 的场景：**
- Rust 后端直接调用 LLM API（绕过 Sidecar 的轻量场景）
- 嵌入式向量搜索（记忆检索）
- 简单的单 Agent 工具调用

**局限：**
- 无内建多 Agent 编排
- 无内建记忆系统
- 无 Human-in-the-loop 原语

### 6.2 swarms-rs

| 指标 | 详情 |
|------|------|
| **定位** | 企业级多 Agent 编排框架 |
| **许可证** | MIT |
| **特点** | 三层架构：Agent → 多 Agent 结构 → 级联系统 |

**核心特性：**
- Agent 层：LLM 集成 + Tool 系统 + 记忆管理
- 多 Agent 结构：Sequential / Concurrent / 通信协议
- 级联系统：Agent 网络、层级组织、群体智能
- **MCP 集成**：支持 stdio 和 SSE 接口
- 可扩展的工具框架

**适合 Claw 的场景：**
- 如果需要在 Rust 端实现复杂多 Agent 系统
- 大规模 Agent 协调

**局限：**
- 社区较小，生产案例有限
- 文档不够完善

### 6.3 MCP Rust SDK (rmcp)

| 指标 | 详情 |
|------|------|
| **组织** | modelcontextprotocol (官方) |
| **最新版本** | v1.4.0 (2026-04-10) |
| **SDK 层级** | Tier 2 (官方维护) |
| **运行时** | tokio async |

**核心特性：**
- 完整 MCP 协议实现（Client + Server）
- Tools / Resources / Prompts / Sampling / Roots
- 过程宏自动生成工具实现 (`rmcp-macros`)
- stdio 和 HTTP 传输

**配置：**
```toml
[dependencies]
rmcp = { version = "1.4.0", features = ["client", "transport-child-process", "transport-sse"] }
```

**对 Claw 的关键价值：** 可以直接在 Rust 后端实现 MCP Client，无需依赖 Python/TypeScript。

---

## 7. 核心功能实现方案

### 7.1 MCP (Model Context Protocol) 实现

#### 方案 A（推荐）：Rust 原生 MCP Client

使用官方 `rmcp` crate 在 Tauri Rust 后端直接实现 MCP Client：

```
Tauri Rust Core
├── MCP Client Manager
│   ├── rmcp Client (Rust)
│   ├── stdio Transport → 子进程管理
│   ├── HTTP/SSE Transport → HTTP 客户端
│   └── 配置加载 (.mcp.json / settings.json)
```

**优势：**
- 性能最优，无跨进程开销
- 与 Tauri 进程管理天然集成
- 类型安全

#### 方案 B：TypeScript MCP Client（前端侧）

使用 `@modelcontextprotocol/client` 在前端：

```
WebView
├── @modelcontextprotocol/client
└── 通过 Tauri shell plugin 管理 MCP Server 子进程
```

#### 推荐：方案 A（Rust 原生 MCP Client）

| 理由 | 说明 |
|------|------|
| 性能 | 原生 Rust 性能，无序列化/跨进程开销 |
| 集成度 | 与 Tauri 进程管理、文件系统插件深度集成 |
| 安全性 | 利用 Tauri 的权限系统控制 MCP 工具访问 |
| 官方支持 | rmcp 是 MCP 官方 Rust SDK |

### 7.2 Skills 系统实现

```
Skills Architecture
├── Skill Registry (Rust 后端)
│   ├── 本地扫描：~/.claw/skills/, .claw/skills/
│   ├── 市场 API：Skyll API 或自建市场
│   └── 安装管理：下载、验证、注册
├── Skill Executor
│   ├── WASM 沙箱执行（安全隔离）
│   ├── 脚本执行（受控环境）
│   └── 工具注入（注入为 MCP 工具）
└── Skill UI (React 前端)
    ├── 浏览和搜索
    ├── 安装和管理
    └── 编辑器（Skill 内容编辑）
```

**实现技术栈：**
- Rust 后端负责 Skill 发现、注册、生命周期管理
- 技能文件格式沿用 `SKILL.md` 标准
- 安全执行通过 WASM 沙箱或受限子进程

### 7.3 沙箱隔离实现

#### 方案 A（推荐）：WebAssembly (WASM) 沙箱

使用 **Wasmtime** 在 Rust 端实现 WASM 沙箱：

```toml
[dependencies]
wasmtime = "41"
```

| 特性 | 实现方式 |
|------|---------|
| **代码隔离** | 每个任务运行在独立 WASM 模块中 |
| **CPU 限制** | Wasmtime 燃料计量机制 |
| **内存限制** | WASM 线性内存上限 |
| **超时控制** | tokio timeout 包装 |
| **文件系统** | WASI 受控的文件挂载 |
| **网络访问** | 默认禁止，白名单放行 |
| **冷启动** | <13ms (AOT 预编译后 ~55μs) |

参考实现：
- **Capsule** (capsulerun/capsule)：基于 Rust + Wasmtime 的 AI Agent 沙箱运行时
- **agent-sandbox**：WASM 沙箱 + 80+ 内建 CLI 工具

#### 方案 B：Docker 容器沙箱

| 优势 | 劣势 |
|------|------|
| 完整的操作系统级隔离 | 启动慢 (秒级) |
| 支持任意语言运行时 | 需要用户安装 Docker |
| 完整的文件系统和网络控制 | 资源开销大 |

#### 推荐：方案 A (WASM 沙箱) + 方案 B (Docker 可选)

日常轻量级代码执行使用 WASM 沙箱（毫秒启动），需要完整环境时可选 Docker。

### 7.4 SubAgent 系统实现

#### 基于 LangGraph 子图的 SubAgent

```python
# LangGraph SubAgent 架构示例
from langgraph.graph import StateGraph

# 子 Agent 定义为独立子图
research_agent = StateGraph(ResearchState)
research_agent.add_node("search", search_node)
research_agent.add_node("analyze", analyze_node)

code_agent = StateGraph(CodeState)
code_agent.add_node("generate", generate_node)
code_agent.add_node("review", review_node)

# 主 Agent 图编排子 Agent
main_graph = StateGraph(MainState)
main_graph.add_node("planner", planner_node)
main_graph.add_node("researcher", research_agent.compile())
main_graph.add_node("coder", code_agent.compile())
main_graph.add_conditional_edges("planner", route_to_agent)
```

#### SubAgent 特性矩阵

| 特性 | 实现方式 | 优先级 |
|------|---------|--------|
| **独立上下文** | LangGraph 子图独立状态 | P0 |
| **任务委托** | 主 Agent 通过条件边路由到子 Agent | P0 |
| **并行执行** | LangGraph Fan-out/Fan-in 节点 | P1 |
| **结果汇总** | 子图输出合并到主图状态 | P0 |
| **独立记忆** | 子图独立 checkpoint_ns | P1 |
| **动态创建** | 运行时根据任务类型派生子 Agent | P2 |

### 7.5 记忆系统实现

#### 三层记忆架构

```
┌───────────────────────────────────────────────┐
│              Memory Architecture               │
├───────────────────────────────────────────────┤
│  Layer 1: 工作记忆 (Working Memory)              │
│  ├── 当前对话上下文                               │
│  ├── 实现：LangGraph Checkpointer               │
│  └── 存储：SQLite (本地)                         │
├───────────────────────────────────────────────┤
│  Layer 2: 短期记忆 (Short-term Memory)           │
│  ├── 最近 N 次会话的关键信息                       │
│  ├── 实现：LangGraph Store (命名空间隔离)          │
│  └── 存储：SQLite / 本地 JSON                    │
├───────────────────────────────────────────────┤
│  Layer 3: 长期记忆 (Long-term Memory)            │
│  ├── 用户偏好、项目知识、历史事实                    │
│  ├── 实现：向量化存储 + 语义检索                    │
│  └── 存储：Qdrant (嵌入式) 或 LanceDB (本地)      │
└───────────────────────────────────────────────┘
```

#### 向量存储选型

| 方案 | 类型 | 特点 | 推荐度 |
|------|------|------|--------|
| **LanceDB** | 嵌入式 | Rust 原生，零依赖，本地文件存储 | ⭐⭐⭐⭐⭐ |
| **Qdrant** | 嵌入式/服务端 | 性能优秀，支持嵌入式模式 | ⭐⭐⭐⭐ |
| **ChromaDB** | Python | 流行但需要额外进程 | ⭐⭐⭐ |
| **SQLite + FTS5** | 嵌入式 | 全文搜索，无向量但够用于关键词 | ⭐⭐⭐⭐ |

**推荐：LanceDB (嵌入式)**
- Rust 原生，可直接集成到 Tauri 后端
- 本地文件存储，无需额外服务
- 支持向量搜索 + 全文搜索混合查询
- Apache 2.0 许可证

---

## 8. 状态管理方案

### 8.1 前端状态管理

#### 推荐：Zustand + TanStack Query

| 状态类型 | 解决方案 | 说明 |
|---------|---------|------|
| **服务器状态** | TanStack Query (React Query) | LLM 响应缓存、会话列表、设置 |
| **客户端 UI 状态** | Zustand (~1.1KB) | 主题、侧边栏、当前会话、模态框 |
| **表单状态** | React Hook Form | 设置表单、路由配置表单 |
| **组件局部状态** | useState/useReducer | 组件内部状态 |
| **URL 状态** | 无需（桌面端非 URL 驱动） | - |

#### Zustand 与 Tauri 集成

```typescript
import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';

interface AppState {
  theme: 'light' | 'dark' | 'system';
  currentSessionId: string | null;
  sidebarOpen: boolean;
  setTheme: (theme: 'light' | 'dark' | 'system') => void;
  setCurrentSession: (id: string | null) => void;
}

const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      theme: 'system',
      currentSessionId: null,
      sidebarOpen: true,
      setTheme: (theme) => set({ theme }),
      setCurrentSession: (id) => set({ currentSessionId: id }),
    }),
    { name: 'claw-app-state' }
  )
);
```

### 8.2 后端状态管理（Rust 侧）

Rust 后端使用 `tauri::Manager` 的状态管理：

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppStateInner {
    pub db: Database,
    pub mcp_manager: McpManager,
    pub agent_sidecar: Option<Child>,
}

pub type AppState = Arc<RwLock<AppStateInner>>;
```

---

## 9. 数据存储方案

### 9.1 本地数据库：SQLite (通过 Tauri 插件或 Rust 直连)

| 方案 | 实现 | 推荐度 |
|------|------|--------|
| **tauri-plugin-sql** | Tauri 官方 SQL 插件 | ⭐⭐⭐ |
| **rusqlite** | Rust 原生 SQLite 绑定 | ⭐⭐⭐⭐⭐ |
| **sqlx** | Rust 异步 SQL 库 | ⭐⭐⭐⭐ |

**推荐：rusqlite** — 直接在 Rust 后端使用，性能最优，WAL 模式支持。

### 9.2 数据模型设计

```sql
-- 会话表
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    title TEXT,
    model TEXT,
    system_prompt TEXT,
    working_directory TEXT,
    project_name TEXT,
    status TEXT DEFAULT 'active',  -- active/archived/hidden
    mode TEXT DEFAULT 'agent',      -- agent/plan/ask
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 消息表
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,             -- user/assistant/system/tool
    content TEXT NOT NULL,          -- JSON blocks
    token_usage TEXT,               -- JSON: {input, output, cache_read, cache_write}
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- 记忆表
CREATE TABLE memories (
    id TEXT PRIMARY KEY,
    namespace TEXT NOT NULL,         -- user/project/global
    key TEXT NOT NULL,
    value TEXT NOT NULL,             -- JSON document
    embedding BLOB,                  -- 向量 (可选，也可用 LanceDB)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(namespace, key)
);

-- 任务表
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    parent_task_id TEXT,            -- 支持子任务
    title TEXT NOT NULL,
    status TEXT DEFAULT 'pending',
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 路由配置表
CREATE TABLE router_configs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider TEXT NOT NULL,
    api_key_encrypted TEXT,
    model TEXT,
    base_url TEXT,
    config_json TEXT,               -- JSON 扩展配置
    is_active BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 设置表
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Schema 版本
CREATE TABLE _schema_version (
    version INTEGER PRIMARY KEY
);
```

### 9.3 向量存储：LanceDB

```rust
// 在 Rust 后端集成 LanceDB 进行语义记忆检索
use lancedb::connect;

async fn search_memories(query_embedding: Vec<f32>, limit: usize) -> Vec<Memory> {
    let db = connect("~/.claw/lance_db").execute().await.unwrap();
    let table = db.open_table("memories").execute().await.unwrap();
    let results = table
        .vector_search(query_embedding)
        .limit(limit)
        .execute()
        .await
        .unwrap();
    // ... 转换结果
}
```

---

## 10. 综合技术栈推荐

### 10.1 推荐技术栈总览

```
┌──────────────────────────────────────────────────────────┐
│                    Claw Desktop App                       │
├──────────────────────────────────────────────────────────┤
│  Frontend (WebView)                                       │
│  ├── React 19 + TypeScript 5.x                           │
│  ├── Tailwind CSS v4 + shadcn/ui                         │
│  ├── Zustand (状态管理)                                    │
│  ├── TanStack Query (服务器状态)                            │
│  ├── Vercel AI SDK (流式 AI UI)                            │
│  ├── Monaco Editor (代码编辑/预览)                          │
│  ├── react-i18next (国际化)                                │
│  └── Vite (构建工具)                                       │
├──────────────────────────────────────────────────────────┤
│  Tauri 2.x Core (Rust)                                    │
│  ├── rmcp (MCP Client, 官方 Rust SDK)                     │
│  ├── rusqlite (SQLite 数据库)                              │
│  ├── lancedb (向量存储/语义记忆)                             │
│  ├── rig-core (LLM API 直调, 多提供商)                      │
│  ├── wasmtime (WASM 沙箱)                                 │
│  ├── serde / serde_json (序列化)                           │
│  ├── tokio (异步运行时)                                     │
│  ├── tauri-plugin-shell (Sidecar 管理)                     │
│  ├── tauri-plugin-fs (文件系统)                             │
│  ├── tauri-plugin-http (HTTP 客户端)                        │
│  └── tauri-plugin-notification (系统通知)                    │
├──────────────────────────────────────────────────────────┤
│  Agent Backend (Python Sidecar, 可选)                      │
│  ├── LangGraph 1.1.x (Agent 编排)                         │
│  ├── FastAPI (HTTP API)                                   │
│  ├── langchain-anthropic / langchain-openai (LLM)         │
│  ├── langgraph-checkpoint-sqlite (持久化)                   │
│  └── PyInstaller / Nuitka (打包为可执行文件)                  │
└──────────────────────────────────────────────────────────┘
```

### 10.2 分层责任划分

| 层 | 技术 | 职责 |
|----|------|------|
| **UI 展示层** | React + shadcn/ui + Tailwind | 页面渲染、交互、动画 |
| **UI 状态层** | Zustand + TanStack Query | 客户端状态、服务器缓存 |
| **AI UI 层** | Vercel AI SDK | 流式消息渲染、Tool Call 展示 |
| **IPC 层** | Tauri invoke + Events | 前后端通信 |
| **系统集成层** | Tauri Rust Core | 文件系统、进程管理、系统通知 |
| **MCP 层** | rmcp (Rust) | MCP Client、Server 管理 |
| **数据层** | rusqlite + LanceDB | 结构化存储 + 向量搜索 |
| **沙箱层** | Wasmtime (Rust) | 安全代码执行 |
| **Agent 编排层** | LangGraph (Python Sidecar) 或 Rig (Rust) | 工作流编排、记忆、SubAgent |
| **LLM 层** | Rig (Rust) / LangChain (Python) | 多模型 API 调用 |

### 10.3 各技术选择理由

| 技术 | 选择理由 |
|------|---------|
| **Tauri 2.x** | 包体积小 96%，内存少 75%，启动快 3.7x，安全模型优于 Electron |
| **React 19** | 最大生态、最多参考实现、shadcn/ui 生态完善 |
| **shadcn/ui** | 2026 年 React UI 标准、0KB 运行时、完全可控、MD3 风格 |
| **Tailwind CSS v4** | 原子化 CSS、极小产物、设计系统一致性 |
| **Zustand** | 1.1KB、最简 API、Tauri 多窗口状态同步已有成熟方案 |
| **Vercel AI SDK** | React 原生 AI UI 组件、流式渲染、MCP 原生支持 |
| **rmcp** | MCP 官方 Rust SDK、Tier 2、tokio async、与 Tauri 无缝集成 |
| **rusqlite** | Rust 原生 SQLite、WAL 模式、零额外依赖 |
| **LanceDB** | 嵌入式向量搜索、Rust 原生、本地文件存储、Apache 2.0 |
| **Rig** | 20+ LLM 提供商统一接口、Rust 原生、6.7K stars |
| **Wasmtime** | WASM 沙箱标准运行时、<13ms 启动、资源限制完善 |
| **LangGraph** | 最成熟 Agent 编排框架、完整记忆系统、SubAgent 子图 |
| **Vite** | 最快的前端构建工具、Tauri 官方推荐 |

---

## 11. 架构设计方案

### 11.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                         Claw Desktop App                         │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                    WebView (React UI)                        │ │
│  │                                                              │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐       │ │
│  │  │ Chat Page │ │Dashboard │ │Extensions│ │ Settings │       │ │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘       │ │
│  │       │             │             │             │             │ │
│  │  ┌────┴─────────────┴─────────────┴─────────────┴────┐       │ │
│  │  │          State Layer (Zustand + TanStack Query)    │       │ │
│  │  └──────────────────────┬────────────────────────────┘       │ │
│  │                         │ Tauri invoke() / listen()           │ │
│  └─────────────────────────┼───────────────────────────────────┘ │
│                            │                                      │
│  ┌─────────────────────────┼───────────────────────────────────┐ │
│  │              Tauri Rust Core (主进程)                         │ │
│  │                         │                                     │ │
│  │  ┌──────────────────────┴─────────────────────────────┐      │ │
│  │  │                  Command Router                     │      │ │
│  │  └──┬──────┬──────┬──────┬──────┬──────┬──────────────┘      │ │
│  │     │      │      │      │      │      │                      │ │
│  │  ┌──┴──┐┌──┴──┐┌──┴──┐┌──┴──┐┌──┴──┐┌──┴──┐                 │ │
│  │  │ MCP ││ DB  ││ FS  ││Sand-││Rig  ││Side-│                  │ │
│  │  │Mgr  ││Layer││Layer││ box ││(LLM)││car  │                  │ │
│  │  │rmcp ││sqlite││     ││wasm ││     ││Mgr  │                  │ │
│  │  └──┬──┘└──┬──┘└─────┘└──┬──┘└──┬──┘└──┬──┘                  │ │
│  │     │      │              │      │      │                      │ │
│  │  ┌──┴──┐┌──┴──┐      ┌───┴──┐   │  ┌───┴─────────────┐       │ │
│  │  │MCP  ││Lance│      │Wasm- │   │  │ Python Sidecar   │       │ │
│  │  │Srvrs││DB   │      │time  │   │  │ ┌─────────────┐  │       │ │
│  │  │(子进 ││     │      │      │   │  │ │ LangGraph   │  │       │ │
│  │  │ 程)  ││     │      │      │   │  │ │ Agent Engine│  │       │ │
│  │  └─────┘└─────┘      └──────┘   │  │ └─────────────┘  │       │ │
│  │                                   │  │ ┌─────────────┐  │       │ │
│  │                                   └──┤ │ FastAPI     │  │       │ │
│  │                                      │ │ HTTP Server │  │       │ │
│  │                                      │ └─────────────┘  │       │ │
│  │                                      └──────────────────┘       │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 11.2 数据流设计

#### 对话流程

```
用户输入 → React UI
    → Zustand (设置 loading 状态)
    → Tauri invoke("send_message", { session_id, content })
    → Rust Core
        → 判断使用 Rig (Rust 直调) 还是 Sidecar (LangGraph)
        ┌─ Rig 路径 ──────────────────────┐
        │  → Rig Client → LLM API         │
        │  → 流式 Token → Tauri Event 推送  │
        └──────────────────────────────────┘
        ┌─ Sidecar 路径 ──────────────────┐
        │  → HTTP POST → FastAPI           │
        │  → LangGraph 编排               │
        │  → SSE/WebSocket 流式返回        │
        │  → Rust 转发 → Tauri Event 推送   │
        └──────────────────────────────────┘
    → React listen("stream_token")
    → Vercel AI SDK 流式渲染
    → 消息持久化 (rusqlite)
```

#### MCP 工具调用流程

```
LLM 返回 tool_use → Rust Core
    → MCP Manager 查找对应 Server
    → rmcp Client 发送 tools/call 请求
    → MCP Server 执行工具
    → 结果返回 → 注入对话上下文
    → 继续 LLM 推理
```

### 11.3 模块划分

```
claw/
├── src-tauri/                     # Rust 后端
│   ├── src/
│   │   ├── main.rs                # 入口
│   │   ├── commands/              # Tauri Commands
│   │   │   ├── chat.rs            # 对话相关命令
│   │   │   ├── session.rs         # 会话管理
│   │   │   ├── mcp.rs             # MCP 管理
│   │   │   ├── settings.rs        # 设置
│   │   │   └── file.rs            # 文件操作
│   │   ├── services/              # 业务逻辑
│   │   │   ├── llm/               # LLM 调用 (via Rig)
│   │   │   ├── mcp/               # MCP Client (via rmcp)
│   │   │   ├── memory/            # 记忆系统
│   │   │   ├── sandbox/           # WASM 沙箱
│   │   │   ├── skills/            # Skills 管理
│   │   │   └── sidecar/           # Python Sidecar 管理
│   │   ├── db/                    # 数据库层
│   │   │   ├── mod.rs
│   │   │   ├── migrations.rs
│   │   │   └── models.rs
│   │   ├── state.rs               # 应用状态
│   │   └── config.rs              # 配置管理
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                            # React 前端
│   ├── components/
│   │   ├── chat/                  # 对话组件
│   │   │   ├── ChatView.tsx
│   │   │   ├── MessageList.tsx
│   │   │   ├── MessageItem.tsx
│   │   │   ├── MessageInput.tsx
│   │   │   ├── StreamingMessage.tsx
│   │   │   ├── CodeBlock.tsx
│   │   │   └── ToolCallBlock.tsx
│   │   ├── layout/                # 布局组件
│   │   │   ├── AppShell.tsx
│   │   │   ├── NavRail.tsx
│   │   │   └── RightPanel.tsx
│   │   ├── settings/              # 设置组件
│   │   ├── extensions/            # 扩展/Skills 组件
│   │   ├── dashboard/             # 仪表盘组件
│   │   └── ui/                    # shadcn/ui 基础组件
│   ├── stores/                    # Zustand stores
│   ├── hooks/                     # 自定义 hooks
│   ├── lib/                       # 工具函数
│   ├── i18n/                      # 国际化
│   └── App.tsx
├── agent/                          # Python Sidecar (可选)
│   ├── agent/
│   │   ├── __init__.py
│   │   ├── graph.py               # LangGraph 图定义
│   │   ├── nodes.py               # 节点实现
│   │   ├── tools.py               # 工具定义
│   │   ├── memory.py              # 记忆管理
│   │   └── state.py               # Agent 状态
│   ├── server.py                  # FastAPI 入口
│   ├── requirements.txt
│   └── build.py                   # PyInstaller 构建脚本
├── package.json
├── vite.config.ts
├── tailwind.config.ts
└── tsconfig.json
```

---

## 12. 风险评估与应对策略

### 12.1 技术风险

| 风险 | 影响 | 概率 | 应对策略 |
|------|------|------|---------|
| **Python Sidecar 打包体积大** | 安装包 +50-80MB | 高 | 使用 Nuitka 代替 PyInstaller；或选择纯 Rust/TS 方案 |
| **跨平台 WebView 差异** | UI 在不同 OS 上表现不一致 | 中 | 充分测试 WebView2 (Win) / WebKit (Mac) / WebKitGTK (Linux) |
| **rmcp 成熟度** | MCP Rust SDK 是 Tier 2，可能有 bug | 中 | 保持跟进官方更新；必要时降级到 TypeScript MCP SDK |
| **Rust 学习曲线** | 团队 Rust 经验不足 | 中 | Rust 核心层保持精简，复杂逻辑放在前端或 Python Sidecar |
| **WASM 沙箱兼容性** | 某些代码无法在 WASM 中运行 | 中 | 提供 Docker 容器作为后备方案 |
| **LangGraph 版本更新** | API 可能有 breaking changes | 低 | 固定版本号，增量升级 |

### 12.2 无 Python Sidecar 的纯 Rust+TS 降级方案

如果最终决定不使用 Python Sidecar，可以采用以下降级方案：

```
Frontend (React + TypeScript)
├── Vercel AI SDK / Mastra (Agent 编排)
├── @modelcontextprotocol/client (MCP)
└── 流式 UI

Tauri Rust Core
├── Rig (多 LLM 调用)
├── rmcp (MCP Client 备选)
├── rusqlite + LanceDB (存储)
└── Wasmtime (沙箱)
```

**此方案的取舍：**
- ✅ 包体积最小 (~10-15MB)
- ✅ 统一技术栈，无需管理多进程
- ❌ Agent 编排能力受限（TS 生态不如 Python 成熟）
- ❌ 记忆系统需要更多手动实现

---

## 13. 结论与最终推荐

### 13.1 推荐技术栈总结

| 层 | 推荐技术 | 备选 |
|----|---------|------|
| **桌面框架** | Tauri 2.x | - |
| **前端框架** | React 19 + TypeScript | Svelte 5 |
| **UI 组件库** | shadcn/ui + Tailwind CSS v4 | Nocta UI |
| **状态管理** | Zustand + TanStack Query | Jotai |
| **AI UI** | Vercel AI SDK | 手动 SSE 处理 |
| **构建工具** | Vite | - |
| **MCP** | rmcp (Rust, 官方 SDK) | TS SDK |
| **数据库** | rusqlite (SQLite, WAL) | tauri-plugin-sql |
| **向量存储** | LanceDB (嵌入式) | Qdrant |
| **LLM 调用** | Rig (Rust, 多提供商) | 直接 HTTP 调用 |
| **Agent 编排** | LangGraph (Python Sidecar) | Mastra (TS) / Rig (Rust) |
| **沙箱** | Wasmtime (WASM) | Docker (可选) |
| **国际化** | react-i18next | - |
| **代码编辑器** | Monaco Editor | CodeMirror |

### 13.2 综合评分

| 评估维度 | 推荐方案评分 (1-10) | 说明 |
|---------|-------------------|------|
| **性能** | 9.5 | Tauri + Rust 后端 + React 前端，全方位性能优势 |
| **UI 美观度** | 9.0 | shadcn/ui + Tailwind CSS v4，现代、优雅、高度可定制 |
| **社区活跃度** | 9.0 | React/Tauri/LangGraph 三个核心框架均有活跃社区 |
| **稳定性** | 8.5 | Tauri 2.x 生产就绪，LangGraph 1.1 稳定；rmcp 稍年轻 |
| **易用性** | 7.5 | Rust 学习曲线是主要门槛，但前端部分对 React 开发者友好 |
| **功能覆盖率** | 9.5 | 完整覆盖 Misaka 所有功能并大幅扩展 |
| **Agent 框架完整性** | 9.0 | LangGraph 提供最完整的编排/记忆/子图能力 |
| **包体积** | 8.0 | Tauri 本体 ~10MB + Python Sidecar 后约 60-80MB |
| **跨平台** | 9.0 | Windows/macOS/Linux 全支持 |
| **可维护性** | 8.5 | 清晰的分层架构，关注点分离 |
| **综合评分** | **8.8 / 10** | |

### 13.3 开发路线图建议

| 阶段 | 时间 | 目标 |
|------|------|------|
| **Phase 1: 基础框架** | 4-6 周 | Tauri 项目搭建、React UI 骨架、基础对话 (Rig 直调)、SQLite |
| **Phase 2: 核心功能** | 6-8 周 | 流式对话、会话管理、MCP (rmcp)、设置系统、主题/i18n |
| **Phase 3: Agent 增强** | 6-8 周 | LangGraph Sidecar 集成、记忆系统、SubAgent |
| **Phase 4: 高级功能** | 4-6 周 | WASM 沙箱、Skills 系统、Dashboard、文件浏览 |
| **Phase 5: 打磨发布** | 4-6 周 | 性能优化、跨平台测试、自动更新、打包发布 |

**总预估周期：24-34 周（6-8 个月）**

---

## 14. Buddy 桌面伴侣系统深度调研

### 14.1 功能目标

在原始 Misaka TODO 中，Buddy 系统仅定义为「16 种物种、稀有度、概率分布、独特视觉身份」的静态头像。Claw 的 Buddy 系统目标远超于此：

| 功能 | 描述 | 优先级 |
|------|------|--------|
| **桌面常驻** | Buddy 作为独立透明窗口常驻在 PC 桌面上，始终置顶 | P0 |
| **动画表现** | Buddy 能播放待机、行走、说话、情绪等动画，不是静态图片 | P0 |
| **可交互** | 点击、拖拽、悬停等交互方式，Buddy 有反馈 | P0 |
| **语音输入** | 用户通过语音与 Buddy 对话，Buddy 识别语音转文字 | P0 |
| **AI 对话** | Buddy 调用大模型进行对话、回答问题、执行任务 | P0 |
| **语音输出** | Buddy 通过 TTS 语音合成朗读 AI 回复 | P0 |
| **情绪系统** | AI 返回的情绪标签驱动 Buddy 的表情/动作切换 | P1 |
| **任务执行** | 通过语音指令让 Buddy 执行 Agent 工具（文件操作、搜索等） | P1 |
| **屏幕感知** | Buddy 能截屏理解用户当前正在做什么（可选，隐私敏感） | P2 |
| **多物种/自定义** | 支持多种 Buddy 形象，用户可自定义/导入 | P1 |
| **人设系统** | Buddy 有可配置的人格、说话风格、角色设定 | P1 |

### 14.2 Tauri 桌面宠物可行性分析

#### 结论：✅ 完全可行，已有成熟的 Tauri 桌面宠物项目验证

##### 已验证项目

| 项目 | 技术栈 | Stars | 特性 | 相关性 |
|------|--------|-------|------|--------|
| **[WindowPet](https://github.com/SeakMengs/WindowPet)** | Tauri + React + Zustand | 573 ⭐ | 45+ 宠物、透明窗口、拖拽、点击穿透、自启动 | 🟢 直接参考 |
| **[Koi Pond](https://crabnebula.dev/blog/building-a-desktop-pet-with-tauri/)** | Tauri + SolidJS | CrabNebula 官方 | 透明背景、点击穿透、动画渲染、跟随点击 | 🟢 官方教程 |
| **[desktop-homunculus](https://github.com/not-elm/desktop_homunculus)** | Rust + Bevy | 69 ⭐ | 3D VRM 角色、AI 集成、MCP Server、MOD 系统 | 🟡 架构参考 |
| **[Live2DPet](https://github.com/x380kkm/Live2DPet)** | Electron + Live2D | 51 ⭐ | Live2D 角色、AI 视觉感知、VOICEVOX 语音、情绪系统 | 🟡 功能参考 |
| **[LLM-Live2D-Desktop-Assistant](https://github.com/ylxmf2005/LLM-Live2D-Desktop-Assitant)** | Electron + Python + Live2D | 160 ⭐ | 语音唤醒、屏幕感知、剪贴板、电脑控制 | 🟡 功能参考 |

##### Tauri 透明窗口配置

```json
{
  "app": {
    "macOSPrivateApi": true,
    "windows": [
      {
        "label": "buddy",
        "title": "Claw Buddy",
        "width": 300,
        "height": 400,
        "alwaysOnTop": true,
        "decorations": false,
        "transparent": true,
        "skipTaskbar": true,
        "shadow": false,
        "resizable": false,
        "visibleOnAllWorkspaces": true
      }
    ]
  }
}
```

##### 关键 Tauri API

| 功能需求 | Tauri API | 说明 |
|---------|-----------|------|
| 透明背景 | `transparent: true` + CSS `background: transparent` | 窗口背景透明 |
| 始终置顶 | `alwaysOnTop: true` | 悬浮于所有窗口之上 |
| 隐藏标题栏 | `decorations: false` | 无系统窗口边框 |
| 隐藏任务栏 | `skipTaskbar: true` | 不出现在任务栏 |
| 拖拽移动 | `data-tauri-drag-region` + `core:window:allow-start-dragging` | 拖拽 Buddy 移动位置 |
| 点击穿透 | `window.setIgnoreCursorEvents(true/false)` | 透明区域不拦截鼠标 |
| 多窗口 | Tauri 多窗口 API | 主窗口 + Buddy 窗口独立 |
| 跨显示器 | `visibleOnAllWorkspaces: true` | macOS 全工作区可见 |

##### 点击穿透的智能处理

WindowPet 和 Koi Pond 的实现方式值得参考：

```rust
// Rust 端：默认点击穿透（透明区域不拦截）
window.set_ignore_cursor_events(true)?;
```

```typescript
// 前端：监听鼠标进入 Buddy 区域时临时关闭穿透
buddyElement.addEventListener('mouseenter', async () => {
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  await getCurrentWindow().setIgnoreCursorEvents(false);
});

buddyElement.addEventListener('mouseleave', async () => {
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  await getCurrentWindow().setIgnoreCursorEvents(true);
});
```

### 14.3 角色动画方案选型

#### 方案对比

| 方案 | 技术 | 动画质量 | 性能 | 自定义度 | 开发复杂度 | 推荐度 |
|------|------|---------|------|---------|-----------|--------|
| **Lottie 动画** | lottie-react | ⭐⭐⭐ | 极高 | 中 (需设计师) | 低 | ⭐⭐⭐⭐ |
| **Live2D** | pixi-live2d-display + PixiJS | ⭐⭐⭐⭐⭐ | 高 | 极高 | 高 | ⭐⭐⭐⭐⭐ |
| **Sprite Sheet** | CSS/Canvas 帧动画 | ⭐⭐ | 极高 | 低 | 低 | ⭐⭐⭐ |
| **3D VRM** | Three.js / Bevy | ⭐⭐⭐⭐ | 中 | 极高 | 极高 | ⭐⭐⭐ |
| **Rive** | @rive-app/react-canvas | ⭐⭐⭐⭐ | 极高 | 高 | 中 | ⭐⭐⭐⭐ |

#### 🏆 推荐方案：Lottie (默认) + Live2D (高级模式)

**Lottie 作为默认方案：**
- `lottie-react` 库仅 <50KB，支持交互式动画
- LottieFiles 平台有大量免费角色动画资源
- 支持 `useLottieInteractivity` 实现鼠标交互
- 状态机控制不同动画段（待机、说话、高兴、思考等）
- 无需 WebGL，CPU 友好

```typescript
import { useLottie, useLottieInteractivity } from 'lottie-react';

type BuddyState = 'idle' | 'talking' | 'thinking' | 'happy' | 'sad';

const FRAME_MAP: Record<BuddyState, [number, number]> = {
  idle:     [0, 60],
  talking:  [61, 120],
  thinking: [121, 180],
  happy:    [181, 240],
  sad:      [241, 300],
};

function BuddyAvatar({ state }: { state: BuddyState }) {
  const [start, end] = FRAME_MAP[state];
  const lottieObj = useLottie({
    animationData: buddyAnimation,
    loop: true,
    initialSegment: [start, end],
  });
  return lottieObj.View;
}
```

**Live2D 作为高级模式（可选）：**
- 适用于需要高质量 2D 角色表现的场景
- `pixi-live2d-display` + PixiJS 渲染
- 支持眼球追踪鼠标、口型同步、表情/动作切换
- 需要 Live2D 模型文件（.moc3 格式）
- Cubism SDK 有免费/商用许可证区分

```typescript
import { Live2DModel } from 'pixi-live2d-display';
import * as PIXI from 'pixi.js';

const app = new PIXI.Application({
  view: canvasRef.current,
  transparent: true,
  autoStart: true,
});

const model = await Live2DModel.from('buddy/model.model3.json');
model.anchor.set(0.5, 0.5);
model.scale.set(0.3);
app.stage.addChild(model);

// 眼球追踪鼠标
model.on('hit', (hitAreas) => {
  if (hitAreas.includes('Body')) model.motion('TapBody');
});
```

### 14.4 语音系统实现方案

#### 14.4.1 语音输入 (STT - Speech to Text)

| 方案 | 技术 | 平台兼容 | 离线 | 准确度 | 延迟 | 推荐度 |
|------|------|---------|------|--------|------|--------|
| **tauri-plugin-stt** | Vosk (离线模型) | ✅ 全平台 | ✅ | 中 | 低 | ⭐⭐⭐⭐ |
| **whisper-cpp-plus** | Whisper.cpp (Rust) | ✅ 全平台 | ✅ | 高 | 中 (~200ms) | ⭐⭐⭐⭐⭐ |
| **Web Speech API** | 浏览器原生 | ❌ Linux 不支持 | ❌ | 高 | 极低 | ⭐⭐⭐ |
| **vox** | Whisper + Silero VAD | ✅ 全平台 | ✅ | 高 | 低 (~250ms) | ⭐⭐⭐⭐ |

##### 🏆 推荐：whisper-cpp-plus (Rust) + tauri-plugin-stt (备选)

**whisper-cpp-plus 集成到 Tauri Rust 后端：**

```toml
[dependencies]
whisper-cpp-plus = { version = "0.1.4", features = ["async"] }
cpal = "0.17"     # 麦克风音频采集
```

```rust
use whisper_cpp_plus::{WhisperContext, WhisperStreamPcm, WhisperStreamPcmConfig};

pub struct VoiceRecognizer {
    ctx: WhisperContext,
}

impl VoiceRecognizer {
    pub fn new(model_path: &str) -> Result<Self> {
        let ctx = WhisperContext::new(model_path)?;
        Ok(Self { ctx })
    }

    pub async fn transcribe_stream(&self, audio: &[f32]) -> Result<String> {
        let text = self.ctx.transcribe(audio)?;
        Ok(text)
    }
}
```

**模型选择：**

| 模型 | 大小 | 速度 | 准确度 | 适用场景 |
|------|------|------|--------|---------|
| `ggml-tiny.en` | 75 MB | 极快 | 中 | 英语快速识别 |
| `ggml-base.en` | 142 MB | 快 | 中高 | 英语日常使用 |
| `ggml-small` | 466 MB | 中 | 高 | 多语言 |
| `ggml-medium` | 1.5 GB | 较慢 | 很高 | 高质量需求 |

推荐默认使用 `ggml-base.en`（142MB），中文需求使用 `ggml-small`（466MB）。

#### 14.4.2 语音输出 (TTS - Text to Speech)

| 方案 | 技术 | 平台兼容 | 离线 | 语音质量 | 延迟 | 推荐度 |
|------|------|---------|------|---------|------|--------|
| **piper-rs** | Piper TTS (Rust) | ✅ 全平台 | ✅ | 高 | ~200ms | ⭐⭐⭐⭐⭐ |
| **@realtimex/piper-tts-web** | Piper WASM (浏览器) | ✅ 全平台 | ✅ | 高 | 中 | ⭐⭐⭐⭐ |
| **Web SpeechSynthesis** | 浏览器原生 | ⚠️ Linux 受限 | ✅ | 中 | 极低 | ⭐⭐⭐ |
| **edge-tts** | 微软 Edge 云端 | ✅ 全平台 | ❌ | 很高 | 网络依赖 | ⭐⭐⭐ |
| **Kokoro** | vox 集成 | ✅ 全平台 | ✅ | 极高 (50+声音) | ~300ms | ⭐⭐⭐⭐ |

##### 🏆 推荐：piper-rs (Rust, 离线) + edge-tts (在线备选)

**piper-rs 集成：**

```toml
[dependencies]
piper-rs = "0.1.9"
rodio = "0.19"    # 音频播放
```

```rust
use piper_rs::PiperModel;

pub struct VoiceSynthesizer {
    model: PiperModel,
}

impl VoiceSynthesizer {
    pub fn new(model_path: &str, config_path: &str) -> Result<Self> {
        let model = PiperModel::new(model_path, config_path)?;
        Ok(Self { model })
    }

    pub fn speak(&self, text: &str) -> Result<Vec<f32>> {
        let audio = self.model.synthesize(text)?;
        Ok(audio)
    }
}
```

**Piper 模型（多语言支持）：**

| 语言 | 模型 | 大小 | 声音数 |
|------|------|------|--------|
| English (US) | en_US-lessac-medium | 63 MB | 1 |
| Chinese | zh_CN-huayan-medium | 63 MB | 1 |
| Japanese | ja_JP-kokoro-medium | 63 MB | 1 |
| 更多 | huggingface.co/rhasspy/piper-voices | - | 900+ |

#### 14.4.3 语音唤醒 (可选)

| 方案 | 技术 | 说明 |
|------|------|------|
| **Silero VAD** | whisper-cpp-plus 内建 | 检测语音活动，自动开始识别 |
| **自定义唤醒词** | Porcupine / Snowboy | 固定唤醒词（如"Hey Claw"） |
| **按键触发** | 全局快捷键 | 最简单可靠的方案 |

推荐初期使用 **全局快捷键** 触发语音输入，后续迭代加入 **Silero VAD** 自动检测。

### 14.5 AI 对话与任务执行集成

#### Buddy 与 Agent 系统的集成架构

```
┌──────────────────────────────────────────────────────────┐
│                  Buddy Window (透明浮窗)                    │
│                                                            │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              Buddy 角色动画 (Lottie/Live2D)            │  │
│  │              ↕ 情绪状态驱动动画切换                      │  │
│  └─────────────────────────────────────────────────────┘  │
│  ┌──────────────┐  ┌───────────────────────────────────┐  │
│  │ 🎤 语音输入   │  │ 💬 对话气泡 / 迷你聊天框             │  │
│  │ (按键/VAD)    │  │ (Buddy 回复文字 + 语音播放)          │  │
│  └──────┬───────┘  └───────────────┬───────────────────┘  │
│         │                           │                      │
│         │    Tauri Event / IPC      │                      │
└─────────┼───────────────────────────┼──────────────────────┘
          │                           │
┌─────────┼───────────────────────────┼──────────────────────┐
│  Tauri Rust Core                    │                       │
│         │                           │                       │
│  ┌──────┴───────┐            ┌──────┴───────┐              │
│  │ STT Engine   │            │ TTS Engine   │              │
│  │ whisper.cpp  │            │ piper-rs     │              │
│  └──────┬───────┘            └──────┬───────┘              │
│         │ 转录文本                    │ 音频数据              │
│         ↓                           ↑                       │
│  ┌──────────────────────────────────────────────────┐      │
│  │              Agent Engine                        │      │
│  │  ├── Rig (Rust) — 简单直调 LLM                    │      │
│  │  └── LangGraph (Python Sidecar) — 复杂编排        │      │
│  │       ├── Tool 调用 (MCP / 内建)                   │      │
│  │       ├── 记忆检索                                 │      │
│  │       └── SubAgent 委托                           │      │
│  └──────────────────────────────────────────────────┘      │
└────────────────────────────────────────────────────────────┘
```

#### 语音→AI→语音 完整流程

```
1. 用户按住快捷键 / 点击麦克风按钮
2. cpal 采集麦克风音频 (16kHz mono f32)
3. whisper-cpp-plus 实时转录 → 文本
4. Buddy 动画切换到 "listening" 状态
5. 转录文本发送到 Agent Engine
6. Agent 调用 LLM (Rig / LangGraph)
   ├── 可能触发工具调用 (MCP tools)
   ├── 可能检索记忆 (LanceDB)
   └── 返回结构化响应: { text, emotion, actions[] }
7. Buddy 动画切换到 emotion 对应状态 (happy/thinking/etc)
8. piper-rs 将回复文本合成语音
9. rodio 播放语音 + Buddy 动画切换到 "talking"
10. 播放完毕 → Buddy 回到 "idle" 状态
```

#### 情绪驱动系统

通过在 LLM System Prompt 中注入情绪提取指令：

```
你是 Buddy，一个桌面伴侣。回复用户时，请以 JSON 格式返回：
{
  "text": "你的回复内容",
  "emotion": "happy|sad|thinking|excited|neutral|surprised",
  "actions": ["optional_tool_calls"]
}
```

前端根据 `emotion` 字段切换 Buddy 动画状态。

### 14.6 Buddy 系统所需的额外依赖

#### Rust 端 (Cargo.toml 新增)

```toml
# 语音识别
whisper-cpp-plus = { version = "0.1.4", features = ["async"] }

# 语音合成
piper-rs = "0.1.9"

# 音频 I/O
cpal = "0.17"       # 麦克风采集
rodio = "0.19"      # 音频播放
hound = "3.5"       # WAV 处理
```

#### 前端 (package.json 新增)

```json
{
  "dependencies": {
    "lottie-react": "^2.4.0",
    "pixi-live2d-display": "^0.4.0",
    "pixi.js": "^7.0.0"
  }
}
```

#### 模型文件（随应用分发或首次运行下载）

| 模型 | 用途 | 大小 | 分发方式 |
|------|------|------|---------|
| `ggml-base.en.bin` | Whisper STT | 142 MB | 首次运行下载 |
| `en_US-lessac-medium.onnx` | Piper TTS (英语) | 63 MB | 首次运行下载 |
| `zh_CN-huayan-medium.onnx` | Piper TTS (中文) | 63 MB | 首次运行下载 |
| `silero_vad.onnx` | VAD 语音检测 | 2 MB | 随应用打包 |
| Buddy Lottie JSON | 角色动画 | <1 MB | 随应用打包 |

### 14.7 性能与资源评估

| 指标 | 预估值 | 说明 |
|------|--------|------|
| **额外内存 (STT)** | ~150 MB | Whisper base 模型加载 |
| **额外内存 (TTS)** | ~80 MB | Piper 模型加载 |
| **Buddy 窗口内存** | ~20 MB | 透明 WebView + Lottie 动画 |
| **STT 延迟** | ~200-300 ms | 3 秒语音片段转录 |
| **TTS 延迟** | ~200 ms | "Hello world" 合成 |
| **CPU 占用 (空闲)** | <1% | Buddy 待机动画 |
| **CPU 占用 (语音处理)** | 10-30% | STT/TTS 处理期间 |
| **额外磁盘空间** | ~270 MB | STT + TTS 模型文件 |

**优化策略：**
- 模型延迟加载：仅在用户首次使用语音功能时加载 STT/TTS 模型
- GPU 加速：`whisper-cpp-plus` 支持 CUDA (NVIDIA) 和 Metal (Apple) 加速
- 模型量化：使用量化版模型减少内存占用
- VAD 预过滤：通过 Silero VAD 避免处理静音段

### 14.8 Buddy 窗口与主窗口的关系

```
┌────────────────────────────────┐
│         Claw 主窗口             │
│  ┌──────────────────────────┐  │
│  │      Chat / Dashboard    │  │
│  │      Settings / etc.     │  │
│  └──────────────────────────┘  │
└────────────────────────────────┘
                                     ┌──────────┐
                                     │  Buddy   │ ← 独立透明窗口
                                     │  (浮窗)   │    始终置顶
                                     │  🐱       │    可拖拽
                                     └──────────┘

两窗口通过 Tauri Event 系统通信：
- 主窗口 → Buddy：推送 AI 回复、情绪状态、通知
- Buddy → 主窗口：语音输入文本、点击事件、工具执行请求
```

Tauri 多窗口通信：

```typescript
// 主窗口发送消息给 Buddy
import { emit } from '@tauri-apps/api/event';
await emit('buddy:set-emotion', { emotion: 'happy', text: '任务完成！' });

// Buddy 窗口接收
import { listen } from '@tauri-apps/api/event';
await listen('buddy:set-emotion', (event) => {
  setBuddyState(event.payload.emotion);
  showSpeechBubble(event.payload.text);
});
```

### 14.9 Buddy 功能实现路线图

| 阶段 | 时间 | 功能 |
|------|------|------|
| **Phase 1: 基础桌面伴侣** | 2-3 周 | 透明悬浮窗、Lottie 待机/拖拽动画、气泡对话框 |
| **Phase 2: 文字对话** | 2-3 周 | 迷你输入框、LLM 调用、流式回复气泡、情绪动画 |
| **Phase 3: 语音输入** | 2-3 周 | Whisper STT 集成、快捷键触发、实时转录 |
| **Phase 4: 语音输出** | 1-2 周 | Piper TTS 集成、口型同步（Lottie 状态切换） |
| **Phase 5: 高级交互** | 3-4 周 | 工具调用、MCP 集成、人设系统、多 Buddy 形象 |
| **Phase 6: Live2D (可选)** | 3-4 周 | PixiJS + pixi-live2d-display、眼球追踪、表情映射 |

**Buddy 系统总预估：13-19 周（3-5 个月），可与主应用并行开发**

### 14.10 Buddy 功能可行性总结

| 评估维度 | 评分 (1-10) | 说明 |
|---------|------------|------|
| **技术可行性** | 9.5 | Tauri 透明窗口+动画+语音已有完整生态验证 |
| **性能影响** | 8.0 | 语音模型占用 ~230MB 内存，但可延迟加载 |
| **用户体验** | 9.0 | 桌面宠物+语音交互是极具差异化的特色功能 |
| **开发成本** | 7.0 | 涉及 STT/TTS/动画/多窗口等多个技术领域 |
| **与主架构兼容性** | 9.5 | 完美融入 Tauri + React + Rust 技术栈 |

**结论：Buddy 桌面伴侣系统可以完美融入 Claw 的技术架构。核心能力（透明窗口、动画、语音）均有成熟的 Tauri/Rust 生态支持，是 Claw 区别于其他 AI Agent 客户端的重要差异化功能。**

---

> **报告结束**
>
> 本报告基于 2026 年 4 月的技术生态进行深度调研，所有数据和推荐均反映当前最新状态。
> 技术选型应根据团队实际能力和项目进展进行灵活调整。
