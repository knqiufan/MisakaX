# MisakaX 桌面端 — 最终架构选型文档

> **项目代号：** MisakaX（基于 Misaka 项目的下一代桌面端 AI Agent 客户端）
> **调研日期：** 2026-04-28（原始）/ 2026-05-07（v2.2 架构重构）
> **文档版本：** v2.2（架构重构：全路径 DeepAgents，消除简单对话概念，工作目录系统）
> **目标：** 回答关键架构问题，给出最终技术栈决策

---

## 目录

1. [项目背景与需求重申](#1-项目背景与需求重申)
2. [Q1 深度解析：Tauri Sidecar 模式与性能影响](#2-q1-深度解析tauri-sidecar-模式与性能影响)
3. [Q2 开发环境要求](#3-q2-开发环境要求)
4. [Q3 长期记忆系统：PowerMem 深度调研与适配性分析](#4-q3-长期记忆系统powermem-深度调研与适配性分析)
5. [Q4 架构模式深度剖析：Tauri 的角色与最佳性能方案](#5-q4-架构模式深度剖析tauri-的角色与最佳性能方案)
6. [Q5 Skills 系统完整方案](#6-q5-skills-系统完整方案)
7. [前端 UI 框架选型](#7-前端-ui-框架选型)
8. [AI Agent 编排框架调研](#8-ai-agent-编排框架调研)
9. [Rust 生态核心框架](#9-rust-生态核心框架)
10. [核心功能实现方案](#10-核心功能实现方案)
    - 10.1 [MCP 实现](#101-mcp-model-context-protocol-实现)
    - 10.2 [SubAgent 系统](#102-subagent-系统实现)
    - 10.3 [沙箱隔离](#103-沙箱隔离实现)
    - 10.4 [长期记忆](#104-长期记忆实现)
    - 10.5 [Skills 系统](#105-skills-系统实现)
    - 10.6 [知识库系统](#106-知识库-knowledge-base-系统实现) 🆕
11. [数据库与存储引擎深度选型：seekdb vs SQLite](#11-数据库与存储引擎深度选型seekdb-vs-sqlite) 🆕
12. [状态管理与数据存储方案](#12-状态管理与数据存储方案)
13. [Buddy 桌面伴侣系统概要](#13-buddy-桌面伴侣系统概要)
14. [综合技术栈推荐](#14-综合技术栈推荐)
15. [最终架构选择](#15-最终架构选择)
16. [风险评估与应对策略](#16-风险评估与应对策略)
17. [开发路线图](#17-开发路线图)
18. [附录：关键决策记录](#18-附录关键决策记录)

---

## 1. 项目背景与需求重申

### 1.1 项目动机

当前 Misaka 项目基于 Python + Flet 构建，依赖 `claude-agent-sdk`，存在以下局限：

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
| **Agent 编排** | 全路径 DeepAgents harness，支持多步骤工作流、条件分支、并行执行 | P0 |
| **SubAgent** | 支持子代理派生、任务委托、独立上下文 | P0 |
| **工作目录** | 每会话绑定本地+远程工作目录，支持目录选择/切换/浏览 | P0 |
| **沙箱隔离** | 安全执行用户/Agent 生成的代码 | P1 |
| **持久化记忆** | 跨会话记忆检索、向量化存储、认知启发的衰减机制 | P0 |
| **Skills 系统** | 渐进式披露架构、可发现、安装、管理、执行 | P0 |
| **流式对话** | 支持实时 token 流式输出和思维链展示 | P0 |
| **高性能 UI** | Material Design 3 风格，流畅的交互体验 | P0 |
| **跨平台** | Windows / macOS / Linux 全平台支持 | P0 |
| **知识库** | 文档管理、全文检索、语义搜索、RAG 问答、混合检索 | P1 |
| **小体积分发** | 安装包尽可能小 | P1 |

> **v2.2 变更说明：** 移除了"简单对话"概念。所有对话均通过 DeepAgents harness 处理。
> Agent 内部自主决定处理复杂度——简单问题直接 LLM 回复，复杂问题自动启用工具/SubAgent/Skills。
> 新增工作目录作为 P0 需求：每个对话绑定工作目录（本地+远程），DeepAgents FilesystemMiddleware 基于此运行。

---

## 2. Q1 深度解析：Tauri Sidecar 模式与性能影响

### 2.1 什么是 Tauri Sidecar 模式？

**Tauri Sidecar** 是 Tauri 提供的一种机制，允许将**外部可执行文件**（用任何语言编写：Python、Go、Node.js、C# 等）与 Tauri 应用一起打包和分发，并由 Tauri Rust Core 管理其生命周期。

```
Tauri App
├── WebView (前端 UI)
├── Rust Core (主进程，负责系统调用、IPC)
└── Sidecar (外部可执行文件)
    ├── AI Agent 逻辑（如 LangGraph）
    ├── MCP Server 管理
    └── 复杂业务逻辑
```

**配置方式（tauri.conf.json）：**

```json
{
  "bundle": {
    "externalBin": ["binaries/agent-server"]
  }
}
```

**Rust 端启动 Sidecar：**

```rust
use tauri_plugin_shell::ShellExt;
let sidecar = app.shell().sidecar("agent-server").unwrap();
let (mut rx, child) = sidecar.spawn().expect("Failed to spawn sidecar");
```

### 2.2 Sidecar 性能影响分析

#### 核心结论：Tauri Sidecar 的进程 spawn 开销可以忽略不计，真正的问题是 PyInstaller 打包

| 指标 | 测量值 | 说明 |
|------|--------|------|
| **Tauri spawn Sidecar overhead** | <50ms | 纯进程创建开销，可忽略 |
| **直接 python.exe 启动** | <1s | 基准 |
| **PyInstaller `--onedir` 启动** | 1.2-2.5s | 首次解压后缓存，可接受 |
| **PyInstaller `--onefile` 启动** | 6-20s | 每次启动都解压整个 Python 环境到 %TEMP% |

**关键发现（来自 Tauri GitHub Discussion #9226 社区实际反馈）：**

> "Running sidecar (.exe generated from .py) takes much more time than running command with python.exe (using std::process)"
>
> "I fixed mine by rewriting in Go-lang, seems like python binaries really have bad load time."
>
> "My CLI app is fast everywhere else except in Tauri" — 问题不在 Tauri，而在 PyInstaller 的 onefile 提取过程。

#### IPC 通信开销

| 通信模式 | 延迟 | 适用场景 |
|---------|------|---------|
| **Tauri IPC (Rust↔前端)** | 0.12ms | 前端与 Rust 通信 |
| **HTTP localhost (Rust↔Python Sidecar)** | <1ms | Sidecar 常驻 HTTP Server |
| **stdio pipe (Rust↔Python Sidecar)** | <0.5ms | 命令行式 Sidecar |

#### 推荐缓解方案

| 方案 | 效果 |
|------|------|
| **使用 `--onedir` 而非 `--onefile`** | 启动从 6-20s → 1.2-2.5s |
| **长期运行 Server 模式**（FastAPI 常驻进程） | 首次启动成本在整个应用生命周期中摊销 |
| **Nuitka 替代 PyInstaller** | 编译为真正的机器码，启动更快 |
| **排除 UPX 压缩核心 DLL** | 避免解压拖慢启动 |
| **应用启动时立即预启动 Sidecar** | 用户感知不到启动延迟 |

**最终的 Sidecar 通信架构（推荐）：**

```
Tauri Rust Core ──HTTP (localhost:18910)──▶ Python FastAPI Server (长期运行)
    │                                            │
    │                                    ┌───────┴────────┐
    │                                    │  LangGraph Agent │
    │                                    │  PowerMem 记忆   │
    │                                    │  MCP 工具执行    │
    │                                    └────────────────┘
    │
    └── Tauri IPC ──▶ React WebView (UI)
```

---

## 3. Q2 开发环境要求

### 3.1 基础工具链

| 工具 | 版本要求 | 用途 |
|------|---------|------|
| **Rust** | 1.80+ (stable) | Tauri 后端、rmcp、Rig、rusqlite、Wasmtime |
| **Node.js** | 18+ (推荐 20 LTS) | React 前端构建 |
| **Python** | 3.10+ (3.12 推荐) | LangGraph Sidecar、PowerMem |
| **包管理器** | Bun 或 pnpm | 前端依赖管理 |
| **tauri-cli** | 2.x | Tauri 项目脚手架与构建 |

### 3.2 平台特定系统依赖

#### Windows

```
必须安装：Microsoft Visual Studio C++ Build Tools
    → 勾选 "Desktop development with C++"
    → MSVC v143 编译器 + Windows 11 SDK

WebView2：Windows 10 (1809+) / Windows 11 已内置
    → 如需手动安装：https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

#### macOS

```bash
xcode-select --install
```

#### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev

# ⚠️ 注意是 libwebkit2gtk-4.1-dev（不是旧版 4.0）
```

### 3.3 推荐的 Rust 工具链

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 必要组件
rustup component add rustfmt clippy

# 国内镜像加速（可选）
# ~/.cargo/config 配置 rsproxy 或 tuna 镜像
```

### 3.4 Python Sidecar 环境

```bash
# 使用 uv（推荐，速度最快）
pip install uv
uv venv
uv pip install langgraph langchain-anthropic langchain-openai powermem fastapi uvicorn

# 或使用 Poetry / pip
```

### 3.5 验证环境就绪

```bash
cargo tauri doctor    # Tauri 官方诊断命令，一键检查所有依赖
```

### 3.6 推荐 IDE

| IDE | Rust 支持 | 前端支持 | Python 支持 |
|-----|----------|---------|------------|
| **VS Code** + 扩展 | rust-analyzer | ✅ | ✅ |
| **Zed** | ✅ 原生 | ✅ | ✅ |
| **RustRover** | ✅ 最佳 | ⚠️ | ⚠️ |
| **Cursor** | rust-analyzer | ✅ 最佳 | ✅ |

---

## 4. Q3 长期记忆系统：PowerMem 深度调研与适配性分析

### 4.1 PowerMem 项目概述

**PowerMem** 是 OceanBase 开源的 AI 长期记忆基础设施，专为解决 AI 应用的"记忆管理"痛点而设计。

| 属性 | 详情 |
|------|------|
| **GitHub** | https://github.com/oceanbase/powermem |
| **许可证** | Apache 2.0 |
| **安装** | `pip install powermem` |
| **最新版本** | v1.1.0 (2026-04-02) |
| **核心依赖** | seekdb（嵌入式向量数据库）、LLM API、Embedding API |

### 4.2 架构深度分析

```
┌─────────────────────────────────┐
│           接入层                  │
│  Python SDK │ CLI(pmem) │ MCP Server │ HTTP API │ Dashboard │
├─────────────────────────────────┤
│         Memory Engine            │
│  智能提取 → 去重合并 → 冲突更新    │
│  艾宾浩斯遗忘曲线 → 时间衰减加权    │
│  混合检索 → 多路召回融合           │
├─────────────────────────────────┤
│          Model 层                │
│  LLM (事实提取) + Embedding (向量化)│
├─────────────────────────────────┤
│         Storage 层               │
│  seekdb (嵌入式) │ PostgreSQL │ SQLite │
└─────────────────────────────────┘
```

#### 关键特性详解

**1. 混合检索架构（三路召回 + RRF 融合）**

| 检索路径 | 技术 | 适用场景 |
|---------|------|---------|
| **向量检索** | HNSW/IVF 索引 + Dense Vector | 语义相似性 |
| **全文检索** | BM25 + Sparse Vector | 关键词精确匹配 |
| **图检索** | 知识图谱多跳遍历 | 实体关系发现 |

**2. 艾宾浩斯遗忘曲线**

这是 PowerMem 区别于传统向量数据库的**核心差异化能力**：

- 基于认知科学的艾宾浩斯遗忘曲线，对记忆进行时间衰减加权
- 优先返回最新、最相关的记忆
- 自动淘汰过时信息

**3. 智能记忆提取**

- LLM 自动从对话中提取关键事实
- 自动去重：检测重复或高度相似的记忆
- 冲突检测与合并：当新记忆与旧记忆不一致时自动处理
- 关联推理：建立记忆之间的语义关联

**4. 子存储 (Sub Stores)**

- 数据分区管理，按命名空间隔离
- 自动路由查询到正确的子存储
- 支持跨存储查询

### 4.3 性能基准测试

**LOCOMO 基准测试结果：**

| 指标 | PowerMem | 全量上下文 | 提升 |
|------|----------|------------|------|
| 准确率 | 78.70% | 52.9% | **+48.77%** |
| p95 延迟 | 1.44s | 17.12s | **-91.83%** |
| Token 用量 | ~0.9K | ~26K | **-96.53%** |

**OpenMisakaX 集成验证：**

| 指标 | PowerMem 插件 | 默认方案 | 节省 |
|------|-------------|---------|------|
| Token 消耗 | 453 万 | 2461 万 | **~82%** |

### 4.4 与 LangGraph 的集成适配性

**结论：PowerMem 已经为 LangGraph 提供了完整的集成支持。**

#### 已验证的架构模式

PowerMem 与 LangGraph 的集成基于 **StateGraph + TypedDict** 实现有状态工作流：

```
┌──────────────┐     ┌────────────────────┐     ┌──────────────────┐
│ load_context │────▶│ generate_response   │────▶│ save_conversation │
│ 检索相关记忆   │     │ LLM 生成（带上下文）   │     │ 持久化到 PowerMem  │
└──────────────┘     └────────────────────┘     └──────────────────┘
```

**三节点工作流：**

| 节点 | 功能 | 关键操作 |
|------|------|---------|
| **load_context** | 提取用户消息，检索 PowerMem | `powermem.search(user_message, user_id)` |
| **generate_response** | 用检索记忆 + 对话历史构造 prompt，调用 LLM | prompt = context + messages |
| **save_conversation** | 持久化本轮对话到 PowerMem | `powermem.add(conversation, user_id, smart_process=True)` |

#### 集成方式

```python
# 在 LangGraph 中使用 PowerMem
from powermem import Memory, auto_config
from langgraph.graph import StateGraph, MessagesState

# 初始化 PowerMem（嵌入式 seekdb 模式，无需外部数据库）
config = auto_config()
memory = Memory(config=config)

# 在 LangGraph 节点中使用
def load_context(state: MessagesState) -> dict:
    last_msg = state["messages"][-1].content
    results = memory.search(last_msg, user_id=state.get("user_id"))
    return {"context": results}

def save_conversation(state: MessagesState) -> dict:
    memory.add(
        state["messages"],
        user_id=state.get("user_id"),
        smart_process=True  # 启用 LLM 智能提取
    )
    return {}
```

#### 五种接入方式

| 接入方式 | 适用场景 |
|---------|---------|
| **Python SDK** | LangGraph Sidecar 直接集成（推荐） |
| **CLI (`pmem`)** | 脚本化操作、调试 |
| **HTTP API Server** | 微服务架构、跨语言调用 |
| **MCP Server** | 通过 MCP 协议暴露给前端/其他 Agent |
| **Web Dashboard** | 可视化记忆管理 |

### 4.5 与原方案（LanceDB）对比

| 维度 | PowerMem (新推荐) | LanceDB (原方案) |
|------|------------------|-----------------|
| **核心价值** | 完整的记忆管理系统 | 嵌入式向量数据库 |
| **语义理解** | ✅ LLM 自动提取+去重+合并 | ❌ 仅存储，无智能处理 |
| **遗忘机制** | ✅ 艾宾浩斯曲线自动衰减 | ❌ 需手动管理 |
| **混合检索** | ✅ 向量+全文+知识图谱 | ⚠️ 向量+全文（较基础） |
| **Agent 集成** | ✅ LangChain/LangGraph 原生集成 | ⚠️ 需手动集成 |
| **MCP 支持** | ✅ 内建 MCP Server | ❌ 无 |
| **嵌入式部署** | ✅ seekdb（v1.1.0+） | ✅ Rust 原生嵌入式 |
| **Rust 原生** | ❌ Python SDK | ✅ Rust SDK |
| **成熟度** | ⭐⭐⭐⭐ (活跃开发中) | ⭐⭐⭐⭐ |
| **语言绑定** | Python（主要） | Rust/Python/Node.js |

### 4.6 PowerMem 适配结论

**强烈推荐使用 PowerMem 替代原方案中的 LanceDB + LangGraph Store 作为三层记忆架构的核心引擎。**

理由：

1. **语义化记忆管理**：PowerMem 不仅是存储，更是完整的记忆生命周期管理（提取→去重→合并→衰减→检索）
2. **LangGraph 原生集成**：已有开箱即用的 StateGraph 工作流模板
3. **嵌入式部署**：v1.1.0 的 seekdb 模式无需独立数据库服务
4. **Token 节省显著**：相比全量上下文节省 96% Token
5. **MCP Server 内建**：可通过 MCP 协议直接暴露给前端和其他 Agent
6. **Apache 2.0 开源**：商业友好

#### 集成方案

```
┌───────────────────────────────────────────────────────┐
│                MisakaX 三层记忆架构                         │
├───────────────────────────────────────────────────────┤
│  Layer 1: 工作记忆 (Working Memory)                      │
│  ├── 当前对话上下文 + 流式 token                           │
│  ├── 实现：React State + Zustand                         │
│  └── 存储：内存（会话生命周期）                              │
├───────────────────────────────────────────────────────┤
│  Layer 2: 短期记忆 (Short-term Memory)                   │
│  ├── 最近 N 次会话的关键信息                                │
│  ├── 实现：LangGraph Checkpointer                        │
│  └── 存储：SQLite (rusqlite, WAL 模式)                    │
├───────────────────────────────────────────────────────┤
│  Layer 3: 长期记忆 (Long-term Memory) → 🆕 PowerMem      │
│  ├── 用户偏好、项目知识、跨会话历史                           │
│  ├── 实现：PowerMem Python SDK + seekdb                  │
│  ├── 特性：艾宾浩斯遗忘曲线、智能提取、混合检索               │
│  └── 暴露：MCP Server ↔ 前端 / Rust 后端                  │
└───────────────────────────────────────────────────────┘
```

---

## 5. Q4 架构模式深度剖析：全路径 DeepAgents 单一架构

### 5.1 Tauri 的角色定位

**Tauri 不是简单的"粘合剂"**，而是应用架构中的**系统集成层** + **IPC 总线** + **安全边界** + **Sidecar 生命周期管理者**。

```
┌─────────────────────────────────────────────────┐
│                  MisakaX Desktop App                │
│                                                  │
│  ┌────────────────────────────────────────────┐ │
│  │     React 19 WebView (表现层 + UI 状态)      │ │
│  └────────────────┬───────────────────────────┘ │
│                   │ Tauri IPC (0.12ms)           │
│  ┌────────────────┴───────────────────────────┐ │
│  │     Tauri Rust Core (系统集成层 + 非对话业务)   │ │
│  │                                             │ │
│  │  ┌─────────┐ ┌─────────┐ ┌──────────────┐  │ │
│  │  │ MCP Mgr │ │ DB Layer│ │ Rig (非对话)  │  │ │
│  │  │ (rmcp)  │ │(rusqlite)│ │ (嵌入/摘要)   │  │ │
│  │  └─────────┘ └─────────┘ └──────────────┘  │ │
│  │  ┌─────────┐ ┌─────────┐ ┌──────────────┐  │ │
│  │  │Sandbox  │ │Skill Mgr│ │Sidecar Mgr   │  │ │
│  │  │(wasmtime)│ │         │ │(预热管理)     │  │ │
│  │  └─────────┘ └─────────┘ └──────┬───────┘  │ │
│  └─────────────────────────────────┼──────────┘ │
│                                    │             │
│                   HTTP localhost   │             │
│  ┌─────────────────────────────────┼──────────┐ │
│  │     Python Sidecar (Agent 引擎层 — 常驻预热)   │ │
│  │  ┌──────────────────────────────────────┐  │ │
│  │  │ DeepAgents Harness (全对话入口)        │  │ │
│  │  │ ├ TodoListMiddleware (任务规划)       │  │ │
│  │  │ ├ SkillsMiddleware (技能加载)         │  │ │
│  │  │ ├ FilesystemMiddleware (工作目录)     │  │ │
│  │  │ ├ SubAgentMiddleware (子代理派生)     │  │ │
│  │  │ ├ SummarizationMiddleware (上下文摘要) │  │ │
│  │  │ ├ MemoryMiddleware (AGENTS.md 记忆)   │  │ │
│  │  │ └ HumanInTheLoopMiddleware (审批)     │  │ │
│  │  └──────────────────────────────────────┘  │ │
│  │  ┌──────────┐ ┌───────────────┐           │ │
│  │  │ PowerMem │ │ LangGraph      │           │ │
│  │  │ (长期记忆) │ │ Checkpointer  │           │ │
│  │  └──────────┘ └───────────────┘           │ │
│  └─────────────────────────────────┘           │
└─────────────────────────────────────────────────┘
```

**Tauri Rust Core 负责的"非对话"系统级功能：**
- 文件系统访问（tauri-plugin-fs）
- 进程管理（tauri-plugin-shell）+ **Sidecar 预热与生命周期管理**
- 系统通知（tauri-plugin-notification）
- 剪贴板操作（tauri-plugin-clipboard）
- 原生对话框（tauri-plugin-dialog）
- 自动更新（tauri-plugin-updater）
- SQLite 数据库操作（rusqlite）+ sqlite-vec 向量搜索
- MCP Client 管理（rmcp）
- WASM 沙箱执行（Wasmtime）
- Skills 发现与注册（文件系统扫描 + 解析）
- **Rig 非对话调用**：Embedding 生成、会话自动标题、摘要生成
- 工作目录管理（本地路径验证、远程连接状态）

**Python Sidecar 负责的 "Agent 对话"功能（全对话入口）：**
- **所有用户对话**通过 DeepAgents harness 处理
- Agent 内部自主决定处理复杂度（简单问题直接 LLM 回复，复杂问题启用工具链）
- 多步骤 Agent 工作流编排（DeepAgents Middleware 链）
- 长期记忆语义管理（PowerMem，通过自定义 Tool 集成）
- SubAgent 派生与委托（SubAgentMiddleware）
- Human-in-the-loop 审批流程（HumanInTheLoopMiddleware）
- 上下文自动摘要压缩（SummarizationMiddleware）
- 文件系统操作（FilesystemMiddleware，基于会话绑定工作目录）

### 5.2 核心架构决策：全路径 DeepAgents，消除"简单对话"

#### 5.2.1 为什么消除"简单对话"概念

v2.1 架构采用双路径智能路由：简单对话走 Rust Rig 直调，复杂编排走 Python Sidecar。这个设计存在根本性问题：

| 问题 | 说明 |
|------|------|
| **用户体验割裂** | 用户无法预知自己的请求会被哪条路径处理。同一句"帮我看看这个文件"可能在 Rig 路径被拒绝，在 Sidecar 路径才能执行 |
| **能力天花板低** | Rig 路径无法访问 Skills、无法派生子代理、无记忆上下文。用户必须"学会"触发复杂路径才能获得完整能力 |
| **路由误判风险** | 基于启发式规则的复杂度判定必然存在误判。简单请求可能实际需要工具支持，复杂判定可能浪费 Sidecar 资源 |
| **架构复杂性** | 两套对话系统、两套工具调用逻辑、两套上下文管理 — 维护成本翻倍 |
| **不符合 Agent 平台定位** | MisakaX 定位是 Agent 平台而非聊天客户端。即使是"简单"对话，Agent 也应能自主决定是否需要调用工具或查阅记忆 |

#### 5.2.2 新架构：单一路径，Agent 自主决策

```
v2.1 (旧):                     v2.2 (新):

用户消息                        用户消息
  │                               │
  ├─ 路由判断 ─┐                   │
  │            │                   ▼
  ▼            ▼          ┌──────────────┐
Rig 直调    Python       │  DeepAgents   │
(简单)     Sidecar       │   Harness     │
           (复杂)        │               │
                         │ Agent 自主判断 │
                         │ ├ 简单→直接回复 │
                         │ ├ 需要工具→调用 │
                         │ ├ 需要记忆→检索 │
                         │ └ 复杂→派子代理 │
                         └──────────────┘
```

**核心原则：用户打开对话框 → 选择工作目录 → 直接进入 DeepAgents harness。Agent 内部自主决定处理策略。**

这与 Claude Code 的工作模式一致：每次对话都在完整的 harness 环境中，Agent 自行判断当前问题是否需要调用工具、查阅技能、检索记忆。

#### 5.2.3 架构对比：三种方案重新评估

| 方案 | 描述 | 评估 |
|------|------|------|
| **方案 A：双路径路由 (v2.1)** | Rig 简单 + Sidecar 复杂 | ❌ 用户割裂、路由误判、维护复杂 |
| **方案 B：纯 Rust (无 Sidecar)** | 全部 Rust 实现 | ❌ 无法使用 DeepAgents/PowerMem 生态 |
| **🏆 方案 C：全路径 DeepAgents (v2.2)** | 所有对话进入 DeepAgents harness | ✅ Agent 完整能力、单一代码路径、无路由误判 |

### 5.3 Sidecar 预热策略

#### 5.3.1 从"按需唤醒"到"应用启动预热"

v2.1 架构中 Sidecar 采用按需延迟启动策略，这导致**首次对话时有 1-2s 冷启动延迟**。在全路径架构中，每次对话都经过 Sidecar，这个冷启动延迟不可接受。

| 策略 | v2.1 按需唤醒 | v2.2 应用启动预热 |
|------|-------------|----------------|
| **启动时机** | 首次复杂请求时 | 应用主窗口渲染时 |
| **用户感知延迟** | 首次对话 +1-2s | 无感知（后台预热） |
| **资源占用** | 空闲时 ~42MB | 空闲时 ~120-200MB |
| **适用性** | 仅复杂请求走 Sidecar | 所有对话走 Sidecar |

**预热流程：**

```
应用启动
  │
  ├── 1. Rust Core 初始化（DB、配置、MCP Manager）
  ├── 2. 前端 WebView 渲染
  ├── 3. 启动 Python Sidecar（后台异步，不阻塞 UI）
  │      ├── uvicorn 启动 FastAPI
  │      ├── DeepAgents create_deep_agent() 初始化
  │      ├── PowerMem 记忆引擎加载
  │      └── 健康检查端点就绪
  │
  └── 4. 用户打开对话框时 Sidecar 已就绪
```

**Rust 端 Sidecar 管理器关键逻辑：**

```rust
// Sidecar 预热管理器
pub struct SidecarManager {
    child: Option<CommandChild>,
    health_status: Arc<AtomicBool>,
    reconnect_attempts: u32,
}

impl SidecarManager {
    /// 应用启动时调用，预启动 Sidecar
    pub async fn preheat(&mut self, app: &tauri::AppHandle) -> Result<()> {
        let sidecar = app.shell().sidecar("agent-server")?;
        let (rx, child) = sidecar.spawn()?;
        self.child = Some(child);
        
        // 等待健康检查通过（最多 10s）
        self.wait_for_healthy(Duration::from_secs(10)).await?;
        
        // 启动后台健康监控
        self.start_health_monitor(rx);
        Ok(())
    }
    
    /// 健康检查循环
    async fn wait_for_healthy(&self, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        loop {
            if start.elapsed() > timeout {
                return Err(anyhow!("Sidecar startup timeout"));
            }
            if reqwest::get("http://127.0.0.1:18910/health")
                .await?.status().is_success() {
                self.health_status.store(true, Ordering::SeqCst);
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }
}
```

### 5.4 Rig 的角色重定义：纯非对话调用

在全路径 DeepAgents 架构中，Rig 不再处理任何用户对话，其职责限定为：

| 用途 | 调用位置 | 说明 |
|------|---------|------|
| **Embedding 生成** | Rust Core | 知识库文档向量化；支持多提供商 Embedding API |
| **会话自动标题** | Rust Core | 新建会话后自动生成标题（`summarize(messages) → title`） |
| **会话摘要** | Rust Core | 关闭会话时生成摘要，供记忆系统使用 |
| **LLM 配置验证** | Rust Core | 验证用户配置的 API Key 是否有效 |
| **简单补全** | Rust Core | 非对话场景的文本补全（如 Dashboard 建议文本） |

**重要：** 在实现过渡期（Phase 2-3），Rig 暂时承担对话功能作为渐进式实现。Phase 4 后正式迁移到 DeepAgents，Rig 退居非对话角色。

### 5.5 工作目录系统

#### 5.5.1 设计目标

每个对话绑定一个工作目录（本地 + 可选远程路径）。这是 MisakaX 作为工程型 Agent 平台区别于普通聊天客户端的核心特性。

```
┌──────────────────────────────────────────────────────────────┐
│                    Working Directory System                     │
│                                                                │
│  对话创建时 ──▶ 目录选择器 ──▶ 绑定到会话                          │
│                                    │                           │
│                    ┌───────────────┴───────────────┐           │
│                    ▼                               ▼           │
│              Local Path                      Remote Path       │
│         (C:\Projects\my-app\)          (ssh://dev-server/opt/) │
│                    │                               │           │
│                    └───────────────┬───────────────┘           │
│                                    ▼                           │
│                    DeepAgents FilesystemMiddleware              │
│                    (工作目录作为 FilesystemBackend 根路径)        │
│                                    │                           │
│                                    ▼                           │
│                    Agent 可以 ls/read/write/edit               │
│                    读写绑定目录内的文件                          │
└──────────────────────────────────────────────────────────────┘
```

#### 5.5.2 数据结构

```rust
// Rust 端：会话模型
pub struct Session {
    pub id: String,
    pub title: String,
    pub working_dir_local: Option<String>,    // 本地工作目录路径
    pub working_dir_remote: Option<String>,   // 远程工作目录连接字符串
    pub working_dir_remote_type: Option<String>, // "ssh" | "s3" | "ftp"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

```python
# Python 端：DeepAgents Backend 配置
def build_session_backend(session: Session) -> CompositeBackend:
    """根据会话的工作目录配置构建 DeepAgents Backend"""
    routes = {}
    if session.working_dir_local:
        routes["/workspace/"] = FilesystemBackend(
            root_dir=session.working_dir_local
        )
    if session.working_dir_remote:
        routes["/remote/"] = RemoteFilesystemBackend(
            connection_string=session.working_dir_remote,
            remote_type=session.working_dir_remote_type,
        )
    return CompositeBackend(
        default=StateBackend(rt),
        routes=routes,
    )
```

#### 5.5.3 前端工作目录选择 UI

| 状态 | 组件 | 行为 |
|------|------|------|
| **新对话** | DirectorySelector 弹窗 | 默认显示上次使用的目录；支持浏览本地文件系统；支持输入远程连接字符串 |
| **已有对话** | 顶栏路径显示 | 显示当前工作目录路径；点击可切换 |
| **无目录模式** | 可选跳过 | 允许不设置工作目录（降级为纯对话，无文件操作能力） |
| **目录验证** | Rust 端检查 | 创建会话前验证路径存在且可读写（本地）；验证连接可用（远程） |

#### 5.5.4 安全约束

| 约束 | 实现 |
|------|------|
| 本地目录白名单 | 用户可在设置中配置允许访问的目录范围 |
| 远程连接凭证 | 加密存储在 SQLite，通过 Tauri secure store |
| 操作审计 | 文件操作日志记录到 `file_operations` 表 |
| 路径遍历防护 | DeepAgents FilesystemMiddleware 自动防止访问根目录外的路径 |

### 5.6 全路径架构性能分析

**关键认知：** 因为所有对话都经过 Sidecar，预热策略是强制要求。预热后，每次对话的额外 IPC 开销为 1-3ms（HTTP localhost），相对于 LLM API 延迟（500-3000ms）可忽略不计。

| 场景 | 延迟构成 | 用户感知 |
|------|---------|---------|
| 简单问答 | HTTP IPC(1ms) + Agent 判断(5ms) + LLM(500-3000ms) | 无明显延迟 |
| MCP 工具调用 | HTTP IPC(1ms) + Tool 执行(50-500ms) + LLM | 正常工具调用 |
| SubAgent 派生 | HTTP IPC(1ms) + 子图创建(10ms) + 子 Agent 执行 | 正常 |
| 记忆检索 | HTTP IPC(1ms) + PowerMem 检索(~1.4s p95) | 搜索感知 |

**资源占用（应用启动后）：**

| 资源 | 空闲时 | 对话中 |
|------|--------|--------|
| 内存 (Rust Core) | ~42 MB | ~60 MB |
| 内存 (Sidecar) | ~120 MB | ~200-300 MB |
| 内存 (总计) | ~162 MB | ~260-360 MB |
| CPU (空闲) | <1% | — |
| Sidecar 启动时间 | ~1-2s（后台，用户不感知） | — |

> **对比：** v2.1 方案空闲时 ~42MB 但首次复杂请求有 1-2s 延迟。v2.2 方案多占 ~120MB 但消除所有冷启动延迟，且每次对话拥有完整 Agent 能力。对于桌面 AI Agent 客户端，这个取舍是正确的。

---

## 6. Q5 Skills 系统完整方案

### 6.1 Skills 系统设计目标

Skills（技能）是可复用的、模块化的 AI Agent 能力包。与 MCP Tool 不同，Skill 不仅包含工具定义，还包含**领域知识、工作流指令和最佳实践**。

**Skill vs MCP Tool vs SubAgent 的关系：**

| 组件 | 角色 | 比喻 |
|------|------|------|
| **MCP Tools** | 底层原子操作 | 锤子和锯子 |
| **Skills** | 封装的工作流+领域知识 | 知道怎么造书架 |
| **SubAgents** | 独立上下文执行 | 派一个学徒去造书架 |

### 6.2 SKILL.md 标准（开源标准，2025 年 10 月发布）

#### 渐进式披露架构（Progressive Disclosure）

这是 Skills 系统的核心架构模式：

```
L1 (Always-On)     → YAML frontmatter: name + description (~50 tokens)
L2 (On-Trigger)    → SKILL.md markdown 正文 (<5,000 tokens)
L3 (On-Demand)     → scripts/, references/, assets/ (按需加载)
L4 (Execution-Only)→ 脚本直接执行，不加载到上下文（零 token）
```

**Token 节省效果：** 60-80% 减少（长链业务工作流），TTFT 降低 45%。

#### SKILL.md 文件格式

```markdown
---
name: deploy
description: >-
  Deploy the application to production or staging.
  Use when the user asks about deploying, shipping, or releasing.
compatibility: Node.js 20+, Vercel CLI
allowed-tools: Bash Read Write
---

# Deploy

## Steps
1. Run tests: `bun run test`
2. Build: `bun run build`
3. Typecheck: `bun run typecheck`
4. Deploy staging: `vercel deploy --env preview`
5. Verify: `curl -s https://myapp.com/health | jq .status`

## Rules
- Never deploy to production without passing tests
- Production deploys require `main` branch
```

### 6.3 Skills 系统架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                   MisakaX Skills System                          │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                  React Frontend (Skills UI)               │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │ │
│  │  │ Skill    │  │ Skill    │  │ Skill    │  │ Skill    │ │ │
│  │  │ Browser  │  │ Search   │  │ Installer│  │ Editor   │ │ │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘ │ │
│  └──────────────────────────┬──────────────────────────────┘ │
│                             │ Tauri IPC                       │
│  ┌──────────────────────────┴──────────────────────────────┐ │
│  │              Tauri Rust Core (Skill Manager)              │ │
│  │                                                           │ │
│  │  ┌─────────────────┐  ┌─────────────────┐                │ │
│  │  │ Skill Registry   │  │ Skill Executor   │               │ │
│  │  │ ├─ 本地扫描       │  │ ├─ WASM 沙箱     │               │ │
│  │  │ ├─ 市场 API       │  │ ├─ 受限子进程     │               │ │
│  │  │ ├─ 注册/注销      │  │ └─ MCP 工具注入   │               │ │
│  │  │ └─ 版本管理       │  │                  │               │ │
│  │  └─────────────────┘  └─────────────────┘                │ │
│  └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

#### 6.3.1 Skill 目录结构

```
~/.claw/skills/                         # 用户级 Skills
├── code-review/
│   ├── SKILL.md                        # 必须：技能定义
│   ├── scripts/
│   │   └── run_linter.sh              # 可执行脚本（L4）
│   ├── references/
│   │   └── code-style-guide.md        # 参考资料（L3）
│   └── assets/
│       └── review_template.json       # 模板文件（L3）
│
.claw/skills/                           # 项目级 Skills
├── deploy/
│   ├── SKILL.md
│   └── scripts/
│       └── deploy.sh
│
├── test/
│   ├── SKILL.md
│   └── references/
│       └── test_conventions.md
```

#### 6.3.2 Skill 生命周期管理（Rust 端）

```rust
// Skill 发现与注册
pub struct SkillManager {
    registry: HashMap<String, Skill>,
    scan_paths: Vec<PathBuf>,
}

impl SkillManager {
    // 扫描本地 Skills 目录
    pub fn scan_local_skills(&mut self) -> Vec<Skill> { /* ... */ }

    // 从远程市场搜索
    pub async fn search_marketplace(&self, query: &str) -> Vec<SkillMeta> { /* ... */ }

    // 安装 Skill（下载 + 验证 + 注册）
    pub async fn install_skill(&mut self, url: &str) -> Result<Skill> { /* ... */ }

    // 注册 Skill 为 MCP 工具（可选）
    pub fn register_as_mcp_tool(&self, skill: &Skill) -> Tool { /* ... */ }

    // 验证 SKILL.md 格式
    pub fn validate_skill(&self, path: &Path) -> Result<ValidationReport> { /* ... */ }
}
```

#### 6.3.3 Skill 执行模式

| 模式 | 说明 | 安全等级 |
|------|------|---------|
| **MCP 工具模式** | Skill 被注册为 MCP 工具，Agent 通过 MCP 协议调用 | 高（MCP 权限模型） |
| **WASM 沙箱模式** | 脚本在 Wasmtime 沙箱中执行 | 最高 |
| **受限子进程模式** | 以受限用户权限运行脚本 | 中 |
| **直接执行模式** | 受信任的 Skill 直接执行（需用户授权） | 低 |

#### 6.3.4 前端 Skill UI 功能矩阵

| 功能 | 组件 | 说明 |
|------|------|------|
| **Skill 浏览** | SkillBrowser | 网格/列表展示已安装和可用的 Skills |
| **Skill 搜索** | SkillSearch | 搜索本地和市场 Skills |
| **Skill 安装** | SkillInstaller | 一键安装、进度显示、依赖检查 |
| **Skill 详情** | SkillDetail | 展示 Skill 描述、参数、使用示例 |
| **Skill 编辑器** | SkillEditor (Monaco) | 编辑 SKILL.md、脚本、参考文档 |
| **Skill 执行日志** | SkillLogs | 查看 Skill 执行历史和结果 |
| **Skill 管理** | SkillManager | 启用/禁用/卸载/更新 |

### 6.4 Skills 市场

#### 自建 MisakaX Skills Hub（类似 Skyll / MisakaXHub）

```
claw-skills-hub/
├── registry.json                    # 索引：所有可用 Skills 的元数据
├── skills/
│   ├── code-review/
│   │   ├── SKILL.md
│   │   └── manifest.json
│   ├── deploy-vercel/
│   │   ├── SKILL.md
│   │   └── manifest.json
│   └── ...
```

#### API 设计

```typescript
// Skills Hub API（Rust 端实现 HTTP Client 调用）
interface SkillsHubAPI {
  search(query: string): Promise<SkillMeta[]>;
  getSkill(id: string): Promise<SkillDetail>;
  getCategories(): Promise<Category[]>;
  getPopular(): Promise<SkillMeta[]>;
}
```

### 6.5 Skills 与原架构的集成点

```
Skills 加载方式：

1. Agent 启动时 → Rust 扫描 .claw/skills/ → 注册到 Skill Registry
2. Agent 执行时 → LLM 根据 Skill description 自动匹配
   → Skill 内容注入 prompt（L2 层级）
3. 工具调用时 → Skill 注册的 MCP 工具被 LLM 发现和调用
4. 用户手动 → 前端 /skill-name 或点击触发
5. 按需加载 → references/ 和 assets/ 仅在需要时加载（L3/L4）
```

---

## 7. 前端 UI 框架选型

### 7.1 结论：React 19 + TypeScript

**选择理由（与原报告一致，补充 2026 年最新数据）：**

| 指标 | React 19 | Svelte 5 | SolidJS | Vue 4 |
|------|----------|----------|---------|-------|
| npm 周下载量 | ~190M | ~2.7M | ~1.5M | ~18M |
| Bundle 大小 | ~45KB | ~5KB | ~7KB | ~16KB |
| UI 组件生态 | ⭐⭐⭐⭐⭐ (shadcn/ui) | ⭐⭐⭐ (Skeleton/Melt) | ⭐⭐ | ⭐⭐⭐⭐ |
| AI UI 库 | Vercel AI SDK | 手动 SSE | 手动 SSE | 手动 SSE |
| 社区资源 | 最大 | 中等 | 较小 | 较大 |
| Tauri 模板 | ✅ 官方 | ✅ 官方 | ✅ 官方 | ✅ 官方 |

### 7.2 UI 技术栈

| 层 | 技术 | 理由 |
|----|------|------|
| **框架** | React 19 | 生态最大，AI UI 参考实现最多 |
| **类型** | TypeScript 5.x | 前后端类型安全 |
| **UI 组件** | shadcn/ui + Tailwind CSS v4 | 0KB 运行时、MD3 风格、完全可控 |
| **状态管理** | Zustand (UI) + TanStack Query (Server) | 最简 API、Tauri 适配成熟 |
| **AI UI** | Vercel AI SDK | React 原生流式 AI UI 组件 |
| **代码编辑器** | Monaco Editor | 语法高亮、Diff 视图、代码预览 |
| **国际化** | react-i18next | 成熟稳定 |
| **构建** | Vite 7+ | Tauri 官方推荐，最快构建速度 |

---

## 8. AI Agent 编排框架调研

### 8.1 2026 年主流框架对比

| 框架 | 语言 | Stars | MCP 支持 | 多 Agent | 记忆系统 | 生产就绪 |
|------|------|-------|---------|---------|---------|---------|
| **LangGraph** | Python/JS | 25K | 适配器 | 子图 | Checkpointer+Store | ⭐⭐⭐⭐⭐ |
| **CrewAI** | Python | 46K | 原生 | 角色团队 | 内建 | ⭐⭐⭐⭐ |
| **AutoGen** | Python/.NET | 36K | 部分 | GroupChat | 对话历史 | ⭐⭐⭐ |
| **OpenAI Agents SDK** | Python | 19K | 原生 | Handoffs | Sessions | ⭐⭐⭐ |
| **Mastra** | TypeScript | 22K | 原生 | Workflows | 内建 | ⭐⭐⭐⭐ |
| **Vercel AI SDK** | TypeScript | N/A (20M+ npm/mo) | 原生 | 手动 | 手动 | ⭐⭐⭐⭐ |

### 8.2 结论：DeepAgents (Python Sidecar，常驻预热) + Vercel AI SDK (前端)

v2.2 架构决策：

- **DeepAgents (基于 LangGraph)** 处理所有用户对话 — 作为全对话入口，Sidecar 应用启动时预热
- **Vercel AI SDK** 负责前端流式渲染（始终使用）
- **Rig** 仅用于非对话调用（Embedding、会话标题/摘要生成）
- 没有"简单对话"概念：所有对话均经过 DeepAgents harness，Agent 自主决定处理策略

---

## 9. Rust 生态核心框架

### 9.1 关键 crate 选型

| 功能 | Crate | 版本 | 说明 |
|------|-------|------|------|
| **LLM 多提供商** | `rig-core` | latest | 20+ LLM 提供商统一接口，支持流式 |
| **MCP Client** | `rmcp` | 1.4+ | MCP 官方 Rust SDK，stdio+HTTP+SSE |
| **SQLite** | `rusqlite` | latest | Rust 原生绑定，WAL 模式 |
| **向量搜索 (SQLite)** | `sqlite-vec` | 0.1+ | 纯 C 零依赖，嵌入 SQLite，全平台 |
| **WASM 沙箱** | `wasmtime` | 41+ | <13ms 冷启动，资源限制完善 |
| **异步运行时** | `tokio` | 1.x | Rust 异步标准 |
| **序列化** | `serde` + `serde_json` | latest | JSON/config 序列化 |
| **语音识别 (STT)** | `whisper-cpp-plus` | 0.1.4 | Whisper.cpp Rust 绑定，离线 |
| **语音合成 (TTS)** | `piper-rs` | 0.1.9 | Piper TTS Rust 绑定，离线 |
| **音频采集** | `cpal` | 0.17 | 跨平台音频 I/O |
| **音频播放** | `rodio` | 0.19 | 跨平台音频播放 |

### 9.2 不再使用的 crate（相比原报告 v1.0）

| 原推荐 | 替代 | 原因 |
|--------|------|------|
| ~~`lancedb`~~ | PowerMem (Python, 长期记忆) + sqlite-vec (Rust, 知识库) | PowerMem 提供语义记忆；sqlite-vec 提供嵌入式向量搜索 |

---

## 10. 核心功能实现方案

### 10.1 MCP (Model Context Protocol) 实现

**方案：Rust 原生 MCP Client (rmcp)**

```
Tauri Rust Core
├── MCP Client Manager
│   ├── rmcp Client (Rust, 官方 SDK Tier 2)
│   ├── stdio Transport → 子进程管理
│   ├── HTTP/SSE Transport → reqwest
│   └── 配置加载 (.mcp.json)
```

**选择理由：**
- 性能最优，无跨进程开销
- 与 Tauri 进程管理自然集成
- 类型安全
- 利用 Tauri 权限系统控制 MCP 工具访问

### 10.2 SubAgent 系统实现

**方案：LangGraph Subgraph（Python Sidecar，按需唤醒）**

```python
# 子 Agent 定义为独立子图
research_agent = StateGraph(ResearchState)
research_agent.add_node("search", search_node)
research_agent.add_node("analyze", analyze_node)

# 主 Agent 编排
main_graph = StateGraph(MainState)
main_graph.add_node("planner", planner_node)
main_graph.add_node("researcher", research_agent.compile())  # 子图
main_graph.add_conditional_edges("planner", route_to_agent)
```

### 10.3 沙箱隔离实现

**方案：Wasmtime WASM 沙箱**

| 特性 | 实现 |
|------|------|
| 代码隔离 | 独立 WASM 模块 |
| CPU 限制 | Wasmtime 燃料计量 |
| 内存限制 | WASM 线性内存上限 |
| 超时控制 | tokio::time::timeout |
| 文件系统 | WASI 受控文件挂载 |
| 冷启动 | <13ms (AOT 预编译后 ~55μs) |

### 10.4 长期记忆实现

**方案：PowerMem (Python SDK，嵌入式 seekdb)**

详见第 4 节完整分析。

### 10.5 Skills 系统实现

**方案：Rust 负责发现/注册/管理 + React 负责 UI + SKILL.md 标准**

详见第 6 节完整方案。

### 10.6 知识库 (Knowledge Base) 系统实现

知识库是 MisakaX 的核心差异化功能之一，允许用户导入本地文档（Markdown、PDF、代码文件等），通过语义搜索和全文检索定位信息，并用 LLM 进行 RAG 问答。

#### 10.6.1 知识库功能矩阵

| 功能 | 描述 | 优先级 |
|------|------|--------|
| **文档导入** | 支持 Markdown、PDF、纯文本、代码文件批量导入 | P0 |
| **全文检索** | 基于关键词的精确和模糊搜索 (BM25) | P0 |
| **语义搜索** | 基于向量嵌入的语义相似性搜索 | P0 |
| **混合检索** | 向量 + 全文 + RRF 融合排序 | P0 |
| **RAG 问答** | 检索增强生成，结合知识库内容回答 | P1 |
| **文档管理** | 文件夹组织、标签、元数据编辑 | P1 |
| **自动同步** | 监听文件夹变化，自动增量索引 | P1 |
| **多格式支持** | PDF、Markdown、纯文本、代码文件 | P0 |
| **Chunk 可视化** | 展示检索到的 Chunk 及来源文档 | P2 |

#### 10.6.2 知识库架构设计

```
┌─────────────────────────────────────────────────────────┐
│                 MisakaX Knowledge Base System                │
│                                                           │
│  ┌─────────────────────────────────────────────────────┐ │
│  │               React Frontend (KB UI)                  │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌─────────┐ │ │
│  │  │ Document │ │ Search   │ │ RAG Chat │ │ Folder  │ │ │
│  │  │ Browser  │ │ Interface│ │ Panel    │ │ Manager │ │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └─────────┘ │ │
│  └──────────────────────┬──────────────────────────────┘ │
│                         │ Tauri IPC                       │
│  ┌──────────────────────┴──────────────────────────────┐ │
│  │            Tauri Rust Core (KB Engine)                │ │
│  │                                                       │ │
│  │  ┌──────────┐ ┌───────────┐ ┌────────────────────┐  │ │
│  │  │ Document │ │ Chunking  │ │ Embedding Pipeline │  │ │
│  │  │ Importer │ │ Engine    │ │ (ONNX local/remote) │  │ │
│  │  └──────────┘ └───────────┘ └────────────────────┘  │ │
│  │  ┌──────────┐ ┌───────────┐ ┌────────────────────┐  │ │
│  │  │ File     │ │ Hybrid    │ │ Relevance Gating   │  │ │
│  │  │ Watcher  │ │ Searcher  │ │ (distance threshold)│  │ │
│  │  └──────────┘ └───────────┘ └────────────────────┘  │ │
│  │                                                       │ │
│  │  ┌─────────────────────────────────────────────────┐ │ │
│  │  │           Single SQLite File                      │ │ │
│  │  │  ┌──────────┐ ┌──────────┐ ┌──────────────────┐ │ │ │
│  │  │  │ chunks   │ │ FTS5     │ │ sqlite-vec       │ │ │ │
│  │  │  │(metadata)│ │(keyword) │ │ (vector + cosine)│ │ │ │
│  │  │  └──────────┘ └──────────┘ └──────────────────┘ │ │ │
│  │  └─────────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

#### 10.6.3 核心检索架构：混合检索 + RRF 融合

遵循 2026 年 local-first RAG 的最佳实践模式：

```
用户查询
    │
    ├──▶ sqlite-vec 语义搜索 ──▶ top_k 向量结果 (余弦相似度)
    │
    ├──▶ FTS5 全文检索 ──▶ top_k 关键词结果 (BM25)
    │
    └──▶ RRF (Reciprocal Rank Fusion) ──▶ 融合排序结果
              │
              score(chunk) = w_vec/(k+r_vec) + w_fts/(k+r_fts)
              │
              ├──▶ 相关性门控 (余弦距离阈值)
              │       distance ≤ 0.95 → 高置信度
              │       0.95-0.98 → 中置信度
              │       > 0.98 → 低置信度，返回 "未找到相关内容"
              │
              └──▶ MMR 去重 (防止同一文档的多个 Chunk 占据结果)
```

**关键参数：**
- `k = 60` (RRF 平滑常数)
- `w_vec = 0.6` / `w_fts = 0.4` （默认权重，可自适应 IDF 调整）
- 余弦距离阈值 0.95（高置信度）至 0.98（低置信度）

#### 10.6.4 Embedding Pipeline

| 模式 | 引擎 | 维度 | 速度 | 适用场景 |
|------|------|------|------|---------|
| **本地 (默认)** | ONNX MiniLM-L6-v2 | 384d | ~700 chunks/s (CPU) | 完全离线、隐私优先 |
| **本地 (高质量)** | ONNX BGE-small-en | 384d | ~500 chunks/s | 离线高质量 |
| **本地 (多语言)** | ONNX BGE-M3 | 1024d | ~200 chunks/s | 中英文混合 |
| **远程** | OpenAI text-embedding-3-small | 1536d | 取决于网络 | 最佳质量、有网络 |

推荐默认使用 **ONNX MiniLM-L6-v2**（384 维，<100MB 模型文件），首次运行时自动下载。

#### 10.6.5 文档处理流水线

```
文件导入 → 格式检测 → 内容提取 → Markdown 感知分块 → Embedding → 索引
    │           │           │              │              │           │
    │  .md      │  解析      │  纯文本       │  按标题边界   │  ONNX     │ sqlite-vec
    │  .pdf     │  markdown  │  提取         │  递归分块     │  local    │ + FTS5
    │  .txt     │  PDF       │               │  512 token    │  or       │
    │  .code    │  代码高亮   │               │  50 overlap   │  remote   │
    └───────────┴────────────┴───────────────┴───────────────┴───────────┘
```

- **分块策略：** Markdown 感知的递归分块（按标题层级优先），512 token/chunk，50 token 重叠
- **增量索引：** 文件监视器 (watcher) 监听变更，基于 content-hash 检测变化，仅重索引变更文件
- **上下文扩展：** 检索时自动 ±1 相邻 Chunk 扩展，为 LLM 提供更完整上下文

#### 10.6.6 sqlite-vec：SQLite 的嵌入式向量扩展

`sqlite-vec` 是 2026 年嵌入式向量搜索的事实标准，纯 C 实现、零依赖、跨全平台：

| 属性 | 详情 |
|------|------|
| **实现** | 纯 C（零依赖，无 Faiss/ML 库依赖） |
| **平台** | Windows / macOS (Intel + Apple Silicon) / Linux / WASM / iOS / Android |
| **向量类型** | float32、int8、binary/bit |
| **距离度量** | L2、Cosine、Hamming |
| **搜索算法** | 暴力 KNN + SIMD 加速（AVX2 on x86, NEON on ARM） |
| **性能 (100K/384d)** | ~50ms (k=20) on M1 |
| **性能 (1M/128d)** | ~33ms (k=20) on M1 |
| **ACID** | ✅ 完整事务支持 |
| **GitHub Stars** | 7,300+ |
| **Rust crate** | `cargo add sqlite-vec`（编译时自动捆绑 C 源码） |

**sqlite-vec Rust 集成示例：**

```rust
use rusqlite::Connection;

fn search_knowledge_base(db: &Connection, query_embedding: &[f32], limit: usize) -> Vec<SearchResult> {
    let mut stmt = db.prepare("
        SELECT c.id, c.text, c.file_path, vec_distance_cosine(v.embedding, ?1) as dist
        FROM chunks c
        JOIN chunks_vec v ON c.id = v.id
        WHERE v.embedding MATCH ?1
        ORDER BY dist
        LIMIT ?2
    ").unwrap();

    // ... bind embedding and return results
}
```

#### 10.6.7 知识库数据模型

```sql
-- 文件注册表
CREATE TABLE kb_files (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    file_type TEXT NOT NULL,          -- 'markdown', 'pdf', 'text', 'code'
    mtime INTEGER NOT NULL,
    size INTEGER NOT NULL,
    content_hash TEXT NOT NULL,       -- SHA-256，用于增量更新检测
    status TEXT DEFAULT 'active',     -- 'active', 'indexing', 'error'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Chunk 元数据表
CREATE TABLE kb_chunks (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    text TEXT NOT NULL,               -- chunk 原始文本
    token_count INTEGER,
    start_byte INTEGER,
    end_byte INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (file_id) REFERENCES kb_files(id) ON DELETE CASCADE
);

-- FTS5 全文索引（关键词搜索）
CREATE VIRTUAL TABLE kb_chunks_fts USING fts5(
    text,
    content='kb_chunks',
    content_rowid='rowid'
);

-- sqlite-vec 向量索引（语义搜索）
CREATE VIRTUAL TABLE kb_chunks_vec USING vec0(
    embedding float[384]              -- 维度取决于选择的 Embedding 模型
);

-- 知识库文件夹/分类
CREATE TABLE kb_folders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 文档-文件夹关联
CREATE TABLE kb_file_folders (
    file_id TEXT NOT NULL,
    folder_id TEXT NOT NULL,
    PRIMARY KEY (file_id, folder_id)
);

-- 索引日志（审计追踪）
CREATE TABLE kb_index_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id TEXT NOT NULL,
    action TEXT NOT NULL,             -- 'indexed', 'updated', 'deleted'
    chunks_count INTEGER,
    duration_ms INTEGER,
    error TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

#### 10.6.8 性能预期

基于 2026 年 sqlite-vec 社区基准测试（M1 Mac Mini 8GB）：

| 知识库规模 | 向量维度 | 搜索延迟 (k=20) | 内存占用 |
|-----------|---------|----------------|---------|
| 1K chunks | 384d | <5ms | <50MB |
| 10K chunks | 384d | ~15ms | <100MB |
| 50K chunks | 384d | ~30ms | <200MB |
| 100K chunks | 384d | ~50ms | <300MB |
| 500K chunks | 384d | ~150ms | <800MB |

对于桌面端知识库的典型规模（1K-100K chunks），延迟完全可接受。单文件模式将一切存放在一个 SQLite 文件中，最大程度简化备份和迁移。

---

## 11. 数据库与存储引擎深度选型：seekdb vs SQLite

### 11.1 核心问题

在架构设计中有两个关联的数据库选型决策：

1. **主数据库**：桌面端的结构化数据（会话、消息、设置、任务）用什么存储？
2. **知识库存储**：知识库的文档、全文索引、向量嵌入用什么存储？

候选方案：
- **方案 A**：SQLite (rusqlite) 作为主数据库
- **方案 B**：seekdb 替代 SQLite，统一结构化数据 + 向量搜索 + 全文检索

### 11.2 seekdb 概述

**seekdb** 是 OceanBase 于 2025 年 11 月开源的 AI 原生混合搜索数据库（Apache 2.0），定位为"面向 AI 时代重新发明的 SQLite"。

| 能力 | seekdb | SQLite |
|------|--------|--------|
| **嵌入式部署** | ✅ `pip install pyseekdb` | ✅ 经典嵌入式 |
| **关系型数据 (SQL)** | ✅ MySQL 兼容 | ✅ 完整 SQL |
| **向量搜索** | ✅ HNSW/IVF, 最高 16000 维 | ❌ 无原生支持 |
| **全文检索** | ✅ BM25, IK/Jieba/Ngram 分词 | ✅ FTS5 |
| **向量+全文混合搜索** | ✅ 原生单 SQL 完成 | ❌ 需手动拼接 |
| **自动 Embedding** | ✅ 内置 AI_EMBED 函数 | ❌ 不支持 |
| **AI Function** | ✅ AI_COMPLETE, AI_RERANK | ❌ 不支持 |
| **MCP 协议** | ✅ 内置 MCP Server | ❌ 不支持 |
| **ACID 事务** | ✅ | ✅ |
| **HTAP 混合负载** | ✅ 行列混存 | ⚠️ 仅轻量 OLTP |
| **包体积** | ~200MB+ (需 1C2G) | ~600KB 单文件 |

如果 seekdb 在三个平台上都可以嵌入式运行，它将是完美的选择——一个引擎同时搞定结构化存储、向量搜索、全文检索、AI 函数。

### 11.3 seekdb 跨平台支持现状（关键调研发现）

| 平台 | 嵌入式原生支持 | 预计时间 | 当前可行方案 |
|------|-------------|---------|------------|
| **Linux (x86_64/aarch64)** | ✅ 完全支持 | 已发布 | `pip install pyseekdb` 嵌入式模式 |
| **macOS (Intel + Apple Silicon)** | ⚠️ 开发中 | **2026 年 4 月左右** | Docker / seekdb 桌面版 |
| **Windows** | ❌ 不支持 | **2026 年底** | WSL2 / Docker Desktop |

> **关键来源 (OceanBase 社区官方回复, ask.oceanbase.com)：**
>
> *"我们现在已经启动 mac 的原生编译，但工作量比较大，估计在明年的 4 月份左右会完成...在 windows 上，需要到明年年底，这个工作量更大。"*

**结论：seekdb 嵌入式模式目前仅 Linux 原生可用。macOS 原生支持可能在近期发布（2026 年 4 月），Windows 原生支持至少还要等 8 个月以上。对于需要同时支持 Windows/macOS/Linux 的桌面应用，seekdb 嵌入式模式还不是可行的主数据库方案。**

### 11.4 桌面端数据库选型决策

#### 🏆 最终决策：SQLite 作为主数据库 + sqlite-vec 扩展

```
桌面端存储架构 (全平台可用)：

  ┌─────────────────────────────────────┐
  │       单个 SQLite 文件 (.claw.db)     │
  │                                     │
  │  ┌───────────────────────────────┐  │
  │  │  rusqlite (Rust 原生绑定)       │  │
  │  │  ├── sessions / messages      │  │
  │  │  ├── tasks / router_configs   │  │
  │  │  ├── settings / memories      │  │
  │  │  └── kb_files / kb_chunks      │  │
  │  ├───────────────────────────────┤  │
  │  │  FTS5 (全文检索)               │  │
  │  │  └── kb_chunks_fts             │  │
  │  ├───────────────────────────────┤  │
  │  │  sqlite-vec (向量搜索)          │  │
  │  │  └── kb_chunks_vec             │  │
  │  └───────────────────────────────┘  │
  │                                     │
  │  ✅ Windows  ✅ macOS  ✅ Linux     │
  └─────────────────────────────────────┘

Python Sidecar (可选，仅 Linux/macOS Docker)：

  ┌─────────────────────────────────────┐
  │  PowerMem + seekdb (嵌入式)          │
  │  ├── 长期语义记忆                    │
  │  ├── 艾宾浩斯遗忘曲线                │
  │  └── LLM 智能提取                    │
  │                                     │
  │  ✅ Linux  ⚠️ macOS (Docker/即将原生) │
  │  ⚠️ Windows (WSL2/Docker)           │
  └─────────────────────────────────────┘
```

**选择理由：**

| 理由 | 说明 |
|------|------|
| **跨平台即用** | SQLite 在所有三个目标平台上原生可用，无需虚拟化 |
| **零运维** | 单个文件，用户备份只需复制一个 `.claw.db` 文件 |
| **sqlite-vec 补全向量能力** | 纯 C、零依赖、全平台，完美嵌入 SQLite |
| **FTS5 内建全文搜索** | 不需要额外的搜索引擎，BM25 排序 |
| **WAL 模式** | 支持并发读写，崩溃恢复 |
| **seekdb 有明确时间表** | 当 Windows 原生支持就绪后，可评估迁移 |

### 11.5 seekdb 的未来迁移路径

```
Phase 1 (现在):     SQLite + sqlite-vec + FTS5     → 全平台可用
Phase 2 (2026 年底): seekdb Windows 原生就绪后评估   → 考虑迁移知识库部分
Phase 3 (2027):     如果 seekdb 成熟稳定            → 可选项：全面迁移到 seekdb
```

**迁移触发条件：**
1. seekdb Windows 原生嵌入式模式稳定发布
2. seekdb 打包尺寸优化到可接受范围 (<100MB)
3. seekdb 提供 Rust SDK（目前仅 Python SDK，需通过 pyseekdb 间接调用）

**seamless 迁移设计：** 知识库数据层通过抽象 trait 设计，方便未来切换底层引擎：

```rust
pub trait KnowledgeBaseStore {
    async fn index_chunks(&self, chunks: Vec<Chunk>) -> Result<()>;
    async fn hybrid_search(&self, query: &str, embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
    async fn delete_file(&self, file_id: &str) -> Result<()>;
}

// 当前实现：SQLite + sqlite-vec + FTS5
pub struct SqliteKnowledgeBase { db: Connection }

// 未来实现：seekdb
// pub struct SeekDbKnowledgeBase { client: SeekDbClient }
```

### 11.6 三方存储方案全景对比

| 维度 | SQLite + sqlite-vec + FTS5 | LanceDB | seekdb (未来) | PowerMem (Python) |
|------|--------------------------|---------|--------------|-------------------|
| **结构化数据** | ✅ 完整 SQL | ⚠️ 非主要用途 | ✅ MySQL 兼容 | ⚠️ 非主要用途 |
| **向量搜索** | ✅ sqlite-vec | ✅ 原生 | ✅ HNSW/IVF | ✅ 通过 seekdb |
| **全文检索** | ✅ FTS5 + BM25 | ⚠️ 基础 | ✅ BM25 | ✅ 通过 seekdb |
| **混合检索** | ⚠️ 应用层 RRF | ⚠️ 需手动实现 | ✅ 原生单 SQL | ✅ 原生 |
| **语义理解** | ❌ 无 | ❌ 无 | ❌ 无 | ✅ 智能提取+衰减 |
| **跨平台** | ✅ 全部 | ✅ 全部 | ❌ 仅 Linux | ⚠️ 依赖 seekdb |
| **Rust SDK** | ✅ rusqlite | ✅ lancedb crate | ❌ 仅 Python | ❌ Python SDK |
| **部署复杂度** | 零 | 零 | 中 (需原生二进制) | 中 (Python 环境) |
| **包体积** | ~600KB | ~2MB | ~200MB+ | ~200MB+ |
| **角色** | **主数据库 + 知识库** | 备选向量引擎 | 未来统一引擎 | **长期记忆引擎** |

**角色分工总结：**

| 引擎 | 负责范围 | 运行位置 |
|------|---------|---------|
| **SQLite + sqlite-vec + FTS5** | 主数据库 + 知识库 + 向量搜索 | Rust Core (始终可用, 全平台) |
| **PowerMem** | 长期语义记忆 | Python Sidecar (按需, Linux/macOS-Docker) |

---

## 12. 状态管理与数据存储方案

### 12.1 前端状态管理

| 状态类型 | 方案 | 说明 |
|---------|------|------|
| **UI 状态** | Zustand | 主题、侧边栏、当前会话 |
| **服务器状态** | TanStack Query | LLM 响应缓存、会话列表 |
| **表单状态** | React Hook Form | 设置表单 |
| **流式状态** | Vercel AI SDK `useChat` | 流式消息、Tool Call 状态 |

### 12.2 Rust 后端状态

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppStateInner {
    pub db: rusqlite::Connection,
    pub mcp_manager: McpManager,
    pub skill_registry: SkillRegistry,
    pub sidecar_handle: Option<SidecarHandle>,
}

pub type AppState = Arc<RwLock<AppStateInner>>;
```

### 12.3 数据库：SQLite (rusqlite, WAL 模式)

保留完整的数据模型设计（sessions、messages、memories、tasks、router_configs、settings 表），详见原报告第 9 节。

知识库相关的表结构（kb_files、kb_chunks、kb_chunks_fts、kb_chunks_vec、kb_folders）详见第 10.6.7 节。

### 12.4 记忆存储

| 记忆类型 | 存储方案 |
|---------|---------|
| 工作记忆 | React State + Zustand (内存) |
| 短期记忆 (Checkpointer) | SQLite (rusqlite, WAL) |
| 长期记忆 | PowerMem + seekdb (嵌入式) |

---

## 13. Buddy 桌面伴侣系统概要

Buddy 系统保持原报告的完整方案，核心要点：

### 13.1 技术选型

| 组件 | 技术 |
|------|------|
| 透明窗口 | Tauri `transparent: true` + `alwaysOnTop` |
| 角色动画 | Lottie (默认) / Live2D (高级模式) |
| 语音输入 (STT) | whisper-cpp-plus (Rust, 离线) |
| 语音输出 (TTS) | piper-rs (Rust, 离线) |
| 语音检测 (VAD) | Silero VAD |
| 动画库 | lottie-react / pixi-live2d-display |

### 13.2 多窗口通信

```
主窗口 ↔ Tauri Event System ↔ Buddy 窗口
```

### 13.3 资源需求

| 指标 | 预估值 |
|------|--------|
| STT 模型内存 | ~150 MB |
| TTS 模型内存 | ~80 MB |
| Buddy 窗口内存 | ~20 MB |
| 模型磁盘空间 | ~270 MB |
| 优化策略 | 延迟加载，首次使用时才加载语音模型 |

---

## 14. 综合技术栈推荐

### 14.1 最终技术栈总览

```
┌──────────────────────────────────────────────────────────────┐
│                    MisakaX Desktop App (v2.2)                     │
├──────────────────────────────────────────────────────────────┤
│  Frontend (WebView)                                           │
│  ├── React 19 + TypeScript 5.x                                │
│  ├── Tailwind CSS v4 + shadcn/ui                              │
│  ├── Zustand + TanStack Query (状态管理)                       │
│  ├── Vercel AI SDK (流式 AI UI)                                │
│  ├── Monaco Editor (代码编辑/预览)                              │
│  ├── react-i18next (国际化)                                    │
│  └── Vite 7+ (构建工具)                                        │
├──────────────────────────────────────────────────────────────┤
│  Tauri 2.x Core (Rust — 系统集成 + 非对话业务)                    │
│  ├── rig-core (Embedding 生成、会话标题/摘要、非对话 LLM)         │
│  ├── rmcp (MCP Client, 官方 Rust SDK)                         │
│  ├── rusqlite + sqlite-vec (SQLite WAL + 向量搜索)              │
│  ├── Knowledge Base Engine (文档索引 + 混合检索 + RRF)           │
│  ├── wasmtime (WASM 沙箱)                                      │
│  ├── whisper-cpp-plus (离线 STT, Buddy)                        │
│  ├── piper-rs (离线 TTS, Buddy)                                │
│  ├── cpal + rodio (音频 I/O, Buddy)                            │
│  ├── tokio (异步运行时)                                         │
│  ├── serde + serde_json (序列化)                                │
│  ├── Skill Manager (本地扫描 + 市场 API + 生命周期)               │
│  ├── Sidecar Manager (预热启动 + 健康检查 + 自动恢复)             │
│  └── Tauri Plugins (shell, fs, http, notification, updater)   │
├──────────────────────────────────────────────────────────────┤
│  Agent Backend (Python Sidecar, 常驻预热)                       │
│  ├── DeepAgents v0.5+ (全对话入口，Middleware 架构)              │
│  │   ├── TodoListMiddleware (任务规划与追踪)                     │
│  │   ├── SkillsMiddleware (技能加载与执行)                       │
│  │   ├── FilesystemMiddleware (工作目录文件操作)                  │
│  │   ├── SubAgentMiddleware (子代理派生)                        │
│  │   ├── SummarizationMiddleware (上下文摘要压缩)                │
│  │   ├── MemoryMiddleware (AGENTS.md 记忆管理)                  │
│  │   └── HumanInTheLoopMiddleware (人工审批)                    │
│  ├── PowerMem (长期记忆引擎，通过自定义 Tool 集成)                 │
│  │   ├── seekdb (嵌入式混合搜索)                                │
│  │   ├── LLM 智能提取 + 去重 + 合并                             │
│  │   └── 艾宾浩斯遗忘曲线                                       │
│  ├── LangGraph 1.1+ (底层编排引擎)                              │
│  ├── FastAPI + uvicorn (HTTP API)                              │
│  ├── langchain-anthropic / langchain-openai (LLM)             │
│  └── Nuitka (编译打包为独立可执行文件)                            │
└──────────────────────────────────────────────────────────────┘
```

### 14.2 各技术选择理由汇总

| 技术 | 选择理由 |
|------|---------|
| **Tauri 2.x** | 包体积小 96%、内存少 75%、启动快 3.7x、安全模型优秀 |
| **React 19** | 最大生态、shadcn/ui、Vercel AI SDK 流式渲染 |
| **shadcn/ui + Tailwind v4** | 0KB 运行时、MD3 风格、完全可控 |
| **Zustand + TanStack Query** | 极简 API、Tauri 适配成熟 |
| **Vercel AI SDK** | React 原生 AI UI、流式渲染、Tool Call 展示 |
| **Rig** | Rust 原生多 LLM 提供商统一接口；v2.2 用于非对话调用（Embedding/标题/摘要） |
| **rmcp** | MCP 官方 Rust SDK、与 Tauri 无缝集成 |
| **rusqlite** | Rust 原生 SQLite、WAL 模式、零额外依赖 |
| **sqlite-vec** | 纯 C 向量扩展、零依赖、全平台、嵌入 SQLite |
| **SQLite FTS5** | 内建全文搜索引擎、BM25 排序、零额外依赖 |
| **Knowledge Base Engine** | 文档处理流水线 + 混合检索 RRF + 相关性门控 |
| **Wasmtime** | WASM 沙箱标准、<13ms 启动、资源限制完善 |
| **DeepAgents** | 全对话入口 Middleware 架构、规划/文件/子代理/技能/记忆/摘要开箱即用 |
| **PowerMem** | 语义记忆引擎、艾宾浩斯遗忘、LangGraph 原生集成、96% Token 节省 |
| **whisper-cpp-plus** | 离线语音识别、Rust 原生、高准确度 |
| **piper-rs** | 离线语音合成、900+ 声音、Rust 原生 |
| **Nuitka** | Python→C 编译、比 PyInstaller 启动更快 |

---

## 15. 最终架构选择

### 15.1 架构模式

```
🏆 最终选择：全路径 DeepAgents 单一架构 (Single-Path DeepAgents Harness)

┌──────────────────────────────────────────────────────────────────────┐
│                         MisakaX Desktop App v2.2                      │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  React 19 WebView (UI)                                           │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │ │
│  │  │  Chat    │ │ 知识库   │ │  Skills  │ │ Settings │           │ │
│  │  │ (含工作目│ │ Browser  │ │ Manager  │ │          │           │ │
│  │  │  录选择) │ │          │ │          │ │          │           │ │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘           │ │
│  └───────┼─────────────┼────────────┼────────────┼─────────────────┘ │
│          │             │            │            │                    │
│          └─────────────┴─────┬──────┴────────────┘                    │
│                              │ Tauri IPC + Events                     │
│  ┌───────────────────────────┴────────────────────────────────────┐ │
│  │  Tauri Rust Core (系统集成 + 非对话业务)                          │ │
│  │                                                                  │ │
│  │  ┌─────────┐ ┌──────────┐ ┌───────────┐ ┌──────────────────┐  │ │
│  │  │ Rig     │ │ MCP Mgr  │ │ DB Layer  │ │ Skill Manager    │  │ │
│  │  │(非对话) │ │ (rmcp)   │ │(rusqlite  │ │ (扫描/解析/注册)  │  │ │
│  │  │• Embed  │ │          │ │+sqlite-vec│ │                  │  │ │
│  │  │• 标题   │ │          │ │+FTS5)     │ │                  │  │ │
│  │  │• 摘要   │ │          │ │           │ │                  │  │ │
│  │  └─────────┘ └──────────┘ └───────────┘ └──────────────────┘  │ │
│  │                                                                  │ │
│  │  ┌──────────────┐ ┌───────────┐ ┌──────────────────────────┐  │ │
│  │  │ Wasmtime     │ │ Sidecar   │ │ Buddy Voice              │  │ │
│  │  │ Sandbox      │ │ Manager   │ │ (whisper + piper)        │  │ │
│  │  └──────────────┘ └─────┬─────┘ └──────────────────────────┘  │ │
│  └─────────────────────────┼──────────────────────────────────────┘ │
│                             │ HTTP localhost (常驻连接)               │
│  ┌─────────────────────────┴──────────────────────────────────────┐ │
│  │  Python Sidecar (Agent 引擎 — 常驻预热，全对话入口)               │ │
│  │                                                                  │ │
│  │  ┌────────────────────────────────────────────────────────────┐ │ │
│  │  │  DeepAgents Harness (create_deep_agent)                     │ │ │
│  │  │  ┌──────────────┐ ┌───────────────┐ ┌──────────────────┐  │ │ │
│  │  │  │ TodoList     │ │ Skills        │ │ Filesystem       │  │ │ │
│  │  │  │ Middleware   │ │ Middleware    │ │ Middleware       │  │ │ │
│  │  │  │ (任务规划)    │ │ (技能加载)     │ │ (工作目录)        │  │ │ │
│  │  │  └──────────────┘ └───────────────┘ └──────────────────┘  │ │ │
│  │  │  ┌──────────────┐ ┌───────────────┐ ┌──────────────────┐  │ │ │
│  │  │  │ SubAgent     │ │ Summarization │ │ Memory           │  │ │ │
│  │  │  │ Middleware   │ │ Middleware    │ │ Middleware       │  │ │ │
│  │  │  │ (子代理派生)  │ │ (上下文摘要)   │ │ (AGENTS.md)      │  │ │ │
│  │  │  └──────────────┘ └───────────────┘ └──────────────────┘  │ │ │
│  │  │  ┌──────────────┐ ┌────────────────────────────────────┐  │ │ │
│  │  │  │ HumanInThe   │ │ Custom Tools                       │  │ │ │
│  │  │  │ LoopMiddleware│ │ powermem_search / powermem_save    │  │ │ │
│  │  │  │ (人工审批)    │ │ mcp_bridge / kb_search             │  │ │ │
│  │  │  └──────────────┘ └────────────────────────────────────┘  │ │ │
│  │  └────────────────────────────────────────────────────────────┘ │ │
│  │                                                                  │ │
│  │  ┌────────────────────┐  ┌────────────────────┐                 │ │
│  │  │ PowerMem           │  │ LangGraph           │                 │ │
│  │  │ (长期记忆引擎)      │  │ Checkpointer (SQLite)│                 │ │
│  │  └────────────────────┘  └────────────────────┘                 │ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  用户对话流程:                                                         │
│  打开对话框 → 选择工作目录 → 发送消息                                    │
│  → Rust 转发 → DeepAgents Harness                                     │
│  → Agent 自主决策 (是否用工具/查记忆/派子代理/读文件)                      │
│  → 流式返回 → React 渲染                                               │
└──────────────────────────────────────────────────────────────────────┘
```

### 15.2 分层职责明确矩阵 (v2.2)

| 层 | 技术 | 职责范围 | 调用频率 |
|----|------|---------|---------|
| **UI 展示层** | React + shadcn/ui | 页面渲染、交互、动画、工作目录选择 UI | 持续 |
| **UI 状态层** | Zustand + TanStack Query | 客户端状态、缓存 | 持续 |
| **AI UI 层** | Vercel AI SDK | 流式消息渲染 | 高频 |
| **IPC 总线** | Tauri invoke + Events | 前后端通信 | 持续 |
| **系统集成层** | Tauri Rust Core | FS、进程、通知、剪贴板、Sidecar 生命周期 | 中频 |
| **非对话 LLM 层** | Rig (Rust) | Embedding、会话标题/摘要、配置验证 | 低频 |
| **MCP 层** | rmcp (Rust) | MCP Client/Server 管理 | 中频 |
| **数据层** | rusqlite + sqlite-vec (Rust) | 结构化数据 + 向量搜索 | 高频 |
| **知识库层** | SQLite FTS5 + sqlite-vec (Rust) | 文档索引、混合检索、RAG | 中频 |
| **沙箱层** | Wasmtime (Rust) | 安全代码执行 | 低频 |
| **Skills 发现层** | Rust Manager | Skill 文件系统扫描/解析/注册 | 低频 |
| **Sidecar 管理层** | Rust Sidecar Manager | 预热启动、健康检查、自动恢复 | 持续 |
| **Agent 编排层** | DeepAgents (Python, 常驻) | **全对话入口**、Middleware 链、SubAgent | 高频（每次对话） |
| **Skills 执行层** | DeepAgents SkillsMiddleware | Skill 内容注入与执行 | 中频 |
| **文件操作层** | DeepAgents FilesystemMiddleware | 基于工作目录的文件读写 | 中频 |
| **长期记忆层** | PowerMem (Python, 常驻) | 语义记忆管理 | 中频 |
| **Buddy 语音层** | whisper + piper (Rust) | 语音识别与合成 | 中频（按需） |
| **Buddy 动画层** | Lottie/Live2D (React) | 角色动画渲染 | 持续 |

### 15.3 请求路由策略 (v2.2)

```rust
// v2.2: 无复杂度判定 — 所有对话统一路由到 DeepAgents Sidecar
// 仅区分"对话类"和"非对话类"请求

fn route_request(req: &AgentRequest) -> ExecutionTarget {
    match req.request_type {
        // 所有用户对话 → DeepAgents Sidecar（常驻预热，无冷启动）
        RequestType::Chat => ExecutionTarget::PythonSidecar,
        
        // 非对话类请求 → Rust 本地处理
        RequestType::KnowledgeBaseSearch => ExecutionTarget::RustKnowledgeBase,
        RequestType::SandboxExecution => ExecutionTarget::RustWasmtime,
        RequestType::EmbeddingGeneration => ExecutionTarget::RustRig,
        RequestType::SessionTitleGeneration => ExecutionTarget::RustRig,
        RequestType::SessionSummary => ExecutionTarget::RustRig,
        RequestType::DatabaseQuery => ExecutionTarget::RustDatabase,
    }
}
```

### 15.4 通信通道

```
用户对话 (所有对话统一路径):
  React → Tauri IPC → Rust 转发 → HTTP POST localhost:18910 → FastAPI
  → DeepAgents Harness (Middleware 链处理)
  → LLM API (通过 LangChain 适配器)
  → SSE Stream → Rust 转发 → Tauri Event → React (Vercel AI SDK 渲染)

非对话调用 (Rust 本地):
  Embedding: Rust → Rig → Embedding API → Vector → SQLite
  会话标题: Rust → Rig → LLM Completion → Title → SQLite
  知识库检索: Rust → sqlite-vec KNN + FTS5 → RRF → Results
```

### 15.5 与历史版本的关键差异

| 维度 | v1.0 | v2.0 | v2.1 | v2.2 | 变化原因 |
|------|------|------|------|------|---------|
| **Architecture** | 大 Sidecar | 智能分层 | 智能分层 | **全路径 DeepAgents** | 消除路由误判，Agent 平台定位 |
| **对话入口** | Sidecar | Rig+Sidecar | Rig+Sidecar | **DeepAgents 唯一** | 每对话都拥有完整 Agent 能力 |
| **Sidecar 启动** | 随应用启动 | 按需延迟唤醒 | 按需延迟唤醒 | **应用启动预热** | 全路径要求 Sidecar 始终就绪 |
| **Rig 角色** | LLM 直调 | 简单对话+工具 | 简单对话+工具 | **非对话专用** | 对话全部迁移至 DeepAgents |
| **路由复杂度** | 无 | 5 路分支 | 5 路分支 | **2 路（对话/非对话）** | 对话统一路径 |
| **工作目录** | 无 | 无 | 无 | **🆕 P0 需求** | Agent 工程平台核心特性 |
| **长期记忆** | LanceDB | PowerMem | PowerMem | PowerMem | 不变 |
| **Skills 执行** | 未设计 | Rust | Rust | **Rust 发现 + DeepAgents 执行** | 双层架构 |
| **Agent 编排** | LangGraph | LangGraph | LangGraph | **DeepAgents (LangGraph)** | Middleware 开箱即用 |

### 15.6 最终综合评分

| 评估维度 | v2.1 评分 | v2.2 评分 | 变化说明 |
|---------|----------|----------|---------|
| **性能** | 9.5 | **9.2** | 全路径走 Sidecar 增加 ~1ms IPC；预热消除冷启动感知 |
| **包体积** | 8.5 | 8.5 | 不变 |
| **Agent 编排完整性** | 9.0 | **9.5** | DeepAgents Middleware 链覆盖规划/文件/子代理/Skills/记忆/摘要/HITL |
| **记忆系统成熟度** | 9.5 | 9.5 | 不变 |
| **Skills 系统完整性** | 9.0 | **9.5** | 双层架构：Rust 发现管理 + DeepAgents SkillsMiddleware 执行 |
| **知识库完整性** | 9.0 | 9.0 | 不变 |
| **数据库方案稳健性** | 9.5 | 9.5 | 不变 |
| **跨平台覆盖** | 9.0 | 9.0 | 不变 |
| **可维护性** | 9.0 | **9.5** | 单一对话路径消除双系统维护；Middleware 集中管理 |
| **用户体验一致性** | 7.0 | **9.5** | 消除简单/复杂路径割裂；每次对话能力一致 |
| **架构简洁性** | 7.5 | **9.0** | 消除 5 路路由判断，对话路径统一 |
| **工作目录能力** | N/A | **9.5** | 🆕 本地+远程目录绑定，Agent 工程化 |
| **综合评分** | **9.2 / 10** | **9.3 / 10** | **+0.1** |

---

## 16. 风险评估与应对策略

### 16.1 风险清单 (v2.2 更新)

| 风险 | 影响 | 概率 | 应对策略 |
|------|------|------|---------|
| **Sidecar 预热失败** | 应用启动后所有对话不可用 | 低 | 重试机制（最多 3 次）；降级为按需启动 + 提示用户等待；显示 Sidecar 状态指示器 |
| **Sidecar 运行时崩溃** | 正在进行的对话中断 | 中 | 自动重启 + 会话状态恢复（LangGraph Checkpointer 持久化）；前端显示重连中 |
| **Sidecar 预热资源占用** | 应用空闲内存 ~162MB（相比纯 Rust ~42MB） | 确定 | 接受此开销。AI Agent 桌面客户端定位下，这是合理的资源使用 |
| **工作目录路径安全** | Agent 可能访问用户未授权的目录 | 低 | 目录白名单配置（设置页）；DeepAgents FilesystemMiddleware 自动限制在绑定路径；操作审计日志 |
| **PowerMem 稳定性** | 作为较新项目，API 可能变动 | 中 | 固定版本号；PowerMem 通过自定义 Tool 集成，API 变动影响可控 |
| **seekdb 跨平台延迟** | Windows 原生支持预计 2026 年底 | 中 | PowerMem 默认使用 SQLite 后端，全平台可用 |
| **DeepAgents API Breaking** | Python Sidecar 代码需要跟进修改 | 中 | 固定 `deepagents==0.5.4`；跟随 Release Notes 增量升级；降级路径：回退手动 LangGraph |
| **DeepAgents Middleware 限制** | Middleware 架构约束了极端自定义需求 | 低 | 可混合使用 DeepAgents + 原生 LangGraph API 操控内部图；必要时 Fork Middleware |
| **SKILL.md 标准演进** | 开源标准仍在快速演进 | 低 | 遵循标准，参与社区，保持兼容 |
| **sqlite-vec 预 v1.0** | 截至 2026 年仍为 pre-v1.0 | 低 | API 已稳定，社区活跃 (7300+ stars)；降级到纯 Rust 余弦相似度 |

### 16.2 保留风险（与 v2.1 一致）

| 风险 | 应对策略 |
|------|---------|
| 跨平台 WebView 差异 | 充分测试 Windows/macOS/Linux |
| rmcp 成熟度 (Tier 2) | 跟进官方更新；备选 TS MCP SDK |
| Rust 学习曲线 | 核心层保持精简；复杂逻辑在 Python/前端 |

### 16.3 降级方案

**如果 Sidecar 预热持续失败：**
```
预热 → 按需启动（保留 v2.1 的延迟启动策略作为降级）
→ 用户首次对话有 1-2s 等待，但功能不受影响
```

**如果 DeepAgents 遇到严重问题：**
```
DeepAgents → 手动 LangGraph StateGraph（v2.1 方案作为降级路径）
```

**如果 PowerMem 遇到严重问题：**
```
PowerMem → SQLite + sqlite-vec (Rust, 全平台) + 手动记忆管理
```

**如果 Python Sidecar 整体不可行：**
```
Python Sidecar → TypeScript LangGraph.js (Node.js Sidecar)
Python PowerMem → SQLite + sqlite-vec (Rust)
```

---

## 17. 开发路线图

### 17.1 最终路线图 (v2.2)

| 阶段 | 时间 | 目标 | 关键技术决策 |
|------|------|------|-------------|
| **Phase 0: 环境搭建** | 1 周 | Tauri 项目脚手架、开发环境就绪、CI 配置 | Rust + React + Python 三端环境 |
| **Phase 1: 基础框架** | 3 周 | React UI 骨架、设置系统、主题/i18n、SQLite Schema | 纯 Rust+TS，无 Python |
| **Phase 2: 过渡对话系统** | 3 周 | Rig 流式对话、会话管理（含工作目录选择 UI）、消息持久化 | Rig 暂时承担对话功能；工作目录概念在此阶段建立 |
| **Phase 3: Sidecar 基础设施 + MCP** | 3 周 | Sidecar 预热管理器、FastAPI 骨架、MCP (rmcp)、会话增强 | 首次引入 Python Sidecar（预热但不接管对话） |
| **Phase 4: DeepAgents 全对话迁移** | 3 周 | DeepAgents 核心集成、PowerMem 记忆、SubAgent、HITL | Rig 对话功能退役；DeepAgents 接管所有对话 |
| **Phase 5: Skills + 知识库** | 4 周 | Skills 双层架构（Rust 发现 + DeepAgents 执行）、知识库 RAG | SkillsMiddleware + 知识库混合搜索 |
| **Phase 6: 高级功能 + Buddy + 发布** | 6 周 | 沙箱、Dashboard、Buddy 桌面伴侣、性能优化、跨平台打包 | 功能补全 + 打磨 |

**总预估周期：23 周（约 5-6 个月）**

**核心节奏：**
- Phase 0-2（前 7 周）：纯 Rust + React，交付可用的桌面 AI 客户端（Rig 过渡对话 + 工作目录选择 UI）
- Phase 3（第 8-10 周）：引入 Sidecar 预热基础设施，MCP 就绪，为 DeepAgents 铺路
- Phase 4（第 11-13 周）：DeepAgents 接管全对话，Rig 退居非对话角色
- Phase 5-6（第 14-23 周）：能力补全 + 打磨发布

> **渐进式过渡说明：** Phase 2 的 Rig 对话系统作为"过渡实现"，验证整个对话 UI 流（包括工作目录选择、流式渲染、会话管理）。Phase 4 时将对话后端从 Rig 替换为 DeepAgents，前端 UI 和会话基础设施基本不变。

---

## 18. 附录：关键决策记录

### 决策 1：全路径 DeepAgents 架构 🆕 (v2.2)

- **决策**：所有用户对话统一经过 DeepAgents harness，消除"简单对话"概念和双路径路由
- **原因**：用户体验割裂（无法预知请求被哪条路径处理）、路由误判风险、不符合 Agent 平台定位
- **影响**：Sidecar 必须预热（应用启动时），空闲内存增加 ~120MB；但每次对话都拥有完整 Agent 能力

### 决策 2：Sidecar 预热策略 🆕 (v2.2)

- **决策**：Python Sidecar 在应用启动时预启动并健康检查，保持常驻
- **原因**：全路径架构下所有对话经过 Sidecar，按需启动的 1-2s 冷启动延迟不可接受
- **影响**：应用启动时 Rust Core 异步启动 Sidecar，不阻塞 UI 渲染

### 决策 3：Rig 角色重定义 🆕 (v2.2)

- **决策**：Rig 仅用于非对话调用（Embedding 生成、会话标题/摘要、LLM 配置验证）
- **原因**：所有对话已迁移至 DeepAgents，Rig 的 LLM 直调能力在非对话场景仍有价值
- **影响**：Phase 2-3 过渡期 Rig 暂时承担对话功能，Phase 4 后正式退居非对话角色

### 决策 4：工作目录系统 🆕 (v2.2)

- **决策**：每会话绑定本地+远程工作目录，作为 P0 核心需求
- **原因**：工程型 Agent 平台的标志性特性；DeepAgents FilesystemMiddleware 基于工作目录运行
- **影响**：新增目录选择 UI、目录验证逻辑、安全白名单机制

### 决策 5：PowerMem 替代 LanceDB (v2.0)

- **决策**：采用 PowerMem 作为长期记忆引擎，放弃原 LanceDB 方案
- **原因**：PowerMem 提供语义级记忆管理（提取、去重、衰减），而 LanceDB 仅是向量存储
- **影响**：增加 Python Sidecar 依赖（PowerMem 是 Python 库），但长期记忆质量质变

### 决策 6：Skills 采用 SKILL.md 开源标准 (v2.0)

- **决策**：基于 agentskills.io 的 SKILL.md 标准，而非自研格式
- **原因**：26+ 平台支持、渐进式披露架构成熟、生态兼容
- **影响**：Skills 可跨平台复用，降低生态建设成本

### 决策 7：Nuitka 优先于 PyInstaller (v2.0)

- **决策**：Sidecar 打包优先使用 Nuitka（编译为 C），PyInstaller `--onedir` 为备选
- **原因**：PyInstaller `--onefile` 启动 6-20s 不可接受；Nuitka 编译为机器码启动更快
- **影响**：构建复杂性增加，但用户体验显著提升

### 决策 8：SQLite 保留为主数据库，seekdb 暂缓采用 (v2.1)

- **决策**：主数据库继续使用 SQLite (rusqlite)，不替换为 seekdb
- **原因**：seekdb 嵌入式模式 Windows 原生支持要到 2026 年底；桌面端必须三平台可用
- **影响**：需引入 sqlite-vec 和 FTS5 补全向量搜索和全文检索能力

### 决策 9：分层存储架构 (v2.1)

- **决策**：SQLite + sqlite-vec + FTS5 (Rust, 全平台) + PowerMem (Python, 常驻)
- **原因**：Rust 侧提供全平台可用的基础向量搜索和知识库；Python 侧提供高级语义记忆
- **影响**：两套存储系统，但职责分明。通过抽象 trait 为未来 seekdb 统一迁移预留空间

### 决策 10：知识库混合检索采用 RRF 融合 (v2.1)

- **决策**：sqlite-vec 向量搜索 + FTS5 全文检索 + RRF 融合排序
- **原因**：2026 年 local-first RAG 社区共识架构；单 SQLite 文件零运维
- **影响**：知识库功能不依赖任何外部服务，完全离线可用

---

> **报告结束**
>
> 本报告经过三个大版本迭代（v2.0 → v2.1 → v2.2），从最初的"大 Sidecar"方案，演进到"智能分层双路径路由"，
> 最终确定为"全路径 DeepAgents 单一架构"。
> 所有技术选型均经过多维度对比和可行性验证。
>
> v2.2 核心架构决策：
> - **全路径 DeepAgents 单一架构**：所有对话进入 DeepAgents harness，Agent 自主决策处理策略
> - **Sidecar 应用启动预热**：消除对话冷启动延迟
> - **Rig 退居非对话角色**：仅用于 Embedding、会话标题/摘要
> - **工作目录作为 P0 需求**：每会话绑定本地+远程目录
> - **渐进式实施**：Phase 2 Rig 作为过渡，Phase 4 DeepAgents 正式接管
>
> 综合评分：**9.3 / 10**

