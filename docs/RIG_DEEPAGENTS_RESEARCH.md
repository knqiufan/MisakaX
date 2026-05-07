# Rig 框架与 DeepAgents 框架深度调研

> **调研日期：** 2026-05-07
> **文档版本：** v3.0（架构重构：全路径 DeepAgents 最终决策）
> **基于：** MISAKAX_ARCHITECTURE_FINAL v2.2（全路径 DeepAgents 单一架构）
> **目标：** 深入研究 Rig 框架（Rust）和 DeepAgents 框架（Python/TypeScript），完成功能对比，
> 评估并确认 DeepAgents 作为全对话入口的架构决策。

---

## 目录

1. [Rig 框架详细介绍](#1-rig-框架详细介绍)
2. [Rig 与 DeepAgents 功能对比](#2-rig-与-deepagents-功能对比)
3. [架构选型：原方案 vs DeepAgents 方案](#3-架构选型原方案-vs-deepagents-方案)

---

## 1. Rig 框架详细介绍

### 1.1 项目概览

| 属性 | 详情 |
|------|------|
| **GitHub** | https://github.com/0xPlaygrounds/rig |
| **Stars** | 7,000+ |
| **许可证** | MIT |
| **最新版本** | v0.36.0 (2026-05-01) |
| **Crate** | `rig-core` |
| **Rust Edition** | 2024 |
| **贡献者** | 180+ |
| **月下载量** | ~45,000 |
| **定位** | Rust 原生 LLM 应用框架 — 构建可扩展、模块化、类型安全的 AI 应用 |

### 1.2 核心架构

Rig 采用分层 trait 抽象设计，核心理念是 **"Agent = Model + Prompt + Tools"**：

```
Application   → Agent<M, P>, Extractor<M, T>
Request       → PromptRequest, StreamingPromptRequest, CompletionRequestBuilder
Trait 抽象    → CompletionModel, EmbeddingModel, Prompt, Chat
Provider 实现 → openai::Client, anthropic::Client, gemini::Client ...
Infrastructure → reqwest, ToolServer, VectorStoreIndexDyn
```

#### 五大核心 Trait

| Trait | 用途 |
|-------|------|
| **`CompletionModel`** | 统一 LLM 调用接口，支持 20+ 提供商 |
| **`EmbeddingModel`** | 向量嵌入生成 |
| **`VectorStoreIndex`** | 向量存储与相似度搜索（10+ 后端） |
| **`Tool`** | 函数调用接口 |
| **`ProviderClient`** | 从环境变量创建模型的工厂 trait |

### 1.3 功能矩阵

#### Agent 系统

| 功能 | 支持 | 说明 |
|------|------|------|
| **单 Agent 对话** | ✅ | `Agent<M, P>` 类型 |
| **多轮对话** | ✅ | 流式 + 非流式，tool calling 循环 |
| **结构化输出** | ✅ v0.31.0+ | `AgentBuilder::schema_output()` |
| **工具调用** | ✅ | 静态/动态/MCP 工具 |
| **并行工具执行** | ✅ | `buffer_unordered(concurrency)` |
| **多 Agent 编排** | ⚠️ 第三方 | `rigs` (DAG 工作流), `graph-flow` (LangGraph 风格) |
| **SubAgent** | ❌ 无原生 | 需手动实现 |
| **Human-in-the-loop** | ❌ 无原生 | 需手动通过 Hooks 实现 |
| **条件分支** | ❌ 无原生 | 需手动实现 |

#### LLM 提供商

| 提供商 | 支持 |
|--------|------|
| OpenAI | ✅ (含 o1/o3/gpt-5 推理模型) |
| Anthropic | ✅ (含 Claude Opus/Sonnet/Haiku 全系列) |
| Gemini | ✅ |
| Cohere | ✅ |
| DeepSeek | ✅ |
| Ollama | ✅ |
| Perplexity | ✅ |
| Azure OpenAI | ✅ |
| AWS Bedrock | ✅ |
| Google Vertex AI | ✅ |
| Groq | ✅ |
| Together | ✅ |
| SambaNova | ✅ |
| OpenRouter | ✅ |

#### 向量存储集成

| 后端 | Crate | 类型 |
|------|-------|------|
| MongoDB Atlas | `rig-mongodb` | 远程 |
| LanceDB | `rig-lancedb` | 嵌入式 |
| Neo4j | `rig-neo4j` | 远程 |
| Qdrant | `rig-qdrant` | 嵌入式/远程 |
| SurrealDB | `rig-surrealdb` | 嵌入式/远程 |
| SQLite + sqlite-vec | `rig-sqlite` | 嵌入式 |
| Milvus | `rig-milvus` | 远程 |
| PostgreSQL | `rig-postgres` | 远程 |
| ScyllaDB | `rig-scylladb` | 远程 |
| HelixDB | `rig-helixdb` | 远程 |
| 内存 | `rig-core`内置 | 嵌入式 |

#### RAG 能力

| 功能 | 支持 |
|------|------|
| `#[derive(Embed)]` 宏 | ✅ 自动标记嵌入字段 |
| `EmbeddingsBuilder` | ✅ 批量文档摄入 |
| `DynamicContextStore` | ✅ Agent 自动检索上下文 |
| 余弦相似度过滤 | ✅ |
| 元数据过滤 | ✅ |
| Top-K 可配置 | ✅ |
| 混合检索（向量+BM25） | ❌ 仅向量搜索 |

#### 其他能力

| 功能 | 支持 |
|------|------|
| **WASM 编译** | ✅ `rig-core` 完全支持 WASM |
| **流式输出** | ✅ 完整流式支持 |
| **多模态** | ✅ 文本/图像/音频生成/转录 |
| **OpenTelemetry** | ✅ GenAI Semantic Convention |
| **Hooks 系统** | ✅ 6 个 hook 点（LLM 调用前后、工具执行前后、流式 delta） |
| **MCP 工具** | ✅ MCP Tool 可作为 Rig Tool |

### 1.4 关键优势

| 优势 | 说明 |
|------|------|
| **类型安全** | Rust 编译期保证，消除大量运行时错误；所有权系统防止内存泄漏 |
| **高性能** | 原生 tokio 异步，零 IPC 开销（在 Tauri Rust 后端内直接调用） |
| **零成本抽象** | Rust trait 设计，无虚拟函数开销 |
| **WASM 可移植** | 核心库完全支持 WASM，可运行在浏览器/边缘 |
| **提供商中立** | 统一接口切换 20+ LLM 提供商，无需改代码 |
| **内存效率** | Rust 原生内存管理，1 个 Agent 约 ~1.1 GB（vs Python ~5.1 GB） |
| **冷启动** | <200ms（vs Python Sidecar 1-2s） |
| **二进制体积** | ~22 MB 单文件（无 Python 环境） |
| **MIT 协议** | 商业友好 |

### 1.5 关键劣势

| 劣势 | 说明 |
|------|------|
| **无多 Agent 原生编排** | 无内建 SubAgent、条件分支、工作流 DAG；需依赖第三方 `rigs`/`graph-flow` |
| **无记忆系统** | 完全无内建记忆管理；需手动实现（或依赖 PowerMem 等外部系统） |
| **无规划工具** | 无任务分解、Todo 追踪等内建能力 |
| **API 不稳定** | 项目文档明确警告"Breaking Changes"，0.x 版本迭代极快（约 weekly releases） |
| **Tool 系统局限** | 不支持一次性注册多个 Tool（无 `AgentBuilder::tools(vec![])`） |
| **嵌套错误链问题** | 多 Agent 嵌套时出错链路冗长，不可恢复性错误多 |
| **Rust 学习曲线** | 团队需要较强的 Rust 基础，快速原型开发速度不如 Python |
| **生态成熟度** | 远不如 Python LangChain/LangGraph 生态，教程/示例/第三方集成少 |
| **无内置 Claude-style Skills** | 无 SKILL.md 标准支持、无渐进式披露、无技能市场 |
| **无 Human-in-the-loop** | 需通过 Hooks 手动实现审批流程 |

### 1.6 适用场景

| ✅ 适合 | ⚠️ 不太适合 |
|---------|-----------|
| 简单 LLM 直调（单轮对话、少工具） | 复杂多步骤 Agent 编排 |
| MCP 工具调用（通过 rmcp + Rig） | 需要 SubAgent 派生的场景 |
| 高性能生产部署（低延迟、低内存） | 快速原型开发 |
| 嵌入式/边缘部署（WASM） | 需要智能记忆管理系统 |
| Rust 技术栈统一的项目 | 需要完整 Skills 生态系统的项目 |

---

## 2. Rig 与 DeepAgents 功能对比

> 本节聚焦功能对比，不讨论语言差异（Rig 是 Rust，DeepAgents 是 Python/TypeScript）。

### 2.1 DeepAgents 项目概览

| 属性 | Python | TypeScript |
|------|--------|-----------|
| **GitHub** | `langchain-ai/deepagents` | `langchain-ai/deepagentsjs` |
| **最新版本** | v0.5.4 (2026-04-29) | v1.9.0 (2026-03) |
| **底层引擎** | LangGraph (StateGraph) | LangGraph.js (StateGraph) |
| **安装** | `pip install deepagents` | `npm install deepagents` |
| **定位** | LangGraph 之上的"开箱即用"Agent Harness |

### 2.2 核心功能逐项对比

| 功能领域 | Rig (Rust) | DeepAgents (Python/TS) |
|---------|------------|------------------------|
| **LLM 统一接口** | ✅ 20+ 提供商 | ✅ 通过 LangChain |
| **流式输出** | ✅ | ✅ 原生 |
| **工具调用** | ✅ 静态/动态/MCP | ✅ LangGraph ToolNode |
| **并行工具执行** | ✅ `buffer_unordered` | ✅ LangGraph Fan-out |
| **结构化输出** | ✅ v0.31.0+ | ✅ `response_format` |
| **多 Agent 编排** | ⚠️ 第三方 crate | ✅ `task` + `SubAgentMiddleware` |
| **SubAgent 上下文隔离** | ❌ | ✅ 独立上下文窗口 |
| **异步 SubAgent** | ❌ | ✅ v0.5 支持（后台委托） |
| **任务规划** | ❌ | ✅ `write_todos` + `TodoListMiddleware` |
| **文件系统操作** | ❌ (不相关) | ✅ `ls`/`read_file`/`write_file`/`edit_file`/`glob`/`grep` |
| **Skills 系统** | ❌ | ✅ `SkillsMiddleware` + SKILL.md 标准 |
| **渐进式披露** | ❌ | ✅ L1(description) → L2(full body) |
| **记忆管理** | ❌ | ✅ `MemoryMiddleware` (AGENTS.md 规范) |
| **记忆持久化** | ❌ | ✅ LangGraph Store (跨线程) |
| **上下文摘要压缩** | ❌ | ✅ `SummarizationMiddleware` |
| **Human-in-the-loop** | ❌ | ✅ `HumanInTheLoopMiddleware` |
| **Prompt 缓存优化** | ❌ | ✅ `AnthropicPromptCachingMiddleware` |
| **可插拔后端** | ❌ (不相关) | ✅ State/Filesystem/Store/Composite/Sandbox |
| **沙箱执行** | ✅ Wasmtime（独立） | ✅ Modal/Daytona/Deno/LocalShell |
| **MCP 工具** | ✅ 通过 rmcp | ✅ 通过 LangChain MCP 适配器 |
| **向量搜索/RAG** | ✅ 10+ 向量后端 | ✅ 通过 LangChain 集成 |
| **嵌入模型统一接口** | ✅ `EmbeddingModel` trait | ✅ 通过 LangChain |
| **WASM 支持** | ✅ | ❌ |
| **OpenTelemetry** | ✅ | ✅ 通过 LangSmith/LangFuse |

### 2.3 关键差异总结

| 维度 | Rig | DeepAgents |
|------|-----|-----------|
| **核心定位** | LLM SDK + 轻量 Agent | 完整 Agent Harness（规划+文件+子代理+记忆+技能） |
| **架构层级** | 基础设施层（Provider 抽象） | 应用框架层（开箱即用的 Agent） |
| **编排能力** | 单 Agent 工具调用循环 | 多 Agent 子图 + SubAgent 委托 + 异步 |
| **开箱即用度** | 低（需手动搭建） | 高（`create_deep_agent()` 一行代码） |
| **可扩展性** | Trait 实现（编译期） | Middleware 模式（运行时） |
| **生态系统** | 集中在 LLM Provider + Vector Store | 完整的 Agent 开发生态（LangChain + LangSmith 等） |
| **性能** | 极致（Rust 原生，无 GIL） | 良好（Python GIL 约束，但 LLM 延迟占主导） |
| **成熟度** | 0.x 快速迭代，API 不稳定 | 建立在成熟的 LangGraph 之上 |

### 2.4 互补关系

Rig 和 DeepAgents 并非直接竞品，它们在不同层级解决不同问题：

```
DeepAgents  ← 应用框架层："我需要一个能规划、写文件、派生子代理的 Agent"
    ↓
LangGraph    ← 编排引擎层："我需要编排多步骤 Agent 工作流"
    ↓
LangChain    ← 组件层："我需要 LLM Provider、Tool、Memory 等组件"
    ↓
（对应 Rust 生态）
    ↓
Rig          ← 基础设施层："我需要统一的 LLM API 和向量存储接口"
```

在当前项目中，**两者可以共存**：Rust 侧用 Rig 做轻量 LLM 直调，Python Sidecar 用 DeepAgents 替代手动 LangGraph 实现。

---

## 3. 架构选型：全路径 DeepAgents 最终方案

### 3.1 架构演进历程

MisakaX 架构经历了三个主要版本的演进：

| 版本 | 架构模式 | 核心问题 |
|------|---------|---------|
| **v2.0** | 大 Sidecar（所有逻辑在 Python） | 简单对话也要启动 Python，资源浪费 |
| **v2.1** | 智能分层（Rig 简单 / Sidecar 复杂） | 用户体验割裂，路由误判风险，维护双系统 |
| **🏆 v2.2** | **全路径 DeepAgents**（所有对话统一入口） | 当前方案 |

### 3.2 全路径 DeepAgents 架构（v2.2 最终方案）

```
React (UI) ──Tauri IPC──▶ Rust Core ──所有对话──▶ Python Sidecar (常驻预热)
                              │                         │
                              │                   ┌─────┴──────────┐
                              │                   │  DeepAgents     │
                              │                   │  Harness        │
                              │                   │  ├ TodoList     │
                              │                   │  ├ Skills       │
                              │                   │  ├ Filesystem   │
                              │                   │  ├ SubAgent     │
                              │                   │  ├ Summarization│
                              │                   │  ├ Memory       │
                              │                   │  └ HITL         │
                              │                   └────────┬────────┘
                              │                            │
                              ├── MCP 工具 ──▶ rmcp       ├── PowerMem
                              │                            │   (自定义Tool)
                              ├── 知识库 ──▶ SQLite        │
                              │              + sqlite-vec  ├── LangGraph
                              │              + FTS5        │   Checkpointer
                              │                            │
                              ├── 代码执行 ──▶ Wasmtime    │
                              │                            │
                              ├── Skills发现 ──▶ 文件同步 ──┘
                              │   (Rust 扫描/注册)
                              │
                              └── Rig (非对话: Embedding/标题/摘要)
```

**核心原则：** 用户打开对话框 → 选择工作目录 → 直接进入 DeepAgents harness。Agent 内部自主决定处理策略（简单直接回复，复杂启用工具/Skills/SubAgent）。没有"简单对话"概念，每次对话都拥有完整 Agent 能力。

### 3.3 DeepAgents 替代手动实现的价值

DeepAgents 本质上不是替代 LangGraph，而是 LangGraph 的"开箱即用"封装。它返回标准的 `CompiledStateGraph`，与 LangGraph Sidecar 运行模式完全兼容。

| 原手动实现 | DeepAgents 替代 | 节省 |
|-----------|----------------|------|
| 手动 StateGraph + Node + Edge 定义 | `create_deep_agent()` 工厂函数 | ~80% 代码量 |
| 手动 SubAgent 子图 | `SubAgent` 类型 + `SubAgentMiddleware` | ~70% 代码量 |
| 手动上下文管理 | `SummarizationMiddleware` 自动摘要压缩 | 省去整个模块 |
| 手动 Human-in-the-loop | `HumanInTheLoopMiddleware` 配置式 | ~60% 代码量 |
| 手动任务规划逻辑 | `TodoListMiddleware` (`write_todos` 工具) | 省去整个模块 |
| 手动记忆注入 | `MemoryMiddleware` (AGENTS.md 自动加载) | ~50% 代码量 |
| 手动文件操作 | `FilesystemMiddleware` (基于工作目录) | 省去整个模块 |
| 手动路由判断 | **不需要路由 — 全对话统一入口** | 省去整个路由模块 |

### 3.4 多维度对比分析（v3.0 更新）

#### 3.4.1 Agent 编排能力

| 维度 | 手动 LangGraph | DeepAgents 全路径 | 评估 |
|------|---------------|-------------------|------|
| **单 Agent 对话** | ✅ 手动构建 StateGraph | ✅ `create_deep_agent()` 一行 | DeepAgents 更快 |
| **多步骤工作流** | ✅ 手动定义节点+边 | ✅ 内建 TodoListMiddleware | DeepAgents 更省力 |
| **SubAgent 派生** | ⚠️ 手动编写子图+条件路由 | ✅ `task` 工具 + SubAgentMiddleware | DeepAgents 更成熟 |
| **子代理上下文隔离** | ⚠️ 需要手动管理命名空间 | ✅ 自动隔离上下文窗口 | DeepAgents 更优 |
| **异步 SubAgent** | ❌ 需手动实现 | ✅ `AsyncSubAgent` (v0.5) | DeepAgents 有优势 |
| **条件分支** | ✅ LangGraph conditional_edges | ✅ LangGraph 底层支持 | 持平 |
| **Human-in-the-loop** | ⚠️ 手动 interrupt + resume | ✅ `HumanInTheLoopMiddleware` | DeepAgents 更简洁 |
| **Agent 状态持久化** | ✅ LangGraph Checkpointer | ✅ 同样基于 LangGraph Checkpointer | 持平 |
| **对话路由** | ❌ 需手动实现路由判断 | ✅ **无路由 — 全对话统一入口** | DeepAgents 架构更简洁 |

#### 3.4.2 工作目录与文件系统

| 维度 | 手动实现 | DeepAgents | 评估 |
|------|---------|-----------|------|
| **工作目录绑定** | ❌ 需手动实现 | ✅ FilesystemMiddleware 基于 root_dir | DeepAgents 内建 |
| **文件读写** | ⚠️ 需通过 Tauri 权限层 | ✅ FilesystemMiddleware 内建 | Agent 内直接文件操作 |
| **上下文卸载** | ❌ 需手动实现 | ✅ 长文本写入文件释放上下文 | DeepAgents 显著优势 |
| **可插拔存储后端** | ❌ 无法更换 | ✅ StateBackend/FilesystemBackend/CompositeBackend | DeepAgents 更灵活 |
| **远程目录支持** | ❌ 需手动实现 | ✅ 通过自定义 Backend 扩展 | 可扩展 |

#### 3.4.3 Skills 系统

| 维度 | 手动实现 | DeepAgents 双层架构 | 评估 |
|------|---------|---------------------|------|
| **Skill 格式** | ✅ SKILL.md | ✅ SKILL.md（SkillsMiddleware） | 持平 |
| **Skill 发现** | ✅ 文件系统扫描 | ✅ Rust 发现 + 同步至 SkillsMiddleware | 双层互补 |
| **渐进式披露** | ⚠️ 手动实现 | ✅ 内建支持 L1→L2 | DeepAgents 更省力 |
| **Skill 执行** | ⚠️ Prompt 注入 | ✅ SkillsMiddleware 自动注入 | DeepAgents 更简洁 |

#### 3.4.4 记忆系统

| 维度 | 手动实现 | DeepAgents + PowerMem | 评估 |
|------|---------|----------------------|------|
| **自动记忆注入** | ❌ 需手动加载 | ✅ MemoryMiddleware 自动注入 AGENTS.md | DeepAgents 更省力 |
| **上下文自动摘要** | ❌ 需手动实现压缩 | ✅ SummarizationMiddleware | DeepAgents 显著优势 |
| **艾宾浩斯遗忘** | ✅ PowerMem | ✅ PowerMem（自定义 Tool 集成，保留） | 持平 |

#### 3.4.5 与 Rust Core 的集成

| 维度 | 双路径路由 (v2.1) | 全路径 DeepAgents (v2.2) | 评估 |
|------|-----------------|-------------------------|------|
| **通信方式** | HTTP localhost → FastAPI | HTTP localhost → FastAPI（常驻连接） | 持平 |
| **Sidecar 生命周期** | 按需唤醒（1-2s 冷启动） | **应用启动预热**（无冷启动） | v2.2 更优 |
| **路由复杂度** | 5 路分支判断 | **2 路（对话/非对话）** | v2.2 更简洁 |
| **Rust 侧改动** | 需维护路由判断 + 两套对话逻辑 | **仅需转发对话到 Sidecar** | v2.2 改造成本更低 |
| **前端 UI 改动** | — | **零改动**（Phase 4 仅替换后端） | 用户无感知迁移 |

#### 3.4.6 开发效率

| 维度 | 手动 LangGraph | DeepAgents 全路径 | 评估 |
|------|---------------|-------------------|------|
| **初始搭建时间** | ~4-6 周 | ~3 周（Phase 2-4 渐进式） | DeepAgents 更高效 |
| **代码量** | 大量样板代码 | 少量配置 + Middleware | DeepAgents 少 60-80% |
| **可维护性** | ⚠️ 双系统维护 | ✅ 单一对话路径 + Middleware 集中 | DeepAgents 更好 |
| **用户体验一致性** | ❌ 简单/复杂路径割裂 | ✅ **每次对话能力一致** | 核心优势 |

### 3.5 最终架构决策（v3.0）

#### 🏆 最终方案：全路径 DeepAgents 单一架构

**核心论据：**

1. **消除用户体验割裂。** 用户无需猜测自己的请求会被哪条路径处理。每次对话都拥有完整的 Agent 能力（工具调用、Skills、记忆检索、文件操作、SubAgent 派生）。

2. **DeepAgents 是 LangGraph 的"开箱即用"封装。** 它返回标准的 `CompiledStateGraph`，与 Sidecar 运行模式完全兼容。Middleware 链覆盖了原方案需要大量手动编码的功能。

3. **Sidecar 预热消除冷启动。** 应用启动时异步启动 Python Sidecar，用户首次对话无等待延迟。

4. **Rig 保留非对话价值。** Embedding 生成、会话标题/摘要等非对话场景继续使用 Rig 的高性能 Rust 直调。

5. **渐进式过渡降低风险。** Phase 2 Rig 作为"过渡对话实现"验证 UI 链路，Phase 4 替换为 DeepAgents。前端 UI 层零改动迁移。

6. **工作目录系统原生集成。** DeepAgents FilesystemMiddleware 天然支持基于工作目录的文件操作，这是手动 LangGraph 方案需要完全自行实现的能力。

#### 集成架构

```
┌─────────────────────────────────────────────────────────────────┐
│  Python Sidecar (agent/)                                          │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │  FastAPI HTTP Server                                         │ │
│  │  ├── POST /agent/chat     → deep_agent.ainvoke(state)       │ │
│  │  ├── POST /agent/stream   → deep_agent.astream_events(state)│ │
│  │  └── WebSocket /agent/ws  → deep_agent.astream_events(state)│ │
│  └──────────────────────────┬──────────────────────────────────┘ │
│                              │                                    │
│  ┌──────────────────────────┴──────────────────────────────────┐ │
│  │  create_deep_agent(                                         │ │
│  │    model="claude-sonnet-4-6",                               │ │
│  │    tools=[powermem_search_tool, powermem_save_tool,         │ │
│  │           mcp_bridge_tools, ...],                           │ │
│  │    system_prompt=system_prompt,                             │ │
│  │    subagents=[                                               │ │
│  │      SubAgent(name="researcher", ...),                      │ │
│  │      SubAgent(name="coder", ...),                           │ │
│  │    ],                                                       │ │
│  │    skills=["/skills/"],                                     │ │
│  │    memory=["/memories/"],                                   │ │
│  │    backend=CompositeBackend(                                │ │
│  │      default=StateBackend(rt),                              │ │
│  │      routes={"/memories/": StoreBackend(rt)}                │ │
│  │    ),                                                       │ │
│  │  )                                                          │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

#### 项目结构变更

```
agent/                               # Python Sidecar（简化后）
├── app/
│   ├── __init__.py
│   ├── main.py                     # FastAPI 入口（不变）
│   ├── agent.py                    # create_deep_agent() 组装（替代 graph.py + nodes.py）
│   ├── tools.py                    # 自定义工具（PowerMem、MCP Bridge）
│   ├── subagents.py               # SubAgent 定义（替代 subagents.py 旧版）
│   ├── middleware.py               # 自定义 Middleware（如需）
│   └── config.py                   # Agent 配置
├── skills/                         # SKILL.md 文件（DeepAgents SkillsMiddleware 加载）
├── memories/                       # AGENTS.md 记忆文件（DeepAgents MemoryMiddleware 加载）
├── server.py                       # 启动入口（简化）
├── requirements.txt
└── build_nuitka.py
```

#### 需要自定义的部分

| 需要自定义 | 实现方式 |
|-----------|---------|
| **PowerMem 集成** | 自定义 LangChain Tool（`@tool` 装饰器） |
| **MCP Bridge** | 自定义 Tool（通过 HTTP 回调 Rust 端 rmcp） |
| **Rust Core 通信** | FastAPI HTTP/WebSocket 接口层（不变） |
| **路由触发** | Rust 端路由逻辑（简化为 2 路：对话→Sidecar / 非对话→Rust） |

### 3.6 风险与应对

| 风险 | 影响 | 概率 | 应对 |
|------|------|------|------|
| **DeepAgents API Breaking** | Python Sidecar 代码需要跟进修改 | 中 | 固定 `deepagents==0.5.4`；跟随 Release Notes 增量升级 |
| **PowerMem Middleware 集成困难** | 需额外开发适配层 | 低 | PowerMem 可以通过 LangChain Tool 标准方式接入；必要时写一个自定义 Middleware |
| **DeepAgents 社区不成熟** | 遇到 Bug 可能需要自行修复或等待 | 中 | LangChain 官方项目有企业级支持；代码开源可自行 Fork |
| **Flexibility 不满足需求** | Middleware 架构限制了自定义行为 | 低 | 可混合使用（DeepAgents 外部用 LangGraph API 直接操控图） |
| **Python/TS DeepAgents 功能差异** | TS 版本可能缺少某些 Middleware | 低 | 使用 Python 版本，它与 LangGraph 集成最成熟 |
| **Sidecar 预热失败/崩溃** | 用户首次对话无法使用，需手动重启 | 中 | 健康检查 + 自动重连机制（最多 3 次）；前端显示 Sidecar 状态指示器 |
| **Phase 4 DeepAgents 迁移不顺利** | 对话功能回退到 Phase 2 Rig 过渡方案 | 低 | Phase 2 Rig 对话链路保持可用作为回退；Phase 4 渐进式迁移而非一次性切换 |
| **工作目录安全风险** | Agent 文件操作越界访问系统敏感目录 | 低 | FilesystemMiddleware root_dir 严格限定；Tauri 文件系统权限层双重校验；远程路径 SSH 密钥隔离 |

### 3.7 最终建议

1. **Phase 1-3（当前阶段）**：纯 Rust + React 快速出 MVP。Phase 2 用 Rig 作为过渡对话实现验证 UI 链路，Phase 3 搭建 Sidecar 预热基础设施但不接入对话。
2. **Phase 4 全对话迁移**：使用 DeepAgents 替代 Phase 2 的 Rig 过渡对话实现。Sidecar 已在 Phase 3 完成预热部署，实现零冷启动切换。
3. **PowerMem 集成**：作为自定义 LangChain Tool 接入 DeepAgents，保留其艾宾浩斯遗忘曲线核心能力。
4. **Rust Core 路由简化**：从 v2.1 的 5 路分支精简为 2 路（对话→Sidecar / 非对话→Rust），删除简单/复杂对话路由判断逻辑。
5. **Rig 角色重定义**：Rig 退役对话功能，保留 Embedding 生成、会话标题/摘要等非对话 LLM 直调。
6. **工作目录系统**：作为 P0 核心需求，Phase 2 即实现 DirectorySelector UI + 会话绑定，Phase 4 接入 DeepAgents FilesystemMiddleware。

6 条建议覆盖了从当前到最终形态的完整渐进式过渡路径，
确保每一阶段都有可工作的产出，同时逐步收敛到最终的全路径 DeepAgents 单一架构。

---

> **报告结束**
>
> 本报告基于 2026 年 5 月的最新技术生态，对 Rig 框架进行了全面介绍，并完成了
> Rig vs DeepAgents 功能对比，最终确认 DeepAgents 作为 MisakaX 全对话入口的架构决策。
>
> **关键结论（v3.0 更新）：**
> - **DeepAgents 作为全对话入口**：所有用户对话统一进入 DeepAgents harness，Agent 自主决策处理策略
> - **Rig 专用于非对话场景**：Embedding 生成、会话标题/摘要等轻量 LLM 直调（高性能 Rust 原生）
> - **Sidecar 预热消除冷启动**：应用启动时异步启动 Python Sidecar + 健康检查，对话零等待
> - **工作目录作为 P0 核心**：Phase 2 实现基础 UI + 绑定，Phase 4 接入 DeepAgents FilesystemMiddleware
> - **渐进式过渡降低风险**：Phase 2 Rig 过渡 → Phase 3 Sidecar 预热 → Phase 4 DeepAgents 全迁移
> - **PowerMem 通过自定义 Tool 集成**：保留艾宾浩斯遗忘曲线核心能力
> - **路由简化为 2 路**：对话→Sidecar / 非对话→Rust（删除简单/复杂对话路由判断）
>
> 综合评分：**全路径 DeepAgents 方案 9.0 vs v2.1 双路径路由方案 7.3**
