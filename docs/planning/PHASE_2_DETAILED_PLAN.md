# Phase 2：对话基础设施骨架与工作目录系统 — 详细实施方案

> **所属项目：** MisakaX
> **阶段：** Phase 2（第 5-7 周）
> **总预估：** ~74 小时（含 Vibe Coding 加速，新增工作目录系统 +12h，模型架构增强 +2h）
> **前置条件：** Phase 1 已完成（完整 UI 骨架、设置系统、主题/国际化、API Key 管理、Zustand/IPC 基础设施就绪）
> **前置文档：** [PHASE_1_DETAILED_PLAN.md](./PHASE_1_DETAILED_PLAN.md)、[MISAKAX_IMPLEMENTATION_PLAN.md](./MISAKAX_IMPLEMENTATION_PLAN%20-%20Opus4.6.md)
> **里程碑：** M1: 核心可用 — Phase 2 完成后，MisakaX 具备完整的对话 UI 基础设施和工作目录系统，为 Phase 4 DeepAgents 全对话接管做好了 UI 层准备。
>
> **⚠️ Phase 2 定位说明（v3.0 全路径 DeepAgents 架构）：**
> Phase 2 **不是在构建"简单对话"产品功能**。MisakaX 的最终形态是全路径 DeepAgents harness——每次对话都在完整的 Agent 环境中运行。
> Phase 2 使用 Rig 作为 **临时 LLM 后端 stub**，目的是：
> 1. **验证对话基础设施**：流式渲染管线（IPC → Event → React）、消息持久化、会话管理
> 2. **建立工作目录系统**：目录选择 UI、会话-目录绑定、目录状态指示
> 3. **预留 Phase 4 兼容接口**：chat Command 采用策略模式，Phase 4 仅切换策略从 Rig→Sidecar，前端零改动
>
> Phase 4 时，Rig 对话功能退役，DeepAgents 正式接管所有对话。Phase 2 建立的 UI 组件和基础设施全部复用。
>
> **代码库现状基线（Phase 1 已交付）：**
> - Rust 侧 11 个 `.rs` 文件，扁平模块结构（`commands/`、`db/`、顶层 `config`/`crypto`/`sidecar`），**无 `services/` 目录**
> - 已注册 15 个 Tauri Command（settings ×8 + router_configs ×5），会话和消息相关命令**未实现**
> - 数据库 Schema v1 已建表（sessions / messages / router_configs / tasks / knowledge_docs + FTS5），sqlite-vec 已加载但未建向量表
> - `db/models.rs` 已定义基础 `Session` / `Message` / `Setting` / `RouterConfig` 结构体
> - `AppState` 使用 `std::sync::Mutex`（非 async Mutex），包含 `db` / `config` / `sidecar` 三个字段
> - 前端 62 个 TS/TSX 文件，`ChatPage.tsx` 为 EmptyState 占位，`chat-store.ts` 仅有基础骨架
> - `lib/ipc/` 已有 `invoke` / `settings` / `router-configs` / `types`，会话和消息 IPC **未封装**

---

## 目录

1. [阶段目标与验收标准](#1-阶段目标与验收标准)
2. [核心数据流设计（含 Phase 4 兼容性）](#2-核心数据流设计)
3. [任务 2.1：LLM Provider 封装（trait 对象分层 + 自定义模型）](#3-任务-21llm-provider-封装)
4. [任务 2.2：Rust 流式 LLM 调用 + Tauri Event 推送](#4-任务-22rust-流式-llm-调用--tauri-event-推送)
5. [任务 2.3：`send_message` Command（核心对话链路 + 策略模式预留）](#5-任务-23send_message-command)
6. [任务 2.4：ChatView 组件（消息列表 + 输入框）](#6-任务-24chatview-组件)
7. [任务 2.5：MessageItem 组件（Markdown 渲染 + 代码高亮 + 复制）](#7-任务-25messageitem-组件)
8. [任务 2.6：MessageInput 组件（多行输入 + 快捷键 + 文件附件）](#8-任务-26messageinput-组件)
9. [任务 2.7：工作目录选择器（本地路径浏览 + 目录验证）](#9-任务-27工作目录选择器) 🆕
10. [任务 2.8：会话-工作目录绑定（Session.working_dir + 创建流程）](#10-任务-28会话-工作目录绑定) 🆕
11. [任务 2.9：工作目录状态指示栏（WorkspaceBar 组件）](#11-任务-29工作目录状态指示栏) 🆕
12. [任务 2.10：会话列表侧边栏（创建/切换/删除/搜索）](#12-任务-210会话列表侧边栏)
13. [任务 2.11：消息持久化（SQLite messages 表读写）](#13-任务-211消息持久化)
14. [任务 2.12：Token 用量统计（单消息 + 单会话累计）](#14-任务-212token-用量统计)
15. [任务 2.13：思维链展示（thinking blocks 折叠/展开）](#15-任务-213思维链展示)
16. [任务 2.14：停止生成 / 重新生成](#16-任务-214停止生成--重新生成)
17. [任务 2.15：多模态输入（图片粘贴/拖拽附件）](#17-任务-215多模态输入)
18. [Phase 2 完整验证清单](#18-phase-2-完整验证清单)
19. [详细 TODO 列表](#19-详细-todo-列表)

---

## 1. 阶段目标与验收标准

### 1.1 核心目标

构建完整的**对话基础设施骨架**和**工作目录系统**：用户新建对话 → 选择工作目录 → 发送消息 → Rig (临时后端 stub) 调用 LLM API → 流式 Token 返回 → React 实时渲染 → 消息持久化。

**Phase 2 的三层目标：**

| 层次 | 目标 | 说明 |
|------|------|------|
| **对话基础设施** | 流式渲染管线 + 消息持久化 + 会话管理 | 所有 UI 组件为 Phase 4 DeepAgents 直接复用 |
| **工作目录系统** | 目录选择 UI + 会话绑定 + 状态指示 | MisakaX 区别于普通聊天客户端的工程化核心特性 |
| **Phase 4 兼容预留** | chat Command 策略模式 + Event 协议兼容 | 确保 Phase 4 切换后端时前端零改动 |

> **关键认知：** Rig 在此阶段是"临时 LLM 后端 stub"——它验证了消息通路的正确性。Phase 4 时只需将 `ChatBackend::Rig` 策略切换为 `ChatBackend::Sidecar`，整个 UI 层和 Event 协议不变。

### 1.2 任务依赖关系

```
Phase 1 产出（settings store / IPC / router_configs / AppShell）
    │
    ├── 2.1 Rig Provider 封装 ──→ 2.2 流式调用 ──→ 2.3 send_message Command (策略模式)
    │                                                       │
    │   2.11 消息持久化 (SQLite) ───────────────────────────┘
    │           │                                            │
    │           ↓                                            ↓
    │   2.10 会话列表侧边栏 ←─── 2.4 ChatView ←──── 2.5 MessageItem
    │           ↑                     ↑                      │
    │           │                     │                      │
    │   2.7 工作目录选择器 ──→ 2.8 会话-目录绑定              │
    │                               ↑                        │
    │                     2.9 工作目录指示栏                   │
    │                               │                        │
    │                       2.6 MessageInput                  │
    │                                                        │
    │   独立功能（与 2.4 并行或后续）：                         │
    │   ├── 2.12 Token 用量统计                               │
    │   ├── 2.13 思维链展示 ←───────────────────────────────┘
    │   ├── 2.14 停止生成 / 重新生成
    │   └── 2.15 多模态输入
    │
    ↓
全部完成 → M1: 核心可用（含工作目录系统）
```

> **关键路径：** 2.1 → 2.2 → 2.3 → 2.11 → 2.4 → 2.5
> **工作目录路径：** 2.7 → 2.8 → 2.9（可与核心链路并行）
> **可并行：** 2.6 / 2.10 在 2.4 骨架完成后可并行；2.12-2.15 可在核心链路通后并行

### 1.3 验收标准（Done Definition）

| # | 验收条件 | 验证方式 |
|---|---------|---------|
| AC-1 | 用户输入消息后，LLM 流式返回 Token 并实时渲染到界面 | 发送"Hello"，观察逐字显示 |
| AC-2 | 支持至少 OpenAI / Anthropic / Gemini 三个 Provider | 分别配置 API Key 后对话 |
| AC-3 | 消息内容支持 Markdown 渲染（标题、列表、粗体、斜体、链接） | 发送"用 Markdown 格式写一段话" |
| AC-4 | 代码块有语法高亮（至少支持 JS/TS/Python/Rust）+ 复制按钮 | 让 LLM 写代码 |
| AC-5 | 可创建新会话、切换会话、删除会话，会话列表可搜索 | CRUD 操作 |
| AC-6 | 关闭应用后重新打开，之前的会话和消息仍然存在 | 重启验证 |
| AC-7 | 每条消息显示 Token 用量（input/output），会话维度有累计统计 | 检查消息底部 |
| AC-8 | Anthropic 模型的 thinking blocks 可折叠/展开显示 | 使用 Claude 并开启 extended thinking |
| AC-9 | 可中途停止生成；可对已完成消息重新生成 | 点击停止/重新生成按钮 |
| AC-10 | 可粘贴图片或拖拽图片到输入框，图片以 Base64 编码发送给支持视觉的模型 | 粘贴截图 |
| AC-11 | 流式输出过程中消息列表自动滚动到底部 | 长回复时观察 |
| AC-12 | 多行输入框支持 Shift+Enter 换行、Enter 发送 | 键盘操作 |
| AC-13 | 🆕 新建会话时弹出工作目录选择器，可选择本地目录或跳过 | 创建会话流程 |
| AC-14 | 🆕 已绑定工作目录的会话，顶栏 WorkspaceBar 显示当前目录路径 | 查看顶栏 |
| AC-15 | 🆕 会话的 working_dir 持久化到 SQLite，重启后绑定关系保留 | 重启验证 |
| AC-16 | 🆕 chat Command 内部采用策略模式（`ChatBackend` trait），可在 Rig/Sidecar 间切换 | 代码审查 |

### 1.4 交付物一览

- **对话基础设施骨架**（Rig 临时后端 + Phase 4 兼容策略接口）
- 支持 OpenAI / Anthropic / Gemini
- 流式输出 + Markdown 渲染 + 代码高亮
- **工作目录选择 UI**（新建会话时选择本地目录）🆕
- **工作目录绑定与指示**（顶栏显示当前会话工作目录）🆕
- 会话创建/切换/删除/搜索
- 消息持久化
- Token 用量统计
- 思维链展示
- 停止生成 / 重新生成
- 图片输入

---

## 2. 核心数据流设计（含 Phase 4 兼容性）

### 2.1 对话请求完整链路

```
[React 前端]                    [Tauri Rust Core]                  [LLM API / Sidecar]
     │                                │                                 │
     │ 1. invoke("send_message",      │                                 │
     │    { session_id, content,      │                                 │
     │      images?, model,           │                                 │
     │      working_dir? })           │  ← 🆕 携带工作目录               │
     ├──────────────────────────────► │                                 │
     │                                │ 2. 保存用户消息到 SQLite           │
     │                                │ 3. 读取 router_config            │
     │                                │    解密 API Key                  │
     │                                │ 4. 🆕 选择 ChatBackend 策略:      │
     │                                │    Phase 2: Rig 直调             │
     │                                │    Phase 4: HTTP → Sidecar       │
     │                                │ 5. 构建请求并调用 LLM             │
     │                                ├────────────────────────────────► │
     │                                │                                 │
     │                                │ 6. 接收流式 Token                 │
     │                                │ ◄─────── chunk1 ────────────────│
     │  7. Tauri Event:               │ ◄─────── chunk2 ────────────────│
     │     "stream_token"             │ ◄─────── chunk3 ────────────────│
     │ ◄─ { delta, msg_id, done }─── │ ◄─────── [DONE] ────────────────│
     │                                │                                 │
     │  8. React 实时更新             │ 9. 保存完整 assistant 消息         │
     │     MessageItem 组件           │    + token_usage 到 SQLite        │
     │     (增量追加 delta)            │                                 │
     │                                │ 10. emit "stream_complete"       │
     │ ◄─ { msg_id, usage } ──────── │                                 │
```

### 2.1.1 Phase 4 兼容性设计 🆕

Phase 2 的 Event 协议和前端组件必须同时兼容 Rig 直调模式和 Phase 4 的 DeepAgents Sidecar 模式。关键设计：

```
Phase 2 (Rig 直调):
  Rust chat.rs → ChatBackend::Rig → rig-core stream → emit Event → React

Phase 4 (DeepAgents Sidecar):
  Rust chat.rs → ChatBackend::Sidecar → HTTP POST Sidecar → SSE/WS → 解析为同格式 Event → React
  ↑ 前端完全不变，仅 Rust 后端策略切换
```

**兼容性保证：**
- Event payload 格式（`stream_token` / `stream_thinking` / `stream_complete` / `stream_error`）在两种模式下完全一致
- 前端 `use-stream-listener.ts` 不感知后端是 Rig 还是 Sidecar
- `send_message` Command 的参数和返回值在两种模式下一致
- 工作目录 `working_dir` 在 Phase 2 存入 Session 但不被 Rig 使用；Phase 4 时由 Sidecar 读取并配置 `FilesystemMiddleware`

### 2.2 Tauri Event 协议定义

Phase 2 使用 Tauri Event System 实现 Rust → React 的流式推送，事件类型：

| Event 名称 | Payload | 触发时机 |
|-----------|---------|---------|
| `stream_token` | `{ session_id, message_id, delta, content_type }` | 每收到一个 LLM token chunk |
| `stream_thinking` | `{ session_id, message_id, thinking_delta }` | Anthropic thinking block 增量 |
| `stream_complete` | `{ session_id, message_id, usage, model }` | 流式传输结束 |
| `stream_error` | `{ session_id, message_id?, error }` | LLM 调用出错 |

`content_type` 枚举：
- `"text"` — 普通文本 token
- `"thinking"` — Anthropic extended thinking content

### 2.3 数据库 Schema 补充

Phase 0/1 已在 `migrate_v1` 中创建基础表。当前 `messages` 表结构如下：

```sql
-- 现有 messages 表（v1）
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,          -- "user" | "assistant" | "system"
    content TEXT NOT NULL,
    token_usage TEXT,            -- JSON string（已预留但未使用）
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- 现有 sessions 表（v1）
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    title TEXT,
    model TEXT,
    system_prompt TEXT,
    working_directory TEXT,
    project_name TEXT,
    status TEXT DEFAULT 'active',  -- "active" | "archived"
    mode TEXT DEFAULT 'agent',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

Phase 2 需要通过 `migrate_v2` 对这两个表进行**增强**：

```sql
-- Phase 2: messages 表增强（migrate_v2）
ALTER TABLE messages ADD COLUMN model TEXT;
ALTER TABLE messages ADD COLUMN thinking_content TEXT;
ALTER TABLE messages ADD COLUMN attachments TEXT;  -- JSON: [{type, data, mime_type}]
ALTER TABLE messages ADD COLUMN status TEXT DEFAULT 'complete';
-- status: "streaming" | "complete" | "error" | "stopped"

-- Phase 2: sessions 表增强
ALTER TABLE sessions ADD COLUMN total_input_tokens INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN total_output_tokens INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN last_message_at DATETIME;
ALTER TABLE sessions ADD COLUMN pinned INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN group_name TEXT;
-- 🆕 工作目录增强（v1 已有 working_directory TEXT，Phase 2 补充远程目录支持）
ALTER TABLE sessions ADD COLUMN working_dir_remote TEXT;         -- 远程工作目录连接字符串
ALTER TABLE sessions ADD COLUMN working_dir_remote_type TEXT;    -- "ssh" | "s3" | "ftp"
```

> **注意：** `sessions.working_directory` 字段在 v1 已建表时创建，存储本地工作目录路径。Phase 2 新增 `working_dir_remote` 和 `working_dir_remote_type` 字段为远程目录预留。Phase 4 时 DeepAgents `FilesystemMiddleware` 将读取这些字段配置 `CompositeBackend`。

> **注意：** `migrations.rs` 中已预留了 v2 注释：`// if current_version < 2 { migrate_v2(conn)?; }`，直接取消注释并实现 `migrate_v2()` 即可。

### 2.4 Zustand Store 设计（chat-store 完善）

当前 `chat-store.ts` 已有基础骨架（sessions / activeSessionId / loading + 3 个 setter），需要大幅增强：

```typescript
// 现有骨架（保留并扩展）：
// sessions: Session[], activeSessionId: string | null, loading: boolean
// setActiveSession, setSessions, setLoading

// Phase 2 完善后的完整接口：
interface ChatState {
  // ============ 会话列表 ============
  sessions: Session[];
  activeSessionId: string | null;
  sessionsLoading: boolean;
  
  // ============ 当前会话的消息 ============
  messages: Message[];
  messagesLoading: boolean;
  
  // ============ 流式状态 ============
  streamingMessageId: string | null;
  streamingContent: string;
  streamingThinking: string;
  isStreaming: boolean;
  
  // ============ Actions — 会话 CRUD ============
  loadSessions: () => Promise<void>;
  createSession: (model?: string) => Promise<string>;
  deleteSession: (id: string) => Promise<void>;
  switchSession: (id: string) => Promise<void>;
  updateSessionTitle: (id: string, title: string) => Promise<void>;
  searchSessions: (query: string) => Promise<Session[]>;
  
  // ============ Actions — 消息 ============
  loadMessages: (sessionId: string) => Promise<void>;
  sendMessage: (content: string, images?: string[]) => Promise<void>;
  stopGeneration: () => void;
  regenerateMessage: (messageId: string) => Promise<void>;
  
  // ============ Actions — 流式处理（由 Event listener 调用）============
  appendStreamDelta: (delta: string) => void;
  appendThinkingDelta: (delta: string) => void;
  completeStream: (messageId: string, usage: TokenUsage) => void;
  handleStreamError: (error: string) => void;
}
```

---

## 3. 任务 2.1：LLM Provider 封装（trait 对象分层 + 自定义模型）

> **预估时间：** 8h（原 6h + 自定义模型支持 +2h）
> **产出：** Rust 端多 Provider 统一接口，支持用户自定义模型 + 接口兼容性选择

### 3.1 设计目标与原则

当前代码存在三个需要解决的设计问题：

| 问题 | 现状 | 改进 |
|------|------|------|
| **概念混淆** | `RouterConfig.model` 仅存一个模型名，不支持多模型 | 引入 `custom_models` 表，一个 Provider 可绑定多个自定义模型 |
| **enum 反模式** | `ProviderClient` enum + 所有 match 分支 | 使用 trait 对象 `Box<dyn LlmProvider>` 替代 enum，遵守开闭原则 |
| **代码扁平** | `provider.rs` 堆砌所有逻辑 | 按职责分层：trait 定义 / 具体实现 / 工厂 / 配置 各自独立文件 |

**设计原则：**
- **开闭原则**：新增 Provider 只需添加一个文件实现 trait，不改动已有代码
- **依赖倒置**：上层（`send_message`）依赖 `LlmProvider` trait 而非具体实现
- **单一职责**：每个文件/模块只做一件事

### 3.2 代码结构设计

```
src-tauri/src/
├── lib.rs                    # 新增: pub mod services;
├── services/
│   ├── mod.rs                # pub mod llm;
│   └── llm/
│       ├── mod.rs            # 模块导出 + re-exports
│       ├── traits.rs         # LlmProvider trait 定义（核心抽象）
│       ├── factory.rs        # ProviderFactory：根据 RouterConfig 创建 Box<dyn LlmProvider>
│       ├── config.rs         # LlmConfig：温度、max_tokens、top_p 等运行时参数
│       ├── registry.rs       # ModelRegistry：可用模型元数据查询（内置 + 自定义）
│       ├── backend.rs        # ChatBackend trait（策略模式，Phase 4 兼容 🆕）
│       └── providers/        # 各 Provider 具体实现（一个文件一个 Provider）
│           ├── mod.rs         # pub mod openai_provider; pub mod anthropic_provider; ...
│           ├── openai_provider.rs
│           ├── anthropic_provider.rs
│           ├── gemini_provider.rs
│           └── openai_compat.rs   # 处理所有 OpenAI 兼容的（DeepSeek, 自定义等）
├── config.rs                 # (已有)
├── crypto.rs                 # (已有)
├── sidecar.rs                # (已有)
└── ...
```

### 3.3 核心 trait 定义

```rust
// src-tauri/src/services/llm/traits.rs

/// LLM Provider 核心抽象
///
/// 每个 Provider 实现此 trait 即可接入系统。
/// 新增 Provider 只需：
///   1. 在 providers/ 下创建新文件
///   2. 实现 LlmProvider trait
///   3. 在 ProviderFactory 中注册一行
pub trait LlmProvider: Send + Sync {
    fn provider_type(&self) -> &str;

    fn completion_model(&self, model_name: &str) -> Box<dyn CompletionModel>;

    fn supports_vision(&self, model_name: &str) -> bool;

    fn supports_thinking(&self, model_name: &str) -> bool;
}
```

### 3.4 Provider 具体实现（以 OpenAI 为例）

```rust
// src-tauri/src/services/llm/providers/openai_provider.rs

use rig::providers::openai;

pub struct OpenAiProvider {
    client: openai::Client,
}

impl OpenAiProvider {
    pub fn new(api_key: &str, base_url: Option<&str>) -> Self {
        let mut client = openai::Client::new(api_key);
        if let Some(url) = base_url {
            client = client.with_base_url(url);
        }
        Self { client }
    }
}

impl LlmProvider for OpenAiProvider {
    fn provider_type(&self) -> &str { "openai" }

    fn completion_model(&self, model_name: &str) -> Box<dyn CompletionModel> {
        Box::new(self.client.completion_model(model_name))
    }

    fn supports_vision(&self, model_name: &str) -> bool {
        model_name.contains("gpt-4o") || model_name.contains("o1")
    }

    fn supports_thinking(&self, _model_name: &str) -> bool {
        false
    }
}
```

```rust
// src-tauri/src/services/llm/providers/openai_compat.rs

/// 处理所有 OpenAI 兼容的 Provider（DeepSeek、自建、第三方中转等）
/// 用户在设置中选择"接口兼容 = OpenAI"时走此实现
pub struct OpenAiCompatProvider {
    client: openai::Client,
    provider_name: String,  // "deepseek" / "custom" / 用户自定义名
}

impl OpenAiCompatProvider {
    pub fn new(api_key: &str, base_url: &str, name: &str) -> Self {
        let client = openai::Client::new(api_key).with_base_url(base_url);
        Self {
            client,
            provider_name: name.to_string(),
        }
    }
}

impl LlmProvider for OpenAiCompatProvider {
    fn provider_type(&self) -> &str { &self.provider_name }
    // ... 同上模式
}
```

### 3.5 工厂模式

```rust
// src-tauri/src/services/llm/factory.rs

/// Provider 工厂 — 根据 RouterConfig + 解密后的 API Key 创建对应 Provider
///
/// 扩展方式：新增 Provider 时只需添加一个 match 分支
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(config: &RouterConfig, decrypted_key: &str) -> Result<Box<dyn LlmProvider>> {
        match config.provider.as_str() {
            "openai" => Ok(Box::new(
                OpenAiProvider::new(decrypted_key, config.base_url.as_deref())
            )),
            "anthropic" => Ok(Box::new(
                AnthropicProvider::new(decrypted_key)
            )),
            "google" => Ok(Box::new(
                GeminiProvider::new(decrypted_key)
            )),
            _ => {
                // 所有未知类型统一走 OpenAI 兼容
                // 用户自定义 Provider 通过 api_compat 字段决定兼容模式
                let compat = config.api_compat.as_deref().unwrap_or("openai");
                let base_url = config.base_url.as_deref()
                    .ok_or_else(|| anyhow!("Custom provider requires base_url"))?;
                match compat {
                    "openai" => Ok(Box::new(
                        OpenAiCompatProvider::new(decrypted_key, base_url, &config.provider)
                    )),
                    "anthropic" => Ok(Box::new(
                        AnthropicProvider::new_with_base_url(decrypted_key, base_url)
                    )),
                    _ => Err(anyhow!("Unsupported api_compat: {}", compat)),
                }
            }
        }
    }
}
```

### 3.6 自定义模型支持

#### 3.6.1 数据库扩展（migrate_v2 新增）

```sql
-- 用户自定义模型（绑定到 router_configs）
CREATE TABLE IF NOT EXISTS custom_models (
    id TEXT PRIMARY KEY,
    router_config_id TEXT NOT NULL,
    model_id TEXT NOT NULL,          -- 模型标识符（如 "gpt-4o-my-finetuned"）
    display_name TEXT NOT NULL,       -- 显示名（如 "我的微调模型"）
    supports_vision INTEGER DEFAULT 0,
    supports_thinking INTEGER DEFAULT 0,
    max_tokens INTEGER,
    context_window INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (router_config_id) REFERENCES router_configs(id) ON DELETE CASCADE
);
```

#### 3.6.2 RouterConfig 扩展

```sql
-- router_configs 表新增接口兼容性字段
ALTER TABLE router_configs ADD COLUMN api_compat TEXT DEFAULT NULL;
-- NULL = 根据 provider 自动判断
-- "openai" = OpenAI 兼容接口
-- "anthropic" = Anthropic 兼容接口
```

#### 3.6.3 Rust 端数据结构

```rust
// db/models.rs 新增
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomModel {
    pub id: String,
    pub router_config_id: String,
    pub model_id: String,
    pub display_name: String,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub max_tokens: Option<i32>,
    pub context_window: Option<i32>,
    pub created_at: String,
}
```

#### 3.6.4 模型注册表（ModelRegistry）

```rust
// src-tauri/src/services/llm/registry.rs

/// 可用模型注册表 — 合并内置模型 + 用户自定义模型
pub struct ModelRegistry;

impl ModelRegistry {
    /// 获取某个 Provider 下所有可用模型（内置 + 自定义）
    pub fn available_models(
        conn: &Connection,
        router_config_id: &str,
        provider: &str,
    ) -> Result<Vec<ModelInfo>> {
        let mut models = Self::builtin_models(provider);
        let custom = Self::custom_models(conn, router_config_id)?;
        models.extend(custom);
        Ok(models)
    }

    fn builtin_models(provider: &str) -> Vec<ModelInfo> {
        match provider {
            "openai" => vec![
                ModelInfo::new("gpt-4o", "GPT-4o", true, false),
                ModelInfo::new("gpt-4o-mini", "GPT-4o Mini", true, false),
                ModelInfo::new("o1", "o1", false, false),
                ModelInfo::new("o3-mini", "o3-mini", false, false),
            ],
            "anthropic" => vec![
                ModelInfo::new("claude-sonnet-4-20250514", "Claude Sonnet 4", true, true),
                ModelInfo::new("claude-opus-4-20250514", "Claude Opus 4", true, true),
                ModelInfo::new("claude-haiku-3-5-20241022", "Claude 3.5 Haiku", true, false),
            ],
            "google" => vec![
                ModelInfo::new("gemini-2.5-pro", "Gemini 2.5 Pro", true, false),
                ModelInfo::new("gemini-2.5-flash", "Gemini 2.5 Flash", true, false),
            ],
            _ => vec![], // custom provider 无内置模型
        }
    }

    fn custom_models(conn: &Connection, config_id: &str) -> Result<Vec<ModelInfo>> {
        // 从 custom_models 表读取
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub display_name: String,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub is_custom: bool,  // 区分内置/自定义
    pub max_tokens: Option<i32>,
}
```

### 3.7 前端自定义模型管理 UI 增强

> Phase 1 的 `ProviderDialog` 仅有一个 `model` 文本框。Phase 2 需要扩展：
> 1. Provider 级别新增 `api_compat` 下拉（OpenAI / Anthropic），仅在 `provider = custom` 时显示
> 2. 新增"自定义模型管理"区域，支持在已有 Provider 下添加/删除自定义模型

```
┌─────────── 编辑 Provider ───────────────────────┐
│                                                  │
│  名称: [My Custom API                        ]   │
│  类型: [Custom (OpenAI Compatible)          ▼]   │
│  接口兼容: [OpenAI ▼]           ← 🆕 仅 custom  │
│  API Key: [••••••••••                  👁️ ]   │
│  Base URL: [https://my-proxy.com/v1       ]   │
│                                                  │
│  ── 自定义模型 ──────────────── [+ 添加模型] ──   │
│  │ gpt-4o-ft-2024    │ 我的微调模型 │ 👁️ 🧠 │ ✕ │ │
│  │ qwen-max          │ 通义千问     │    🧠 │ ✕ │ │
│  └──────────────────────────────────────────────┘ │
│                                                  │
│                          [取消] [保存]             │
└──────────────────────────────────────────────────┘
```

### 3.8 前端模型选择器交互（对话中）

```
┌─── 模型选择器（MessageInput 内嵌）───────────────────┐
│  Provider A: My OpenAI                              │
│    ├── gpt-4o              ✅默认  👁️              │
│    ├── gpt-4o-mini                 👁️              │
│    └── o3-mini                                     │
│  Provider B: Anthropic                              │
│    ├── Claude Sonnet 4             👁️ 🧠           │
│    └── Claude Opus 4               👁️ 🧠           │
│  Provider C: My Proxy                               │
│    ├── 我的微调模型（自定义）         👁️             │
│    └── qwen-max（自定义）               🧠          │
└─────────────────────────────────────────────────────┘
```

对话中选择模型时，前端需传递 `router_config_id + model_id`（而非仅 model 名），以确定走哪个 Provider 的 API Key 和端点。

### 3.9 新增 Rust Commands（模型相关）

```rust
// src-tauri/src/commands/models.rs 🆕

#[tauri::command]
pub fn list_available_models(
    state: State<'_, AppState>,
    router_config_id: Option<String>,
) -> Result<Vec<ProviderModels>, String>;
// 返回所有 Provider 及其可用模型（内置 + 自定义）

#[tauri::command]
pub fn add_custom_model(
    state: State<'_, AppState>,
    router_config_id: String,
    model: CreateCustomModel,
) -> Result<String, String>;

#[tauri::command]
pub fn delete_custom_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<(), String>;

// 响应结构
#[derive(Serialize)]
pub struct ProviderModels {
    pub provider: RouterConfigView,
    pub models: Vec<ModelInfo>,
}
```

### 3.10 依赖新增（Cargo.toml）

> 当前 `Cargo.toml` 已有 tokio(full)、reqwest(json,stream,blocking)、serde、dashmap 等依赖。
> Phase 2 仅需新增 `rig-core`、`futures` 和 `async-trait`。

```toml
# LLM 框架
rig-core = { version = "0.36", features = ["derive"] }

# 流式处理
futures = "0.3"

# trait 异步方法支持
async-trait = "0.1"
```

### 3.11 内置模型参考表

| Provider | 模型 | 视觉 | Thinking |
|----------|------|------|----------|
| OpenAI | gpt-4o, gpt-4o-mini, o1, o3-mini | ✅ gpt-4o | ❌ |
| Anthropic | claude-sonnet-4-20250514, claude-opus-4-20250514, claude-haiku | ✅ 全系列 | ✅ claude-sonnet-4/opus-4 |
| Gemini | gemini-2.5-pro, gemini-2.5-flash | ✅ 全系列 | ❌ |
| DeepSeek | deepseek-chat, deepseek-reasoner | ❌ | ✅ deepseek-reasoner |

> **自定义模型** 的能力（视觉/思维链）由用户在创建时手动标注。

---

## 4. 任务 2.2：Rust 流式 LLM 调用 + Tauri Event 推送

> **预估时间：** 8h
> **产出：** 流式 Token 通过 Tauri Event 推送到前端

### 4.1 架构设计

> **关于 AppState 与异步的兼容性：** 当前 `AppState` 使用 `std::sync::Mutex`，这在 async 上下文中持有锁时可能阻塞线程。
> 流式调用中，应**最小化锁持有时间**：进入 async 流式循环前先读取数据库/解密 Key，然后释放锁；流式过程中通过 Tauri Event 推送不需要访问 AppState。
> 如果后续遇到性能问题，可考虑将 `db` 字段迁移为 `tokio::sync::Mutex` 或使用 connection pool。

```
src-tauri/src/services/llm/
├── streaming.rs        # 流式调用核心逻辑
└── ...
```

### 4.2 流式调用核心逻辑

```rust
// src-tauri/src/services/llm/streaming.rs

use futures::StreamExt;
use tauri::{AppHandle, Emitter};

pub struct StreamSession {
    message_id: String,
    session_id: String,
    app_handle: AppHandle,
    abort_flag: Arc<AtomicBool>,
    accumulated_content: String,
    accumulated_thinking: String,
}

impl StreamSession {
    pub async fn execute_stream(
        &mut self,
        model: Box<dyn CompletionModel>,
        request: CompletionRequest,
    ) -> Result<StreamResult> {
        let stream = model.stream(request).await?;
        
        tokio::pin!(stream);
        
        while let Some(chunk) = stream.next().await {
            if self.abort_flag.load(Ordering::Relaxed) {
                break;  // 用户点击了"停止生成"
            }
            
            match chunk {
                Ok(delta) => {
                    self.handle_delta(&delta)?;
                }
                Err(e) => {
                    self.emit_error(&e.to_string())?;
                    return Err(e.into());
                }
            }
        }
        
        self.finalize().await
    }
    
    fn handle_delta(&mut self, delta: &StreamDelta) -> Result<()> {
        // 区分 thinking 和普通文本
        if delta.is_thinking() {
            self.accumulated_thinking.push_str(&delta.text);
            self.app_handle.emit("stream_thinking", StreamThinkingPayload {
                session_id: self.session_id.clone(),
                message_id: self.message_id.clone(),
                thinking_delta: delta.text.clone(),
            })?;
        } else {
            self.accumulated_content.push_str(&delta.text);
            self.app_handle.emit("stream_token", StreamTokenPayload {
                session_id: self.session_id.clone(),
                message_id: self.message_id.clone(),
                delta: delta.text.clone(),
                content_type: "text".to_string(),
            })?;
        }
        Ok(())
    }
}
```

### 4.3 停止生成机制

> `DashMap` 已在 `Cargo.toml` 中引入（dashmap = "6"），可直接使用。
> `StreamRegistry` 需要注册到 `AppState` 中（新增字段 `pub stream_registry: StreamRegistry`）。

```rust
/// 全局流式会话注册表，用于支持"停止生成"
pub struct StreamRegistry {
    active_streams: DashMap<String, Arc<AtomicBool>>,
    // key: session_id, value: abort_flag
}

impl StreamRegistry {
    pub fn new() -> Self {
        Self {
            active_streams: DashMap::new(),
        }
    }

    pub fn register(&self, session_id: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.active_streams.insert(session_id.to_string(), flag.clone());
        flag
    }
    
    pub fn abort(&self, session_id: &str) -> bool {
        if let Some(flag) = self.active_streams.get(session_id) {
            flag.store(true, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
    
    pub fn unregister(&self, session_id: &str) {
        self.active_streams.remove(session_id);
    }
}
```

需要更新 `lib.rs` 中的 `AppState`：

```rust
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub config: Mutex<AppConfig>,
    pub sidecar: Mutex<Option<SidecarManager>>,
    pub stream_registry: StreamRegistry,  // Phase 2 新增，无需 Mutex（DashMap 自带并发安全）
}
```

### 4.4 Anthropic Thinking Blocks 处理

Anthropic Claude 的 extended thinking 需要特殊处理：

```rust
/// Anthropic thinking blocks 通过 content_block 类型区分
fn classify_anthropic_delta(event: &StreamEvent) -> ContentType {
    match event {
        StreamEvent::ContentBlockStart { content_block } => {
            if content_block.r#type == "thinking" {
                ContentType::Thinking
            } else {
                ContentType::Text
            }
        }
        StreamEvent::ContentBlockDelta { delta } => {
            match &delta {
                Delta::ThinkingDelta { .. } => ContentType::Thinking,
                Delta::TextDelta { .. } => ContentType::Text,
                _ => ContentType::Text,
            }
        }
        _ => ContentType::Text,
    }
}
```

---

## 5. 任务 2.3：`send_message` Command（核心对话链路 + 策略模式预留）

> **预估时间：** 8h（含策略模式设计 +2h）
> **产出：** 核心对话链路 — 保存消息 → 调用 LLM → 流式返回。采用策略模式预留 Phase 4 Sidecar 切换。

### 5.1 策略模式设计（Phase 4 兼容性关键） 🆕

chat Command 的后端调用采用 `ChatBackend` trait 抽象，Phase 2 实现 `RigBackend`，Phase 4 新增 `SidecarBackend`：

```rust
// src-tauri/src/services/llm/backend.rs 🆕

/// 对话后端策略 trait — Phase 4 兼容性的关键抽象
#[async_trait]
pub trait ChatBackend: Send + Sync {
    async fn send_and_stream(
        &self,
        app: &AppHandle,
        session: &Session,
        messages: &[Message],
        user_content: &str,
        images: &Option<Vec<ImageAttachment>>,
        model: &str,
        abort_flag: Arc<AtomicBool>,
    ) -> Result<StreamResult>;
}

/// Phase 2: Rig 直调实现（临时后端 stub）
pub struct RigBackend {
    provider: LlmProvider,
}

/// Phase 4 预留接口：Sidecar 代理实现（Phase 4 时实现）
/// pub struct SidecarBackend {
///     sidecar_url: String, // "http://127.0.0.1:9527"
/// }
```

**切换逻辑：** Phase 2 硬编码使用 `RigBackend`。Phase 4 时在 `AppState` 中根据配置选择 Backend：

```rust
// Phase 4 时的切换（当前 Phase 2 不需要实现）
fn get_chat_backend(state: &AppState) -> Box<dyn ChatBackend> {
    if state.sidecar_manager.is_healthy() {
        Box::new(SidecarBackend::new(&state.sidecar_url))
    } else {
        Box::new(RigBackend::new(provider)) // 降级回退
    }
}
```

### 5.2 Command 定义

> **命令注册：** 新增的 chat commands 需要在 `commands/mod.rs` 中添加 `pub mod chat;` 和 `pub mod session;`，
> 并在 `lib.rs` 的 `invoke_handler` 中注册（追加到现有的 15 个命令之后）。

```rust
// src-tauri/src/commands/chat.rs

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    content: String,
    images: Option<Vec<ImageAttachment>>,
    model_override: Option<String>,
) -> Result<SendMessageResult, String> {
    // 整体流程（注意 Mutex 锁的生命周期管理）：
    //
    // 1. {lock db} 保存用户消息到 SQLite {unlock db}
    // 2. {lock db} 加载会话上下文（历史消息 + working_dir）{unlock db}
    // 3. 解析模型标识：model_override > session.model > default_model
    //    格式为 "config_id:model_id"，拆分得到 router_config_id 和 model_id
    // 4. {lock db} 按 router_config_id 读取 RouterConfig {unlock db}
    //    {lock config} 获取加密 master key {unlock config}
    //    调用 crypto::decrypt() 解密 API Key
    // 5. 通过 ProviderFactory::create() 构建 Box<dyn LlmProvider>
    //    再构建 ChatBackend（Phase 2: RigBackend）
    // 6. 调用 backend.send_and_stream()（全程无锁，通过 Tauri Event 推送）
    // 7. 流式完成后：{lock db} 保存 assistant 消息 + usage {unlock db}
    // 8. {lock db} 更新 session 的 last_message_at 和 token 计数 {unlock db}
    //
    // 🆕 Phase 4 切换说明：
    // 步骤 5 从 RigBackend 替换为 SidecarBackend
    // 步骤 6 从 Rig 流式调用替换为 HTTP POST Sidecar + SSE 解析
    // 步骤 1-4, 7-8 完全不变
}
```

### 5.2 消息上下文构建

```rust
/// 构建发送给 LLM 的消息上下文
fn build_chat_context(
    session: &Session,
    messages: &[Message],
    user_content: &str,
    images: &Option<Vec<ImageAttachment>>,
    max_context_messages: usize,
) -> CompletionRequest {
    let mut request = CompletionRequestBuilder::new();
    
    // System prompt
    if let Some(sp) = &session.system_prompt {
        request = request.system(sp);
    }
    
    // 历史消息（取最近 N 条，避免超上下文窗口）
    let history = messages.iter()
        .rev()
        .take(max_context_messages)
        .rev();
    for msg in history {
        match msg.role.as_str() {
            "user" => request = request.user(msg.content.clone()),
            "assistant" => request = request.assistant(msg.content.clone()),
            _ => {}
        }
    }
    
    // 当前用户消息（可能带图片）
    if let Some(imgs) = images {
        // 多模态消息：文本 + 图片
        request = request.user_multimodal(user_content, imgs);
    } else {
        request = request.user(user_content);
    }
    
    request.build()
}
```

### 5.3 其他 Chat Commands

```rust
#[tauri::command]
pub async fn stop_generation(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String>;

#[tauri::command]
pub async fn regenerate_message(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    message_id: String,
) -> Result<(), String>;

#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    session_id: String,
    limit: Option<u32>,
    before_id: Option<String>,
) -> Result<Vec<Message>, String>;
```

---

## 6. 任务 2.4：ChatView 组件

> **预估时间：** 6h
> **产出：** 完整的对话界面（消息列表 + 输入框 + 流式渲染）

### 6.1 文件结构

> **现有文件对照：**
> - `src/pages/ChatPage.tsx` — 已存在但为 EmptyState 占位，需**完全重写**
> - `src/components/layout/EmptyState.tsx` — 已有通用空状态组件，Chat 专属 EmptyState 应复用或新建
> - `src/components/chat/` — **目录不存在**，需新建

```
src/pages/
├── ChatPage.tsx                 # 重写：Chat 页面（左右分栏：会话列表 + 对话区）

src/components/chat/             # 新建目录
├── ChatView.tsx                 # 对话主视图容器
├── MessageList.tsx              # 消息列表（自动滚底）
├── MessageItem.tsx              # 单条消息渲染
├── MessageInput.tsx             # 输入框组件
├── StreamingIndicator.tsx       # 流式输出指示器（打字动画）
├── ChatEmptyState.tsx           # Chat 专属空会话欢迎页（区别于 layout/EmptyState）
├── SessionPanel.tsx             # 左侧会话列表面板
├── SessionItem.tsx              # 单个会话列表项
├── ThinkingBlock.tsx            # 思维链折叠块
├── CodeBlock.tsx                # 代码块（高亮 + 复制）
├── ImagePreview.tsx             # 图片预览（输入框中的附件预览）
└── TokenBadge.tsx               # Token 用量标签
```

### 6.2 ChatPage 布局

```
┌─────────────────────────────────────────────────────────────┐
│ Chat                                                         │
├──────────────┬──────────────────────────────────────────────┤
│ SessionPanel │  ChatView                                     │
│ (260px)      │  ┌──────────────────────────────────────────┐ │
│              │  │ MessageList                               │ │
│ ┌──────────┐ │  │                                          │ │
│ │ 🔍 搜索   │ │  │  ┌─ UserMessage ─────────────────────┐  │ │
│ │ + 新会话  │ │  │  │ 你好，请介绍一下自己               │  │ │
│ ├──────────┤ │  │  └────────────────────────────────────┘  │ │
│ │ 会话1    │ │  │                                          │ │
│ │ 会话2 ●  │ │  │  ┌─ AssistantMessage ─────────────────┐  │ │
│ │ 会话3    │ │  │  │ 🤖 你好！我是 MisakaX 的 AI 助手...  │  │ │
│ │ ...      │ │  │  │ ┌─ ThinkingBlock (可折叠) ──────┐   │  │ │
│ │          │ │  │  │ │ 💭 让我思考一下用户的意图...    │   │  │ │
│ │          │ │  │  │ └─────────────────────────────────┘   │  │ │
│ │          │ │  │  │ TokenBadge: 156 in / 423 out          │  │ │
│ │          │ │  │  └────────────────────────────────────┘  │ │
│ └──────────┘ │  │                                          │ │
│              │  └──────────────────────────────────────────┘ │
│              │  ┌──────────────────────────────────────────┐ │
│              │  │ MessageInput                              │ │
│              │  │ ┌──────────────────────────────────────┐ │ │
│              │  │ │ 📎  输入消息... (Shift+Enter 换行)    │ │ │
│              │  │ │                            [模型▼][⏎] │ │ │
│              │  │ └──────────────────────────────────────┘ │ │
│              │  └──────────────────────────────────────────┘ │
└──────────────┴──────────────────────────────────────────────┘
```

### 6.3 流式渲染机制

前端通过 Tauri Event listener 接收增量 Token，使用 Zustand 状态驱动 React 重渲染：

```typescript
// src/hooks/use-stream-listener.ts
export function useStreamListener() {
  const { appendStreamDelta, appendThinkingDelta, completeStream, handleStreamError } = useChatStore();

  useEffect(() => {
    const unlistenToken = listen<StreamTokenPayload>("stream_token", (event) => {
      appendStreamDelta(event.payload.delta);
    });
    
    const unlistenThinking = listen<StreamThinkingPayload>("stream_thinking", (event) => {
      appendThinkingDelta(event.payload.thinking_delta);
    });
    
    const unlistenComplete = listen<StreamCompletePayload>("stream_complete", (event) => {
      completeStream(event.payload.message_id, event.payload.usage);
    });
    
    const unlistenError = listen<StreamErrorPayload>("stream_error", (event) => {
      handleStreamError(event.payload.error);
    });
    
    return () => {
      unlistenToken.then(f => f());
      unlistenThinking.then(f => f());
      unlistenComplete.then(f => f());
      unlistenError.then(f => f());
    };
  }, []);
}
```

### 6.4 自动滚动策略

```typescript
// 消息列表自动滚动逻辑
// - 用户在底部时：新消息自动滚到底
// - 用户主动上滚查看历史：不自动滚动
// - 用户点击"回到底部"按钮后恢复自动滚动

function useAutoScroll(ref: RefObject<HTMLDivElement>, isStreaming: boolean) {
  const [isAtBottom, setIsAtBottom] = useState(true);
  const [showScrollButton, setShowScrollButton] = useState(false);
  
  // 监听滚动位置
  // 流式输出时若 isAtBottom 则自动滚动
}
```

---

## 7. 任务 2.5：MessageItem 组件

> **预估时间：** 6h
> **产出：** Markdown 渲染 + 代码高亮 + 复制按钮

### 7.1 Markdown 渲染方案

| 库 | 用途 | 大小 |
|---|------|------|
| `react-markdown` | Markdown → React 组件 | ~15KB |
| `remark-gfm` | GFM 扩展（表格、任务列表、删除线） | ~3KB |
| `rehype-raw` | 允许嵌入原始 HTML | ~2KB |
| `shiki` | 代码语法高亮（100+ 语言） | 按需加载 |

### 7.2 代码块组件

```typescript
// src/components/chat/CodeBlock.tsx
interface CodeBlockProps {
  code: string;
  language?: string;
}

export function CodeBlock({ code, language }: CodeBlockProps) {
  // 1. shiki 高亮
  // 2. 语言标签显示
  // 3. 复制按钮（点击后显示"已复制"反馈）
  // 4. 行号显示
  // 5. 代码块可横向滚动
}
```

### 7.3 消息渲染变体

| 角色 | 视觉 | 特殊处理 |
|------|------|---------|
| `user` | 右对齐，强调色背景圆角气泡 | 图片附件预览 |
| `assistant` | 左对齐，卡片背景 + 模型图标 | Markdown 渲染、ThinkingBlock、TokenBadge |
| `system` | 居中，小字灰色 | 一般不显示 |

### 7.4 需要安装的 npm 依赖

```bash
npm install react-markdown remark-gfm rehype-raw shiki
```

---

## 8. 任务 2.6：MessageInput 组件

> **预估时间：** 4h
> **产出：** 多行文本输入框 + 快捷键 + 模型选择 + 附件

### 8.1 功能矩阵

| 功能 | 快捷键 | 说明 |
|------|--------|------|
| 发送消息 | `Enter` | 发送当前输入 |
| 换行 | `Shift + Enter` | 插入换行符 |
| 粘贴图片 | `Ctrl/Cmd + V` | 从剪贴板粘贴图片 |
| 选择文件 | 点击 📎 | 打开文件选择对话框 |
| 切换模型 | 点击模型下拉 | 临时切换当前对话使用的模型 |

### 8.2 组件结构

```
┌─────────────────────────────────────────────────────────────┐
│ ┌─ 图片预览区（有附件时显示）──────────────────────────────┐ │
│ │ [🖼 img1.png ✕]  [🖼 img2.png ✕]                        │ │
│ └──────────────────────────────────────────────────────────┘ │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ 📎 │ 输入消息... (Shift+Enter 换行)         [模型▼] [⏎] │ │
│ │    │                                                     │ │
│ │    │ (自动高度，最大 200px)                                │ │
│ └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 8.3 Textarea 自适应高度

```typescript
// textarea 高度随内容自动增长，最小 1 行，最大 ~200px
function autoResize(textarea: HTMLTextAreaElement) {
  textarea.style.height = "auto";
  textarea.style.height = Math.min(textarea.scrollHeight, 200) + "px";
}
```

### 8.4 模型选择器（分组下拉 + 能力标识）

在输入框右下角嵌入一个模型选择 Popover，按 Provider 分组展示所有可用模型：

```typescript
// src/components/chat/ModelSelector.tsx

interface ModelSelectorProps {
  selectedConfigId: string | null;
  selectedModelId: string | null;
  onChange: (configId: string, modelId: string) => void;
}

export function ModelSelector({ selectedConfigId, selectedModelId, onChange }: ModelSelectorProps) {
  const [providerModels, setProviderModels] = useState<ProviderModels[]>([]);

  useEffect(() => {
    modelsIpc.listAvailableModels().then(setProviderModels);
  }, []);

  // Popover 内容按 Provider 分组
  // 每个模型行显示: 模型名 + 能力标签（👁️视觉 🧠思维链）+ 是否自定义
  // 选中后回传 router_config_id + model_id
}
```

**交互要点：**
- 对话中选择模型时传递 `router_config_id + model_id`（非仅 model 名），确定走哪个 Provider
- 切换模型仅影响当前会话，不改变全局默认
- 已有的 `session.model` 字段格式从纯 model 名变为 `config_id:model_id`
- 当 Provider 被删除时，对话自动降级到默认模型

---

## 9. 任务 2.7：工作目录选择器 🆕

> **预估时间：** 6h
> **产出：** 本地路径浏览 + 目录验证 + 新建会话时的目录选择 UI

### 9.1 设计目标

工作目录选择器是 MisakaX 区别于普通聊天客户端的核心 UX 差异点。新建对话时弹出目录选择器，用户可以：

1. 选择一个本地项目目录作为工作目录
2. 使用"最近使用的目录"快速选择
3. 选择"无目录模式"跳过（降级为纯对话，Phase 4 时无文件操作能力）

### 9.2 前端组件

```
┌─────────────── 新建对话 ─────────────────────┐
│                                               │
│  选择工作目录                                   │
│  ┌───────────────────────────────────────────┐ │
│  │ 📂 最近使用                                │ │
│  │  ├── C:\Projects\my-app          (3天前)   │ │
│  │  ├── D:\code\Misaka-Tauri        (今天)   │ │
│  │  └── ~/Documents/notes           (1周前)   │ │
│  ├───────────────────────────────────────────┤ │
│  │ [📁 浏览本地目录...]                       │ │
│  │ [🔗 远程连接...] (Phase 4+)                │ │
│  ├───────────────────────────────────────────┤ │
│  │ [跳过，不设置工作目录]                      │ │
│  └───────────────────────────────────────────┘ │
│                                               │
│  当前选择: D:\code\Misaka-Tauri               │
│                                [取消] [确认]   │
└───────────────────────────────────────────────┘
```

### 9.3 Rust 端目录操作 Commands

```rust
// src-tauri/src/commands/workspace.rs

#[tauri::command]
pub fn browse_directory(
    state: State<'_, AppState>,
    start_path: Option<String>,
) -> Result<Option<String>, String>;
// 调用 Tauri 原生文件夹选择对话框

#[tauri::command]
pub fn validate_directory(
    path: String,
) -> Result<DirectoryInfo, String>;
// 验证路径存在、可读写，返回目录元信息

#[tauri::command]
pub fn get_recent_directories(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<RecentDirectory>, String>;
// 从 SQLite 读取最近使用的工作目录
```

### 9.4 数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryInfo {
    pub path: String,
    pub name: String,           // 目录名（最后一段路径）
    pub exists: bool,
    pub readable: bool,
    pub writable: bool,
    pub file_count: Option<u32>, // 目录下文件数（可选）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentDirectory {
    pub path: String,
    pub name: String,
    pub last_used_at: String,
    pub use_count: u32,
}
```

### 9.5 数据库表（migrate_v2 新增）

```sql
-- 最近使用的工作目录（独立表，跨会话共享）
CREATE TABLE IF NOT EXISTS recent_directories (
    path TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    last_used_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    use_count INTEGER DEFAULT 1
);
```

---

## 10. 任务 2.8：会话-工作目录绑定 🆕

> **预估时间：** 3h
> **产出：** Session 创建时绑定工作目录，持久化到 SQLite

### 10.1 创建会话流程变更

原流程：点击"新建" → 直接创建空会话
新流程：点击"新建" → 弹出工作目录选择器 → 选择后创建会话（working_dir 已绑定）

```rust
#[tauri::command]
pub fn create_session(
    state: State<'_, AppState>,
    title: Option<String>,
    model: Option<String>,
    working_directory: Option<String>,      // 🆕 本地工作目录
    working_dir_remote: Option<String>,     // 🆕 远程目录（Phase 4+）
    working_dir_remote_type: Option<String>,// 🆕 远程类型（Phase 4+）
) -> Result<Session, String>;
```

### 10.2 目录验证与记录

```rust
// 创建会话时的目录处理逻辑
fn bind_working_directory(
    conn: &Connection,
    session_id: &str,
    working_dir: &Option<String>,
) -> Result<()> {
    if let Some(dir) = working_dir {
        // 1. 验证目录存在且可访问
        // 2. 更新 session 的 working_directory 字段
        // 3. 更新 recent_directories 表（use_count +1, last_used_at 更新）
    }
    Ok(())
}
```

---

## 11. 任务 2.9：工作目录状态指示栏 🆕

> **预估时间：** 3h
> **产出：** WorkspaceBar 组件，显示当前会话的工作目录路径

### 11.1 组件位置

WorkspaceBar 位于 ChatView 的顶部，MessageList 之上：

```
┌──────────────────────────────────────────────────┐
│ 📂 D:\code\Misaka-Tauri  ·  Misaka-Tauri  [切换] │  ← WorkspaceBar
├──────────────────────────────────────────────────┤
│ MessageList                                       │
│ ...                                              │
```

### 11.2 组件设计

```typescript
// src/components/chat/WorkspaceBar.tsx
interface WorkspaceBarProps {
  workingDir: string | null;
  onChangeDir: () => void;
}

export function WorkspaceBar({ workingDir, onChangeDir }: WorkspaceBarProps) {
  if (!workingDir) {
    return (
      <div className="flex items-center gap-2 px-4 py-1.5 border-b text-xs text-muted-foreground">
        <FolderOpen className="h-3.5 w-3.5" />
        <span>未设置工作目录</span>
        <button onClick={onChangeDir} className="text-primary hover:underline">
          选择目录
        </button>
      </div>
    );
  }
  return (
    <div className="flex items-center gap-2 px-4 py-1.5 border-b text-xs">
      <FolderOpen className="h-3.5 w-3.5 text-primary" />
      <span className="text-muted-foreground truncate max-w-[300px]">{workingDir}</span>
      <span className="text-muted-foreground">·</span>
      <span className="font-medium">{dirName(workingDir)}</span>
      <button onClick={onChangeDir} className="ml-auto text-muted-foreground hover:text-foreground">
        切换
      </button>
    </div>
  );
}
```

### 11.3 Phase 4 增强预留

Phase 4 时 WorkspaceBar 将额外显示：
- 远程目录连接状态（绿/红点）
- DeepAgents FilesystemMiddleware 状态
- 当前 Agent 正在操作的文件路径

---

## 12. 任务 2.10：会话列表侧边栏

> **预估时间：** 5h
> **产出：** 会话 CRUD + Rust 后端

### 12.1 Rust Commands

```rust
// src-tauri/src/commands/session.rs

#[tauri::command]
pub fn create_session(
    state: State<'_, AppState>,
    title: Option<String>,
    model: Option<String>,
    working_directory: Option<String>,      // 🆕 绑定工作目录
) -> Result<Session, String>;

#[tauri::command]
pub fn list_sessions(
    state: State<'_, AppState>,
    status: Option<String>,  // "active" | "archived"
) -> Result<Vec<Session>, String>;

#[tauri::command]
pub fn update_session(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    model: Option<String>,
    pinned: Option<bool>,
    status: Option<String>,
) -> Result<(), String>;

#[tauri::command]
pub fn delete_session(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String>;

#[tauri::command]
pub fn search_sessions(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<Session>, String>;
```

### 12.2 会话列表 UI 设计

**SessionPanel 组件：**
- 顶部：搜索框 + "新建会话"按钮
- 中间：会话列表（按 `last_message_at` 倒序排列）
- 每个 SessionItem 显示：标题（或自动生成的摘要）、最后消息时间、消息数
- 右键菜单：重命名、删除、归档
- 当前活跃会话高亮显示

**自动命名策略：**
- 新会话标题为 "New Chat"
- 第一条用户消息发送后，使用 LLM 自动生成一个简短标题（<=10字）
- 自动命名不阻塞主对话流，后台异步执行

```rust
// 自动命名逻辑（在 send_message 完成后异步执行）
async fn auto_title_session(provider: &LlmProvider, model: &str, first_message: &str) -> String {
    let prompt = format!(
        "Based on this message, generate a concise title (max 10 words, no quotes):\n{}",
        first_message
    );
    // 调用 LLM 生成标题
}
```

---

## 13. 任务 2.11：消息持久化

> **预估时间：** 3h
> **产出：** 完整的消息 SQLite 存储层

### 13.1 数据库操作

```rust
// src-tauri/src/db/messages.rs

pub fn save_message(conn: &Connection, msg: &NewMessage) -> Result<String>;
pub fn get_messages(conn: &Connection, session_id: &str, limit: u32, before_id: Option<&str>) -> Result<Vec<Message>>;
pub fn update_message_content(conn: &Connection, id: &str, content: &str) -> Result<()>;
pub fn update_message_status(conn: &Connection, id: &str, status: &str) -> Result<()>;
pub fn update_message_usage(conn: &Connection, id: &str, usage: &TokenUsage) -> Result<()>;
pub fn delete_message(conn: &Connection, id: &str) -> Result<()>;
pub fn delete_messages_after(conn: &Connection, session_id: &str, message_id: &str) -> Result<()>;
```

### 13.2 Schema 迁移

在 `migrations.rs` 中取消 v2 注释（已预留 `// if current_version < 2 { migrate_v2(conn)?; }`）并实现 `migrate_v2()`：

```rust
// 在 run_migrations() 中取消注释：
if current_version < 2 {
    migrate_v2(conn)?;
}

fn migrate_v2(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        ALTER TABLE messages ADD COLUMN model TEXT;
        ALTER TABLE messages ADD COLUMN thinking_content TEXT;
        ALTER TABLE messages ADD COLUMN attachments TEXT;
        ALTER TABLE messages ADD COLUMN status TEXT DEFAULT 'complete';

        ALTER TABLE sessions ADD COLUMN total_input_tokens INTEGER DEFAULT 0;
        ALTER TABLE sessions ADD COLUMN total_output_tokens INTEGER DEFAULT 0;
        ALTER TABLE sessions ADD COLUMN last_message_at DATETIME;
        ALTER TABLE sessions ADD COLUMN pinned INTEGER DEFAULT 0;
        ALTER TABLE sessions ADD COLUMN group_name TEXT;

        INSERT INTO _schema_version (version) VALUES (2);
    ")?;
    tracing::info!("Database migrated to version 2");
    Ok(())
}
```

### 13.3 消息 FTS 同步

用户消息和 assistant 消息写入 `messages` 表时，同步写入 `messages_fts` 虚拟表用于全文搜索。

> `messages_fts` 已在 v1 迁移中创建：`CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(content, session_id, role);`
> 注意 FTS5 不使用 rowid 参数，直接 INSERT 即可（会自动分配内部 rowid）。

```rust
fn sync_message_fts(conn: &Connection, id: &str, content: &str, session_id: &str, role: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO messages_fts(content, session_id, role) VALUES (?1, ?2, ?3)",
        params![content, session_id, role],
    )?;
    Ok(())
}
```

---

## 14. 任务 2.12：Token 用量统计

> **预估时间：** 2h
> **产出：** 单消息 + 单会话累计 Token 统计

### 14.1 Token 数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: Option<u32>,
    pub cache_creation_tokens: Option<u32>,
    pub total_tokens: u32,
}
```

### 14.2 显示位置

- **消息级别：** 每条 assistant 消息底部显示 `TokenBadge`，格式：`↑156 ↓423`
- **会话级别：** ChatView 顶部的 TopBar 区域显示当前会话累计 Token

### 14.3 TokenBadge 组件

```typescript
// src/components/chat/TokenBadge.tsx
function TokenBadge({ usage }: { usage: TokenUsage }) {
  return (
    <div className="flex items-center gap-2 text-xs text-muted-foreground">
      <span>↑{usage.input_tokens}</span>
      <span>↓{usage.output_tokens}</span>
      {usage.cache_read_tokens && <span>📦{usage.cache_read_tokens}</span>}
    </div>
  );
}
```

---

## 15. 任务 2.13：思维链展示

> **预估时间：** 3h
> **产出：** Anthropic extended thinking blocks 折叠/展开

### 15.1 ThinkingBlock 组件

```typescript
// src/components/chat/ThinkingBlock.tsx
interface ThinkingBlockProps {
  content: string;
  isStreaming?: boolean;
}

export function ThinkingBlock({ content, isStreaming }: ThinkingBlockProps) {
  const [expanded, setExpanded] = useState(false);
  
  return (
    <Collapsible open={expanded} onOpenChange={setExpanded}>
      <CollapsibleTrigger className="flex items-center gap-2 text-sm text-muted-foreground">
        <Brain className="h-4 w-4" />
        <span>Thinking{isStreaming ? "..." : ""}</span>
        <ChevronRight className={cn("h-4 w-4 transition-transform", expanded && "rotate-90")} />
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div className="mt-2 rounded-md bg-muted/30 p-3 text-sm text-muted-foreground italic">
          {content}
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
}
```

### 15.2 流式思维链

思维链在流式过程中实时更新：
- 正在流式时，ThinkingBlock 标题显示 `Thinking...` + 旋转图标
- 内容增量追加
- 默认折叠，用户可点击展开查看过程
- 流式结束后，`isStreaming` 设为 `false`

### 15.3 需要安装的 shadcn/ui 组件

```bash
npx shadcn@latest add collapsible
```

---

## 16. 任务 2.14：停止生成 / 重新生成

> **预估时间：** 2h
> **产出：** 中途停止 + 重新生成

### 16.1 停止生成

**前端：** 流式输出中，输入框区域的"发送"按钮变为"停止"按钮（⏹），点击调用 `invoke("stop_generation")`。

**后端：** `StreamRegistry` 设置 abort flag，流式循环检测到 flag 后中断，保存已生成的部分内容，消息状态标记为 `"stopped"`。

### 16.2 重新生成

**前端：** 每条 assistant 消息的操作栏中显示"重新生成"按钮 (🔄)。

**后端逻辑：**
1. 删除目标 assistant 消息
2. 重新构建上下文（到该消息之前的用户消息为止）
3. 发起新的流式调用
4. 流式结果作为新的 assistant 消息保存

```rust
#[tauri::command]
pub async fn regenerate_message(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    message_id: String,
) -> Result<(), String> {
    // 1. 获取目标消息的前一条用户消息
    // 2. 删除目标 assistant 消息
    // 3. 以前一条用户消息为基础重新调用 send_message 逻辑
}
```

---

## 17. 任务 2.15：多模态输入

> **预估时间：** 4h
> **产出：** 图片粘贴/拖拽附件

### 17.1 图片输入方式

| 方式 | 实现 |
|------|------|
| 剪贴板粘贴 | 监听 `paste` 事件，提取 `clipboardData.items` 中的图片 |
| 文件拖拽 | 监听 `dragover` + `drop` 事件 |
| 文件选择 | 点击 📎 按钮，打开文件选择对话框 |

### 17.2 图片处理流程

```
用户粘贴/拖拽图片
    │
    ├── 检查文件类型（仅支持 image/png, image/jpeg, image/gif, image/webp）
    ├── 检查文件大小（限制 <=10MB）
    ├── 转换为 Base64 编码
    ├── 在输入框上方显示缩略图预览（可删除）
    │
    └── 发送时作为 images 参数传递给 send_message
```

### 17.3 Rust 端传递给 LLM

```rust
// Rig 多模态消息构建
fn build_multimodal_message(
    content: &str,
    images: &[ImageAttachment],
) -> Vec<ContentBlock> {
    let mut blocks = vec![];
    
    for img in images {
        blocks.push(ContentBlock::Image {
            source: ImageSource::Base64 {
                media_type: img.mime_type.clone(),
                data: img.data.clone(),
            }
        });
    }
    
    blocks.push(ContentBlock::Text {
        text: content.to_string(),
    });
    
    blocks
}
```

### 17.4 ImageAttachment 类型

```typescript
// src/lib/ipc/types.ts
interface ImageAttachment {
  data: string;       // Base64 encoded
  mime_type: string;  // "image/png" | "image/jpeg" | etc.
  name?: string;      // 文件名
  size?: number;      // 原始文件大小
}
```

---

## 18. Phase 2 完整验证清单

### 18.1 核心对话验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-1 | 选择 OpenAI 模型发送消息 | Token 流式显示，回复完整 |
| V-2 | 选择 Anthropic 模型发送消息 | Token 流式显示，回复完整 |
| V-3 | 选择 Gemini 模型发送消息 | Token 流式显示，回复完整 |
| V-4 | 连续多轮对话 | 模型正确引用之前的对话内容 |
| V-5 | 发送包含代码需求的消息 | 返回代码块，有语法高亮 |
| V-6 | 代码块复制按钮 | 点击后代码复制到剪贴板 |
| V-7 | 流式输出期间自动滚动 | 消息列表保持滚动到底部 |
| V-8 | 手动上滚后不自动滚动 | 上滚查看历史时不被新 token 打断 |

### 18.2 会话管理验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-9 | 点击"新建会话" | 创建空白会话，清空对话区 |
| V-10 | 切换会话 | 消息列表切换到对应会话的历史消息 |
| V-11 | 删除会话 | 确认后删除，消息也清除 |
| V-12 | 搜索会话 | 输入关键词过滤会话列表 |
| V-13 | 会话自动命名 | 第一条消息发送后，会话标题自动更新 |
| V-14 | 重启验证 | 关闭再打开应用，所有会话和消息仍存在 |

### 18.3 工作目录验证 🆕

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-15 | 新建会话弹出目录选择器 | 显示最近目录列表 + 浏览按钮 + 跳过选项 |
| V-16 | 选择本地目录 | 目录验证通过，会话 working_directory 正确保存 |
| V-17 | 浏览目录按钮 | 弹出系统文件夹选择对话框 |
| V-18 | 跳过目录选择 | 创建无工作目录的会话，WorkspaceBar 显示"未设置" |
| V-19 | WorkspaceBar 显示 | 显示当前目录路径 + 目录名 + 切换按钮 |
| V-20 | 最近目录记录 | 使用过的目录在下次新建会话时出现在最近列表 |
| V-21 | 切换工作目录 | 点击切换后重新选择目录，session 更新 |

### 18.4 Token 统计验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-22 | 单消息 Token 显示 | assistant 消息底部显示 input/output token 数 |
| V-23 | 会话累计 Token | 多次对话后累计值正确增长 |

### 18.5 思维链验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-24 | Anthropic thinking 显示 | 使用 Claude 时，thinking block 可见 |
| V-25 | 折叠/展开 | 默认折叠，点击可展开/收起 |
| V-26 | 流式 thinking | thinking 内容实时增量显示 |

### 18.6 控制流验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-27 | 停止生成 | 点击停止后，流式中断，已生成内容保留 |
| V-28 | 重新生成 | 点击后删除旧回复，重新调用 LLM |
| V-29 | 快速连续发送 | 不会出现消息错乱 |

### 18.7 多模态验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-30 | 粘贴图片 | 输入框上方显示缩略图预览 |
| V-31 | 拖拽图片 | 同粘贴效果 |
| V-32 | 带图片发送 | 视觉模型正确理解图片内容 |
| V-33 | 删除附件 | 缩略图上的 × 可移除 |

### 18.8 错误处理验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-34 | 无效 API Key | 友好错误提示（Toast），不崩溃 |
| V-35 | 网络断开 | 超时后提示错误，可重试 |
| V-36 | 模型不存在 | 友好错误提示 |

---

## 19. 详细 TODO 列表

> 按任务分组的可执行 TODO，推荐按顺序逐项完成。每项完成时勾选 `[x]`。

### 2.1 LLM Provider 封装（trait 对象分层 + 自定义模型） [预估 8h] 🆕重写

- [x] **2.1.1** 在 `Cargo.toml` 末尾新增依赖：`rig-core`、`futures`、`async-trait`
- [x] **2.1.2** 在 `lib.rs` 添加 `pub mod services;`
- [x] **2.1.3** 创建 `src-tauri/src/services/mod.rs`（`pub mod llm;`）
- [x] **2.1.4** 创建 `src-tauri/src/services/llm/mod.rs`（模块导出 + re-exports）
- [x] **2.1.5** 创建 `traits.rs`（`LlmProvider` trait 定义：`provider_type` / `build_agent` / `supports_vision` / `supports_thinking`）
- [x] **2.1.6** 创建 `providers/` 子目录 + `mod.rs`
- [x] **2.1.7** 实现 `providers/openai_provider.rs`（`OpenAiProvider` struct + `LlmProvider` trait 实现）
- [x] **2.1.8** 实现 `providers/anthropic_provider.rs`（`AnthropicProvider`）
- [x] **2.1.9** 实现 `providers/gemini_provider.rs`（`GeminiProvider`）
- [x] **2.1.10** 实现 `providers/openai_compat.rs`（`OpenAiCompatProvider`，处理 DeepSeek/自定义等所有兼容接口）
- [x] **2.1.11** 创建 `factory.rs`（`ProviderFactory::create()` 工厂方法，根据 `RouterConfig.provider` + `api_compat` 创建 `Box<dyn LlmProvider>`）
- [x] **2.1.12** 创建 `registry.rs`（`ModelRegistry`：内置模型列表 + 从 `custom_models` 表读取自定义模型）
- [x] **2.1.13** 创建 `config.rs`（`LlmConfig`：温度、max_tokens、top_p 等运行时参数）
- [x] **2.1.14** 在 `migrate_v2()` 中新增 `custom_models` 表 + `ALTER TABLE router_configs ADD COLUMN api_compat`
- [x] **2.1.15** 在 `db/models.rs` 新增 `CustomModel` 和 `ModelInfo` 结构体
- [x] **2.1.16** 创建 `src-tauri/src/commands/models.rs`（`list_available_models` / `add_custom_model` / `delete_custom_model`）
- [x] **2.1.17** 在 `commands/mod.rs` 添加 `pub mod models;` + `lib.rs` 注册新 commands
- [x] **2.1.18** 编写单元测试验证 ProviderFactory + ModelRegistry
- [x] **2.1.19** `cargo check` 编译通过

### 2.2 Rust 流式 LLM 调用 + Tauri Event 推送 [预估 8h]

- [x] **2.2.1** 创建 `src-tauri/src/services/llm/streaming.rs`（StreamSession struct）
- [x] **2.2.2** 定义 Event Payload 结构体（StreamTokenPayload / StreamThinkingPayload / StreamCompletePayload / StreamErrorPayload），全部派生 `Clone + Serialize`
- [x] **2.2.3** 实现 `StreamSession::execute_stream()` — 主流式循环 + 通过 `AppHandle::emit()` 推送 Event
- [x] **2.2.4** 实现 `StreamSession::handle_delta()` — 区分 text / thinking 类型
- [x] **2.2.5** 实现 `StreamSession::finalize()` — 流式结束后汇总并 emit stream_complete
- [x] **2.2.6** 实现 `StreamSession::emit_error()` — 错误 Event 推送
- [x] **2.2.7** 创建 `StreamRegistry`（使用已有的 `dashmap::DashMap`，支持 abort）
- [x] **2.2.8** 在 `lib.rs` 的 `AppState` 中新增 `pub stream_registry: StreamRegistry` 字段（无需 Mutex）
- [x] **2.2.9** 在 `run()` 函数的 `.manage(AppState {...})` 中初始化 `stream_registry: StreamRegistry::new()`
- [x] **2.2.10** 实现 Anthropic thinking blocks 的特殊处理逻辑
- [x] **2.2.11** `cargo check` 编译通过

### 2.3 send_message Command + 策略模式预留 [预估 8h] 🆕更新

- [x] **2.3.1** 创建 `src-tauri/src/services/llm/backend.rs`（ChatBackend trait + RigBackend struct）🆕
- [x] **2.3.2** 实现 `ChatBackend` trait（`send_and_stream` 方法签名，Phase 4 兼容性关键抽象）🆕
- [x] **2.3.3** 实现 `RigBackend`（Phase 2 的临时 Rig 直调实现，实现 ChatBackend trait）🆕
- [x] **2.3.4** 创建 `src-tauri/src/commands/chat.rs`
- [x] **2.3.5** 实现 `send_message` command（通过 ChatBackend trait 调用，注意 Mutex 锁分段获取/释放）
- [x] **2.3.6** 实现消息上下文构建函数 `build_chat_context()`
- [x] **2.3.7** 实现多模态消息构建（文本 + 图片 Base64）
- [x] **2.3.8** 实现 `stop_generation` command（调用 `state.stream_registry.abort()`）
- [x] **2.3.9** 实现 `regenerate_message` command
- [x] **2.3.10** 实现 `get_messages` command（带分页：limit + before_id）
- [x] **2.3.11** 在 `commands/mod.rs` 中添加 `pub mod chat;`（现有：`pub mod router_configs; pub mod settings;`）
- [x] **2.3.12** 在 `lib.rs` 的 `invoke_handler` 中注册新 commands（追加到现有 13 条之后）
- [x] **2.3.13** `cargo check` 编译通过

### 2.7 工作目录选择器 [预估 6h] 🆕

- [ ] **2.7.1** 创建 `src-tauri/src/commands/workspace.rs`（browse_directory / validate_directory / get_recent_directories）
- [ ] **2.7.2** 在 `commands/mod.rs` 添加 `pub mod workspace;`
- [ ] **2.7.3** 在 `lib.rs` 的 `invoke_handler` 中注册 workspace commands
- [ ] **2.7.4** 定义 `DirectoryInfo` 和 `RecentDirectory` 数据结构
- [ ] **2.7.5** 在 `migrate_v2()` 中新增 `recent_directories` 表
- [ ] **2.7.6** 实现 `browse_directory`：调用 Tauri 原生文件夹选择对话框
- [ ] **2.7.7** 实现 `validate_directory`：验证路径存在、可读写，返回目录元信息
- [ ] **2.7.8** 实现 `get_recent_directories`：从 SQLite 读取最近使用的工作目录
- [ ] **2.7.9** 创建 `src/lib/ipc/workspace.ts`（workspaceIpc：browseDirectory/validateDirectory/getRecentDirectories）
- [ ] **2.7.10** 创建 `src/components/chat/WorkspaceSelector.tsx`（工作目录选择器弹窗组件）
- [ ] **2.7.11** 实现最近目录列表 + 浏览按钮 + 跳过选项 UI
- [ ] **2.7.12** `cargo check` 编译通过

### 2.8 会话-工作目录绑定 [预估 3h] 🆕

- [ ] **2.8.1** 修改 `create_session` command，新增 `working_directory` 参数
- [ ] **2.8.2** 实现 `bind_working_directory()` 逻辑（验证 + 更新 session + 更新 recent_directories）
- [ ] **2.8.3** 在 `migrate_v2()` 中添加 `ALTER TABLE sessions ADD COLUMN working_dir_remote TEXT` 和 `working_dir_remote_type TEXT`
- [ ] **2.8.4** 更新前端 `sessionsIpc.create()` 传递 `working_directory` 参数
- [ ] **2.8.5** 修改新建会话流程：点击"新建"→ 弹出工作目录选择器 → 选择后创建会话
- [ ] **2.8.6** 验证会话创建时工作目录正确持久化到 SQLite

### 2.9 工作目录状态指示栏 [预估 3h] 🆕

- [ ] **2.9.1** 创建 `src/components/chat/WorkspaceBar.tsx`（工作目录状态指示栏组件）
- [ ] **2.9.2** 实现已设置目录 / 未设置目录两种显示状态
- [ ] **2.9.3** 实现"切换目录"按钮功能（复用 WorkspaceSelector 弹窗）
- [ ] **2.9.4** 在 ChatView 中集成 WorkspaceBar（MessageList 之上）
- [ ] **2.9.5** 验证目录显示、切换流程完整性

### 2.10 会话列表侧边栏 [预估 5h]

- [ ] **2.10.1** 创建 `src-tauri/src/commands/session.rs`（create / list / update / delete / search，create 需含 working_directory）
- [ ] **2.10.2** 在 `commands/mod.rs` 添加 `pub mod session;`
- [ ] **2.10.3** 在 `lib.rs` 的 `invoke_handler` 中注册 session commands
- [ ] **2.10.4** 创建 `src/lib/ipc/sessions.ts`（sessionsIpc：create/list/update/delete/search）
- [ ] **2.10.5** 创建 `src/lib/ipc/chat.ts`（chatIpc：sendMessage/stopGeneration/regenerateMessage/getMessages）
- [ ] **2.10.6** 更新 `src/lib/ipc/types.ts`（新增 Message / TokenUsage / ImageAttachment / StreamPayload 类型，Session 已有但需增强）
- [ ] **2.10.7** 更新 `src/lib/ipc/index.ts`（导出 sessionsIpc / chatIpc / workspaceIpc）
- [ ] **2.10.8** 创建 `src/components/chat/SessionPanel.tsx`（会话列表面板 + 搜索框 + 新建按钮）
- [ ] **2.10.9** 创建 `src/components/chat/SessionItem.tsx`（单个会话项 + 右键菜单：重命名/删除/归档）
- [ ] **2.10.10** 实现会话搜索过滤（前端调用 Rust search_sessions FTS 查询）
- [ ] **2.10.11** 实现会话自动命名逻辑（后台异步调用 LLM 生成标题，不阻塞主对话流）
- [ ] **2.10.12** 验证 CRUD 全流程 + 持久化（重启后会话仍在）

### 2.11 消息持久化 [预估 3h]

- [ ] **2.11.1** 在 `migrations.rs` 中实现 `migrate_v2()`（ALTER TABLE messages + ALTER TABLE sessions + CREATE TABLE recent_directories）
- [ ] **2.11.2** 在 `run_migrations()` 中取消 v2 注释：`if current_version < 2 { migrate_v2(conn)?; }`
- [ ] **2.11.3** 更新 `db/models.rs` 中的 `Message` 结构体（新增 model / thinking_content / attachments / status 字段）
- [ ] **2.11.4** 更新 `db/models.rs` 中的 `Session` 结构体（新增 total_input_tokens / total_output_tokens / last_message_at / pinned / group_name 字段）
- [ ] **2.11.5** 新增 `db/models.rs` 中的 `TokenUsage` 和 `ImageAttachment` 结构体
- [ ] **2.11.6** 创建 `src-tauri/src/db/messages.rs`（save / get / update_content / update_status / update_usage / delete / delete_after）
- [ ] **2.11.7** 创建 `src-tauri/src/db/sessions.rs`（create / list / update / delete / search / update_token_counts）
- [ ] **2.11.8** 在 `db/mod.rs` 中添加 `pub mod messages; pub mod sessions;`
- [ ] **2.11.9** 实现 FTS 同步写入（messages_fts，已在 v1 建表）
- [ ] **2.11.10** 编写单元测试验证消息/会话读写
- [ ] **2.11.11** `cargo check` 编译通过

### 2.4 ChatView 组件 [预估 6h]

- [ ] **2.4.1** 重写 `src/pages/ChatPage.tsx`（从 EmptyState 占位替换为左右分栏：SessionPanel + ChatView）
- [ ] **2.4.2** 创建 `src/components/chat/ChatView.tsx`（对话主视图容器）
- [ ] **2.4.3** 创建 `src/components/chat/MessageList.tsx`（消息列表 + 自动滚动）
- [ ] **2.4.4** 创建 `src/components/chat/ChatEmptyState.tsx`（Chat 专属空会话欢迎页，可复用 `layout/EmptyState` 风格）
- [ ] **2.4.5** 创建 `src/components/chat/StreamingIndicator.tsx`（流式输出指示器）
- [ ] **2.4.6** 完善 `src/stores/chat-store.ts`（从现有 22 行骨架扩展为完整 ChatState + 所有 Actions）
- [ ] **2.4.7** 创建 `src/hooks/use-stream-listener.ts`（`@tauri-apps/api` listen() 监听 4 种 Event）
- [ ] **2.4.8** 实现自动滚动策略（isAtBottom 检测 + 回到底部按钮）
- [ ] **2.4.9** 验证完整对话流程（发送 → 流式渲染 → 完成）

### 2.5 MessageItem 组件 [预估 6h]

- [ ] **2.5.1** 安装依赖：`npm install react-markdown remark-gfm rehype-raw shiki`
- [ ] **2.5.2** 创建 `src/components/chat/MessageItem.tsx`（消息渲染主组件）
- [ ] **2.5.3** 创建 `src/components/chat/CodeBlock.tsx`（shiki 代码高亮 + 复制按钮 + 语言标签）
- [ ] **2.5.4** 实现 user / assistant 两种消息变体的视觉样式
- [ ] **2.5.5** 集成 react-markdown + remark-gfm 渲染 Markdown 内容
- [ ] **2.5.6** 实现 CodeBlock 的自定义渲染器（替换 react-markdown 默认代码块）
- [ ] **2.5.7** 创建 `src/components/chat/TokenBadge.tsx`（Token 用量标签）
- [ ] **2.5.8** 实现消息操作栏（复制消息 / 重新生成按钮）
- [ ] **2.5.9** 验证 Markdown 渲染效果（标题/列表/表格/代码/链接）

### 2.6 MessageInput 组件 [预估 4h]

- [ ] **2.6.1** 创建 `src/components/chat/MessageInput.tsx`（多行输入框 + 发送/停止按钮）
- [ ] **2.6.2** 实现 Enter 发送 / Shift+Enter 换行
- [ ] **2.6.3** 实现 Textarea 自适应高度（最小 1 行，最大 200px）
- [ ] **2.6.4** 实现模型选择器下拉（读取已配置的 Provider 模型列表）
- [ ] **2.6.5** 实现流式中 发送按钮→停止按钮 的状态切换
- [ ] **2.6.6** 验证键盘快捷键和发送逻辑

### 2.12 Token 用量统计 [预估 2h]

- [ ] **2.12.1** 定义 `TokenUsage` 结构体（Rust + TypeScript 两端）
- [ ] **2.12.2** 在 `stream_complete` 事件中传递 usage 数据
- [ ] **2.12.3** 在 `save_message` 时写入 token_usage 字段（JSON）
- [ ] **2.12.4** 在 assistant 消息完成后更新 session 的 total_input_tokens / total_output_tokens
- [ ] **2.12.5** 在 MessageItem 底部显示 TokenBadge 组件
- [ ] **2.12.6** 验证 Token 统计数据正确

### 2.13 思维链展示 [预估 3h]

- [ ] **2.13.1** 安装 shadcn/ui 组件：`npx shadcn@latest add collapsible`
- [ ] **2.13.2** 创建 `src/components/chat/ThinkingBlock.tsx`（折叠/展开组件）
- [ ] **2.13.3** 在 `MessageItem` 中集成 ThinkingBlock（当 thinking_content 非空时显示）
- [ ] **2.13.4** 实现流式 thinking 内容的实时更新（通过 stream_thinking 事件）
- [ ] **2.13.5** 验证 Anthropic Claude thinking blocks 完整流程

### 2.14 停止生成 / 重新生成 [预估 2h]

- [ ] **2.14.1** 在 MessageInput 中实现 发送→停止 按钮切换
- [ ] **2.14.2** 实现前端 `stopGeneration()` 调用 → Rust `stop_generation` command
- [ ] **2.14.3** 在 MessageItem 操作栏中添加"重新生成"按钮
- [ ] **2.14.4** 实现前端 `regenerateMessage()` 调用 → Rust `regenerate_message` command
- [ ] **2.14.5** 验证停止生成后已有内容保留 + 重新生成后旧内容替换

### 2.15 多模态输入 [预估 4h]

- [ ] **2.15.1** 在 MessageInput 中实现剪贴板粘贴图片（paste 事件 → Base64）
- [ ] **2.15.2** 在 MessageInput 中实现拖拽图片（drag/drop 事件 → Base64）
- [ ] **2.15.3** 在 MessageInput 中添加 📎 按钮（文件选择对话框）
- [ ] **2.15.4** 创建 `src/components/chat/ImagePreview.tsx`（缩略图预览 + 删除按钮）
- [ ] **2.15.5** 在 send_message 时传递 images 参数
- [ ] **2.15.6** 实现文件大小检查（<=10MB）和类型校验
- [ ] **2.15.7** 验证粘贴/拖拽/选择三种方式均可正常工作
- [ ] **2.15.8** 验证视觉模型正确理解图片内容

### i18n 补充 [预估 1h]

- [ ] **i18n-1** 在 `en/chat.json` 中添加对话相关翻译键
- [ ] **i18n-2** 在 `zh-CN/chat.json` 中添加对话相关翻译键
- [ ] **i18n-3** 所有新增 UI 文字使用 `useTranslation()` 的 `t()` 函数

### 收尾验证 [预估 2h]

- [ ] **F-1** 工作目录冒烟测试：新建会话 → 选择工作目录 → WorkspaceBar 显示正确 → 切换目录
- [ ] **F-2** 完整对话冒烟测试：创建会话 → 发送消息 → 流式回复 → 切换会话 → 重启验证持久化
- [ ] **F-3** 三 Provider 测试：分别使用 OpenAI / Anthropic / Gemini 完整对话
- [ ] **F-4** 思维链测试：使用 Claude 模型验证 thinking blocks 流式显示
- [ ] **F-5** 控制流测试：停止生成 → 重新生成 → 多轮对话连续性
- [ ] **F-6** 多模态测试：粘贴截图 → 发送给视觉模型 → 验证理解正确
- [ ] **F-7** 错误处理测试：使用无效 API Key 发送消息 → 友好错误提示
- [ ] **F-8** 性能验证：长回复（1000+ token）流式输出不卡顿
- [ ] **F-9** 对照第 18 节验证清单，逐项确认全部通过

---

## 附录 A：新增依赖汇总

### 前端 (npm)

```bash
npm install react-markdown remark-gfm rehype-raw shiki
```

### 后端 (Cargo.toml 新增，追加到现有 64 行依赖列表末尾)

```toml
# --- LLM framework (Phase 2) ---
rig-core = { version = "0.36", features = ["derive"] }

# --- Stream processing ---
futures = "0.3"
```

### shadcn/ui 组件

```bash
npx shadcn@latest add collapsible context-menu
```

---

## 附录 B：文件创建清单（精确对齐现有代码库）

> **标注说明：** (新增) = 文件不存在需创建; (修改) = 文件已存在需修改; (重写) = 文件存在但内容需完全替换

```
src-tauri/src/
├── lib.rs                          (修改：添加 pub mod services; + AppState 新增 stream_registry + 注册新 commands)
├── services/                       (新增目录)
│   ├── mod.rs                      (新增：pub mod llm;)
│   └── llm/
│       ├── mod.rs                  (新增：模块导出 + re-exports)
│       ├── traits.rs               (新增：LlmProvider trait 核心抽象 🆕)
│       ├── factory.rs              (新增：ProviderFactory 工厂方法 🆕)
│       ├── registry.rs             (新增：ModelRegistry 内置+自定义模型注册表 🆕)
│       ├── streaming.rs            (新增：StreamSession 流式调用)
│       ├── config.rs               (新增：LlmConfig 运行时参数)
│       ├── backend.rs              (新增：ChatBackend trait + RigBackend)
│       └── providers/              (新增目录：各 Provider 独立文件 🆕)
│           ├── mod.rs              (新增：pub mod 声明)
│           ├── openai_provider.rs  (新增：OpenAiProvider)
│           ├── anthropic_provider.rs (新增：AnthropicProvider)
│           ├── gemini_provider.rs  (新增：GeminiProvider)
│           └── openai_compat.rs    (新增：OpenAiCompatProvider 通用兼容层)
├── commands/
│   ├── mod.rs                      (修改：添加 pub mod chat/session/workspace/models;)
│   ├── chat.rs                     (新增)
│   ├── session.rs                  (新增)
│   ├── workspace.rs                (新增：目录浏览/验证/最近记录)
│   └── models.rs                   (新增：list_available_models/add_custom_model/delete_custom_model 🆕)
├── db/
│   ├── mod.rs                      (可能修改：导出新子模块)
│   ├── messages.rs                 (新增)
│   ├── sessions.rs                 (新增)
│   ├── migrations.rs               (修改：migrate_v2 含 custom_models + recent_directories + api_compat 字段)
│   └── models.rs                   (修改：新增 CustomModel/ModelInfo/DirectoryInfo/RecentDirectory 等)

src/
├── components/chat/                (新增目录)
│   ├── ChatView.tsx                (新增)
│   ├── MessageList.tsx             (新增)
│   ├── MessageItem.tsx             (新增)
│   ├── MessageInput.tsx            (新增)
│   ├── ModelSelector.tsx           (新增：分组模型选择 Popover 🆕)
│   ├── SessionPanel.tsx            (新增)
│   ├── SessionItem.tsx             (新增)
│   ├── CodeBlock.tsx               (新增)
│   ├── ThinkingBlock.tsx           (新增)
│   ├── TokenBadge.tsx              (新增)
│   ├── StreamingIndicator.tsx      (新增)
│   ├── ChatEmptyState.tsx          (新增)
│   ├── ImagePreview.tsx            (新增)
│   ├── WorkspaceSelector.tsx       (新增：工作目录选择器弹窗)
│   └── WorkspaceBar.tsx            (新增：工作目录状态指示栏)
├── hooks/
│   ├── use-ipc.ts                  (已有)
│   └── use-stream-listener.ts      (新增)
├── lib/ipc/
│   ├── invoke.ts                   (已有)
│   ├── settings.ts                 (已有)
│   ├── router-configs.ts           (已有)
│   ├── chat.ts                     (新增：chatIpc)
│   ├── sessions.ts                 (新增：sessionsIpc)
│   ├── workspace.ts                (新增：workspaceIpc)
│   ├── models.ts                   (新增：modelsIpc — listAvailableModels/addCustomModel/deleteCustomModel 🆕)
│   ├── types.ts                    (修改：新增 CustomModel/ModelInfo/ProviderModels 等类型)
│   └── index.ts                    (修改：导出新模块)
├── stores/
│   ├── chat-store.ts               (修改：从 22 行骨架扩展为完整 ChatState)
│   └── index.ts                    (修改：可能需要导出新类型)
├── pages/
│   ├── ChatPage.tsx                (重写：从 EmptyState 占位替换为完整对话页面)
│   ├── settings/
│   │   └── ProviderDialog.tsx      (修改：新增 api_compat 下拉 + 自定义模型管理区域 🆕)
│   └── index.ts                    (已有导出，无需修改)
└── locales/
    ├── en/chat.json                (新增)
    └── zh-CN/chat.json             (新增)
```

**统计：** 新增 ~30 个文件，修改 ~12 个已有文件

---

> **文档结束**
>
> 本文档是 Phase 2 的详细执行指南，覆盖了从 LLM 集成到前端对话 UI 的完整方案。
> 所有 TODO 共 **~110 项**（Rust 后端 ~43 项 + React 前端 ~50 项 + i18n 3 项 + 收尾验证 8 项），
> 按关键路径顺序完成后即可交付 M1 里程碑：核心可用的 AI 对话客户端。
>
> **代码库对齐说明：**
> 本文档基于 Phase 1 完成后的实际代码库状态编写（11 个 Rust 文件、62 个 TS/TSX 文件、15 个已注册 Tauri Command、Schema v1），
> 所有路径、模块引用、结构体字段均已与现有代码精确对齐。修改指令明确标注了"新增"与"修改"。
>
> 下一阶段为 Phase 3：MCP 协议集成与高级会话管理。
