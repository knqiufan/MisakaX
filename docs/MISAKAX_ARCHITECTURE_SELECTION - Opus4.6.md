# MisakaX 项目架构选型文档 v2.1

> **项目代号：** MisakaX（基于 Misaka 项目的下一代桌面端 AI Agent 客户端）
> **文档版本：** v2.1
> **调研日期：** 2026-04-28
> **基于：** [MISAKAX_TECH_SELECTION_REPORT.md v1.0](./MISAKAX_TECH_SELECTION_REPORT.md) 的深入修订
> **目标：** 针对 v1.0 报告中遗漏和不够深入的关键问题，补充调研并给出最终技术决策

---

## 目录

1. [核心问题深度分析](#1-核心问题深度分析)
2. [Tauri Sidecar 模式深度剖析](#2-tauri-sidecar-模式深度剖析)
3. [运行环境与依赖矩阵](#3-运行环境与依赖矩阵)
4. [PowerMem 记忆引擎调研](#4-powermem-记忆引擎调研)
5. [Rust 最大化架构方案](#5-rust-最大化架构方案)
6. [Skills 系统完整技术方案](#6-skills-系统完整技术方案)
7. [Buddy 桌面伴侣架构选型](#7-buddy-桌面伴侣架构选型)
8. [数据库与知识库存储方案](#8-数据库与知识库存储方案)
9. [修订后的综合技术栈](#9-修订后的综合技术栈)
10. [修订后的架构设计](#10-修订后的架构设计)
11. [最终技术决策](#11-最终技术决策)

---

## 1. 核心问题深度分析

本文档针对 v1.0 报告中以下 8 个核心问题进行深入调研：

| # | 问题 | v1.0 覆盖度 | v2.x 处理 |
|---|------|-----------|----------|
| 1 | Tauri Sidecar 模式是什么？Python Sidecar 对性能有何影响？ | 仅概述，无性能量化 | 深度剖析原理与性能基准 |
| 2 | 运行整个技术架构需要什么环境？ | 未覆盖 | 完整的环境依赖矩阵 |
| 3 | PowerMem 是否适合作为长期记忆引擎？是否兼容 LangGraph？ | 未涉及 | 完整调研 + 对比分析 |
| 4 | 能否最大化用 Rust，仅在 Agent 相关场景使用 Python？ | 提及但未深入 | 分层职责划分 + 最佳实践 |
| 5 | Skills 系统的加载与使用方案在选型中缺失 | 未覆盖 | 完整技术方案 |
| 6 | Buddy 桌面伴侣功能的架构选型 | v1.0 有深度调研但未纳入 v2.0 | **[v2.1 新增]** 完整架构选型 |
| 7 | SeekDB 嵌入式能否替换 SQLite？跨平台兼容性如何？ | 未涉及 | **[v2.1 新增]** 深度调研 + 结论 |
| 8 | 知识库功能需要什么向量存储方案？ | 未涉及 | **[v2.1 新增]** 方案对比 + 最终决策 |

---

## 2. Tauri Sidecar 模式深度剖析

### 2.1 什么是 Sidecar 模式

Tauri Sidecar 是 Tauri 框架提供的 **外部二进制嵌入机制**，允许将任意可执行文件（Python/Go/Node.js 编译产物）与 Tauri 应用一起打包分发。Sidecar 本质上是一个 **由 Tauri Rust 主进程管理生命周期的子进程**。

```
┌─────────────────────────────────────────────────────────────┐
│                     Tauri 应用进程结构                         │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐    │
│  │  主进程 (Rust Core)                                    │    │
│  │  ├── WebView 管理 (前端 UI 渲染)                        │    │
│  │  ├── IPC 消息路由                                       │    │
│  │  ├── Tauri Commands (暴露给前端的 API)                   │    │
│  │  ├── 系统原生 API 调用                                   │    │
│  │  └── Sidecar 生命周期管理器 ─────────────────┐          │    │
│  └──────────────────────────────────────────────┤          │    │
│                                                  │          │    │
│  ┌──────────────────────────────────────────────┤          │    │
│  │  子进程 A (Python Sidecar)                    ◄── spawn  │    │
│  │  ├── 独立的操作系统进程                                   │    │
│  │  ├── 独立的内存空间                                       │    │
│  │  ├── 通过 stdin/stdout 或 HTTP 与主进程通信                │    │
│  │  └── 可被主进程启动 / 停止 / 健康检查 / 重启               │    │
│  └───────────────────────────────────────────────────────┘    │
│                                                               │
│  ┌───────────────────────────────────────────────────────┐    │
│  │  子进程 B (其他 Sidecar, 如 MCP Server)                 │    │
│  └───────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

**核心机制：**

1. **打包阶段**：通过 `tauri.conf.json` 的 `bundle.externalBin` 字段声明外部二进制文件路径
2. **分发阶段**：Tauri 构建工具自动按平台后缀（如 `agent-server-x86_64-pc-windows-msvc.exe`）匹配并打包
3. **运行阶段**：通过 `tauri-plugin-shell` 的 `app.shell().sidecar("name")` 启动子进程
4. **通信阶段**：主进程与 Sidecar 通过 stdin/stdout 管道或 localhost HTTP/WebSocket 通信
5. **终止阶段**：主窗口关闭时，Tauri 自动清理所有 Sidecar 子进程

### 2.2 性能影响量化分析

Sidecar 模式引入的性能开销来自三个维度：**进程启动**、**通信延迟**、**资源占用**。

#### 2.2.1 进程间通信 (IPC) 延迟

| 通信方式 | 单次延迟 | 适用场景 | 吞吐量 |
|---------|---------|---------|--------|
| **Tauri invoke()** (Rust 直调) | 0.12 ms | 前端 → Rust 后端 | 极高 |
| **stdin/stdout 管道** | 0.3 - 0.8 ms | Rust → Sidecar 简单命令 | 高 |
| **localhost HTTP** | 1 - 3 ms | Rust → Sidecar REST API | 中高 |
| **localhost WebSocket** | 0.5 - 1.5 ms | 双向流式通信 | 高 |
| **WebView Bridge** (JS→Rust) | 1 - 5 ms | 前端 → Rust Core | 中 |

> **对比基线**：直接在 Rust 内部函数调用的延迟约为 0.001 ms（纳秒级），因此 Sidecar 通信引入的额外开销约为 **1-5ms 量级**。

#### 2.2.2 对 AI Agent 场景的实际影响

关键认知：**Sidecar 的 IPC 延迟相对于 LLM API 调用延迟可以忽略不计**。

| 操作 | 典型延迟 | Sidecar 额外开销占比 |
|------|---------|-------------------|
| LLM API 调用 (首 Token) | 500 - 3,000 ms | < 0.3% |
| LLM 完整响应 | 2,000 - 30,000 ms | < 0.1% |
| MCP 工具调用 | 50 - 500 ms | 1 - 5% |
| 向量搜索 (记忆检索) | 10 - 100 ms | 3 - 15% |
| 文件 I/O 操作 | 5 - 50 ms | 5 - 30% |

**结论：对于 Agent 编排、LLM 调用、记忆管理等 AI 相关任务，Sidecar 引入的 1-3ms IPC 延迟在整个请求链路中占比极小，不构成性能瓶颈。**

#### 2.2.3 资源占用影响

| 资源指标 | 纯 Rust (无 Sidecar) | + Python Sidecar | 增量 |
|---------|---------------------|-----------------|------|
| 空闲内存 | ~42 MB | ~120 - 200 MB | +80 - 160 MB |
| 磁盘占用 (安装包) | ~10 MB | ~60 - 90 MB | +50 - 80 MB |
| 冷启动时间 | ~380 ms | ~1,200 - 2,000 ms | +800 - 1,600 ms |
| CPU 空闲占用 | < 0.1% | < 0.5% | 可忽略 |

> Python Sidecar 通过 PyInstaller/Nuitka 编译后的体积和启动时间是主要代价。使用 Nuitka AOT 编译可将启动时间降低至 ~600ms，体积降低至 ~40MB。

#### 2.2.4 优化策略

| 策略 | 效果 | 复杂度 |
|------|------|--------|
| **延迟启动**：仅在需要 Agent 功能时才启动 Sidecar | 主窗口秒开，无感知延迟 | 低 |
| **预热连接**：启动后立即建立 WebSocket 长连接 | 消除首次请求的连接建立开销 | 低 |
| **连接池**：复用 HTTP 连接 | 减少 TCP 握手开销 | 低 |
| **Nuitka 替代 PyInstaller** | 启动快 30-50%，体积小 20-30% | 中 |
| **消息批处理**：合并多个小请求 | 减少 IPC 次数 | 中 |
| **共享内存**：大数据通过 mmap 传递 | 避免序列化大对象 | 高 |

### 2.3 Sidecar 模式的核心价值

| 价值 | 说明 |
|------|------|
| **生态复用** | 直接使用 Python 丰富的 AI/ML 库（LangGraph、PowerMem 等），无需 Rust 重写 |
| **关注点分离** | Rust 负责高性能系统层，Python 专注 AI Agent 业务逻辑 |
| **独立迭代** | Agent 逻辑可独立于 UI 进行开发和测试 |
| **故障隔离** | Sidecar 崩溃不影响主进程，可自动重启 |
| **无需用户安装 Python** | PyInstaller/Nuitka 打包为独立可执行文件 |

---

## 3. 运行环境与依赖矩阵

### 3.1 开发环境要求

| 组件 | 必需/可选 | 最低版本 | 用途 |
|------|---------|---------|------|
| **Rust 工具链** | 必需 | 1.82+ (stable) | Tauri 后端编译 |
| **Node.js** | 必需 | 20 LTS+ | 前端构建 (Vite)、npm 包管理 |
| **Python** | 必需 | 3.11+ | Agent Sidecar 开发 |
| **系统 WebView** | 必需 | WebView2 (Win) / WebKit (Mac/Linux) | 前端渲染 |
| **Git** | 必需 | 2.x | 版本管理 |
| **C/C++ 编译器** | 必需 | MSVC (Win) / Xcode CLT (Mac) / gcc (Linux) | Rust 原生依赖编译 |

#### 各平台详细要求

**Windows：**
```
- Windows 10 1803+ (WebView2 内置于 Windows 11)
- Visual Studio Build Tools 2022+ (含 C++ 工作负载)
- Rust: rustup 安装，stable 工具链
- Node.js 20+ (通过 nvm-windows 或官方安装包)
- Python 3.11+ (通过官方安装包或 pyenv-win)
- WebView2 运行时 (Windows 10 需手动安装，Windows 11 已内置)
```

**macOS：**
```
- macOS 10.15 Catalina+ (WKWebView)
- Xcode Command Line Tools: xcode-select --install
- Rust: rustup 安装
- Node.js 20+ (通过 nvm 或 Homebrew)
- Python 3.11+ (通过 pyenv 或 Homebrew)
```

**Linux：**
```
- WebKitGTK 4.1+ 和相关系统库:
  sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file
  sudo apt install libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
- Rust: rustup 安装
- Node.js 20+ (通过 nvm)
- Python 3.11+ (通常系统自带或通过 pyenv)
```

### 3.2 用户运行环境要求（安装包分发后）

| 组件 | Windows | macOS | Linux |
|------|---------|-------|-------|
| **系统版本** | Win 10 1803+ | macOS 10.15+ | Ubuntu 20.04+ 或同等 |
| **WebView** | WebView2 (自动安装) | WKWebView (内置) | WebKitGTK 4.1 |
| **额外依赖** | 无 | 无 | 可能需安装 WebKitGTK |
| **Python** | 无 (Sidecar 自带) | 无 (Sidecar 自带) | 无 (Sidecar 自带) |
| **磁盘空间** | ~120 MB | ~100 MB | ~110 MB |
| **内存推荐** | 4 GB+ | 4 GB+ | 4 GB+ |

> 最终用户 **无需安装 Rust、Node.js 或 Python**。所有依赖通过 Tauri 构建流程和 Sidecar 打包完整嵌入。

### 3.3 Sidecar Python 环境构建

Agent Sidecar 的 Python 环境在构建阶段通过以下方式打包为独立可执行文件：

| 打包工具 | 产物体积 | 启动速度 | 兼容性 | 推荐度 |
|---------|---------|---------|--------|-------|
| **Nuitka** | ~40-55 MB | ~600 ms | 良好 | ⭐⭐⭐⭐⭐ |
| **PyInstaller** | ~55-80 MB | ~1,000-1,500 ms | 极好 | ⭐⭐⭐⭐ |
| **cx_Freeze** | ~50-70 MB | ~800 ms | 良好 | ⭐⭐⭐ |

**推荐 Nuitka**：作为 AOT 编译器，Nuitka 将 Python 代码编译为 C 代码再编译为机器码，性能和启动速度优于 PyInstaller 的解释打包模式。

---

## 4. PowerMem 记忆引擎调研

### 4.1 PowerMem 项目概述

| 指标 | 详情 |
|------|------|
| **项目方** | OceanBase 团队 (阿里巴巴旗下) |
| **GitHub Stars** | ~620 |
| **许可证** | Apache 2.0 |
| **语言** | Python |
| **最新版本** | v1.1.0 (2026 年初) |
| **定位** | AI 应用的智能长期记忆管理系统 |

### 4.2 核心架构

PowerMem 实现了一个受认知科学启发的五层记忆架构：

```
┌──────────────────────────────────────────────────────┐
│ External Layer: Multi-Agents & Users                  │
└───────────────────────┬──────────────────────────────┘
                        ↓
┌──────────────────────────────────────────────────────┐
│ API Layer                                             │
│ ├── Python SDK (pip install powermem)                 │
│ ├── CLI (pmem) — 命令行交互式 Shell                    │
│ ├── HTTP API Server — 带 Dashboard 的 Web 服务         │
│ └── MCP Server — 标准化协议接口                         │
└───────────────────────┬──────────────────────────────┘
                        ↓
┌──────────────────────────────────────────────────────┐
│ Core Layer (Memory Engine)                            │
│ ├── Memory Lifecycle Management                      │
│ │   ├── 艾宾浩斯遗忘曲线算法 (R = e^(-t/S))           │
│ │   └── 时间/频率强化学习                               │
│ ├── Intelligent Memory Processor                     │
│ │   ├── 记忆添加/更新/查询/压缩                         │
│ │   └── AI 驱动的重要性评估                             │
│ └── Layered Memory Structure                         │
│     ├── 工作记忆 (Working Memory)                     │
│     ├── 短期记忆 (Short-term Memory)                  │
│     ├── 长期记忆 (Long-term Memory)                   │
│     └── 共享记忆 (Shared Memory)                      │
└───────────────────────┬──────────────────────────────┘
                        ↓
┌──────────────────────────────────────────────────────┐
│ Model Layer                                           │
│ ├── Embedding: Qwen / OpenAI / HuggingFace / Ollama │
│ └── LLM: Qwen / OpenAI / Anthropic / DeepSeek       │
└───────────────────────┬──────────────────────────────┘
                        ↓
┌──────────────────────────────────────────────────────┐
│ Storage Layer                                         │
│ ├── SQLite (本地开发默认)                              │
│ ├── SeekDB (嵌入式 OceanBase, v1.1.0+)               │
│ ├── PostgreSQL (pgvector)                             │
│ └── OceanBase (企业级生产推荐)                          │
└──────────────────────────────────────────────────────┘
```

### 4.3 PowerMem 的核心优势

| 能力 | 详情 | 对 MisakaX 的价值 |
|------|------|--------------|
| **艾宾浩斯遗忘曲线** | 基于 `R = e^(-t/S)` 实现记忆衰减、强化、遗忘 | 智能记忆管理，自动清理无关记忆 |
| **重要性智能评估** | LLM 驱动的多维度重要性评分 (0.0-1.0) | 自动区分高价值记忆和临时信息 |
| **混合检索** | 向量搜索 + 全文搜索 + 知识图谱 | 多维度精准记忆召回 |
| **多 Agent 隔离** | 每个 Agent 独立记忆空间 + 跨 Agent 共享 | SubAgent 记忆管理 |
| **多模态** | 文本 + 图片 + 音频记忆 | Buddy 语音交互记忆 |
| **SQLite 本地模式** | 默认 SQLite 存储，零外部依赖 | 桌面端开箱即用 |
| **MCP Server** | 标准化记忆访问接口 | 可作为 MCP 工具集成 |
| **LOCOMO 基准** | 78.7 准确度 vs 全上下文 52.9 (+25.8) | 超越"全部塞进上下文"的暴力方案 |

### 4.4 与 LangGraph 的集成可行性

**结论：完全兼容，官方已提供集成示例。**

PowerMem 官方仓库中包含 `examples/langgraph/` 目录，演示了使用 PowerMem + LangGraph + OceanBase 构建 AI 客服机器人的完整示例。

集成方式有两种：

**方式 A：作为 LangGraph 的外部记忆存储（推荐）**

```python
from powermem import Memory, create_memory
from langgraph.graph import StateGraph

memory = create_memory()

async def memory_node(state):
    user_id = state["user_id"]
    query = state["messages"][-1].content

    # 检索相关记忆
    relevant = memory.search(query, user_id=user_id, limit=5)

    # 将记忆注入上下文
    state["context"] = format_memories(relevant)
    return state

async def post_response_node(state):
    # 从对话中自动提取并存储新记忆
    memory.add(
        state["messages"][-1].content,
        user_id=state["user_id"],
        metadata={"session_id": state["session_id"]}
    )
    return state
```

**方式 B：通过 MCP Server 接入**

PowerMem 提供 MCP Server 模式，可以作为 MCP 工具被 Agent 直接调用，实现记忆的读写操作。

### 4.5 与竞品对比

| 维度 | PowerMem | Mem0 | Zep/Graphiti | LangMem |
|------|----------|------|-------------|---------|
| **GitHub Stars** | ~620 | ~47.8K | ~2.6K | ~1.5K |
| **架构模式** | 向量+全文+知识图谱 | 向量+知识图谱+KV | 时序知识图谱 | 扁平 KV+向量 |
| **遗忘曲线** | ✅ 内建 | ❌ | ❌ | ❌ |
| **本地 SQLite** | ✅ 默认支持 | ❌ 需要服务端 | ❌ 需要 Neo4j+PostgreSQL | ✅ |
| **LangGraph 集成** | ✅ 官方示例 | ✅ 适配器 | ✅ 适配器 | ✅ 原生 |
| **MCP Server** | ✅ 内建 | ❌ | ❌ | ❌ |
| **多 Agent 隔离** | ✅ 内建 | ✅ 付费 Pro | ✅ | ❌ |
| **多模态** | ✅ 文本/图片/音频 | ❌ 文本 | ❌ 文本 | ❌ 文本 |
| **免费/开源** | ✅ Apache 2.0 | ⚠️ 图谱记忆需付费 | ✅ MIT | ✅ MIT |
| **桌面端友好度** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| **LongMemEval 分数** | N/A | 49% | 63.8% | N/A |
| **LOCOMO 分数** | 78.7 | N/A | N/A | N/A |

### 4.6 PowerMem 的潜在风险

| 风险 | 影响 | 应对策略 |
|------|------|---------|
| **社区规模小** (~620 stars) | 问题响应可能较慢 | OceanBase 团队背书，企业级支持；代码开源可自行维护 |
| **OceanBase 优化偏重** | 高级特性依赖 OceanBase | SQLite 模式已满足桌面端需求，无需 OceanBase |
| **Python 依赖** | 必须在 Python Sidecar 中运行 | 正好与 LangGraph Sidecar 共享运行环境 |
| **Embedding 模型需求** | 需要 Embedding 服务 | 可使用 Ollama 本地模型或云端 API |

### 4.7 PowerMem 采用建议

**推荐采用 PowerMem 作为 MisakaX 的长期记忆引擎。** 理由：

1. **艾宾浩斯遗忘曲线**：在所有竞品中独有，最贴近人类记忆的管理方式
2. **SQLite 本地模式**：桌面端无需额外数据库服务，开箱即用
3. **MCP Server 模式**：可同时作为 Agent 工具和独立服务使用
4. **与 LangGraph 无缝集成**：官方示例已验证
5. **多 Agent 隔离**：天然支持 SubAgent 的独立记忆空间
6. **多模态支持**：为 Buddy 语音交互提供记忆基础
7. **完全开源**：Apache 2.0，无付费墙

**替换 v1.0 中的 LanceDB 方案**：v1.0 推荐 LanceDB 仅提供原始的向量搜索能力，而 PowerMem 在此基础上增加了智能记忆管理（衰减、强化、压缩、分层），是更完整的记忆解决方案。LanceDB 仍可作为 PowerMem 的底层向量存储 adapter 使用。

---

## 5. Rust 最大化架构方案

### 5.1 核心理解

对于 v1.0 报告中的架构：**Tauri 确实是"粘合剂"角色，前端 React 负责 UI，Python Sidecar 负责 Agent 逻辑，Rust Core 居中协调**。但这并非唯一选择。

v1.0 的架构可以概括为：

```
React (UI) ←→ Tauri Rust (中间层/粘合剂) ←→ Python (Agent 大脑)
```

你提出的"Rust 最大化"思路不仅可行，而且从 2026 年 Rust AI Agent 生态的发展来看，是一个性能上限更高的方案。

### 5.2 2026 年 Rust vs Python Agent 性能基准

根据 2026 Q1 的独立基准测试数据：

| 指标 | Rust (AutoAgents/Rig) | Python (LangGraph/CrewAI) | 差异 |
|------|---------------------|-------------------------|------|
| **峰值内存 (单 Agent)** | ~1.1 GB | ~5.1 GB | Rust 低 5x |
| **50 实例内存** | ~51 GB | ~272 GB (LangGraph) | Rust 低 5.3x |
| **平均延迟** | 5,714 ms | 6,046 - 10,155 ms | Rust 低 25-44% |
| **冷启动** | ~180 ms | 3,200 - 5,800 ms | Rust 快 18-32x |
| **吞吐量** | ~2,400 tasks/s | ~180 tasks/s | Rust 高 13x |
| **二进制体积** | ~22 MB (单文件) | N/A (需 Python 环境) | 结构性优势 |

> 注意：这里的延迟差异主要来自框架自身开销，不包含 LLM API 的网络延迟。在实际 Agent 使用中，LLM API 延迟 (500-3000ms) 占主导，框架本身的差异会被大幅稀释。

### 5.3 分层职责划分方案（推荐）

**核心原则：Rust 承担一切能用 Rust 高效实现的部分，Python 仅承担 Rust 生态目前不成熟的 Agent 编排层。**

```
┌─────────────────────────────────────────────────────────────┐
│                     职责分层矩阵                               │
├──────────────┬──────────────────────────────────────────────┤
│ 完全 Rust    │ • MCP Client (rmcp, 官方 Tier 2 SDK)         │
│ 实现的部分    │ • 数据库存储 (rusqlite, SQLite WAL)            │
│              │ • 文件系统操作                                  │
│              │ • 系统通知 / 剪贴板 / 对话框                     │
│              │ • WASM 沙箱 (Wasmtime)                        │
│              │ • LLM 直调 (rig-core, 简单请求)                │
│              │ • 应用状态管理                                   │
│              │ • 进程管理 / Sidecar 生命周期                    │
│              │ • 权限控制 / ACL                                │
│              │ • 安全模型 (Tauri 权限 ACL)                     │
│              │ • 语音引擎 (whisper-cpp, piper-rs)             │
│              │ • Skills 注册/发现/生命周期                      │
├──────────────┼──────────────────────────────────────────────┤
│ Python       │ • Agent 图编排 (LangGraph StateGraph)         │
│ Sidecar      │ • 复杂工作流 (条件分支/并行/子图)                │
│ 仅处理       │ • SubAgent 管理 (LangGraph 子图)               │
│              │ • Human-in-the-loop 流程                       │
│              │ • 长期记忆引擎 (PowerMem)                       │
│              │ • 记忆智能管理 (衰减/强化/压缩)                   │
├──────────────┼──────────────────────────────────────────────┤
│ React 前端   │ • UI 渲染 / 交互                               │
│ 仅处理       │ • 流式消息展示 (Vercel AI SDK)                  │
│              │ • 状态管理 (Zustand)                            │
│              │ • Skills UI (浏览/安装/管理)                     │
│              │ • 主题系统 / 国际化                              │
└──────────────┴──────────────────────────────────────────────┘
```

### 5.4 三种架构方案对比

#### 方案 Alpha：Rust 最大化 + Python 最小化（推荐）

```
React UI ←→ Tauri Rust Core (承担大部分逻辑) ←→ Python Sidecar (仅 Agent 编排+记忆)
                    ↕
            Rig (LLM 直调)
            rmcp (MCP Client)
            rusqlite (数据库)
            Wasmtime (沙箱)
            Skills Manager
```

| 优势 | 劣势 |
|------|------|
| 性能最优的折中方案 | Rust 层代码量较大 |
| Python 仅做最擅长的事 | 需维护 Rust + Python 两套代码 |
| 安装包 ~60 MB (Python 部分被压缩到最小) | Agent 编排调试需跨进程 |
| 启动快 (Rust 部分即时就绪，Python 延迟启动) | |
| LLM 直调无需经过 Sidecar | |

#### 方案 Beta：完全 Python Sidecar（v1.0 原方案）

```
React UI ←→ Tauri Rust Core (薄中间层) ←→ Python Sidecar (所有后端逻辑)
```

| 优势 | 劣势 |
|------|------|
| Rust 层最简单 | 所有请求都经过 IPC |
| Python 开发效率高 | 安装包 80+ MB |
| AI 库选择最多 | 冷启动慢 |

#### 方案 Gamma：纯 Rust + TypeScript（无 Python）

```
React UI ←→ Tauri Rust Core (所有后端逻辑)
                  ↕
           Rig (LLM)
           rmcp (MCP)
           Ractor (Multi-Agent)
           自研记忆系统
```

| 优势 | 劣势 |
|------|------|
| 安装包最小 (~15 MB) | 无法使用 PowerMem、LangGraph |
| 性能最高 | Agent 编排能力有限 (Rig 无内建多 Agent) |
| 技术栈最统一 | 需要大量自研，开发周期长 |
| 冷启动最快 (~380 ms) | Rust AI Agent 生态仍在成长期 |

### 5.5 最佳实践建议

**推荐方案 Alpha：Rust 最大化 + Python 最小化。** 理由：

1. **渐进式架构**：初期可以从更多 Python 开始，逐步将稳定模块迁移到 Rust
2. **LLM 直调 via Rig**：简单的 LLM 对话（无需复杂编排）直接在 Rust 端完成，无 IPC 开销
3. **MCP 原生 Rust**：rmcp 是官方 Tier 2 SDK，MCP Client 在 Rust 端实现性能最优
4. **Python 做 Python 最擅长的事**：复杂 Agent 编排（LangGraph）和智能记忆（PowerMem）是 Python 生态的强项
5. **延迟启动策略**：Python Sidecar 仅在用户触发 Agent 功能时启动，普通设置/浏览等操作纯 Rust 处理

**数据流路径选择逻辑：**

```
用户发送消息
    │
    ├── 简单对话 (无工具、无编排)?
    │   └── Rig (Rust 端直调 LLM)  ← 0 IPC 开销
    │
    ├── 需要 MCP 工具调用?
    │   └── rmcp (Rust 端 MCP Client) → Rig 编排 ← 0 IPC 开销
    │
    └── 需要复杂编排 (多步骤/SubAgent/条件分支/记忆)?
        └── Python Sidecar (LangGraph + PowerMem) ← ~2ms IPC 开销
```

---

## 6. Skills 系统完整技术方案

### 6.1 Skills 在 v1.0 中的缺失

v1.0 报告第 7.2 节仅给出了 Skills 的目录结构草图，但在第 10 节"综合技术栈推荐"和第 13 节"结论与最终推荐"中 **完全未体现 Skills 系统的技术选型**。Skills 作为 P1 优先级的核心功能，需要完整的技术方案。

### 6.2 Skills 系统需求分析

基于 Misaka 现有 Skills 功能和 Claude Code Skills 标准：

| 需求 | 说明 | 优先级 |
|------|------|--------|
| **Skill 发现** | 从多个目录源 + 市场 API 发现 Skills | P0 |
| **Skill 加载** | 解析 SKILL.md frontmatter + body，注册到系统 | P0 |
| **Skill 注入** | 将可用 Skill 列表注入 LLM 系统提示词 | P0 |
| **Skill 执行** | 用户显式调用 (/skill-name) 或 Agent 自动匹配触发 | P0 |
| **Skill 安装** | 从市场下载、验证、注册到本地 | P1 |
| **Skill 管理** | 启用/禁用、更新、卸载、配置 | P1 |
| **Skill 沙箱** | Skill 中的可执行逻辑在受控环境中运行 | P1 |
| **Skill 上下文修改** | Skill 可指定需要读取的项目文件作为上下文 | P1 |
| **条件激活** | Skill 基于触发条件 (triggers) 自动激活 | P2 |

### 6.3 SKILL.md 标准格式

沿用 Claude Code 社区的 SKILL.md 标准格式：

```yaml
---
name: my-skill
description: "这个 Skill 的功能描述"
triggers:
  - "当用户请求做某事时"
  - "当需要处理某种类型的任务时"
tools:
  - "Read"
  - "Write"
  - "Shell"
model: "claude-sonnet-4-20250514"
temperature: 0.3
---

你是一个专门处理 XXX 任务的助手。

## 输入格式
用户会提供...

## 输出格式
你应该返回...

## 约束
1. ...
2. ...

## 示例
输入: ...
输出: ...
```

### 6.4 Skills 加载架构

```
┌──────────────────────────────────────────────────────────────┐
│                    Skills Loading Pipeline                      │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  1. Discovery (发现层) — Rust 实现                        │   │
│  │                                                          │   │
│  │  来源优先级 (高 → 低):                                     │   │
│  │  ① 内建 Skills    — 随应用打包的默认技能                    │   │
│  │  ② 托管 Skills    — ~/.claw/managed/skills/              │   │
│  │  ③ 用户全局 Skills — ~/.claw/skills/                      │   │
│  │  ④ 项目 Skills    — .claw/skills/ (向上遍历到 HOME)       │   │
│  │  ⑤ 插件 Skills    — ~/.claw/plugins/*/skills/            │   │
│  │  ⑥ 市场 Skills    — 通过 API 远程发现                      │   │
│  │                                                          │   │
│  │  扫描策略: glob *.md → 过滤有效 SKILL.md → 按路径去重      │   │
│  └──────────────────────────┬────────────────────────────────┘   │
│                              ↓                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  2. Parsing (解析层) — Rust 实现                           │   │
│  │                                                           │   │
│  │  ├── YAML Frontmatter 解析 (serde_yaml)                   │   │
│  │  │   ├── name (必需) → 作为 /slash-command 标识            │   │
│  │  │   ├── description (必需) → 用于语义匹配                  │   │
│  │  │   ├── triggers (可选) → 自动激活条件                      │   │
│  │  │   ├── tools (可选) → 允许使用的工具列表                   │   │
│  │  │   ├── model (可选) → 覆盖默认模型                        │   │
│  │  │   └── temperature (可选) → 覆盖默认温度                  │   │
│  │  └── Markdown Body → 作为 System Prompt 注入                │   │
│  └──────────────────────────┬────────────────────────────────┘   │
│                              ↓                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  3. Registry (注册层) — Rust 实现                          │   │
│  │                                                           │   │
│  │  SkillRegistry {                                          │   │
│  │    skills: HashMap<String, Skill>,   // name → Skill      │   │
│  │    triggers: Vec<(TriggerPattern, SkillName)>,             │   │
│  │  }                                                        │   │
│  │                                                           │   │
│  │  ├── 缓存策略: 使用 memoize 避免重复磁盘 I/O               │   │
│  │  ├── 热重载: 监听目录变化，增量更新注册表                     │   │
│  │  └── 去重: 同名 Skill 按来源优先级覆盖                      │   │
│  └──────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

### 6.5 Skills 执行流程

```
┌─────────────────────────────────────────────────────────────────┐
│                     Skills Execution Pipeline                     │
│                                                                   │
│  触发方式:                                                         │
│  ├── 显式: 用户输入 /skill-name → 直接匹配                         │
│  └── 隐式: Agent 分析用户意图 → 语义匹配 triggers → 自动推荐       │
│                                                                   │
│  执行流程:                                                         │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │ 1. Skill 匹配                                            │     │
│  │    ├── 精确匹配: /skill-name                              │     │
│  │    └── 语义匹配: triggers + description 向量相似度         │     │
│  └────────────────────────┬─────────────────────────────────┘     │
│                            ↓                                       │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │ 2. 权限检查                                               │     │
│  │    ├── Skill 声明的 tools 是否被允许?                       │     │
│  │    ├── Skill 是否来自受信任的来源?                           │     │
│  │    └── 用户是否授权了该 Skill?                              │     │
│  └────────────────────────┬─────────────────────────────────┘     │
│                            ↓                                       │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │ 3. 上下文准备                                              │     │
│  │    ├── 注入 Skill Body 作为 System Prompt                  │     │
│  │    ├── 如果 Skill 指定了项目文件，自动读取并注入             │     │
│  │    ├── 注入用户的原始消息作为 User Prompt                   │     │
│  │    └── 覆盖 model / temperature (如果 Skill 指定了)         │     │
│  └────────────────────────┬─────────────────────────────────┘     │
│                            ↓                                       │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │ 4. Fork 执行 (独立上下文)                                   │     │
│  │    ├── 创建独立的对话上下文 (不污染主对话)                    │     │
│  │    ├── 使用 Skill 指定的 model + temperature                │     │
│  │    ├── 限制可用工具为 Skill 声明的 tools                     │     │
│  │    └── 执行并收集结果                                        │     │
│  └────────────────────────┬─────────────────────────────────┘     │
│                            ↓                                       │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │ 5. 结果合并                                                │     │
│  │    ├── Skill 输出注入回主对话上下文                          │     │
│  │    └── UI 展示 Skill 执行结果                               │     │
│  └──────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────┘
```

### 6.6 Skills 技术实现方案

| 模块 | 实现语言 | 技术选型 | 说明 |
|------|---------|---------|------|
| **Skill 发现** | Rust | `walkdir` + `glob` | 文件系统扫描，高性能 |
| **YAML 解析** | Rust | `serde_yaml` + `gray_matter` | Frontmatter 解析 |
| **Skill 注册表** | Rust | `DashMap` (并发 HashMap) | 线程安全的注册表 |
| **触发匹配** | Rust/Python | Rig embedding (Rust) / PowerMem (Python) | 简单匹配 Rust，语义匹配可走 Python |
| **权限控制** | Rust | Tauri 权限 ACL | 与 Tauri 安全模型集成 |
| **Skill 执行** | Rust → Python | 简单 Skill 在 Rust 端直接注入 prompt；复杂 Skill 通过 Sidecar 编排 | 分级处理 |
| **市场 API** | Rust | `reqwest` (HTTP Client) | Skyll 市场或自建 API |
| **Skill UI** | React | shadcn/ui 组件 | 浏览、安装、管理界面 |
| **热重载** | Rust | `notify` (文件系统监听) | 目录变化自动刷新 |

### 6.7 Skills 与 Agent 编排的集成

Skills 在 Agent 编排中有两种使用方式：

**方式 1：Prompt 注入式（轻量，Rust 端处理）**

Skill 的 body 作为 System Prompt 片段，在 LLM 调用前注入。适用于大多数 Skill。

```
System Prompt = 基础 System Prompt
              + Skill Listing (可用 Skill 清单)
              + Active Skill Body (如果当前激活了某个 Skill)
              + 用户上下文
```

**方式 2：工作流编排式（重量，Python Sidecar 处理）**

Skill 作为 LangGraph 图中的一个节点参与编排。适用于需要多步骤执行、工具调用的复杂 Skill。

```python
# LangGraph 中的 Skill 执行节点
async def skill_execution_node(state):
    skill = skill_registry.get(state["active_skill"])
    result = await execute_skill_in_context(
        skill=skill,
        user_message=state["messages"][-1],
        available_tools=skill.tools
    )
    state["messages"].append(result)
    return state
```

---

## 7. Buddy 桌面伴侣架构选型

> v1.0 报告第 14 节已对 Buddy 系统进行了深度调研（透明窗口、动画方案、语音系统、AI 集成）。本节将其核心技术选型纳入 v2.1 的统一架构决策。

### 7.1 Buddy 系统定位

Buddy 不是一个独立应用，而是 MisakaX 主应用的 **独立透明浮窗**，通过 Tauri 多窗口 API 与主窗口通信，共享同一个 Rust Core 后端。

```
┌─────────────────────────────────┐
│         MisakaX 主窗口              │          ┌──────────────┐
│  ┌───────────────────────────┐  │          │  Buddy 窗口   │
│  │   Chat / Dashboard /      │  │  Event   │  (透明浮窗)    │
│  │   Settings / Skills       │◄─┼─────────►│  🐱 角色动画   │
│  └───────────────────────────┘  │  系统     │  💬 对话气泡   │
└─────────────────────────────────┘          │  🎤 语音输入   │
                                             └──────────────┘
            两窗口共享 Tauri Rust Core 后端
```

### 7.2 Buddy 技术选型决策

| 模块 | 最终选型 | 备选方案 | 选择理由 |
|------|---------|---------|---------|
| **窗口管理** | Tauri 多窗口 (transparent + alwaysOnTop) | — | 原生支持透明背景、置顶、点击穿透 |
| **默认动画** | Lottie (`lottie-react`) | Rive | <50KB，CPU 友好，LottieFiles 大量免费资源 |
| **高级动画** | Live2D (`pixi-live2d-display` + PixiJS) | 3D VRM (Three.js) | 眼球追踪、口型同步、表情映射，2D 性价比最高 |
| **语音识别 (STT)** | `whisper-cpp-plus` (Rust) | tauri-plugin-stt (Vosk) | 高准确度、多语言、Rust 原生集成 |
| **语音合成 (TTS)** | `piper-rs` (Rust) | edge-tts (在线) | 离线、Rust 原生、900+ 声音、~200ms 延迟 |
| **音频采集** | `cpal` (Rust) | — | 跨平台麦克风采集标准库 |
| **音频播放** | `rodio` (Rust) | — | Rust 音频播放标准库 |
| **语音唤醒** | 全局快捷键 (初期) → Silero VAD (后续) | Porcupine | 快捷键最简可靠，VAD 后续渐进加入 |
| **情绪驱动** | LLM 返回结构化 JSON (emotion 字段) | 独立情感分析模型 | 零额外模型开销，直接复用 Agent LLM |
| **窗口通信** | Tauri Event (`emit` / `listen`) | IPC Channel | Tauri 原生事件系统，简洁可靠 |

### 7.3 Buddy 与 Agent 系统集成

Buddy 不是独立的 AI 系统，而是 MisakaX Agent 系统的一个 **交互前端**：

```
语音 / 文字输入
    │
    ↓
┌─────────────────────────────────────────────────┐
│ Buddy 窗口 (React)                                │
│ ├── 语音录制 → Tauri invoke("transcribe")         │
│ └── 文字输入 → Tauri invoke("buddy_chat")         │
└──────────────────────┬──────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────┐
│ Tauri Rust Core                                   │
│ ├── whisper-cpp-plus → 语音转文字                  │
│ ├── Command Router → 同一套路由逻辑                │
│ │   ├── 简单对话 → Rig (Rust LLM 直调)            │
│ │   └── 复杂任务 → Python Sidecar (LangGraph)     │
│ ├── 结果返回: { text, emotion, actions[] }        │
│ ├── piper-rs → 文字转语音                          │
│ └── Tauri Event → Buddy 窗口更新动画 + 播放语音    │
└─────────────────────────────────────────────────┘
```

### 7.4 Buddy 资源开销评估

| 资源 | 空闲态 | 语音处理态 | 说明 |
|------|--------|-----------|------|
| **内存 (动画)** | ~20 MB | ~20 MB | 透明 WebView + Lottie |
| **内存 (STT 模型)** | 0 (延迟加载) | ~150 MB | Whisper base 模型 |
| **内存 (TTS 模型)** | 0 (延迟加载) | ~80 MB | Piper medium 模型 |
| **CPU** | < 1% | 10 - 30% | 仅语音处理期间高占用 |
| **磁盘 (模型)** | — | ~270 MB | STT + TTS 模型文件，首次使用时下载 |

**关键优化**：STT/TTS 模型采用延迟加载策略，仅在用户首次使用语音功能时加载到内存。

### 7.5 Buddy 开发阶段

| 阶段 | 时间 | 核心交付物 |
|------|------|-----------|
| Phase 1 | 2-3 周 | 透明浮窗 + Lottie 待机/拖拽动画 + 气泡对话框 |
| Phase 2 | 2-3 周 | 迷你输入框 + LLM 对话 + 情绪动画切换 |
| Phase 3 | 2-3 周 | Whisper STT + 快捷键触发 + 实时转录 |
| Phase 4 | 1-2 周 | Piper TTS + 口型状态切换 |
| Phase 5 | 3-4 周 | MCP 工具调用 + 人设系统 + 多形象 |
| Phase 6 (可选) | 3-4 周 | Live2D 高级模式 + 眼球追踪 |

**Buddy 可与主应用 Phase 2-5 并行开发，共享 Rust Core 后端。**

---

## 8. 数据库与知识库存储方案

### 8.1 问题背景

v2.0 文档中数据库层选择了 `rusqlite (SQLite WAL)`，记忆层选择了 `PowerMem`（默认 SQLite 模式）。但存在以下未回答的问题：

1. 能否用 SeekDB（OceanBase 嵌入式）统一替换所有 SQLite？
2. 知识库功能（RAG）需要向量搜索，SQLite 方案是否需要额外的向量库？
3. 如果 SeekDB 能同时提供关系数据 + 向量搜索 + 全文搜索，是否是更优选择？

### 8.2 SeekDB 嵌入式深度调研

#### 8.2.1 SeekDB 是什么

SeekDB 是 OceanBase 团队推出的 **AI 原生混合数据库**，基于 OceanBase 引擎构建，在单一引擎中同时支持关系型数据、向量搜索、全文搜索、JSON 和 GIS 数据。

| 指标 | 详情 |
|------|------|
| **项目方** | OceanBase (阿里巴巴旗下) |
| **最新版本** | v1.2.0 (2026-04-15) |
| **Python SDK** | `pyseekdb` (pip install pyseekdb) |
| **核心能力** | 关系型 + 向量搜索 + 全文搜索 + 图搜索，单引擎 |
| **嵌入式模式** | 作为 Python 库内嵌运行，零配置 |

#### 8.2.2 跨平台兼容性（关键问题）

| 平台 | 嵌入式模式 | 服务器模式 | 备注 |
|------|-----------|-----------|------|
| **Linux** | ✅ 完全支持 | ✅ | glibc >= 2.28，x86_64 / aarch64 |
| **macOS** | ✅ 支持 | ✅ | 通过 pyseekdb |
| **Windows** | ❌ **不支持** | ⚠️ 需 WSL2 | **这是核心障碍** |

> **关键发现：SeekDB 嵌入式模式在 Windows 上不可用。** Windows 用户只能通过 WSL2 运行服务器模式，或连接远程 OceanBase 实例。

#### 8.2.3 SeekDB 在 Windows 上不可用的影响分析

MisakaX 的核心需求之一是 **P0 优先级的跨平台支持 (Windows / macOS / Linux)**。如果采用 SeekDB 作为唯一数据库：

- macOS 和 Linux 用户：可以使用嵌入式模式，体验良好
- **Windows 用户**：必须安装 WSL2 并运行一个 SeekDB 服务端进程，严重破坏"开箱即用"的用户体验

**结论：SeekDB 嵌入式模式无法作为 MisakaX 的统一数据库方案，因为它不满足 Windows 跨平台需求。**

#### 8.2.4 macOS 上的已知问题

根据 GitHub Issue #174，SeekDB 嵌入式模式在 macOS 上也曾出现兼容性问题（部分版本无法正常工作），虽然后续版本已修复，但稳定性仍需关注。

### 8.3 知识库功能的向量存储需求

知识库（RAG）功能要求数据库支持：

| 能力 | 必要性 | 说明 |
|------|--------|------|
| **向量存储** | 必需 | 存储文档的 Embedding 向量 |
| **向量相似度搜索 (KNN)** | 必需 | 检索与查询最相似的文档片段 |
| **全文搜索 (FTS)** | 强需求 | 关键词搜索，与向量搜索互补 |
| **混合搜索 (Hybrid)** | 强需求 | 向量 + 全文联合排序 (RRF) |
| **元数据过滤** | 需要 | 按来源、日期、标签等过滤 |
| **跨平台** | 必需 | Windows / macOS / Linux |
| **嵌入式** | 必需 | 桌面端无需外部服务 |

### 8.4 方案对比：SQLite + sqlite-vec vs SeekDB vs LanceDB

| 维度 | SQLite + sqlite-vec + FTS5 | SeekDB (嵌入式) | LanceDB (Rust 嵌入式) |
|------|--------------------------|----------------|---------------------|
| **Windows** | ✅ 完全支持 | ❌ 不支持 | ✅ 完全支持 |
| **macOS** | ✅ 完全支持 | ✅ 支持 (有过兼容问题) | ✅ 完全支持 |
| **Linux** | ✅ 完全支持 | ✅ 完全支持 | ✅ 完全支持 |
| **向量搜索** | ✅ float32/int8/binary | ✅ HNSW/IVFFLAT | ✅ HNSW, IVF_PQ |
| **全文搜索** | ✅ FTS5 (SQLite 内建) | ✅ 内建 | ✅ Tantivy |
| **混合搜索** | ✅ RRF 手动实现 | ✅ 内建 RRF | ✅ 内建 |
| **关系型数据** | ✅ 完整 SQL | ✅ MySQL 兼容 | ❌ 仅 KV + 向量 |
| **Rust 集成** | ✅ rusqlite + sqlite-vec crate | ❌ 仅 Python SDK | ✅ lancedb crate (原生) |
| **Python 集成** | ✅ sqlite3 标准库 | ✅ pyseekdb | ✅ lancedb Python SDK |
| **单一数据库** | ✅ 一个 .db 文件 | ✅ 一个数据目录 | ❌ 独立 Lance 目录 |
| **额外依赖** | sqlite-vec 扩展 (~200KB) | pyseekdb (~大型) | lancedb crate |
| **成熟度** | ⭐⭐⭐⭐⭐ SQLite 最成熟 | ⭐⭐⭐ 较新 | ⭐⭐⭐⭐ 活跃 |
| **社区规模** | SQLite: 全球最广泛 | ~700 stars | ~9.8K stars |
| **桌面端案例** | 已有 Tauri + sqlite-vec 实战 | 无桌面端案例 | 少量 |

### 8.5 推荐方案：SQLite (rusqlite) + sqlite-vec + FTS5

#### 8.5.1 方案架构

```
┌────────────────────────────────────────────────────────────┐
│              MisakaX 统一存储层 (单一 SQLite 数据库文件)          │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  结构化数据 (rusqlite, 标准 SQL 表)                     │  │
│  │  ├── sessions (会话)                                   │  │
│  │  ├── messages (消息)                                   │  │
│  │  ├── tasks (任务)                                      │  │
│  │  ├── router_configs (路由配置)                          │  │
│  │  ├── settings (设置)                                   │  │
│  │  └── skills_metadata (Skills 元数据)                    │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  向量搜索 (sqlite-vec, vec0 虚拟表)                     │  │
│  │  ├── knowledge_vectors (知识库文档向量)                  │  │
│  │  │   └── vec0(embedding float[1536])                   │  │
│  │  └── memory_vectors (记忆向量, 供 Rust 端检索)           │  │
│  │      └── vec0(embedding float[1536])                   │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  全文搜索 (FTS5, fts5 虚拟表)                           │  │
│  │  ├── knowledge_fts (知识库全文索引)                      │  │
│  │  └── messages_fts (消息全文索引)                         │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  混合搜索: 通过 RRF (Reciprocal Rank Fusion) 合并两个索引     │
│  的结果，使用 FULL OUTER JOIN + CTE 实现                     │
└────────────────────────────────────────────────────────────┘
```

#### 8.5.2 选择理由

1. **全平台支持**：SQLite 是世界上部署最广泛的数据库，Windows / macOS / Linux 零问题
2. **单文件架构**：所有数据（结构化 + 向量 + 全文索引）在一个 `.db` 文件中，备份和迁移极简
3. **Rust 原生**：`rusqlite` + `sqlite-vec` crate 均可直接在 Tauri Rust 后端使用，无需 Python
4. **已验证**：已有开发者在 Tauri 应用中成功实现 SQLite + sqlite-vec 的 RAG 系统
5. **极小开销**：sqlite-vec 扩展编译为纯 C，无外部依赖，~200KB
6. **与 PowerMem 互补**：Rust 端 sqlite-vec 负责知识库 RAG，Python 端 PowerMem 负责智能记忆管理，职责清晰

#### 8.5.3 Rust 端集成示例

```rust
use rusqlite::Connection;
use sqlite_vec::sqlite3_vec_init;

fn setup_database(db: &Connection) -> Result<()> {
    // 加载 sqlite-vec 扩展
    unsafe {
        let ext = std::mem::transmute(sqlite3_vec_init as usize);
        rusqlite::auto_extension::register_auto_extension(ext)?;
    }

    // 创建知识库向量表
    db.execute_batch("
        CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_vectors
        USING vec0(embedding float[1536]);

        CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_fts
        USING fts5(title, content, source);
    ")?;

    Ok(())
}

fn hybrid_search(db: &Connection, query_vec: &[f32], query_text: &str, limit: usize)
    -> Result<Vec<SearchResult>>
{
    // RRF 混合搜索: 向量 + 全文
    db.prepare("
        WITH vec_results AS (
            SELECT rowid, distance, ROW_NUMBER() OVER (ORDER BY distance) AS rank
            FROM knowledge_vectors
            WHERE embedding MATCH ?1
            ORDER BY distance LIMIT ?3
        ),
        fts_results AS (
            SELECT rowid, rank AS fts_rank, ROW_NUMBER() OVER (ORDER BY rank) AS rank
            FROM knowledge_fts
            WHERE knowledge_fts MATCH ?2
            LIMIT ?3
        )
        SELECT COALESCE(v.rowid, f.rowid) AS id,
               (COALESCE(1.0 / (60 + v.rank), 0) * 0.7 +
                COALESCE(1.0 / (60 + f.rank), 0) * 0.3) AS rrf_score
        FROM vec_results v
        FULL OUTER JOIN fts_results f ON v.rowid = f.rowid
        ORDER BY rrf_score DESC
        LIMIT ?3
    ")?.query_map(params![query_vec.as_bytes(), query_text, limit], ...)?
}
```

#### 8.5.4 知识库数据流

```
用户导入文档 (PDF/MD/TXT/DOCX)
    │
    ↓
┌─────────────────────────────────────────────────────────┐
│ Rust Core: 文档处理流水线                                  │
│ ├── 1. 文档解析 (提取纯文本)                               │
│ ├── 2. 智能分块 (按语义边界切分 chunks)                     │
│ ├── 3. Embedding 生成                                     │
│ │   ├── Rig → 云端 Embedding API (OpenAI / Qwen)         │
│ │   └── 或 本地 Embedding 模型 (Ollama)                   │
│ ├── 4. 向量存入 sqlite-vec (knowledge_vectors)             │
│ ├── 5. 文本存入 FTS5 (knowledge_fts)                      │
│ └── 6. 元数据存入 SQL 表 (knowledge_docs)                  │
└─────────────────────────────────────────────────────────┘

用户提问 → 知识库检索
    │
    ↓
┌─────────────────────────────────────────────────────────┐
│ Rust Core: 混合检索                                       │
│ ├── 1. 查询文本 → Embedding 向量                          │
│ ├── 2. sqlite-vec KNN 搜索 (向量相似度)                    │
│ ├── 3. FTS5 全文搜索 (关键词匹配)                          │
│ ├── 4. RRF 融合排序                                       │
│ └── 5. Top-K 结果注入 LLM 上下文                           │
└─────────────────────────────────────────────────────────┘
```

### 8.6 SeekDB 的定位：PowerMem 的可选升级存储

虽然 SeekDB 不适合作为 MisakaX 的统一数据库，但它仍然有价值：

**PowerMem 的存储后端升级路径：**

```
PowerMem 存储选择:
├── 默认: SQLite (开箱即用, 全平台)
├── 进阶: SeekDB 嵌入式 (仅 macOS / Linux 用户)
│   └── 优势: 混合检索更强、图搜索、企业级性能
└── 生产: OceanBase 服务端 (团队/企业部署)
```

在 MisakaX 的设置界面中，可以为 macOS / Linux 用户提供一个可选的"切换到 SeekDB 模式"选项，以获得 PowerMem 更强的混合检索能力。但默认方案必须是 SQLite，以确保 Windows 用户的开箱即用体验。

### 8.7 数据库方案最终决策

| 用途 | 方案 | 技术 | 说明 |
|------|------|------|------|
| **应用主数据库** | SQLite (WAL) | rusqlite (Rust) | 会话/消息/任务/设置等结构化数据 |
| **知识库向量搜索** | sqlite-vec | sqlite-vec crate (Rust) | 与主数据库同一文件，向量 KNN 搜索 |
| **知识库全文搜索** | FTS5 | SQLite 内建 | 与主数据库同一文件，关键词搜索 |
| **知识库混合搜索** | RRF 融合 | SQL CTE (Rust) | 向量 + 全文联合排序 |
| **智能记忆 (PowerMem)** | SQLite (默认) / SeekDB (可选) | Python Sidecar | 默认 SQLite 全平台；macOS/Linux 可选 SeekDB |
| **Agent 状态持久化** | SQLite | langgraph-checkpoint-sqlite | LangGraph Checkpointer |

---

## 9. 修订后的综合技术栈

### 9.1 技术栈总览（v2.1 修订版）

与 v1.0 的关键差异用 **[NEW]** / **[CHANGED]** 标注：

```
┌───────────────────────────────────────────────────────────────────┐
│                        MisakaX Desktop App v2.1                        │
├───────────────────────────────────────────────────────────────────┤
│  Frontend (WebView)                                                 │
│  ├── React 19 + TypeScript 5.x                                     │
│  ├── Tailwind CSS v4 + shadcn/ui                                   │
│  ├── Zustand (状态管理)                                              │
│  ├── TanStack Query (服务器状态)                                     │
│  ├── Vercel AI SDK (流式 AI UI)                                     │
│  ├── Monaco Editor (代码编辑/预览)                                   │
│  ├── react-i18next (国际化)                                         │
│  ├── lottie-react (Buddy 默认动画)                                   │
│  └── Vite (构建工具)                                                 │
├───────────────────────────────────────────────────────────────────┤
│  Tauri 2.x Core (Rust) — 承担主要后端职责                             │
│  ├── rmcp (MCP Client, 官方 Rust SDK)                               │
│  ├── rusqlite (SQLite 数据库, WAL 模式)                              │
│  ├── [v2.1] sqlite-vec (向量搜索扩展, 知识库 RAG)                    │
│  ├── rig-core (LLM API 直调, 简单对话+MCP 工具调用)                   │
│  ├── wasmtime (WASM 沙箱, 安全代码执行)                              │
│  ├── Skills Manager (发现/解析/注册/执行)                             │
│  │   ├── serde_yaml (YAML frontmatter 解析)                         │
│  │   ├── walkdir + glob (文件系统扫描)                               │
│  │   ├── DashMap (并发 Skill 注册表)                                 │
│  │   └── notify (文件系统监听, 热重载)                                │
│  ├── [v2.1] whisper-cpp-plus (Buddy 语音识别 STT)                    │
│  ├── [v2.1] piper-rs (Buddy 语音合成 TTS)                           │
│  ├── [v2.1] cpal + rodio (音频采集 + 播放)                           │
│  ├── serde / serde_json (序列化)                                    │
│  ├── tokio (异步运行时)                                              │
│  ├── tauri-plugin-shell (Sidecar 管理)                              │
│  ├── tauri-plugin-fs (文件系统)                                      │
│  ├── tauri-plugin-http (HTTP 客户端)                                 │
│  ├── tauri-plugin-notification (系统通知)                             │
│  └── tauri-plugin-updater (自动更新)                                 │
├───────────────────────────────────────────────────────────────────┤
│  Agent Backend (Python Sidecar) — 仅 Agent 编排+记忆                 │
│  ├── LangGraph 1.1.x (Agent 图编排, SubAgent 子图)                   │
│  ├── FastAPI (HTTP API, WebSocket 流式)                              │
│  ├── PowerMem (长期记忆引擎)                                         │
│  │   ├── SQLite 本地模式 (默认, 全平台)                               │
│  │   ├── SeekDB 嵌入式 (可选, macOS/Linux)                          │
│  │   ├── 艾宾浩斯遗忘曲线记忆管理                                     │
│  │   └── 混合检索 (向量+全文+图谱)                                    │
│  ├── langchain-anthropic / langchain-openai (LLM 适配)               │
│  ├── langgraph-checkpoint-sqlite (图状态持久化)                       │
│  └── Nuitka (AOT 编译为独立可执行文件)                                │
└───────────────────────────────────────────────────────────────────┘
```

### 9.2 v1.0 → v2.1 变更清单

| 变更 | v1.0 | v2.1 | 理由 |
|------|------|------|------|
| **记忆系统** | LanceDB (纯向量搜索) | PowerMem (智能记忆引擎) | 遗忘曲线 + 多层记忆 + 智能管理 |
| **知识库向量** | 未涉及 | sqlite-vec + FTS5 | **[v2.1]** 全平台嵌入式混合搜索 |
| **Buddy 选型** | v1.0 调研未纳入 | 完整技术决策 | **[v2.1]** Lottie + whisper + piper |
| **数据库** | rusqlite | rusqlite + sqlite-vec | **[v2.1]** 统一 SQLite 架构 |
| **Python 打包** | PyInstaller | Nuitka | 启动快 30-50%，体积小 20-30% |
| **Rust 职责** | 薄中间层 | 承担大部分后端逻辑 | 性能最大化 |
| **Skills 方案** | 仅目录草图 | 完整技术方案 | v1.0 缺失 |
| **LLM 调用路径** | 全部走 Sidecar | 简单走 Rig (Rust)，复杂走 Sidecar | 减少 IPC，性能优化 |
| **数据流设计** | 单路径 | 双路径 (Rust 直调 / Sidecar) | 灵活性 |

### 9.3 分层责任划分（v2.1 修订）

| 层 | 技术 | 职责 | v2.1 变更 |
|----|------|------|----------|
| **UI 展示层** | React + shadcn/ui + Tailwind | 页面渲染、交互、动画 | + lottie-react (Buddy) |
| **UI 状态层** | Zustand + TanStack Query | 客户端状态、服务器缓存 | 无变更 |
| **AI UI 层** | Vercel AI SDK | 流式消息渲染、Tool Call 展示 | 无变更 |
| **IPC 层** | Tauri invoke + Events | 前后端通信、多窗口事件 | + Buddy 窗口通信 |
| **系统集成层** | Tauri Rust Core | 文件系统、进程管理、系统通知 | 扩展为主要后端 |
| **MCP 层** | rmcp (Rust) | MCP Client、Server 管理 | 无变更 |
| **数据层** | rusqlite + sqlite-vec + FTS5 | 结构化存储 + 向量搜索 + 全文搜索 | **[v2.1]** 统一 SQLite 方案 |
| **知识库层** | sqlite-vec + FTS5 + RRF | **[v2.1 NEW]** 文档 RAG、混合检索 | 新增 |
| **沙箱层** | Wasmtime (Rust) | 安全代码执行 | 无变更 |
| **Skills 层** | Rust (发现/注册) + Python (复杂执行) | Skill 生命周期管理 | 无变更 |
| **LLM 直调层** | Rig (Rust) | 简单 LLM 对话，无需 Sidecar | 无变更 |
| **Agent 编排层** | LangGraph (Python Sidecar) | 复杂工作流、SubAgent、条件分支 | 无变更 |
| **记忆层** | PowerMem (Python Sidecar) | 智能长期记忆管理 | + SeekDB 可选 |
| **语音层** | whisper-cpp-plus + piper-rs (Rust) | **[v2.1 NEW]** Buddy STT/TTS | 新增 |

---

## 10. 修订后的架构设计

### 10.1 整体架构图（v2.1）

```
┌──────────────────────────────────────────────────────────────────────┐
│                          MisakaX Desktop App v2.0                         │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                     WebView (React 19 UI)                         │ │
│  │                                                                   │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────┐ │ │
│  │  │ Chat     │ │Dashboard │ │ Skills   │ │ Settings │ │ Buddy  │ │ │
│  │  │ Page     │ │ Page     │ │ Market   │ │ Page     │ │ Window │ │ │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └───┬────┘ │ │
│  │       │             │            │             │            │      │ │
│  │  ┌────┴─────────────┴────────────┴─────────────┴────────────┴──┐  │ │
│  │  │      State: Zustand (UI) + TanStack Query (Server)          │  │ │
│  │  │      AI UI: Vercel AI SDK (Streaming)                       │  │ │
│  │  └─────────────────────────┬───────────────────────────────────┘  │ │
│  │                             │ invoke() / listen()                  │ │
│  └─────────────────────────────┼─────────────────────────────────────┘ │
│                                │                                       │
│  ┌─────────────────────────────┼─────────────────────────────────────┐ │
│  │           Tauri Rust Core (主进程, 承担大部分后端职责)                │ │
│  │                             │                                      │ │
│  │  ┌──────────────────────────┴────────────────────────────────┐    │ │
│  │  │                    Command Router                          │    │ │
│  │  │         (路由决策: Rust 直处理 or 转发 Sidecar)             │    │ │
│  │  └──┬──────┬──────┬──────┬──────┬──────┬──────┬──────────────┘    │ │
│  │     │      │      │      │      │      │      │                    │ │
│  │  ┌──┴───┐┌─┴───┐┌─┴───┐┌─┴───┐┌─┴───┐┌─┴───┐┌─┴──────┐          │ │
│  │  │ Rig  ││ MCP ││ DB  ││ FS  ││Sand-││Skill││Sidecar │          │ │
│  │  │ LLM  ││ Mgr ││Layer││Layer││ box ││ Mgr ││  Mgr   │          │ │
│  │  │直调   ││rmcp ││sqlit││     ││wasm ││     ││        │          │ │
│  │  └──┬───┘└──┬──┘└──┬──┘└─────┘└──┬──┘└──┬──┘└──┬─────┘          │ │
│  │     │       │      │              │      │      │                  │ │
│  │     │    ┌──┴──┐   │          ┌───┴──┐   │      │                  │ │
│  │     │    │ MCP │   │          │Wasm- │   │      │                  │ │
│  │     │    │Srvrs│   │          │time  │   │      │                  │ │
│  │     │    └─────┘   │          └──────┘   │      │                  │ │
│  │     │              │                     │      │                  │ │
│  │  ┌──┴──────────────┴─────────────────────┴──────┤                  │ │
│  │  │         简单请求路径 (Rust 内部完成)            │                  │ │
│  │  │  LLM 直调 / MCP 调用 / DB 读写 / Skill 注入   │                  │ │
│  │  └────────────────────────────────────────────────┘                 │ │
│  │                                                                     │ │
│  │  ┌─────────────────── 复杂请求路径 ───────────────────────┐         │ │
│  │  │                    ↕ HTTP / WebSocket                   │         │ │
│  │  │  ┌───────────────────────────────────────────────────┐ │         │ │
│  │  │  │          Python Sidecar (延迟启动)                  │ │         │ │
│  │  │  │                                                    │ │         │ │
│  │  │  │  ┌─────────────┐  ┌────────────────────────────┐  │ │         │ │
│  │  │  │  │ FastAPI     │  │ LangGraph Agent Engine      │  │ │         │ │
│  │  │  │  │ HTTP Server │→│ ├── StateGraph (主图)         │  │ │         │ │
│  │  │  │  │ + WebSocket │  │ ├── SubGraph (子 Agent)      │  │ │         │ │
│  │  │  │  └─────────────┘  │ ├── Human-in-the-loop       │  │ │         │ │
│  │  │  │                   │ └── Conditional Edges        │  │ │         │ │
│  │  │  │                   └────────────────────────────┘  │ │         │ │
│  │  │  │  ┌────────────────────────────────────────────┐   │ │         │ │
│  │  │  │  │ PowerMem (长期记忆引擎)                      │   │ │         │ │
│  │  │  │  │ ├── SQLite 本地存储 (默认)                   │   │ │         │ │
│  │  │  │  │ ├── 艾宾浩斯遗忘曲线管理                      │   │ │         │ │
│  │  │  │  │ ├── 混合检索 (向量+全文+图谱)                 │   │ │         │ │
│  │  │  │  │ └── Multi-Agent 记忆隔离                     │   │ │         │ │
│  │  │  │  └────────────────────────────────────────────┘   │ │         │ │
│  │  │  └───────────────────────────────────────────────────┘ │         │ │
│  │  └────────────────────────────────────────────────────────┘         │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
```

### 10.2 双路径数据流

```
用户发送消息
    │
    ├─── 路由判断 (Rust Command Router) ───┐
    │                                       │
    │  [简单路径 — Rust 内部完成]             │  [复杂路径 — 转发 Sidecar]
    │                                       │
    │  条件:                                 │  条件:
    │  • 单轮对话                             │  • 需要 Agent 编排 (多步骤)
    │  • 仅需 LLM 回复                       │  • 需要 SubAgent
    │  • MCP 工具调用                        │  • 需要记忆检索/存储
    │  • 数据库读写                           │  • 需要条件分支
    │  • 文件操作                             │  • 需要 Human-in-the-loop
    │                                       │
    ↓                                       ↓
┌────────────────┐              ┌─────────────────────┐
│ Rig → LLM API │              │ HTTP → FastAPI       │
│ rmcp → MCP    │              │ → LangGraph 编排     │
│ rusqlite → DB │              │ → PowerMem 记忆      │
│ 流式 Token     │              │ → SSE/WS 流式返回    │
└───────┬────────┘              └──────────┬──────────┘
        │                                  │
        ↓                                  ↓
┌──────────────────────────────────────────────────┐
│ Tauri Event 推送 → React listen() → Vercel AI SDK│
│ → 流式渲染 UI → 消息持久化 (rusqlite)               │
└──────────────────────────────────────────────────┘
```

### 10.3 模块划分（v2.1 修订）

```
claw/
├── src-tauri/                        # Rust 后端 (承担主要职责)
│   ├── src/
│   │   ├── main.rs                   # 入口
│   │   ├── commands/                 # Tauri Commands (暴露给前端)
│   │   │   ├── chat.rs              # 对话 (路由判断: Rig 或 Sidecar)
│   │   │   ├── session.rs           # 会话管理
│   │   │   ├── mcp.rs              # MCP 管理
│   │   │   ├── skills.rs           # Skills 管理
│   │   │   ├── knowledge.rs        # [v2.1] 知识库管理 (导入/检索)
│   │   │   ├── memory.rs           # 记忆查询 (代理转发)
│   │   │   ├── buddy.rs            # [v2.1] Buddy 交互 (STT/TTS/对话)
│   │   │   ├── settings.rs         # 设置
│   │   │   └── file.rs             # 文件操作
│   │   ├── services/                # 业务逻辑 (Rust 实现)
│   │   │   ├── llm/                # LLM 直调 (via Rig)
│   │   │   │   ├── provider.rs     # 多提供商统一接口
│   │   │   │   ├── streaming.rs    # 流式 Token 处理
│   │   │   │   └── router.rs       # 路由判断逻辑
│   │   │   ├── mcp/                # MCP Client (via rmcp)
│   │   │   │   ├── client.rs       # MCP Client 管理器
│   │   │   │   ├── transport.rs    # stdio / HTTP / SSE 传输
│   │   │   │   └── config.rs       # MCP 配置加载
│   │   │   ├── skills/             # Skills 系统 (Rust 实现)
│   │   │   │   ├── discovery.rs    # Skill 发现 (多目录扫描)
│   │   │   │   ├── parser.rs       # SKILL.md 解析 (YAML + MD)
│   │   │   │   ├── registry.rs     # Skill 注册表 (DashMap)
│   │   │   │   ├── executor.rs     # Skill 执行器
│   │   │   │   ├── watcher.rs      # 文件监听 (热重载)
│   │   │   │   └── market.rs       # 市场 API 客户端
│   │   │   ├── knowledge/          # [v2.1] 知识库系统
│   │   │   │   ├── ingest.rs       # 文档导入 (解析/分块/嵌入)
│   │   │   │   ├── search.rs       # 混合搜索 (sqlite-vec + FTS5 + RRF)
│   │   │   │   ├── embedding.rs    # Embedding 生成 (via Rig)
│   │   │   │   └── chunker.rs      # 智能文档分块
│   │   │   ├── voice/              # [v2.1] 语音系统 (Buddy)
│   │   │   │   ├── stt.rs          # whisper-cpp-plus 语音识别
│   │   │   │   ├── tts.rs          # piper-rs 语音合成
│   │   │   │   └── audio.rs        # cpal 采集 + rodio 播放
│   │   │   ├── sandbox/            # WASM 沙箱
│   │   │   └── sidecar/            # Python Sidecar 管理
│   │   │       ├── manager.rs      # 启动/停止/健康检查/重启
│   │   │       ├── protocol.rs     # HTTP/WS 通信协议
│   │   │       └── health.rs       # 健康检查与自动恢复
│   │   ├── db/                     # 数据库层
│   │   │   ├── mod.rs              # 数据库初始化 + sqlite-vec 加载
│   │   │   ├── migrations.rs       # Schema 迁移
│   │   │   └── models.rs           # 数据模型
│   │   ├── state.rs                # 应用状态
│   │   └── config.rs               # 配置管理
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                              # React 前端
│   ├── components/
│   │   ├── chat/                    # 对话组件
│   │   ├── layout/                  # 布局组件
│   │   ├── skills/                  # Skills 组件
│   │   │   ├── SkillMarket.tsx     # Skills 市场浏览
│   │   │   ├── SkillCard.tsx       # 单个 Skill 卡片
│   │   │   ├── SkillManager.tsx    # 已安装 Skills 管理
│   │   │   ├── SkillEditor.tsx     # Skill 内容编辑器
│   │   │   └── SkillTrigger.tsx    # Skill 触发提示 UI
│   │   ├── knowledge/              # [v2.1] 知识库组件
│   │   │   ├── KnowledgeBase.tsx   # 知识库管理界面
│   │   │   ├── DocImporter.tsx     # 文档导入
│   │   │   └── SearchResult.tsx    # 检索结果展示
│   │   ├── buddy/                  # [v2.1] Buddy 组件
│   │   │   ├── BuddyAvatar.tsx    # 角色动画 (Lottie)
│   │   │   ├── SpeechBubble.tsx   # 对话气泡
│   │   │   ├── MiniInput.tsx      # 迷你输入框
│   │   │   └── VoiceButton.tsx    # 语音按钮
│   │   ├── settings/
│   │   ├── dashboard/
│   │   └── ui/                      # shadcn/ui 基础组件
│   ├── stores/
│   │   ├── app.ts                  # 应用状态
│   │   ├── chat.ts                 # 对话状态
│   │   ├── skills.ts              # Skills 状态
│   │   ├── knowledge.ts           # [v2.1] 知识库状态
│   │   └── buddy.ts              # [v2.1] Buddy 状态
│   ├── hooks/
│   ├── lib/
│   ├── i18n/
│   └── App.tsx
│
├── agent/                            # Python Sidecar (精简职责)
│   ├── agent/
│   │   ├── __init__.py
│   │   ├── graph.py                # LangGraph 主图定义
│   │   ├── nodes.py                # 节点实现
│   │   ├── subagents.py           # SubAgent 子图定义
│   │   ├── tools.py                # 工具定义
│   │   ├── memory.py              # PowerMem 集成
│   │   └── state.py                # Agent 状态
│   ├── server.py                   # FastAPI 入口
│   ├── requirements.txt
│   └── build_nuitka.py            # Nuitka 构建脚本
│
├── skills/                           # 内建 Skills
│   ├── code-review.md
│   ├── brainstorming.md
│   └── debugging.md
│
├── models/                           # [v2.1] 模型文件 (首次使用时下载)
│   ├── README.md                    # 模型下载说明
│   └── .gitkeep
│
├── package.json
├── vite.config.ts
├── tailwind.config.ts
└── tsconfig.json
```

---

## 11. 最终技术决策

### 11.1 最终推荐技术栈

| 层 | 最终选型 | 备选方案 | 选择理由 |
|----|---------|---------|---------|
| **桌面框架** | **Tauri 2.x** | — | 体积小 96%，内存少 75%，安全模型优于 Electron |
| **前端框架** | **React 19 + TypeScript** | Svelte 5 | 最大生态，shadcn/ui 生态完善，AI 参考实现最多 |
| **UI 组件库** | **shadcn/ui + Tailwind CSS v4** | Nocta UI | 0KB 运行时，完全可控，2026 年 React 标配 |
| **状态管理** | **Zustand + TanStack Query** | Jotai | 1.1KB，最简 API，Tauri 多窗口同步成熟 |
| **AI UI** | **Vercel AI SDK** | 手动 SSE | React 原生 AI UI，流式渲染，MCP 原生支持 |
| **构建工具** | **Vite** | — | 最快的前端构建，Tauri 官方推荐 |
| **MCP Client** | **rmcp (Rust 官方 SDK)** | TypeScript SDK | Tier 2 官方维护，Rust 原生性能 |
| **数据库** | **rusqlite (SQLite WAL)** | SeekDB (不满足跨平台) | Rust 原生，全平台，零额外依赖 |
| **知识库向量** | **sqlite-vec** | LanceDB | **[v2.1]** 与主库同文件，全平台，Rust 原生 |
| **知识库全文** | **FTS5** (SQLite 内建) | — | **[v2.1]** 零额外依赖 |
| **混合搜索** | **RRF (Reciprocal Rank Fusion)** | — | **[v2.1]** 向量+全文联合排序标准算法 |
| **LLM 直调** | **Rig (Rust)** | 直接 HTTP 调用 | 20+ 提供商统一接口，Rust 原生 |
| **Agent 编排** | **LangGraph (Python Sidecar)** | Mastra (TS) | 最成熟编排框架，完整子图/记忆/中断 |
| **长期记忆** | **PowerMem** | Mem0 / LangMem | 遗忘曲线，SQLite 本地，MCP Server，LangGraph 集成 |
| **记忆存储** | **SQLite (默认) / SeekDB (可选)** | — | **[v2.1]** 默认全平台；macOS/Linux 可选 SeekDB |
| **沙箱** | **Wasmtime (WASM)** | Docker (可选) | <13ms 启动，Rust 原生，资源限制完善 |
| **Skills 管理** | **Rust (发现/注册) + Python (复杂执行)** | 纯 Rust | 平衡性能与灵活性 |
| **Buddy 动画** | **Lottie (默认) + Live2D (高级)** | Rive | **[v2.1]** CPU 友好，丰富的免费资源 |
| **语音识别** | **whisper-cpp-plus (Rust)** | Vosk | **[v2.1]** 高准确度、多语言、Rust 原生 |
| **语音合成** | **piper-rs (Rust)** | edge-tts | **[v2.1]** 离线、Rust 原生、900+ 声音 |
| **Python 打包** | **Nuitka** | PyInstaller | AOT 编译，启动快 30-50%，体积小 20-30% |
| **国际化** | **react-i18next** | — | React 生态标准 |
| **代码编辑器** | **Monaco Editor** | CodeMirror | VS Code 同款，功能最全 |

### 11.2 架构核心原则

| 原则 | 实施方式 |
|------|---------|
| **Rust 做重活** | MCP、数据库、文件系统、沙箱、Skills 注册、LLM 直调、知识库检索、语音引擎全部 Rust 实现 |
| **Python 做巧活** | 仅 Agent 编排 (LangGraph) 和智能记忆 (PowerMem) 使用 Python |
| **延迟启动** | Python Sidecar 仅在需要时启动，普通操作纯 Rust，用户无感知 |
| **双路径分流** | 简单请求 Rust 内部完成 (0 IPC)，复杂请求转发 Sidecar (~2ms IPC) |
| **统一 SQLite** | 结构化数据 + 向量搜索 + 全文搜索，一个 .db 文件搞定 |
| **Skills 原生支持** | SKILL.md 标准格式，多来源发现，热重载，与 Agent 编排深度集成 |
| **记忆智能化** | PowerMem 遗忘曲线 + 重要性评估，非暴力全文塞进上下文 |
| **Buddy 独立窗口** | 透明浮窗 + Tauri Event 通信，共享 Rust Core 后端 |

### 11.3 综合评分（v2.1 修订）

| 评估维度 | v1.0 评分 | v2.0 评分 | v2.1 评分 | 变更说明 |
|---------|----------|----------|----------|---------|
| **性能** | 9.5 | 9.7 | **9.7** | 无变更 |
| **UI 美观度** | 9.0 | 9.0 | 9.0 | 无变更 |
| **社区活跃度** | 9.0 | 9.0 | 9.0 | 无变更 |
| **稳定性** | 8.5 | 8.5 | **8.7** | SQLite 统一方案更稳定，避免 SeekDB 跨平台问题 |
| **易用性** | 7.5 | 7.5 | 7.5 | 无变更 |
| **功能覆盖率** | 9.5 | 9.8 | **10.0** | 新增知识库 RAG + Buddy 完整选型 |
| **Agent 框架完整性** | 9.0 | 9.3 | 9.3 | 无变更 |
| **包体积** | 8.0 | 8.3 | 8.3 | 无变更 |
| **跨平台** | 9.0 | 9.0 | **9.5** | 确认 SQLite 方案全平台零问题，排除 SeekDB 风险 |
| **可维护性** | 8.5 | 8.7 | 8.7 | 无变更 |
| **Skills 系统** | N/A | 9.0 | 9.0 | 无变更 |
| **记忆系统** | 7.5 | 9.0 | 9.0 | 无变更 |
| **知识库** | N/A | N/A | **9.0** | sqlite-vec + FTS5 + RRF 混合搜索 |
| **Buddy 系统** | N/A | N/A | **9.0** | Lottie + whisper + piper 完整选型 |
| **综合评分** | **8.8** | **9.0** | **9.1** | |

### 11.4 开发路线图（v2.1 修订）

| 阶段 | 时间 | 目标 | v2.1 更新 |
|------|------|------|----------|
| **Phase 1: 基础框架** | 4-6 周 | Tauri 项目搭建、React UI 骨架、Rig 简单对话、SQLite + sqlite-vec | + sqlite-vec 初始化 |
| **Phase 2: 核心功能** | 6-8 周 | 流式对话、会话管理、rmcp MCP、设置系统、主题/i18n | 无变更 |
| **Phase 3: Skills 系统** | 3-4 周 | Skill 发现/解析/注册/执行/UI | 无变更 |
| **Phase 4: Agent 增强** | 6-8 周 | LangGraph Sidecar、PowerMem 记忆、SubAgent | 无变更 |
| **Phase 5: 知识库** | 3-4 周 | **[v2.1 NEW]** 文档导入、分块、Embedding、混合搜索、RAG UI | 新增阶段 |
| **Phase 6: 高级功能** | 4-6 周 | WASM 沙箱、Dashboard、文件浏览 | 无变更 |
| **Phase 7: Buddy 系统** | 13-19 周 | 桌面伴侣（可与 Phase 2-6 并行） | 无变更 |
| **Phase 8: 打磨发布** | 4-6 周 | 性能优化、跨平台测试、自动更新、打包 | 无变更 |

**总预估周期：30-41 周（7.5-10 个月），其中 Buddy 可并行开发**

---

> **文档结束**
>
> 本文档是对 [MISAKAX_TECH_SELECTION_REPORT.md v1.0](./MISAKAX_TECH_SELECTION_REPORT.md) 的深度修订。
> - v2.0 (2026-04-27)：针对 5 个核心问题补充调研
> - v2.1 (2026-04-28)：新增 Buddy 架构选型、SeekDB 调研、知识库向量方案
>
> v1.0 中 Buddy 系统的详细实现方案（动画代码、语音集成、AI 对话流程等）请参考原报告第 14 节。
