# MisakaX 桌面端 — 最终架构选型文档

> **项目代号：** MisakaX（基于 Misaka 项目的下一代桌面端 AI Agent 客户端）
> **调研日期：** 2026-04-28
> **文档版本：** v2.1（v2.0 深度修订：新增知识库方案、数据库引擎深度选型）
> **目标：** 回答 7 个关键架构问题，给出最终技术栈决策

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
| **Agent 编排** | 支持多步骤工作流、条件分支、并行执行 | P0 |
| **SubAgent** | 支持子代理派生、任务委托、独立上下文 | P0 |
| **沙箱隔离** | 安全执行用户/Agent 生成的代码 | P1 |
| **持久化记忆** | 跨会话记忆检索、向量化存储、认知启发的衰减机制 | P0 |
| **Skills 系统** | 渐进式披露架构、可发现、安装、管理、执行 | P0 |
| **流式对话** | 支持实时 token 流式输出和思维链展示 | P0 |
| **高性能 UI** | Material Design 3 风格，流畅的交互体验 | P0 |
| **跨平台** | Windows / macOS / Linux 全平台支持 | P0 |
| **知识库** | 文档管理、全文检索、语义搜索、RAG 问答、混合检索 | P1 |
| **小体积分发** | 安装包尽可能小 | P1 |

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

## 5. Q4 架构模式深度剖析：Tauri 的角色与最佳性能方案

### 5.1 Tauri 的角色定位

用户的理解基本正确，但需要做一些精确化调整：

**Tauri 不是简单的"粘合剂"**，而是应用架构中的**系统集成层** + **IPC 总线** + **安全边界**。

```
┌─────────────────────────────────────────────────┐
│                  MisakaX Desktop App                │
│                                                  │
│  ┌────────────────────────────────────────────┐ │
│  │     React 19 WebView (表现层 + UI 状态)      │ │
│  └────────────────┬───────────────────────────┘ │
│                   │ Tauri IPC (0.12ms)           │
│  ┌────────────────┴───────────────────────────┐ │
│  │     Tauri Rust Core (系统集成层)              │ │
│  │                                             │ │
│  │  ┌─────────┐ ┌─────────┐ ┌──────────────┐  │ │
│  │  │ MCP Mgr │ │ DB Layer│ │ LLM Client   │  │ │
│  │  │ (rmcp)  │ │(rusqlite)│ │ (Rig)        │  │ │
│  │  └─────────┘ └─────────┘ └──────────────┘  │ │
│  │  ┌─────────┐ ┌─────────┐ ┌──────────────┐  │ │
│  │  │Sandbox  │ │Skill Mgr│ │Sidecar Mgr   │  │ │
│  │  │(wasmtime)│ │         │ │(shell plugin) │  │ │
│  │  └─────────┘ └─────────┘ └──────┬───────┘  │ │
│  └─────────────────────────────────┼──────────┘ │
│                                    │             │
│                   HTTP localhost   │             │
│  ┌─────────────────────────────────┼──────────┐ │
│  │     Python Sidecar (Agent 引擎层) │           │ │
│  │  ┌──────────┐ ┌───────────────┐ │           │ │
│  │  │LangGraph │ │ PowerMem      │ │           │ │
│  │  │(Agent编排)│ │ (长期记忆)     │ │           │ │
│  │  └──────────┘ └───────────────┘ │           │ │
│  └─────────────────────────────────┘           │
└─────────────────────────────────────────────────┘
```

**Tauri Rust Core 负责的"非 Agent"系统级功能：**
- 文件系统访问（tauri-plugin-fs）
- 进程管理（tauri-plugin-shell）
- 系统通知（tauri-plugin-notification）
- 剪贴板操作（tauri-plugin-clipboard）
- 原生对话框（tauri-plugin-dialog）
- 自动更新（tauri-plugin-updater）
- SQLite 数据库操作（rusqlite）
- MCP Client 管理（rmcp）
- LLM API 调用（Rig）— **轻量级直调**
- WASM 沙箱执行（Wasmtime）
- Skills 发现与生命周期管理

**Python Sidecar 负责的 "Agent 专属"功能：**
- 复杂多步骤 Agent 工作流编排（LangGraph）
- 长期记忆语义管理（PowerMem）
- SubAgent 派生与委托（LangGraph Subgraph）
- Human-in-the-loop 审批流程
- 高级 Tool 编排（条件分支、并行执行、Fan-out/Fan-in）

### 5.2 三种架构方案深度对比

#### 方案 A：大 Sidecar（原推荐方案）

```
React UI ↔ Rust Core (轻量) ↔ Python Sidecar (重，处理大部分逻辑)
```

| 优点 | 缺点 |
|------|------|
| Python AI 生态完整 | Sidecar 臃肿（打包 200MB+） |
| 开发效率高 | 大部分 LLM 调用穿过进程边界 |
| LangGraph 能力充沛 | 简单对话也要启动 Python |

#### 方案 B：纯 Rust/TS（无 Python Sidecar）

```
React UI ↔ Rust Core (Rig LLM + rmcp MCP + Mastra 编排)
```

| 优点 | 缺点 |
|------|------|
| 包体积最小 (~12MB) | Agent 编排能力受限 |
| 统一技术栈 | 记忆系统需手动实现 |
| 无跨进程开销 | Rust AI 生态不如 Python |

#### 方案 C：智能分层（🏆 新推荐方案）

```
                简单调用 ──▶ Rust Rig 直接处理（无 Sidecar 开销）
               /
React ↔ Rust Core
               \
                复杂编排 ──▶ Python Sidecar（仅按需唤醒）
```

**核心设计原则：按复杂度路由**

| 场景 | 处理层 | 延迟 |
|------|--------|------|
| 简单问答 / 单步工具调用 | Rust Rig (直调 LLM) | <100ms |
| 流式对话 | Rust Rig (SSE 转发) | 实时 |
| 多步工作流 / SubAgent | Python LangGraph Sidecar | 1-2s 启动 + 执行 |
| 记忆检索 | PowerMem MCP Server | ~1.4s (p95) |
| 沙箱代码执行 | Rust Wasmtime | <13ms |

### 5.3 方案 C 的关键技术决策

#### 路由逻辑（Rust 端）

```rust
// 伪代码：根据请求复杂度路由到不同执行路径
fn route_request(request: &AgentRequest) -> ExecutionTarget {
    match request.complexity {
        Complexity::Simple => ExecutionTarget::RustRig,        // 无 Sidecar
        Complexity::MultiStep => ExecutionTarget::PythonLangGraph, // 唤醒 Sidecar
        Complexity::SubAgent => ExecutionTarget::PythonLangGraph,
    }
}
```

#### 通信通道

```
Rust Rig (简单调用):
  React → Tauri IPC → Rig Client → LLM API → Stream Token → Tauri Event → React

Python LangGraph (复杂编排):
  React → Tauri IPC → Rust → HTTP POST localhost:18910 → FastAPI
  → LangGraph Graph Execution → SSE Stream → Rust 转发 → React
```

### 5.4 性能最大化方案总结

**用户提出的思路"只在需要时用 Python，其他用 Rust"是完全可行且推荐的最佳实践。**

具体实现：

| 模块 | 实现位置 | 理由 |
|------|---------|------|
| 简单 LLM 对话 | Rust (Rig) | 低延迟，无 Sidecar 开销 |
| 流式渲染 | React (Vercel AI SDK) | 原生流式 UI 支持 |
| MCP Client | Rust (rmcp) | 与进程管理天然集成 |
| 数据库 | Rust (rusqlite) | 原生性能 |
| Skills 管理 | Rust (文件系统扫描+注册) | 系统集成 |
| 沙箱 | Rust (Wasmtime) | 毫秒启动 |
| Agent 编排 | Python (LangGraph) | 只在多步工作流时调用 |
| 长期记忆 | Python (PowerMem) | 仅在需要检索/存储时调用 |
| Buddy 语音 | Rust (whisper-cpp-plus + piper-rs) | 性能敏感 |

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

### 8.2 结论：LangGraph (Python Sidecar) + Vercel AI SDK (前端)

保持原报告的推荐，但增加约束：

- **LangGraph** 仅用于复杂编排场景（按需唤醒 Sidecar）
- **Vercel AI SDK** 负责前端流式渲染（始终使用）
- **简单调用走 Rust Rig** 直接调用 LLM API（无 Sidecar）

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
│                    MisakaX Desktop App (v2)                       │
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
│  Tauri 2.x Core (Rust — 系统集成 + 轻量 LLM)                    │
│  ├── rig-core (简单 LLM 直调, 20+ 提供商)                       │
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
│  └── Tauri Plugins (shell, fs, http, notification, updater)   │
├──────────────────────────────────────────────────────────────┤
│  Agent Backend (Python Sidecar, 按需唤醒)                       │
│  ├── LangGraph 1.1+ (复杂 Agent 编排)                          │
│  ├── PowerMem (长期记忆引擎)                                    │
│  │   ├── seekdb (嵌入式混合搜索)                                │
│  │   ├── LLM 智能提取 + 去重 + 合并                             │
│  │   └── 艾宾浩斯遗忘曲线                                       │
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
| **Rig** | Rust 原生多 LLM 提供商统一接口、6.7K+ stars |
| **rmcp** | MCP 官方 Rust SDK、与 Tauri 无缝集成 |
| **rusqlite** | Rust 原生 SQLite、WAL 模式、零额外依赖 |
| **sqlite-vec** | 纯 C 向量扩展、零依赖、全平台、嵌入 SQLite |
| **SQLite FTS5** | 内建全文搜索引擎、BM25 排序、零额外依赖 |
| **Knowledge Base Engine** | 文档处理流水线 + 混合检索 RRF + 相关性门控 |
| **Wasmtime** | WASM 沙箱标准、<13ms 启动、资源限制完善 |
| **LangGraph** | 最成熟 Agent 编排、子图 SubAgent、仅复杂场景调用 |
| **PowerMem** | 语义记忆引擎、艾宾浩斯遗忘、LangGraph 原生集成、96% Token 节省 |
| **whisper-cpp-plus** | 离线语音识别、Rust 原生、高准确度 |
| **piper-rs** | 离线语音合成、900+ 声音、Rust 原生 |
| **Nuitka** | Python→C 编译、比 PyInstaller 启动更快 |

---

## 15. 最终架构选择

### 15.1 架构模式

```
🏆 最终选择：智能分层架构 (Hybrid Smart Routing)

React (UI)  ──Tauri IPC──▶  Rust Core  ──简单调用──▶  Rig → LLM API
                 ▲              │
                 │              ├──复杂编排──▶  Python Sidecar (LangGraph)
                 │              │                    │
                 │              │              ┌─────┴──────┐
                 │              │              │ PowerMem   │
                 │              │              │ 长期记忆    │
                 │              │              └────────────┘
                 │              │
                 │              ├──MCP 工具──▶  rmcp → MCP Servers
                 │              │
                 │              ├──知识库──▶  SQLite + sqlite-vec + FTS5
                 │              │
                 │              ├──代码执行──▶  Wasmtime 沙箱
                 │              │
                 │              └──Skills───▶  Skill Registry + Executor
                 │
                 └──Tauri Events── 流式 Token / 状态推送
```

### 15.2 分层职责明确矩阵

| 层 | 技术 | 职责范围 | 调用频率 |
|----|------|---------|---------|
| **UI 展示层** | React + shadcn/ui | 页面渲染、交互、动画 | 持续 |
| **UI 状态层** | Zustand + TanStack Query | 客户端状态、缓存 | 持续 |
| **AI UI 层** | Vercel AI SDK | 流式消息渲染 | 高频 |
| **IPC 总线** | Tauri invoke + Events | 前后端通信 | 持续 |
| **系统集成层** | Tauri Rust Core | FS、进程、通知、剪贴板 | 中频 |
| **LLM 轻量层** | Rig (Rust) | 简单问答、单步工具 | 高频 |
| **MCP 层** | rmcp (Rust) | MCP Client/Server 管理 | 中频 |
| **数据层** | rusqlite + sqlite-vec (Rust) | 结构化数据 + 向量搜索 | 高频 |
| **知识库层** | SQLite FTS5 + sqlite-vec (Rust) | 文档索引、混合检索、RAG | 中频 |
| **沙箱层** | Wasmtime (Rust) | 安全代码执行 | 低频 |
| **Skills 层** | Rust Manager + React UI | Skill 发现/注册/执行 | 低频 |
| **Agent 编排层** | LangGraph (Python) | 复杂多步工作流 | 低频（按需） |
| **长期记忆层** | PowerMem (Python) | 语义记忆管理 | 低频（按需） |
| **Buddy 语音层** | whisper + piper (Rust) | 语音识别与合成 | 中频（按需） |
| **Buddy 动画层** | Lottie/Live2D (React) | 角色动画渲染 | 持续 |

### 15.3 请求路由策略

```rust
// 请求复杂度判定规则
fn classify_request(req: &AgentRequest) -> ExecutionPath {
    if req.requires_multi_step_workflow() {
        return ExecutionPath::PythonSidecar;  // LangGraph 编排
    }
    if req.requires_subagent() {
        return ExecutionPath::PythonSidecar;  // LangGraph Subgraph
    }
    if req.requires_long_term_memory() {
        return ExecutionPath::PythonSidecar;  // PowerMem 检索
    }
    if req.requires_knowledge_base_search() {
        return ExecutionPath::RustKnowledgeBase;  // SQLite + sqlite-vec + FTS5
    }
    if req.requires_sandbox_execution() {
        return ExecutionPath::RustWasmtime;   // WASM 沙箱
    }
    ExecutionPath::RustRig  // 默认：Rust 直调 LLM
}
```

### 15.4 与原报告方案的关键差异

| 维度 | 原推荐 (v1.0) | v2.0 推荐 | v2.1 推荐 | 变化原因 |
|------|-------------|----------|----------|---------|
| **Architecture** | 大 Sidecar | 智能分层 | 智能分层 | 性能最大化 |
| **长期记忆** | LanceDB | PowerMem | PowerMem | 艾宾浩斯遗忘、智能提取 |
| **Skills 系统** | 未设计 | SKILL.md 标准 | SKILL.md 标准 | 补充缺口 |
| **Python 打包** | PyInstaller | Nuitka/--onedir | Nuitka/--onedir | 解决启动慢 |
| **主数据库** | SQLite (rusqlite) | SQLite (rusqlite) | SQLite + sqlite-vec + FTS5 | 增加向量搜索 |
| **知识库** | N/A | N/A | 🆕 完整方案 | 新增核心功能 |
| **向量引擎** | LanceDB (Rust) | 移除 (PowerMem替代) | sqlite-vec (Rust) + PowerMem (Python) | 分层处理 |
| **seekdb 评估** | N/A | N/A | 🆕 深度调研，暂缓采用 | 跨平台限制 |
| **Sidecar 启动** | 随应用启动 | 按需延迟唤醒 | 按需延迟唤醒 | 减少资源占用 |

### 15.5 最终综合评分

| 评估维度 | v2.1 评分 | v2.0 评分 | v1.0 评分 | 变化说明 |
|---------|----------|----------|----------|---------|
| **性能** | 9.5 | 9.5 | 9.5 | — |
| **包体积** | 8.5 | 8.5 | 8.0 | — |
| **Agent 编排完整性** | 9.0 | 9.0 | 9.0 | — |
| **记忆系统成熟度** | 9.5 | 9.5 | 7.0 | PowerMem |
| **Skills 系统完整性** | 9.0 | 9.0 | 5.0 | 补充设计 |
| **知识库完整性** | 9.0 | N/A | N/A | 🆕 全新功能 |
| **数据库方案稳健性** | 9.5 | 7.0 | 7.0 | sqlite-vec 解决向量需求 |
| **跨平台覆盖** | 9.0 | 9.0 | 9.0 | SQLite 全平台 |
| **可维护性** | 9.0 | 9.0 | 8.5 | 分层清晰 |
| **架构灵活性** | 9.5 | 9.5 | 7.5 | 智能路由 + 抽象 trait |
| **综合评分** | **9.2 / 10** | **9.1 / 10** | **8.8 / 10** | **+0.1** |

---

## 16. 风险评估与应对策略

### 16.1 新增风险（v2.1 特有）

| 风险 | 影响 | 概率 | 应对策略 |
|------|------|------|---------|
| **PowerMem 稳定性** | 作为较新项目 (2025.11 首发)，API 可能变动 | 中 | 固定版本号；维护降级适配层；关注 GitHub Release |
| **seekdb 跨平台延迟** | Windows 原生支持预计 2026 年底，macOS 刚刚发布可能不稳定 | 中 | SQLite + sqlite-vec 作为全平台主力；seekdb 仅用于 Python Sidecar |
| **PowerMem/seekdb Windows 兼容** | PowerMem 内部依赖 seekdb，Windows 原生不可用 | 中 | PowerMem 支持 SQLite 后端作为备选；短期 Windows 用户可 Docker/WSL2 |
| **智能路由误判** | 简单请求错误路由到 Sidecar 或反之 | 低 | 保守策略（默认走 Rust）；加降级开关 |
| **Python Sidecar 按需启动延迟** | 首次复杂请求时 1-2s Sidecar 启动延迟 | 中 | 预热策略：后台预启动，首屏等待时不明显 |
| **SKILL.md 标准演进** | 开源标准 (agentskills.io) 仍在快速演进 | 低 | 遵循标准，参与社区，保持兼容 |
| **sqlite-vec 预 v1.0** | 截至 2026 年 4 月仍为 pre-v1.0 | 低 | API 已稳定，社区活跃 (7300+ stars)；降级到纯 JS 余弦相似度 |

### 16.2 保留风险（与原报告一致）

| 风险 | 应对策略 |
|------|---------|
| 跨平台 WebView 差异 | 充分测试 Windows/macOS/Linux |
| rmcp 成熟度 (Tier 2) | 跟进官方更新；备选 TS MCP SDK |
| Rust 学习曲线 | 核心层保持精简；复杂逻辑在 Python/前端 |
| LangGraph 版本更新 | 固定版本，增量升级 |

### 16.3 降级方案

**如果 PowerMem 遇到严重问题：**
```
PowerMem → SQLite + sqlite-vec (Rust, 全平台) + 手动记忆管理
```

**如果 Python Sidecar 性能不可接受：**
```
Python LangGraph → Mastra (TypeScript) 或 LangGraph.js
Python PowerMem → SQLite + sqlite-vec (Rust, 知识库兼做记忆)
```

**如果 sqlite-vec 遇到跨平台问题：**
```
sqlite-vec → 纯 JS/Rust 余弦相似度计算 (退化为暴力搜索，功能不受影响)
```

**如果 seekdb 提前完成 Windows 支持：**
```
评估 SQLite + sqlite-vec → seekdb 迁移可行性 (通过抽象 trait 切换)
```

---

## 17. 开发路线图

### 17.1 最终路线图

| 阶段 | 时间 | 目标 | 关键技术决策 |
|------|------|------|-------------|
| **Phase 1: 基础框架** | 4-6 周 | Tauri 项目搭建、React UI 骨架、Rig 直调 LLM、SQLite | 纯 Rust+TS，无 Python |
| **Phase 2: 核心功能** | 6-8 周 | 流式对话、会话管理、MCP (rmcp)、设置、主题/i18n | Rust 继续承担全部后端 |
| **Phase 3: Skills 系统** | 3-4 周 | SKILL.md 标准实现、Skill Manager、前端 Skill UI | Rust + React，独立模块 |
| **Phase 4: 知识库系统** | 4-6 周 | 文档导入、sqlite-vec 集成、FTS5、混合检索 RRF、RAG 问答 | 🆕 SQLite + sqlite-vec + FTS5 |
| **Phase 5: Python Sidecar 集成** | 4-6 周 | LangGraph Sidecar、PowerMem 集成、智能路由 | 首次引入 Python |
| **Phase 6: Agent 增强** | 4-6 周 | SubAgent、Human-in-the-loop、记忆系统完善 | Python Agent 编排完善 |
| **Phase 7: Buddy 基础** | 3-4 周 | 透明悬浮窗、Lottie 动画、气泡对话 | 独立 Buddy 窗口 |
| **Phase 8: Buddy 语音** | 3-4 周 | STT (whisper-cpp-plus)、TTS (piper-rs)、语音交互 | Rust 语音引擎集成 |
| **Phase 9: 高级功能** | 3-4 周 | WASM 沙箱、Dashboard、文件浏览、代码高亮 | Rust 沙箱 + React 展示 |
| **Phase 10: 打磨发布** | 4-6 周 | 性能优化、跨平台测试、自动更新、打包发布 | 全栈性能调优 |

**总预估周期：38-54 周（9-13 个月）**

> 注：相比 v2.0 的 34-48 周，增加了知识库系统（Phase 4，4-6 周）。Phase 1-4（前 17-24 周）即可交付一个含知识库的完整纯 Rust+TS 桌面 AI 客户端（含 MCP、Skills、知识库、基础对话），无需 Python Sidecar。

---

## 18. 附录：关键决策记录

### 决策 1：Python Sidecar 定位

- **决策**：Python 不作为默认执行路径，仅处理复杂 Agent 编排和长期记忆
- **原因**：90% 的用户请求是简单问答或单步工具调用，无需 Python 的编排能力
- **影响**：前端 90% 的请求不穿过进程边界，延迟更低

### 决策 2：PowerMem 替代 LanceDB

- **决策**：采用 PowerMem 作为长期记忆引擎，放弃原 LanceDB 方案
- **原因**：PowerMem 提供语义级记忆管理（提取、去重、衰减），而 LanceDB 仅是向量存储
- **影响**：增加 Python Sidecar 依赖（PowerMem 是 Python 库），但长期记忆质量质变

### 决策 3：Skills 采用 SKILL.md 开源标准

- **决策**：基于 agentskills.io 的 SKILL.md 标准，而非自研格式
- **原因**：26+ 平台支持、渐进式披露架构成熟、生态兼容
- **影响**：Skills 可跨平台复用，降低生态建设成本

### 决策 4：Nuitka 优先于 PyInstaller

- **决策**：Sidecar 打包优先使用 Nuitka（编译为 C），PyInstaller `--onedir` 为备选
- **原因**：PyInstaller `--onefile` 启动 6-20s 不可接受；Nuitka 编译为机器码启动更快
- **影响**：构建复杂性增加，但用户体验显著提升

### 决策 5：SQLite 保留为主数据库，seekdb 暂缓采用 🆕

- **决策**：主数据库继续使用 SQLite (rusqlite)，不替换为 seekdb
- **原因**：seekdb 嵌入式模式 Windows 原生支持要到 2026 年底；桌面端必须三平台可用
- **影响**：需引入 sqlite-vec 和 FTS5 补全向量搜索和全文检索能力

### 决策 6：分层存储架构 🆕

- **决策**：SQLite + sqlite-vec + FTS5 (Rust, 全平台) + PowerMem (Python, 按需)
- **原因**：Rust 侧提供全平台可用的基础向量搜索和知识库；Python 侧提供高级语义记忆
- **影响**：两套存储系统，但职责分明。通过抽象 trait 为未来 seekdb 统一迁移预留空间

### 决策 7：知识库混合检索采用 RRF 融合 🆕

- **决策**：sqlite-vec 向量搜索 + FTS5 全文检索 + RRF 融合排序
- **原因**：2026 年 local-first RAG 社区共识架构；单 SQLite 文件零运维
- **影响**：知识库功能不依赖任何外部服务，完全离线可用

---

> **报告结束**
>
> 本报告基于 2026 年 4 月的最新技术生态进行深度调研，结合 PowerMem、seekdb 跨平台分析、
> SKILL.md 标准、sqlite-vec 向量搜索、Tauri Sidecar 社区实践等最新信息，对
> MISAKAX_TECH_SELECTION_REPORT.md (v1.0) 进行了重大修订和扩展。
> 所有技术选型均经过多维度对比和可行性验证。
>
> 最终架构选择：**智能分层 Hybrid Smart Routing (React + Tauri Rust Core + Python Sidecar on-demand)**
>
> 知识库方案：**SQLite + sqlite-vec + FTS5 (全平台嵌入式混合检索)**
>
> 综合评分：**9.2 / 10**

