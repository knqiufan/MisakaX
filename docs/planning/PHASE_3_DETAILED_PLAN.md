# Phase 3：Sidecar 预热基础设施 + MCP + 会话高级管理 — 详细实施方案

> **所属项目：** MisakaX
> **阶段：** Phase 3（第 8-10 周）
> **总预估：** ~64 小时（含 Vibe Coding 加速）
> **前置条件：** Phase 2 已完成（完整对话基础设施、流式渲染管线、工作目录系统、会话/消息持久化）
> **前置文档：** [PHASE_2_DETAILED_PLAN.md](./PHASE_2_DETAILED_PLAN.md)、[MISAKAX_IMPLEMENTATION_PLAN.md](./MISAKAX_IMPLEMENTATION_PLAN%20-%20Opus4.6.md)
> **里程碑：** M2: 基础设施就绪 — Sidecar 已预热、MCP 已就绪、通信协议已定义，一切准备就绪等待 Phase 4 DeepAgents 接管对话。
>
> **代码库审计状态（2026-07-06）：** Phase 3 **约 85% 完成，尚未交付 M2**。**未完成项的执行清单见 [`PHASE_3_REMAINING_TODO.md`](./PHASE_3_REMAINING_TODO.md)**（按 Epic A–F 拆分，含验收勾选表）。完成后再进入 Phase 4。
>
> **⚠️ Phase 3 定位说明：**
> Phase 3 是**"基础设施铺路"**阶段，有三条并行主线：
> 1. **Sidecar 预热**：Python Sidecar 已启动并健康运行，但**尚未参与对话**（Rig 仍处理对话）。这确保 Phase 4 迁移时平滑切换。
> 2. **MCP 完整支持**：通过 rmcp（Rust 原生 MCP SDK）集成 MCP Client，支持 stdio/HTTP/SSE transport，完成工具调用全链路。
> 3. **会话高级管理**：分组/归档/置顶、FTS5 全文搜索、导入/导出，补全用户日常使用中的会话管理需求。
>
> **编写时基线（Phase 2 结束时）：** 见各 Sprint 章节「当前状态」小节。  
> **审计时基线（2026-07-06）：** Rust 47 个 `.rs`、~50+ Tauri Commands、DB Schema **v6**、前端 ~94 个 TSX；Sidecar/MCP/会话高级 API 与 UI 已大量落地，详见 §1.3。

---

## 目录

1. [阶段目标与验收标准](#1-阶段目标与验收标准)
   - [1.3 剩余工作索引](#13-剩余工作索引)
2. [任务依赖关系与执行顺序](#2-任务依赖关系与执行顺序)
3. [Sprint 1：Sidecar 预热增强 (3.1–3.4)](#3-sprint-1sidecar-预热增强)
4. [Sprint 2：MCP 核心集成 (3.5–3.8)](#4-sprint-2mcp-核心集成)
5. [Sprint 3：MCP 前端 + Tool Call UI (3.9–3.11)](#5-sprint-3mcp-前端--tool-call-ui)
6. [Sprint 4：会话高级管理 (3.12–3.14)](#6-sprint-4会话高级管理)
7. [Sprint 5：Nuitka 打包验证 (3.15)](#7-sprint-5nuitka-打包验证)
8. [数据库 Schema v3 迁移](#8-数据库-schema-v3-迁移)
9. [新增依赖清单](#9-新增依赖清单)
10. [文件创建/修改清单](#10-文件创建修改清单)
11. [Phase 3 完整验证清单](#11-phase-3-完整验证清单)
12. [详细 TODO 列表（历史记录）](#12-详细-todo-列表历史记录)
13. [**Phase 3 剩余 TODO（执行入口）**](./PHASE_3_REMAINING_TODO.md)

---

## 1. 阶段目标与验收标准

### 1.1 核心目标

构建 Phase 4 DeepAgents 接管的**全部前置基础设施**：Sidecar 预热可靠运行 + MCP 工具调用全链路打通 + 会话管理补全。

**Phase 3 的三条并行主线：**

| 主线 | 目标 | Phase 4 衔接点 |
|------|------|--------------|
| **Sidecar 预热** | 应用启动时异步预热 Sidecar，健康检查循环，前端状态指示 | Phase 4 仅需在 Sidecar 上实现 DeepAgents agent.py |
| **MCP 完整支持** | rmcp Client 集成，stdio/HTTP/SSE transport，工具发现/调用/结果展示 | Phase 4 DeepAgents 通过 mcp_bridge_tool 复用 MCP 连接 |
| **会话高级管理** | 分组/归档/置顶、FTS5 搜索、导入/导出 | 所有会话基础设施 Phase 4 直接复用 |

### 1.2 验收标准

**图例：** ✅ 已通过代码审计 · 🟡 部分完成 · ❌ 未完成 · ⚪ 端到端待人工复验

| # | 验收项 | 验收方式 | 状态 | 说明 |
|---|--------|---------|------|------|
| AC-1 | 应用启动时 Sidecar 自动后台预热，3 秒内完成健康检查 | 观察启动日志 + StatusBadge 变绿 | 🟡 | `SidecarManager::preheat` + 异步健康轮询已实现；最坏等待 ~10s（20×500ms），需实机确认 Badge 变绿时机 |
| AC-2 | Sidecar 异常退出后 ≤5s 自动重启（最多 3 次） | kill 进程后观察恢复 | ❌ | 仅有**启动阶段**最多 3 次重试；**就绪后无子进程 watchdog**，kill 后不会自动重启 |
| AC-3 | 前端 StatusBadge 实时反映 Sidecar 状态 | UI 观察 | ✅ | `SidecarStatusBadge` + `sidecar:status` 事件 + 手动重启 |
| AC-4 | Rust → Sidecar HTTP 协议 + 占位 `/agent/chat`、`/agent/stream` | curl 测试 | ✅ | `SidecarClient` + `agent/routers/agent.py` 501 占位 |
| AC-5 | rmcp stdio 连接 MCP Server | 工具列表可见 | ⚪ | `McpManager::connect_stdio` 已实现；需配置 filesystem 等 Server 人工验证 |
| AC-6 | rmcp HTTP/SSE 连接远程 MCP Server | 工具列表可见 | ⚪ | Http/Sse 均走 `StreamableHttpClientTransport`；需远程 Server 人工验证 |
| AC-7 | MCP 配置（`~/.misakax/mcp.json` + Settings） | 配置 → 连接成功 | ✅ | `McpConfigLoader` + `McpSettings.tsx` + DB `mcp_servers` |
| AC-8 | MCP Server 生命周期（启停/重启/健康检查） | Settings 页面操作 | ✅ | commands + `start_mcp_health_loop`（60s 重连，最多 3 次） |
| AC-9 | Tool Call UI（工具名 + 参数 + 结果 + 折叠） | 对话中触发工具调用 | 🟡 | `ToolCallBlock` / DB `tool_calls` / 流式 listener 已就绪；**Rust 对话流未 emit `stream:tool_call`**，Rig 未闭环执行 MCP 工具 |
| AC-10 | Tool Call 权限审批（approve/deny/always allow） | 首次工具调用弹出审批 | 🟡 | `mcp_call_tool` + `ToolApprovalDialog` 完整；**需经对话或 Settings 手动触发** `mcp_call_tool` 验证 |
| AC-11 | MCP 管理页面 | Settings → MCP | ✅ | `McpSettings.tsx`（~585 行） |
| AC-12 | 会话分组/归档/置顶 | 右键菜单 | ✅ | `SessionPanel` + `SessionItem` + session commands |
| AC-13 | FTS5 全文搜索消息 | 搜索框 → 高亮 | ✅ | `search_messages` + `MessageSearchResults` |
| AC-14 | 会话 JSON 导入/导出 | 导出 → 导入 → 恢复 | 🟡 | 导出：`SessionItem` 右键；导入：**仅** `AboutSettings`，原计划 SessionPanel 入口未做 |
| AC-15 | Nuitka 打包 Sidecar 可执行文件 | 独立 exe + health check | ❌ | `build_nuitka.py` + `run.py` 已有；**未验收**（无 `agent/dist/` 产物；`SidecarManager` 仍 spawn `python -m uvicorn`） |

**M2 里程碑：** 15 项中 ✅ 6 · 🟡 5 · ❌ 2 · ⚪ 2 — **不可标记为已交付**。

### 1.3 剩余工作索引

> **执行入口：** 所有未完成项的详细步骤、文件路径、验收标准见 **[`PHASE_3_REMAINING_TODO.md`](./PHASE_3_REMAINING_TODO.md)**。

| 优先级 | Epic | 核心任务 | 预估 |
|--------|------|----------|------|
| **P0** | A | Sidecar 运行时 watchdog（AC-2） | 3–4h |
| **P0** | B | Rig 对话 MCP 工具闭环 + 流式 Event + DB 持久化（AC-9/10） | 8–10h |
| **P1** | C | Nuitka 打包验收（AC-15） | 2–3h |
| **P2** | D | 导入后会话刷新（AC-14） | 0.5h |
| **P1** | E | V1–V30 全量验证 + TODO-FINAL | 4–5h |
| **P2** | F | Sidecar / Tool loop 自动化测试 | 4–5h |

**Sprint 完成度速查（2026-07-06）：**

| Sprint | 范围 | 状态 | 缺口摘要 |
|--------|------|------|----------|
| Sprint 1 | 3.1–3.4 Sidecar 预热 + 协议 | 🟡 ~90% | 缺运行时 watchdog（3.2） |
| Sprint 2 | 3.5–3.8 MCP 核心 | ✅ ~95% | 代码就绪；远程 transport 待 E2E |
| Sprint 3 | 3.9–3.11 MCP 前端 + Tool UI | 🟡 ~80% | UI/审批就绪；对话内工具闭环与流式 Event 未接通 |
| Sprint 4 | 3.12–3.14 会话高级管理 | 🟡 ~95% | 导入 UI 入口偏离计划 |
| Sprint 5 | 3.15 Nuitka | ❌ ~40% | 脚本有，验收与集成未完成 |
| 收尾 | Schema + i18n + FINAL | ✅ | Schema 已到 v6（超出 v3–v5 计划） |

---

## 2. 任务依赖关系与执行顺序

### 2.1 依赖图

```
Phase 2 产出（Rig 对话 / 会话管理 / 工作目录 / 流式渲染）
    │
    ├── Sprint 1: Sidecar 预热增强 ─────────────────────────────────────
    │   ├── 3.1 Python Sidecar 端点扩展 ──→ 3.2 Sidecar 预热管理器 ──→ 3.3 状态指示器
    │   │                                       │
    │   │                                       └──→ 3.4 通信协议定义
    │   │
    │   │   （Sprint 1 完成后 Sprint 2 可立即启动）
    │   │
    ├── Sprint 2: MCP 核心集成 ──────────────────────────────────────────
    │   ├── 3.5 rmcp stdio transport ──→ 3.8 Server 生命周期管理
    │   │           │                         │
    │   │           └──→ 3.6 HTTP/SSE ────────┘
    │   │                                     │
    │   └── 3.7 MCP 配置加载 ────────────────┘
    │
    │   （Sprint 2 核心完成后 Sprint 3 可启动）
    │
    ├── Sprint 3: MCP 前端 + Tool Call UI ──────────────────────────────
    │   ├── 3.9 Tool Call UI 展示 ──→ 3.10 权限审批流程
    │   └── 3.11 MCP 管理页面（与 3.9 并行）
    │
    ├── Sprint 4: 会话高级管理（与 Sprint 1-2 可部分并行）──────────────
    │   ├── 3.12 会话分组/归档/置顶
    │   ├── 3.13 FTS5 全文搜索（独立）
    │   └── 3.14 会话导入/导出（独立）
    │
    └── Sprint 5: Nuitka 打包验证（Sprint 1 完成后可启动）─────────────
        └── 3.15 Nuitka 打包脚本初版
```

### 2.2 推荐执行顺序（按周分配）

| 周 | 主要任务 | 说明 |
|----|---------|------|
| **第 8 周** | Sprint 1 全部 (3.1→3.2→3.3→3.4) + Sprint 4 部分 (3.12) | Sidecar 预热是最高优先级，会话分组可并行 |
| **第 9 周** | Sprint 2 全部 (3.5→3.6→3.7→3.8) + Sprint 4 余下 (3.13, 3.14) | MCP 核心是最复杂模块，会话搜索/导出可穿插 |
| **第 10 周** | Sprint 3 全部 (3.9→3.10→3.11) + Sprint 5 (3.15) | 前端 UI 收尾 + 打包验证 |

### 2.3 为什么这个顺序

1. **Sidecar 预热先行**：Phase 3 的核心定位是"为 Phase 4 铺路"，Sidecar 可靠运行是最关键前置条件。当前 `sidecar.rs` 已有基础版本（同步 start + 简单健康检查），需要升级为异步预热 + 自动重启 + 状态通知。

2. **MCP 集成紧随其后**：rmcp v1.6.0 是成熟的 Rust MCP SDK，stdio/HTTP/SSE transport 均已原生支持。MCP 是 Phase 4 DeepAgents 通过 `mcp_bridge_tool` 调用外部工具的通道，必须在 Phase 3 打通。

3. **会话高级管理可并行**：分组/搜索/导出是纯 Rust 后端 + 前端 UI 工作，与 Sidecar/MCP 无依赖，可利用 Sprint 间隙穿插完成。

4. **Nuitka 打包放最后**：依赖 Sidecar 端点完善（Sprint 1），且属于低风险的工程化任务。

---

## 3. Sprint 1：Sidecar 预热增强 · 🟡 ~90%

### 3.1 任务 3.1：Python Sidecar 端点扩展（4h）

#### 3.1.1 目标

在现有 FastAPI 骨架上扩展：增强 `/health` 端点（返回详细状态）、创建 `/agent/chat` 和 `/agent/stream` 占位端点（Phase 4 实现具体逻辑）、添加 `/info` 端点（版本/能力信息）。

#### 3.1.2 当前状态

`agent/` 目录已存在：
- `app/main.py` — FastAPI 入口，仅注册 `health_router`
- `app/routers/health.py` — 仅返回 `{"status": "ok", "service": "misaka-agent"}`
- `app/config.py` — pydantic-settings 基础配置
- `pyproject.toml` — 依赖已声明 `fastapi>=0.115, uvicorn>=0.34, pydantic>=2`

#### 3.1.3 实现方案

**`/health` 增强（返回结构化状态）：**

```python
# agent/app/routers/health.py
@router.get("/health")
async def health_check():
    return {
        "status": "ok",
        "service": "misaka-agent",
        "version": "0.1.0",
        "uptime_seconds": get_uptime(),
        "capabilities": ["chat", "stream"],
        "agent_ready": False,  # Phase 4 时改为动态检测
    }
```

**`/agent/chat` 和 `/agent/stream` 占位端点：**

```python
# agent/app/routers/agent.py
@router.post("/agent/chat")
async def agent_chat(request: ChatRequest):
    """Phase 4 实现：DeepAgents 同步对话"""
    raise HTTPException(
        status_code=501,
        detail="Agent chat not implemented. Will be available in Phase 4."
    )

@router.post("/agent/stream")
async def agent_stream(request: ChatRequest):
    """Phase 4 实现：DeepAgents 流式对话"""
    raise HTTPException(
        status_code=501,
        detail="Agent stream not implemented. Will be available in Phase 4."
    )
```

**`/info` 端点：**

```python
# agent/app/routers/info.py
@router.get("/info")
async def get_info():
    return {
        "name": "misaka-agent",
        "version": "0.1.0",
        "python_version": sys.version,
        "deepagents_available": False,  # Phase 4 时动态检测
        "powermem_available": False,
    }
```

#### 3.1.4 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `agent/app/routers/health.py` | 修改 | 增强返回结构 |
| `agent/app/routers/agent.py` | 新增 | 占位 `/agent/chat`、`/agent/stream` |
| `agent/app/routers/info.py` | 新增 | `/info` 端点 |
| `agent/app/models.py` | 新增 | Pydantic 请求/响应模型（ChatRequest 等） |
| `agent/app/main.py` | 修改 | 注册新 router、添加 startup 计时 |
| `agent/app/config.py` | 修改 | 新增 `debug` 配置项 |

---

### 3.2 任务 3.2：Sidecar 预热管理器（6h）

#### 3.2.1 目标

将现有的同步 `SidecarManager` 升级为**异步预热管理器**，支持：应用启动时异步后台预热（不阻塞 UI）、健康检查循环（定期轮询）、自动重启（最多 N 次）、状态事件通知（通过 Tauri Event 推送至前端）。

#### 3.2.2 当前状态

`sidecar.rs` 已有：
- `SidecarManager::start()` — 同步启动，阻塞等待 10s 健康检查
- `SidecarManager::is_healthy()` — 单次 HTTP GET `/health`
- `Drop` trait — 进程清理
- **问题**：同步阻塞、无重试机制、无状态通知、无健康检查循环

#### 3.2.3 架构设计

```
┌─────────────────────────────────────────────────────────┐
│ SidecarManager (异步)                                     │
│                                                           │
│ ┌─── preheat() ───────────────────────────────────────┐ │
│ │ 1. 后台 tokio::spawn 启动 Python 进程               │ │
│ │ 2. 循环健康检查（间隔 500ms，超时 15s）              │ │
│ │ 3. 成功 → emit SidecarStatus::Ready                 │ │
│ │ 4. 失败 → 重试（最多 3 次）→ SidecarStatus::Error   │ │
│ └──────────────────────────────────────────────────────┘ │
│                                                           │
│ ┌─── health_loop() ──────────────────────────────────┐  │
│ │ tokio::spawn 定期健康检查（间隔 30s）               │  │
│ │ 失败 → 尝试自动重启 → 通知前端状态变化              │  │
│ └──────────────────────────────────────────────────────┘ │
│                                                           │
│ ┌─── shutdown() ─────────────────────────────────────┐  │
│ │ 优雅关闭：先 SIGTERM → 等待 5s → 强制 kill         │  │
│ └──────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

#### 3.2.4 状态机

```
          preheat()              healthy?
 Stopped ──────────▶ Starting ──────────▶ Ready
                      │    ▲               │
                      │    │ retry         │ health check fail
                      │    │ (≤3次)        ▼
                      └────┘             Error ──▶ Restarting ──▶ Starting
                                                    (auto)
```

#### 3.2.5 核心 API 设计

```rust
// src-tauri/src/sidecar.rs — 重写

/// Sidecar 运行状态
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SidecarStatus {
    Stopped,
    Starting,
    Ready,
    Error { message: String },
    Restarting { attempt: u32 },
}

/// Sidecar 状态变化事件（推送至前端）
#[derive(Debug, Clone, Serialize)]
pub struct SidecarStatusEvent {
    pub status: SidecarStatus,
    pub port: u16,
    pub uptime_ms: Option<u64>,
}

pub struct SidecarManager {
    port: u16,
    agent_dir: PathBuf,
    max_retries: u32,
    status: Arc<Mutex<SidecarStatus>>,
    child: Arc<Mutex<Option<Child>>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl SidecarManager {
    pub fn new(agent_dir: PathBuf, port: u16) -> Self;

    /// 异步预热：后台启动 Sidecar，返回 JoinHandle
    pub async fn preheat(&self, app: AppHandle) -> Result<()>;

    /// 获取当前状态
    pub fn status(&self) -> SidecarStatus;

    /// 手动重启
    pub async fn restart(&self, app: AppHandle) -> Result<()>;

    /// 优雅关闭
    pub async fn shutdown(&self) -> Result<()>;
}
```

#### 3.2.6 Tauri Event 协议

| 事件名 | Payload | 触发时机 |
|--------|---------|---------|
| `sidecar:status` | `SidecarStatusEvent` | 每次状态变化 |

#### 3.2.7 AppState 变更

```rust
// lib.rs 修改
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub config: Mutex<AppConfig>,
    pub sidecar: Arc<SidecarManager>,  // 从 Mutex<Option<SidecarManager>> → Arc<SidecarManager>
    pub stream_registry: StreamRegistry,
}
```

#### 3.2.8 启动流程变更

```rust
// lib.rs run() 修改
// 旧：同步阻塞启动 Sidecar
// 新：创建 SidecarManager 实例，setup 阶段异步预热

.setup(|app| {
    let sidecar = app.state::<AppState>().sidecar.clone();
    let handle = app.handle().clone();
    tokio::spawn(async move {
        if let Err(e) = sidecar.preheat(handle).await {
            tracing::error!("Sidecar preheat failed: {e}");
        }
    });
    Ok(())
})
```

#### 3.2.9 新增 Tauri Command

```rust
// src-tauri/src/commands/sidecar.rs (新增)

#[tauri::command]
pub fn get_sidecar_status(state: State<'_, AppState>) -> SidecarStatus;

#[tauri::command]
pub async fn restart_sidecar(app: AppHandle, state: State<'_, AppState>) -> Result<(), String>;
```

---

### 3.3 任务 3.3：Sidecar 状态指示器（2h）

#### 3.3.1 目标

前端 StatusBadge 组件实时显示 Sidecar 状态，嵌入到 AppShell 底部状态栏。

#### 3.3.2 实现方案

**StatusBadge 组件：**

```tsx
// src/components/layout/SidecarStatusBadge.tsx

interface SidecarStatusEvent {
  status: 'stopped' | 'starting' | 'ready' | 'error' | 'restarting';
  port: number;
  uptime_ms?: number;
}

// 状态 → 颜色 + 图标 映射
const STATUS_MAP = {
  stopped:    { color: 'text-muted-foreground', icon: CircleOff,  label: '已停止' },
  starting:   { color: 'text-amber-500',        icon: Loader2,    label: '启动中...' },
  ready:      { color: 'text-emerald-500',       icon: CircleCheck, label: '就绪' },
  error:      { color: 'text-destructive',       icon: CircleAlert, label: '异常' },
  restarting: { color: 'text-amber-500',         icon: RefreshCw,  label: '重启中...' },
};
```

**监听 Tauri Event：**

```tsx
useEffect(() => {
  const unlisten = listen<SidecarStatusEvent>('sidecar:status', (event) => {
    setStatus(event.payload);
  });
  // 初始化时获取一次当前状态
  invoke('get_sidecar_status').then(setStatus);
  return () => { unlisten.then(fn => fn()); };
}, []);
```

**嵌入位置：**

- 嵌入 `SidebarFooter.tsx` 底部，紧挨版本号
- 点击可展开 Tooltip 显示详细信息（端口、运行时长）
- Error 状态下点击提供「重启」操作按钮

---

### 3.4 任务 3.4：Rust ↔ Sidecar 通信协议定义（4h）

#### 3.4.1 目标

定义 Rust Core ↔ Python Sidecar 之间的 HTTP/WebSocket 通信协议，供 Phase 4 使用。Phase 3 实现协议定义 + 基础 HTTP 客户端 + 占位端点联通测试。

#### 3.4.2 协议设计

**请求/响应协议（HTTP JSON）：**

```
POST /agent/chat
Content-Type: application/json

Request:
{
  "session_id": "uuid",
  "message": "用户消息",
  "model": "claude-sonnet-4-20250514",
  "working_dir": "/path/to/workspace",
  "history": [
    {"role": "user", "content": "..."},
    {"role": "assistant", "content": "..."}
  ],
  "config": {
    "temperature": 0.7,
    "max_tokens": 4096,
    "tools_enabled": true
  }
}

Response:
{
  "message_id": "uuid",
  "content": "助手回复",
  "thinking": "思维链内容（可选）",
  "tool_calls": [...],
  "usage": { "input_tokens": 100, "output_tokens": 200 }
}
```

**流式协议（SSE）：**

```
POST /agent/stream
Content-Type: application/json
Accept: text/event-stream

Request: 同 /agent/chat

Response (SSE):
event: token
data: {"delta": "Hello"}

event: thinking
data: {"delta": "Let me think..."}

event: tool_call
data: {"id": "tc_1", "name": "read_file", "arguments": {"path": "/src/main.rs"}}

event: tool_result
data: {"id": "tc_1", "content": "file contents..."}

event: usage
data: {"input_tokens": 100, "output_tokens": 200}

event: done
data: {"message_id": "uuid", "finish_reason": "stop"}

event: error
data: {"code": "rate_limit", "message": "Rate limited"}
```

#### 3.4.3 Rust 侧实现

```rust
// src-tauri/src/services/sidecar_client.rs (新增)

pub struct SidecarClient {
    base_url: String,
    client: reqwest::Client,
}

impl SidecarClient {
    pub fn new(port: u16) -> Self;

    /// 健康检查
    pub async fn health(&self) -> Result<HealthResponse>;

    /// 同步对话（Phase 4 使用）
    pub async fn chat(&self, request: AgentChatRequest) -> Result<AgentChatResponse>;

    /// 流式对话（Phase 4 使用），返回 SSE 事件流
    pub async fn stream(&self, request: AgentChatRequest) -> Result<impl Stream<Item = AgentStreamEvent>>;

    /// 获取 Sidecar 信息
    pub async fn info(&self) -> Result<InfoResponse>;
}
```

#### 3.4.4 Python 侧 Pydantic 模型

```python
# agent/app/models.py (新增)

class ChatRequest(BaseModel):
    session_id: str
    message: str
    model: str = "claude-sonnet-4-20250514"
    working_dir: str | None = None
    history: list[ChatMessage] = []
    config: ChatConfig = ChatConfig()

class ChatMessage(BaseModel):
    role: Literal["user", "assistant", "system"]
    content: str
    thinking: str | None = None
    tool_calls: list[ToolCall] | None = None

class ChatConfig(BaseModel):
    temperature: float = 0.7
    max_tokens: int = 4096
    tools_enabled: bool = True

class ChatResponse(BaseModel):
    message_id: str
    content: str
    thinking: str | None = None
    tool_calls: list[ToolCall] = []
    usage: TokenUsage | None = None

class ToolCall(BaseModel):
    id: str
    name: str
    arguments: dict
    result: str | None = None
    status: Literal["pending", "approved", "denied", "complete"] = "pending"
```

---

## 4. Sprint 2：MCP 核心集成 · ✅ ~95%

### 4.1 任务 3.5：rmcp Client 集成 — stdio transport（8h）

#### 4.1.1 目标

通过 rmcp v1.6.0 的 `client` feature 集成 MCP Client，实现 stdio transport 连接本地 MCP Server（通过子进程管理）。

#### 4.1.2 技术选型

| 方案 | 说明 | 选择 |
|------|------|------|
| **rmcp (Rust 原生)** | 官方 MCP Rust SDK v1.6.0，原生 async，stdio/HTTP/SSE 全支持 | ✅ 选用 |
| TypeScript MCP SDK | 需要额外 Node.js 进程 | 降级备选 |

rmcp v1.6.0 关键 feature flags：
- `client` — 启用 MCP Client 端
- `transport-child-process` — stdio transport（TokioChildProcess）
- `transport-streamable-http` — HTTP/SSE transport
- `native-tls` — TLS 后端

#### 4.1.3 架构设计

```
┌──────────────────────────────────────────────────┐
│ McpManager                                         │
│                                                    │
│ ┌── McpServerHandle ──────────────────────────┐  │
│ │ server_id: String                            │  │
│ │ config: McpServerConfig                      │  │
│ │ client: Option<rmcp::service::RunningService> │  │
│ │ tools: Vec<Tool>                             │  │
│ │ status: McpServerStatus                      │  │
│ └──────────────────────────────────────────────┘  │
│                                                    │
│ 方法：                                              │
│ - connect(config) → 启动 MCP Server 连接           │
│ - disconnect(server_id)                            │
│ - list_tools(server_id) → Vec<Tool>                │
│ - call_tool(server_id, tool_name, args) → Result   │
│ - list_all_tools() → Vec<(server_id, Tool)>        │
│ - reconnect(server_id)                             │
└──────────────────────────────────────────────────┘
```

#### 4.1.4 核心代码设计

```rust
// src-tauri/src/services/mcp/mod.rs (新增)
pub mod config;
pub mod manager;
pub mod types;

// src-tauri/src/services/mcp/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub transport: McpTransport,
    pub auto_connect: bool,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpTransport {
    #[serde(rename = "stdio")]
    Stdio {
        command: String,
        args: Vec<String>,
    },
    #[serde(rename = "http")]
    Http {
        url: String,
        headers: Option<HashMap<String, String>>,
    },
    #[serde(rename = "sse")]
    Sse {
        url: String,
        headers: Option<HashMap<String, String>>,
    },
}

// src-tauri/src/services/mcp/manager.rs
pub struct McpManager {
    servers: DashMap<String, McpServerHandle>,
}

impl McpManager {
    pub fn new() -> Self;

    pub async fn connect(&self, config: McpServerConfig) -> Result<()> {
        match &config.transport {
            McpTransport::Stdio { command, args } => {
                let service = rmcp::transport::TokioChildProcess::new(
                    tokio::process::Command::new(command).args(args)
                )?;
                let client = rmcp::ServiceExt::serve(service).await?;
                // 获取工具列表
                let tools = client.list_tools(None).await?;
                // 存储连接
                self.servers.insert(config.id.clone(), McpServerHandle {
                    config,
                    client: Some(client),
                    tools: tools.tools,
                    status: McpServerStatus::Connected,
                });
            }
            // HTTP/SSE 在 3.6 中实现
            _ => unimplemented!()
        }
        Ok(())
    }

    pub async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value>;

    pub fn list_all_tools(&self) -> Vec<McpToolInfo>;
}
```

#### 4.1.5 AppState 扩展

```rust
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub config: Mutex<AppConfig>,
    pub sidecar: Arc<SidecarManager>,
    pub stream_registry: StreamRegistry,
    pub mcp_manager: Arc<McpManager>,  // 新增
}
```

---

### 4.2 任务 3.6：rmcp HTTP/SSE transport（4h）

#### 4.2.1 目标

扩展 McpManager，支持通过 HTTP/SSE transport 连接远程 MCP Server。

#### 4.2.2 实现方案

rmcp v1.6.0 提供 `StreamableHttpClientTransport`，支持可配置的 TLS 后端。

```rust
// manager.rs 中 connect() 扩展
McpTransport::Http { url, headers } => {
    let transport = rmcp::transport::StreamableHttpClientTransport::builder()
        .uri(url)
        .build()?;
    let client = transport.serve().await?;
    // 后续逻辑同 stdio
}
```

#### 4.2.3 关键细节

- HTTP transport 支持长连接 + 自动重连
- SSE transport 用于接收服务端推送事件
- 需要处理认证 headers（Bearer token 等）
- 超时配置（连接 10s / 请求 30s）

---

### 4.3 任务 3.7：MCP 配置加载（4h）

#### 4.3.1 目标

实现 MCP Server 配置的多来源加载：`~/.misakax/mcp.json` 文件 + Settings 页面 UI 配置。

#### 4.3.2 配置文件格式

```json
// ~/.misakax/mcp.json
{
  "mcpServers": {
    "filesystem": {
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed/dir"],
      "auto_connect": true
    },
    "github": {
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxx"
      },
      "auto_connect": false
    },
    "remote-tools": {
      "transport": "http",
      "url": "https://mcp.example.com/sse",
      "headers": {
        "Authorization": "Bearer xxx"
      },
      "auto_connect": true
    }
  }
}
```

#### 4.3.3 实现方案

```rust
// src-tauri/src/services/mcp/config.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct McpConfigFile {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerEntry>,
}

/// 加载 MCP 配置：文件 + 数据库合并
pub fn load_mcp_configs(config_dir: &Path, db: &Connection) -> Result<Vec<McpServerConfig>> {
    let mut configs = Vec::new();

    // 1. 从 mcp.json 加载
    let file_path = config_dir.join("mcp.json");
    if file_path.exists() {
        let content = std::fs::read_to_string(&file_path)?;
        let file_config: McpConfigFile = serde_json::from_str(&content)?;
        for (name, entry) in file_config.mcp_servers {
            configs.push(entry.into_server_config(name));
        }
    }

    // 2. 从数据库加载（Settings 页面配置的）
    // Schema v3 新增 mcp_servers 表
    let db_configs = McpServerRepo::list_all(db)?;
    configs.extend(db_configs);

    Ok(configs)
}
```

#### 4.3.4 Settings 页面 MCP 配置

- 提供 "Add MCP Server" 对话框
- 支持选择 transport 类型（stdio / HTTP / SSE）
- stdio: 输入 command + args
- HTTP/SSE: 输入 URL + 可选 headers
- 配置存储到 SQLite（Schema v3 新增 `mcp_servers` 表）

---

### 4.4 任务 3.8：MCP Server 生命周期管理（6h）

#### 4.4.1 目标

完整的 MCP Server 进程管理：启动时自动连接 `auto_connect` 的 Server、健康检查与自动重连、用户手动启动/停止/重启。

#### 4.4.2 架构设计

```
应用启动
    │
    ├── 加载 MCP 配置 (mcp.json + DB)
    │
    ├── 遍历 auto_connect=true 的 Server
    │   └── McpManager::connect(config) （异步，不阻塞启动）
    │
    └── 启动健康检查循环 (每 60s)
        ├── 检测已连接 Server 的存活状态
        │   ├── stdio: 检查子进程是否存活
        │   └── HTTP/SSE: ping 请求
        │
        └── 断连 → 自动重连（最多 3 次）→ 通知前端状态变化
```

#### 4.4.3 Tauri Command

```rust
// src-tauri/src/commands/mcp.rs (新增)

#[tauri::command]
pub async fn mcp_list_servers(state: State<'_, AppState>) -> Result<Vec<McpServerInfo>, String>;

#[tauri::command]
pub async fn mcp_connect_server(state: State<'_, AppState>, server_id: String) -> Result<(), String>;

#[tauri::command]
pub async fn mcp_disconnect_server(state: State<'_, AppState>, server_id: String) -> Result<(), String>;

#[tauri::command]
pub async fn mcp_restart_server(state: State<'_, AppState>, server_id: String) -> Result<(), String>;

#[tauri::command]
pub async fn mcp_list_tools(state: State<'_, AppState>, server_id: Option<String>) -> Result<Vec<McpToolInfo>, String>;

#[tauri::command]
pub async fn mcp_call_tool(
    state: State<'_, AppState>,
    server_id: String,
    tool_name: String,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, String>;

#[tauri::command]
pub async fn mcp_add_server_config(
    state: State<'_, AppState>,
    config: McpServerConfig,
) -> Result<(), String>;

#[tauri::command]
pub async fn mcp_remove_server_config(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<(), String>;
```

#### 4.4.4 Tauri Event

| 事件名 | Payload | 触发时机 |
|--------|---------|---------|
| `mcp:server_status` | `{ server_id, status, tools_count }` | Server 状态变化 |
| `mcp:tool_call_request` | `{ call_id, server_id, tool_name, arguments }` | 需要用户审批的工具调用 |

---

## 5. Sprint 3：MCP 前端 + Tool Call UI · 🟡 ~80%

### 5.1 任务 3.9：Tool Call UI 展示（5h）

#### 5.1.1 目标

在消息流中展示 Tool Call 信息：工具名 + 参数 + 结果，支持折叠/展开。兼容 Rig 和 DeepAgents 两种模式。

#### 5.1.2 组件设计

```
┌───────────────────────────────────────────┐
│ ToolCallBlock                              │
│ ┌─────────────────────────────────────────┐│
│ │ 🔧 read_file                    ▼ 折叠 ││
│ │ 📂 filesystem-server                    ││
│ ├─────────────────────────────────────────┤│
│ │ 参数:                                   ││
│ │ ┌─────────────────────────────────────┐ ││
│ │ │ { "path": "/src/main.rs" }         │ ││
│ │ └─────────────────────────────────────┘ ││
│ │ 结果:                                   ││
│ │ ┌─────────────────────────────────────┐ ││
│ │ │ (代码内容，语法高亮)               │ ││
│ │ └─────────────────────────────────────┘ ││
│ │ ⏱️ 120ms                ✅ 执行成功     ││
│ └─────────────────────────────────────────┘│
└───────────────────────────────────────────┘
```

#### 5.1.3 数据模型

```typescript
// src/lib/ipc/types.ts 扩展

interface ToolCall {
  id: string;
  server_id: string;
  tool_name: string;
  arguments: Record<string, unknown>;
  result?: string;
  status: 'pending' | 'running' | 'approved' | 'denied' | 'complete' | 'error';
  duration_ms?: number;
  error?: string;
}
```

#### 5.1.4 与消息系统集成

- `Message` 模型新增 `tool_calls: ToolCall[]` 字段
- `MessageItem` 组件内嵌 `ToolCallBlock` 组件
- 流式阶段通过 Tauri Event `stream:tool_call` / `stream:tool_result` 实时更新
- 消息持久化时 `tool_calls` 序列化为 JSON 存入 `messages.tool_calls` 字段

---

### 5.2 任务 3.10：Tool Call 权限审批流程（4h）

#### 5.2.1 目标

实现 Human-in-the-loop 前端：首次调用某工具时弹出审批对话框（approve / deny / always allow），权限记录持久化。

#### 5.2.2 权限模型

```
┌── ToolPermission ─────────────────────────────────┐
│ server_id: string    — MCP Server 标识             │
│ tool_name: string    — 工具名                      │
│ policy: 'ask' | 'allow' | 'deny'                  │
│ updated_at: datetime                               │
└────────────────────────────────────────────────────┘
```

**审批流程：**

```
工具调用请求
    │
    ├── 查询 ToolPermission
    │   ├── policy = 'allow' → 直接执行
    │   ├── policy = 'deny'  → 拒绝，返回错误
    │   └── policy = 'ask' 或 不存在 → 弹出审批对话框
    │
    └── 用户操作：
        ├── ✅ 允许一次 → 执行，不记录 policy
        ├── ✅ 始终允许 → 执行，记录 policy='allow'
        └── ❌ 拒绝     → 不执行，记录 policy='deny'
```

#### 5.2.3 前端组件

```tsx
// src/components/chat/ToolApprovalDialog.tsx

// 弹出式对话框，显示：
// - 工具名 + 所属 MCP Server
// - 调用参数（JSON 格式化展示）
// - 三个按钮：允许一次 / 始终允许 / 拒绝
// - 倒计时自动拒绝（可配置，默认 60s）
```

#### 5.2.4 Rust 侧存储

Schema v3 新增 `tool_permissions` 表：

```sql
CREATE TABLE IF NOT EXISTS tool_permissions (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    policy TEXT NOT NULL DEFAULT 'ask',
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (server_id, tool_name)
);
```

---

### 5.3 任务 3.11：MCP 管理页面（4h）

#### 5.3.1 目标

Settings 子页面，展示已配置的 MCP Server、连接状态、可用工具列表，支持增删改查。

#### 5.3.2 页面布局

```
┌──────────────────────────────────────────────────────┐
│ MCP Servers                                    + 添加 │
├──────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────┐ │
│ │ 🟢 filesystem     stdio    12 tools   [断开][⋮] │ │
│ │ ├── read_file       读取文件内容                  │ │
│ │ ├── write_file      写入文件内容                  │ │
│ │ ├── list_directory  列出目录内容                  │ │
│ │ └── ...（展开/收起）                              │ │
│ ├──────────────────────────────────────────────────┤ │
│ │ 🔴 github          stdio    断开      [连接][⋮] │ │
│ ├──────────────────────────────────────────────────┤ │
│ │ 🟢 remote-api      HTTP     8 tools   [断开][⋮] │ │
│ └──────────────────────────────────────────────────┘ │
│                                                      │
│ ── 权限管理 ────────────────────────────────────────── │
│ ┌──────────────────────────────────────────────────┐ │
│ │ filesystem / read_file     ✅ 始终允许 [重置]    │ │
│ │ filesystem / write_file    ❓ 每次询问 [重置]    │ │
│ │ filesystem / delete_file   ❌ 已拒绝   [重置]    │ │
│ └──────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

#### 5.3.3 前端 IPC

```typescript
// src/lib/ipc/mcp.ts (新增)

export const mcpIpc = {
  listServers: () => invoke<McpServerInfo[]>('mcp_list_servers'),
  connectServer: (serverId: string) => invoke('mcp_connect_server', { serverId }),
  disconnectServer: (serverId: string) => invoke('mcp_disconnect_server', { serverId }),
  restartServer: (serverId: string) => invoke('mcp_restart_server', { serverId }),
  listTools: (serverId?: string) => invoke<McpToolInfo[]>('mcp_list_tools', { serverId }),
  addServerConfig: (config: McpServerConfig) => invoke('mcp_add_server_config', { config }),
  removeServerConfig: (serverId: string) => invoke('mcp_remove_server_config', { serverId }),
};
```

---

## 6. Sprint 4：会话高级管理 · 🟡 ~95%

### 6.1 任务 3.12：会话分组/归档/置顶（3h）

#### 6.1.1 目标

会话列表支持：置顶（pinned 字段已存在于 Schema v2）、分组（group_name 字段已存在）、归档（新增 status='archived'）。

#### 6.1.2 当前状态

Schema v2 已有字段：
- `sessions.pinned INTEGER DEFAULT 0`
- `sessions.group_name TEXT`
- `sessions.status TEXT DEFAULT 'active'`

需要新增的是**前端 UI 交互**和**Rust Command**。

#### 6.1.3 实现方案

**Rust 侧（扩展 SessionRepo）：**

```rust
// 已有的 session.rs commands 扩展

#[tauri::command]
pub fn pin_session(state: State<'_, AppState>, session_id: String, pinned: bool) -> Result<(), String>;

#[tauri::command]
pub fn archive_session(state: State<'_, AppState>, session_id: String) -> Result<(), String>;

#[tauri::command]
pub fn set_session_group(state: State<'_, AppState>, session_id: String, group: Option<String>) -> Result<(), String>;

#[tauri::command]
pub fn list_session_groups(state: State<'_, AppState>) -> Result<Vec<String>, String>;
```

**前端侧：**

- `SessionPanel` 右键菜单增加：置顶/取消置顶、归档、设置分组
- 会话列表分区渲染：📌 置顶区 → 分组区（按 group_name 折叠）→ 普通区
- 归档会话默认隐藏，提供"显示归档"开关

---

### 6.2 任务 3.13：会话搜索 — FTS5 全文搜索（3h）

#### 6.2.1 目标

基于 SQLite FTS5 实现消息内容全文搜索（Schema v1 已建 `messages_fts` 虚拟表）。

#### 6.2.2 当前状态

- `messages_fts` FTS5 表已创建（Schema v1）
- 但**消息写入时未同步写入 FTS5 索引**（需要在 MessageRepo 中补充）

#### 6.2.3 实现方案

**Step 1：FTS5 索引同步**

```rust
// db/repository/message_repo.rs — 修改 insert_user_message / update_assistant_content
// 每次写入/更新消息后，同步写入 FTS5 索引

fn sync_to_fts(conn: &Connection, msg_id: &str, content: &str, session_id: &str, role: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO messages_fts(rowid, content, session_id, role)
         SELECT rowid, ?1, ?2, ?3 FROM messages WHERE id = ?4",
        rusqlite::params![content, session_id, role, msg_id],
    )?;
    Ok(())
}
```

**Step 2：搜索 Command**

```rust
// src-tauri/src/commands/session.rs — 扩展

#[tauri::command]
pub fn search_messages(
    state: State<'_, AppState>,
    query: String,
    session_id: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<MessageSearchResult>, String>;
```

```rust
// db/repository/message_repo.rs — 新增搜索方法

pub fn search_fts(
    conn: &Connection,
    query: &str,
    session_id: Option<&str>,
    limit: u32,
) -> Result<Vec<MessageSearchResult>> {
    let sql = if session_id.is_some() {
        "SELECT m.id, m.session_id, m.role, m.content, m.created_at,
                snippet(messages_fts, 0, '<mark>', '</mark>', '...', 32) as snippet
         FROM messages_fts fts
         JOIN messages m ON m.rowid = fts.rowid
         WHERE messages_fts MATCH ?1 AND fts.session_id = ?2
         ORDER BY rank
         LIMIT ?3"
    } else {
        "SELECT m.id, m.session_id, m.role, m.content, m.created_at,
                snippet(messages_fts, 0, '<mark>', '</mark>', '...', 32) as snippet
         FROM messages_fts fts
         JOIN messages m ON m.rowid = fts.rowid
         WHERE messages_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2"
    };
    // ... execute and map
}
```

**Step 3：前端搜索 UI**

- `SessionPanel` 顶部搜索框，输入时防抖 300ms 调用 `search_messages`
- 搜索结果按会话分组展示，点击跳转到对应会话和消息
- 搜索词高亮显示（利用 FTS5 snippet 返回的 `<mark>` 标签）

---

### 6.3 任务 3.14：会话导入/导出（3h）

#### 6.3.1 目标

支持将会话（含全部消息）导出为 JSON 文件，并从 JSON 文件导入。

#### 6.3.2 JSON 格式

```json
{
  "version": "1.0",
  "exported_at": "2026-05-10T15:30:00Z",
  "app": "MisakaX",
  "sessions": [
    {
      "id": "uuid",
      "title": "会话标题",
      "model": "config_id:model_id",
      "working_directory": "/path/to/workspace",
      "created_at": "2026-05-10T10:00:00Z",
      "messages": [
        {
          "id": "uuid",
          "role": "user",
          "content": "消息内容",
          "thinking_content": null,
          "attachments": null,
          "token_usage": null,
          "created_at": "2026-05-10T10:00:01Z"
        }
      ]
    }
  ]
}
```

#### 6.3.3 Rust Command

```rust
// src-tauri/src/commands/session.rs — 扩展

#[tauri::command]
pub fn export_sessions(
    state: State<'_, AppState>,
    session_ids: Vec<String>,
    file_path: String,
) -> Result<String, String>;

#[tauri::command]
pub fn import_sessions(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String>;
```

#### 6.3.4 前端交互

- 导出：会话右键菜单「导出」→ 选择保存路径（tauri-plugin-dialog）→ 调用 `export_sessions`
- 批量导出：Settings 页面「数据管理」区域提供「导出所有会话」
- 导入：Settings 页面「数据管理」区域提供「导入会话」→ 选择 JSON 文件 → 预览 → 确认导入

---

## 7. Sprint 5：Nuitka 打包验证 · ❌ ~40%

### 7.1 任务 3.15：Nuitka 打包脚本初版（4h）

#### 7.1.1 目标

验证 Python Sidecar 可通过 Nuitka 打包为独立可执行文件，确认打包体积、启动速度、功能正常。

#### 7.1.2 关键注意事项

基于调研，Nuitka + FastAPI + uvicorn 的已知问题和解决方案：

| 问题 | 解决方案 |
|------|---------|
| uvicorn 模块导入失败 | 使用 `uvicorn.run()` 直接调用 app 对象，不用字符串引用 |
| worker 模块缺失 | `--include-module=uvicorn.workers` |
| multiprocessing 兼容 | 添加 `freeze_support()` |
| 隐式导入丢失 | `--include-package=app` + `--include-module` 逐一指定 |

#### 7.1.3 打包脚本

```python
# agent/build_nuitka.py

"""Nuitka build script for MisakaX Agent Sidecar."""

import subprocess
import sys

NUITKA_ARGS = [
    sys.executable,
    "-m", "nuitka",
    "--standalone",
    "--onefile",
    "--output-filename=misaka-agent",
    # FastAPI/uvicorn 必要模块
    "--include-package=app",
    "--include-module=uvicorn.workers",
    "--include-module=uvicorn.lifespan.on",
    "--include-module=uvicorn.protocols.http.auto",
    "--include-module=uvicorn.protocols.websockets.auto",
    "--include-module=uvicorn.logging",
    "--include-package=pydantic",
    "--include-package=fastapi",
    "--include-package=starlette",
    # 入口
    "run.py",
]


def main():
    print("Building MisakaX Agent Sidecar with Nuitka...")
    result = subprocess.run(NUITKA_ARGS, cwd="agent")
    sys.exit(result.returncode)


if __name__ == "__main__":
    main()
```

```python
# agent/run.py (新增入口点)

"""Standalone entry point for Nuitka builds."""
import multiprocessing
import uvicorn

from app.main import app
from app.config import settings


def main():
    multiprocessing.freeze_support()
    uvicorn.run(
        app,
        host=settings.host,
        port=settings.port,
        log_level=settings.log_level,
    )


if __name__ == "__main__":
    main()
```

#### 7.1.4 验证步骤

1. 安装 Nuitka：`pip install nuitka`
2. 执行打包：`python build_nuitka.py`
3. 运行打包产物：`./misaka-agent.exe`
4. 健康检查：`curl http://127.0.0.1:9527/health`
5. 记录：打包体积、启动时间、内存占用

---

## 8. 数据库 Schema v3 迁移

### 8.1 迁移内容

```sql
-- migrate_v3

-- MCP Server 配置表
CREATE TABLE IF NOT EXISTS mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    transport_type TEXT NOT NULL,  -- 'stdio' | 'http' | 'sse'
    command TEXT,                   -- stdio: 可执行文件路径
    args TEXT,                     -- stdio: JSON 数组参数
    url TEXT,                      -- http/sse: 服务端 URL
    headers TEXT,                  -- http/sse: JSON 对象 headers
    env TEXT,                      -- JSON 对象环境变量
    auto_connect INTEGER DEFAULT 0,
    source TEXT DEFAULT 'user',    -- 'file' (mcp.json) | 'user' (Settings 页面)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 工具权限表
CREATE TABLE IF NOT EXISTS tool_permissions (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    policy TEXT NOT NULL DEFAULT 'ask',  -- 'ask' | 'allow' | 'deny'
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (server_id, tool_name)
);

-- messages 表新增 tool_calls 字段
ALTER TABLE messages ADD COLUMN tool_calls TEXT;

-- 更新 schema 版本
INSERT INTO _schema_version (version) VALUES (3);
```

### 8.2 实现位置

```rust
// src-tauri/src/db/migrations.rs — 新增 migrate_v3

if current_version < 3 {
    migrate_v3(conn)?;
}
```

---

## 9. 新增依赖清单

### 9.1 Rust (Cargo.toml 追加)

```toml
# --- MCP Client (Phase 3) ---
rmcp = { version = "1.6", features = ["client", "transport-child-process", "transport-streamable-http", "native-tls"] }
```

### 9.2 前端 (package.json)

Phase 3 前端无新增 npm 依赖。所有 UI 组件使用现有 shadcn/ui + Lucide React。

### 9.3 shadcn/ui 组件

```bash
npx shadcn@latest add alert-dialog progress
```

- `alert-dialog` — Tool Call 权限审批对话框
- `progress` — MCP Server 连接进度指示

### 9.4 Python (agent/pyproject.toml)

Phase 3 不新增 Python 依赖。现有 `fastapi + uvicorn + pydantic` 已满足需求。
Nuitka 为开发依赖：`pip install nuitka`

---

## 10. 文件创建/修改清单

> **标注说明：** (新增) = 文件不存在需创建; (修改) = 文件已存在需修改

```
src-tauri/src/
├── lib.rs                            (修改：AppState 新增 mcp_manager + sidecar 类型变更 + 注册新 commands + setup 异步预热)
├── sidecar.rs                        (重写：同步→异步预热管理器，SidecarStatus 状态机)
├── services/
│   ├── mod.rs                        (修改：pub mod mcp; pub mod sidecar_client;)
│   ├── sidecar_client.rs             (新增：Rust→Sidecar HTTP 客户端)
│   └── mcp/
│       ├── mod.rs                    (新增：模块导出)
│       ├── types.rs                  (新增：McpServerConfig / McpTransport / McpToolInfo / McpServerStatus)
│       ├── config.rs                 (新增：mcp.json 加载 + DB 合并)
│       └── manager.rs               (新增：McpManager — 连接/断开/工具调用/生命周期)
├── commands/
│   ├── mod.rs                        (修改：pub mod sidecar; pub mod mcp;)
│   ├── sidecar.rs                    (新增：get_sidecar_status / restart_sidecar)
│   ├── mcp.rs                        (新增：8 个 MCP 相关 commands)
│   └── session.rs                    (修改：新增 pin/archive/group/search_messages/export/import commands)
├── db/
│   ├── migrations.rs                 (修改：新增 migrate_v3)
│   ├── models.rs                     (修改：新增 McpServer / ToolPermission / MessageSearchResult / ExportData 模型)
│   └── repository/
│       ├── mod.rs                    (修改：pub mod mcp_server_repo; pub mod tool_permission_repo;)
│       ├── mcp_server_repo.rs        (新增：MCP Server 配置 CRUD)
│       ├── tool_permission_repo.rs   (新增：工具权限 CRUD)
│       ├── message_repo.rs           (修改：FTS5 索引同步 + 搜索方法 + 导出查询)
│       └── session_repo.rs           (修改：pin/archive/group 方法 + 导出/导入)

src/
├── components/
│   ├── layout/
│   │   ├── SidebarFooter.tsx         (修改：嵌入 SidecarStatusBadge)
│   │   └── SidecarStatusBadge.tsx    (新增：Sidecar 状态指示器)
│   ├── chat/
│   │   ├── ToolCallBlock.tsx         (新增：Tool Call 展示组件)
│   │   ├── ToolApprovalDialog.tsx    (新增：权限审批对话框)
│   │   ├── MessageItem.tsx           (修改：嵌入 ToolCallBlock)
│   │   ├── SessionPanel.tsx          (修改：右键菜单增加置顶/归档/分组/导出、搜索增强)
│   │   └── MessageSearchResults.tsx  (新增：搜索结果展示)
│   └── settings/
│       └── McpSettingsPage.tsx       (新增：MCP 管理页面)
├── lib/ipc/
│   ├── mcp.ts                        (新增：MCP IPC 封装)
│   ├── sidecar.ts                    (新增：Sidecar IPC 封装)
│   ├── sessions.ts                   (修改：新增 pin/archive/group/searchMessages/export/import)
│   ├── types.ts                      (修改：新增 MCP/Sidecar/ToolCall/搜索结果 类型)
│   └── index.ts                      (修改：导出新模块)
├── stores/
│   └── chat-store.ts                 (修改：会话分组/归档/置顶 状态管理)
├── hooks/
│   └── use-sidecar-status.ts         (新增：监听 sidecar:status 事件)
└── locales/
    ├── en/
    │   ├── chat.json                 (修改：新增 tool call / 搜索 / 导入导出 翻译)
    │   └── settings.json             (修改：新增 MCP 管理页面翻译)
    └── zh-CN/
        ├── chat.json                 (修改)
        └── settings.json             (修改)

agent/
├── app/
│   ├── main.py                       (修改：注册新 router、startup 计时)
│   ├── config.py                     (修改：新增 debug 配置项)
│   ├── models.py                     (新增：Pydantic 请求/响应模型)
│   └── routers/
│       ├── health.py                 (修改：增强返回结构)
│       ├── agent.py                  (新增：占位 /agent/chat、/agent/stream)
│       └── info.py                   (新增：/info 端点)
├── run.py                            (新增：Nuitka 打包入口)
└── build_nuitka.py                   (新增：打包脚本)
```

**统计：** 新增 ~23 个文件，修改 ~20 个已有文件

---

## 11. Phase 3 完整验证清单

> **图例：** ✅ 代码已具备 · 🟡 部分 · ❌ 未具备 · ⚪ 待人工执行  
> 审计日期：2026-07-06

### 11.1 Sidecar 预热验证

| # | 验证项 | 操作 | 预期 | 状态 |
|---|--------|------|------|------|
| V1 | Sidecar 自动预热 | 启动应用 | StatusBadge 从 "启动中" → "就绪" | 🟡 |
| V2 | 异常自动重启 | 手动 kill Python 进程 | StatusBadge "重启中" → "就绪"（≤15s） | ❌ |
| V3 | 重试上限 | 反复 kill 超过 3 次 | StatusBadge "异常"，不再自动重启 | ❌ |
| V4 | 手动重启 | 点击异常状态的重启按钮 | Sidecar 重启成功 | ✅ |
| V5 | 禁用自动启动 | `auto_start_sidecar: false` | 不启动 Sidecar，Badge "已停止" | ⚪ |

### 11.2 通信协议验证

| # | 验证项 | 操作 | 预期 | 状态 |
|---|--------|------|------|------|
| V6 | /health 增强 | `curl …/health` | version/uptime/capabilities JSON | ✅ |
| V7 | /info 端点 | `curl …/info` | Python 版本、langgraph/powermem 检测 | ✅ |
| V8 | /agent/chat 占位 | `POST /agent/chat` | 501 + Phase 4 提示 | ✅ |
| V9 | /agent/stream 占位 | `POST /agent/stream` | 501 + Phase 4 提示 | ✅ |

### 11.3 MCP 集成验证

| # | 验证项 | 操作 | 预期 | 状态 |
|---|--------|------|------|------|
| V10 | stdio 连接 | 配置 filesystem MCP → 连接 | 工具列表显示 | ⚪ |
| V11 | HTTP 连接 | 配置远程 MCP → 连接 | 工具列表显示 | ⚪ |
| V12 | 对话中工具调用 | 对话触发 read_file 等 | ToolCallBlock 展示参数+结果 | ❌ |
| V13 | 配置文件加载 | 编辑 `mcp.json` → 重启 | auto_connect Server 连接 | ✅ |
| V14 | Settings 配置 | UI 添加 MCP Server | 连接并显示工具 | ✅ |
| V15 | Server 生命周期 | Settings 断开/重连 | 状态实时更新 | ✅ |

### 11.4 Tool Call 权限验证

| # | 验证项 | 操作 | 预期 | 状态 |
|---|--------|------|------|------|
| V16 | 首次审批 | 首次调用工具 | 弹出审批对话框 | 🟡 |
| V17 | 允许一次 | 点击"允许一次" | 执行，下次仍询问 | 🟡 |
| V18 | 始终允许 | 点击"始终允许" | 后续自动放行 | 🟡 |
| V19 | 拒绝 | 点击"拒绝" | 未执行 | 🟡 |
| V20 | 权限重置 | MCP 页重置权限 | 恢复 ask | ✅ |

> V16–V19：通过 Settings 或未来 `mcp_call_tool` 路径可测；**对话内触发（V12）未通则无法完整验收 AC-10**。

### 11.5 会话管理验证

| # | 验证项 | 操作 | 预期 | 状态 |
|---|--------|------|------|------|
| V21 | 置顶 | 右键 → 置顶 | 📌 区 | ✅ |
| V22 | 归档 | 右键 → 归档 | 归档列表可见 | ✅ |
| V23 | 分组 | 右键 → 设置分组 | 折叠组 | ✅ |
| V24 | 全文搜索 | 搜索关键词 | 消息高亮 | ✅ |
| V25 | 导出 | 右键 → 导出 | JSON 含会话+消息 | ✅ |
| V26 | 导入 | Settings → 关于 → 导入 | 会话恢复 | 🟡 |

### 11.6 Nuitka 打包验证

| # | 验证项 | 操作 | 预期 | 状态 |
|---|--------|------|------|------|
| V27 | 打包成功 | `python build_nuitka.py` | 生成 `misaka-agent.exe` | ❌ |
| V28 | 独立运行 | 运行 exe | 服务启动 | ❌ |
| V29 | 健康检查 | `curl …/health` | 200 OK | ❌ |
| V30 | 体积记录 | 检查 exe 大小 | 30–60MB 量级记录 | ❌ |

---

## 12. 详细 TODO 列表（历史记录）

> ⚠️ **续做 Phase 3 请使用 [`PHASE_3_REMAINING_TODO.md`](./PHASE_3_REMAINING_TODO.md)**，而非本节。  
> 下列条目为开发过程中的原始记录；部分 ✅ 标记与 2026-07-06 代码审计不符（已在上文 §1.2 / §11 校正）。

### Sprint 1：Sidecar 预热增强（第 8 周前半）

#### 3.1 Python Sidecar 端点扩展（4h）

```
TODO-3.1.1  [1.0h] [无依赖] ✅ 已完成 (2026-05-11)
    创建 agent/app/models.py
    - 定义 ChatMessage / ChatConfig / ChatRequest / ChatResponse / ToolCall / TokenUsage Pydantic 模型
    - 定义 HealthResponse / InfoResponse 响应模型
    - 所有模型使用 pydantic v2 语法（model_config 而非 class Config）

TODO-3.1.2  [0.5h] [依赖 3.1.1] ✅ 已完成 (2026-05-11)
    修改 agent/app/routers/health.py
    - 增强 /health 返回结构：添加 version / uptime_seconds / capabilities / agent_ready 字段
    - 在 app 启动时记录 startup_time（修改 main.py 的 startup 事件）

TODO-3.1.3  [1.0h] [依赖 3.1.1] ✅ 已完成 (2026-05-11)
    创建 agent/app/routers/agent.py
    - 实现占位 POST /agent/chat 端点（返回 501 HTTPException）
    - 实现占位 POST /agent/stream 端点（返回 501 HTTPException）
    - 请求参数类型使用 models.py 中定义的 ChatRequest

TODO-3.1.4  [0.5h] [无依赖] ✅ 已完成 (2026-05-11)
    创建 agent/app/routers/info.py
    - GET /info 返回 name / version / python_version / deepagents_available / powermem_available
    - deepagents_available / powermem_available 通过 try-import 动态检测

TODO-3.1.5  [0.5h] [依赖 3.1.2-3.1.4] ✅ 已完成 (2026-05-11)
    修改 agent/app/main.py
    - 注册 agent_router 和 info_router
    - startup 事件记录 app.state.startup_time（使用 lifespan 替代已弃用的 on_event）
    - 修改 config.py 新增 debug: bool = False

TODO-3.1.6  [0.5h] [依赖 3.1.5] ✅ 已完成 (2026-05-11)
    验证所有端点
    - 启动 Sidecar：cd agent && python -m uvicorn app.main:app --port 9527
    - 测试 /health、/info、/agent/chat、/agent/stream 返回值正确
```

#### 3.2 Sidecar 预热管理器（6h）

```
TODO-3.2.1  [2.0h] [依赖 3.1 完成] ✅ 已完成 (2026-05-11)
    重写 src-tauri/src/sidecar.rs
    - 定义 SidecarStatus 枚举（Stopped / Starting / Ready / Error / Restarting）
    - 定义 SidecarStatusEvent 结构体（用于 Tauri Event）
    - 重构 SidecarManager：
      - 字段：port / agent_dir / max_retries / status (Arc<Mutex>) / child (Arc<Mutex>) / shutdown_tx
      - new() 构造函数
      - async fn preheat(app: AppHandle)：后台 tokio::spawn 启动进程 + 健康检查循环 + 重试机制
      - fn status() → SidecarStatus：获取当前状态
      - async fn restart(app: AppHandle)：手动重启
      - async fn shutdown()：优雅关闭（SIGTERM → wait → kill）
    - 保留 Drop trait 用于进程清理
    - 每次状态变化通过 app.emit("sidecar:status", event) 通知前端

TODO-3.2.2  [1.5h] [依赖 3.2.1] ✅ 已完成 (2026-05-11)
    修改 src-tauri/src/lib.rs
    - AppState.sidecar 类型从 Mutex<Option<SidecarManager>> 改为 Arc<SidecarManager>
    - 构建 SidecarManager 实例（传入 agent_dir + port）
    - setup 闭包中 tokio::spawn 异步调用 sidecar.preheat(app_handle)
    - 移除旧的同步 Sidecar 启动逻辑

TODO-3.2.3  [1.0h] [依赖 3.2.1] ✅ 已完成 (2026-05-11)
    创建 src-tauri/src/commands/sidecar.rs
    - get_sidecar_status command：返回当前 SidecarStatus
    - restart_sidecar command：调用 sidecar.restart()
    - 在 commands/mod.rs 中注册新模块

TODO-3.2.4  [0.5h] [依赖 3.2.2, 3.2.3] ✅ 已完成 (2026-05-11)
    修改 src-tauri/src/lib.rs invoke_handler
    - 注册 get_sidecar_status / restart_sidecar 两个新 command

TODO-3.2.5  [1.0h] [依赖 3.2.4] 🟡 部分完成
    集成测试
    - cargo check 编译通过 ✅
    - 启动应用，观察日志确认 Sidecar 异步预热成功 ✅
    - kill Python 进程，观察自动重启 ❌（缺运行时 watchdog，见 §1.3 P0）
    - config.yaml 设置 auto_start_sidecar: false，验证不启动 ⚪
```

#### 3.3 Sidecar 状态指示器（2h）

```
TODO-3.3.1  [0.5h] [无依赖] ✅ 已完成 (2026-05-11)
    创建 src/lib/ipc/sidecar.ts
    - 定义 SidecarStatus / SidecarStatusEvent TypeScript 类型
    - 封装 getSidecarStatus() / restartSidecar() IPC 调用
    - 在 index.ts 中导出

TODO-3.3.2  [0.5h] [依赖 3.3.1] ✅ 已完成 (2026-05-11)
    创建 src/hooks/use-sidecar-status.ts
    - useSidecarStatus() hook：
      - 监听 'sidecar:status' Tauri Event
      - 初始化时调用 getSidecarStatus()
      - 返回 { status, restartSidecar }

TODO-3.3.3  [0.5h] [依赖 3.3.2] ✅ 已完成 (2026-05-11)
    创建 src/components/layout/SidecarStatusBadge.tsx
    - 使用 useSidecarStatus() hook
    - 状态→颜色+图标映射（emerald=就绪 / amber=启动中或重启中 / destructive=异常 / muted=停止）
    - Tooltip 显示详细信息（端口、运行时长）
    - Error 状态显示"重启"按钮

TODO-3.3.4  [0.5h] [依赖 3.3.3] ✅ 已完成 (2026-05-11)
    修改 src/components/layout/SidebarFooter.tsx
    - 在版本号旁嵌入 SidecarStatusBadge 组件
    - 确保布局紧凑，不干扰已有 UI
    - 添加 i18n key：sidecar.status.ready / starting / error / stopped / restarting
```

#### 3.4 Rust ↔ Sidecar 通信协议（4h）

```
TODO-3.4.1  [1.5h] [无依赖] ✅ 已完成 (2026-05-11)
    创建 src-tauri/src/services/sidecar_client.rs
    - SidecarClient 结构体（base_url + reqwest::Client）
    - new(port) 构造函数
    - async fn health() → Result<HealthResponse>
    - async fn info() → Result<InfoResponse>
    - async fn chat(request: AgentChatRequest) → Result<AgentChatResponse>（Phase 4 使用）
    - async fn stream(request: AgentChatRequest) → Result<impl Stream>（Phase 4 使用，SSE 解析）
    - 定义 AgentChatRequest / AgentChatResponse / AgentStreamEvent 类型

TODO-3.4.2  [0.5h] [依赖 3.4.1] ✅ 已完成 (2026-05-11)
    修改 src-tauri/src/services/mod.rs
    - 添加 pub mod sidecar_client;
    - 注：pub mod mcp 将在 Sprint 2 中添加

TODO-3.4.3  [1.0h] [依赖 3.4.1, 3.1 完成] ✅ 已完成 (2026-05-11)
    修改 agent/app/routers/agent.py
    - /agent/chat 占位端点改为接受完整 ChatRequest 并返回 501（而非简单 422）
    - /agent/stream 占位端点同上
    - 确保请求参数与 Rust 侧 AgentChatRequest 对齐

TODO-3.4.4  [1.0h] [依赖 3.4.1-3.4.3] ✅ 已完成 (2026-05-11)
    端到端联通测试
    - 在 sidecar.rs preheat 成功后，调用 SidecarClient::health() 验证
    - 记录日志：Sidecar health check via SidecarClient: OK
    - 测试 /agent/chat 占位端点返回 501
```

### Sprint 2：MCP 核心集成（第 8 周后半 + 第 9 周前半）

#### 3.5 rmcp stdio transport 集成（8h）

```
TODO-3.5.1  [0.5h] [无依赖] ✅ DONE
    修改 src-tauri/Cargo.toml
    - 添加 rmcp 依赖：rmcp = { version = "1.6", features = ["client", "transport-child-process", "transport-streamable-http-client-reqwest", "reqwest-native-tls"] }
    - 添加 http = "1" 依赖（修复 http::HeaderName 编译问题）

TODO-3.5.2  [1.5h] [无依赖] ✅ DONE
    创建 src-tauri/src/services/mcp/types.rs
    - McpServerConfig（id / name / transport / auto_connect / env）
    - McpTransport 枚举（Stdio / Http / Sse，每种含各自参数）
    - McpServerStatus 枚举（Disconnected / Connecting / Connected / Error）
    - McpServerInfo（config + status + tools_count + auto_connect）
    - McpToolInfo（server_id / name / description / input_schema）
    - McpServerHandle（config / client / tools / status / retry_count）

TODO-3.5.3  [3.0h] [依赖 3.5.1, 3.5.2] ✅ DONE
    创建 src-tauri/src/services/mcp/manager.rs
    - McpManager 结构体（servers: DashMap<String, McpServerHandle>）
    - new() / default() 构造函数
    - async fn connect(config: McpServerConfig)：Stdio transport
    - async fn disconnect(server_id)：关闭连接，更新状态
    - async fn call_tool(server_id, tool_name, arguments)：调用工具，返回结果
    - fn list_all_tools()：遍历所有连接的 Server，收集工具信息
    - fn server_info(server_id) → Option<McpServerInfo>
    - fn list_servers() → Vec<McpServerInfo>
    - fn find_server_for_tool() / connected_count() / retry_count() / increment_retry() / reset_retry()

TODO-3.5.4  [0.5h] [依赖 3.5.3] ✅ DONE
    创建 src-tauri/src/services/mcp/mod.rs
    - pub mod types / config / manager
    - re-export 核心类型

TODO-3.5.5  [1.0h] [依赖 3.5.4] ✅ DONE
    修改 src-tauri/src/lib.rs
    - AppState 添加 mcp_manager: Arc<McpManager>
    - 初始化 McpManager::new()
    - setup 闭包中加载 MCP 配置并自动连接 auto_connect Server（异步）

TODO-3.5.6  [1.5h] [依赖 3.5.5] ✅ DONE（单元测试覆盖）
    集成测试：stdio transport
    - 通过 mcp_types_tests / mcp_manager_tests / mcp_config_tests 覆盖
    - 实际 MCP Server 集成测试需要安装外部 MCP Server，属于端到端测试范畴
```

#### 3.6 rmcp HTTP/SSE transport（4h）

```
TODO-3.6.1  [2.0h] [依赖 3.5.3] ✅ DONE
    扩展 src-tauri/src/services/mcp/manager.rs connect()
    - McpTransport::Http 分支：使用 rmcp::transport::StreamableHttpClientTransport
    - McpTransport::Sse 分支：复用 StreamableHttpClientTransport（rmcp 统一处理）
    - 支持自定义 headers（认证 token 等）

TODO-3.6.2  [1.0h] [依赖 3.6.1] ✅ DONE
    错误处理与重连
    - 连接失败时记录错误，状态设为 Error
    - 提供 reconnect(server_id) 方法
    - 调用工具失败时区分"连接断开"和"工具执行失败"

TODO-3.6.3  [1.0h] [依赖 3.6.1] ✅ DONE（单元测试覆盖）
    集成测试：HTTP/SSE transport
    - 通过 mcp_manager_tests 中的连接逻辑测试覆盖
    - 实际远程 MCP Server 测试属于端到端测试范畴
```

#### 3.7 MCP 配置加载（4h）

```
TODO-3.7.1  [1.5h] [无依赖] ✅ DONE
    创建 src-tauri/src/services/mcp/config.rs
    - McpConfigFile / McpServerEntry 结构体（对应 mcp.json 格式）
    - load_from_file(path) → Vec<McpServerConfig>：解析 mcp.json
    - load_from_db(conn) → Vec<McpServerConfig>：查询 mcp_servers 表
    - load_all(config_dir, conn) → Vec<McpServerConfig>：合并两个来源，去重（file 优先）

TODO-3.7.2  [0.5h] [依赖 TODO-3.7.1] ✅ DONE
    创建默认 mcp.json 模板
    - 在 config::ensure_directories() 中，若 mcp.json 不存在则创建空模板
    - 模板内容：{ "mcpServers": {} }

TODO-3.7.3  [1.0h] [依赖 Schema v3] ✅ DONE
    创建 src-tauri/src/db/repository/mcp_server_repo.rs
    - McpServerRecord 结构体 + McpServerRepo CRUD（insert / find_by_id / list_all / update / delete）
    - 在 db/repository/mod.rs 中注册
    - 数据库迁移 v3：创建 mcp_servers 表

TODO-3.7.4  [1.0h] [依赖 3.7.1, 3.5.5] ✅ DONE
    集成：启动时自动加载配置并连接
    - 修改 lib.rs setup：加载 MCP configs → 遍历 auto_connect → McpManager::connect
    - 全部异步执行，不阻塞启动
```

#### 3.8 MCP Server 生命周期管理（6h）

```
TODO-3.8.1  [2.0h] [依赖 3.5.3] ✅ DONE
    McpManager 扩展：健康检查循环
    - start_mcp_health_loop(app, manager)：tokio::spawn 定期（60s）检查所有连接的 Server
    - 通过 list_all_tools() 验证连接活跃性
    - 断连检测 → 自动重连（最多 3 次）→ emit mcp:server_status 事件

TODO-3.8.2  [2.0h] [依赖 3.8.1] ✅ DONE
    创建 src-tauri/src/commands/mcp.rs
    - 实现 8 个 MCP commands：
      - mcp_list_servers / mcp_connect_server / mcp_disconnect_server / mcp_restart_server
      - mcp_list_tools / mcp_call_tool
      - mcp_add_server_config / mcp_remove_server_config
    - 在 commands/mod.rs 注册

TODO-3.8.3  [0.5h] [依赖 3.8.2] ✅ DONE
    修改 lib.rs invoke_handler
    - 注册全部 8 个 MCP commands

TODO-3.8.4  [1.0h] [依赖 3.8.3] 🟡 部分完成
    将 MCP 工具调用集成到现有 Rig 对话流
    - 创建 services/mcp_bridge.rs（McpToolBridge）✅
    - 修改 commands/chat.rs：注入 MCP 工具描述到 system prompt ✅
    - ❌ 未完成：LLM 返回工具调用 → 审批 → call_tool → 写回 message.tool_calls → emit stream:tool_call/result

TODO-3.8.5  [0.5h] [依赖 3.8.1-3.8.4] ✅ DONE（单元测试覆盖）
    端到端验证
    - 41 个 MCP 单元测试全部通过
    - 169 个全量测试全部通过（含既有测试回归验证）
    - 实际 MCP Server 端到端集成测试待配置外部 Server 后执行
```

### Sprint 3：MCP 前端 + Tool Call UI（第 9 周后半）

#### 3.9 Tool Call UI 展示（5h）🟡 部分完成

```
TODO-3.9.1  [2.0h] [无依赖] ✅ DONE
    创建 src/components/chat/ToolCallBlock.tsx
    - 接收 ToolCall 对象作为 props
    - 展示：工具名 + 所属 Server + 参数（JSON 格式化）+ 结果（支持代码高亮）
    - 折叠/展开控制（默认折叠结果区域，展开参数摘要）
    - 状态指示：pending（loading）/ running（动画）/ complete（✅）/ error（❌）
    - 执行耗时显示

TODO-3.9.2  [1.0h] [依赖 3.9.1] ✅ DONE
    修改 src/lib/ipc/types.ts
    - 新增 ToolCall / McpServerInfo / McpToolInfo / ToolPermission 类型
    - Message 类型新增 tool_calls?: ToolCall[]

TODO-3.9.3  [1.0h] [依赖 3.9.1, 3.9.2] ✅ DONE
    修改 src/components/chat/MessageItem.tsx
    - 在 assistant 消息内容下方嵌入 ToolCallBlock 列表
    - 渲染逻辑：message.tool_calls?.map(tc => <ToolCallBlock key={tc.id} toolCall={tc} />)
    - 确保与 ThinkingBlock 的布局协调（thinking 在上，tool calls 在内容之间）

TODO-3.9.4  [1.0h] [依赖 3.9.3] 🟡 部分完成
    流式 Tool Call 更新
    - use-stream-listener.ts 监听 stream:tool_call / stream:tool_result ✅
    - chat-store addToolCall / updateToolCall ✅
    - ❌ Rust 侧 streaming/chat 未 emit 上述事件（grep 无匹配）
```

#### 3.10 Tool Call 权限审批流程（4h）✅ COMPLETED

```
TODO-3.10.1 [1.0h] [依赖 Schema v3] ✅ DONE
    创建 src-tauri/src/db/repository/tool_permission_repo.rs
    - find_policy(server_id, tool_name) → Option<String>
    - upsert_policy(server_id, tool_name, policy)
    - list_all() → Vec<ToolPermission>
    - reset(server_id, tool_name)
    - 在 db/repository/mod.rs 注册

TODO-3.10.2 [1.0h] [依赖 3.10.1] ✅ DONE
    扩展 mcp.rs commands
    - mcp_call_tool 增加权限检查逻辑：
      1. 查询 ToolPermissionRepo::find_policy
      2. allow → 直接执行
      3. deny → 返回拒绝
      4. ask / 不存在 → emit mcp:tool_call_request 事件，等待前端响应
    - 新增 mcp_approve_tool_call / mcp_deny_tool_call commands
    - 新增 mcp_list_permissions / mcp_reset_permission commands

TODO-3.10.3 [1.5h] [依赖 3.9.1] ✅ DONE
    创建 src/components/chat/ToolApprovalDialog.tsx
    - AlertDialog 组件，显示：工具名 / Server / 参数 JSON
    - 三个按钮：允许一次 / 始终允许 / 拒绝
    - 60s 倒计时自动拒绝
    - 调用 mcp_approve_tool_call 或 mcp_deny_tool_call

TODO-3.10.4 [0.5h] [依赖 3.10.3] ✅ DONE
    集成到 ChatView
    - 监听 mcp:tool_call_request 事件
    - 触发时弹出 ToolApprovalDialog
    - 用户操作后继续执行或拒绝
```

#### 3.11 MCP 管理页面（4h）✅ COMPLETED

```
TODO-3.11.1 [0.5h] [无依赖] ✅ DONE
    创建 src/lib/ipc/mcp.ts
    - 封装所有 MCP IPC 调用（listServers / connectServer / disconnectServer / restartServer / listTools / addServerConfig / removeServerConfig / listPermissions / resetPermission）
    - 在 index.ts 导出

TODO-3.11.2 [2.5h] [依赖 3.11.1] ✅ DONE
    创建 src/pages/settings/McpSettings.tsx（替换占位组件）
    - Server 列表区域：每个 Server 显示名称 / transport 类型 / 连接状态 / 工具数量
    - 操作按钮：连接 / 断开 / 重启 / 删除
    - 工具列表展开（每个 Server 下可展开查看工具详情）
    - "添加 MCP Server" 对话框：选择 transport 类型 → 填写配置 → 保存
    - 权限管理区域：显示已设置的权限，支持重置

TODO-3.11.3 [0.5h] [依赖 3.11.2] ✅ DONE
    将 MCP 管理页面集成到 Settings 路由
    - Settings 页面导航已有 "MCP" tab（Phase 2 已注册占位）
    - 路由配置已接入 McpSettings 组件

TODO-3.11.4 [0.5h] [依赖 3.11.2] ✅ DONE
    i18n
    - en/settings.json 新增 MCP 相关翻译 key
    - zh-CN/settings.json 对应翻译
    - en/chat.json 新增 toolCall / toolApproval 相关翻译
    - zh-CN/chat.json 对应翻译
```

### Sprint 4：会话高级管理（与 Sprint 1-2 穿插进行）

#### 3.12 会话分组/归档/置顶（3h） ✅ DONE

```
TODO-3.12.1 ✅ [1.0h] [无依赖]
    扩展 src-tauri/src/db/repository/session_repo.rs
    - pin_session(conn, session_id, pinned: bool)
    - archive_session(conn, session_id)：设置 status='archived'
    - unarchive_session(conn, session_id)：恢复 status='active'
    - set_group(conn, session_id, group: Option<&str>)
    - list_groups(conn) → Vec<String>：SELECT DISTINCT group_name
    - list_all_for_export(conn) → Vec<Session>（导出用）

TODO-3.12.2 ✅ [0.5h] [依赖 3.12.1]
    扩展 src-tauri/src/commands/session.rs
    - 新增 pin_session / archive_session / set_session_group / list_session_groups commands
    - 注册到 lib.rs invoke_handler

TODO-3.12.3 ✅ [1.0h] [依赖 3.12.2]
    修改 src/components/chat/SessionPanel.tsx
    - 会话列表分区渲染：📌 置顶区 → 分组区（按 group_name 折叠）→ 普通区
    - 归档会话默认隐藏，底部"显示 N 个归档"开关
    - 分组折叠/展开控制

TODO-3.12.4 ✅ [0.5h] [依赖 3.12.3]
    修改 src/components/chat/SessionItem.tsx
    - 右键菜单 (DropdownMenu) 新增：置顶 / 归档(含取消归档) / 设置分组（子菜单选择或新建分组）/ 导出
    - 置顶状态显示 📌 图标
    - 归档状态显示灰色半透明样式
```

#### 3.13 FTS5 全文搜索（3h） ✅ DONE

```
TODO-3.13.1 ✅ [1.0h] [无依赖]
    修改 src-tauri/src/db/repository/message_repo.rs
    - sync_fts() 已在 Phase 2 实现（insert_user_message / update_assistant_content 后同步）
    - 新增 search_fts(conn, query, session_id, limit) → Vec<MessageSearchResult>
      - 使用 FTS5 snippet() 函数返回高亮片段（<mark>标签包裹）
      - 支持按 session_id 过滤
      - build_fts_query() 辅助：将用户输入转为安全的 FTS5 前缀匹配表达式

TODO-3.13.2 ✅ [0.5h] [依赖 3.13.1]
    修改 src-tauri/src/db/models.rs
    - 新增 MessageSearchResult { id, session_id, session_title, role, snippet, created_at }

TODO-3.13.3 ✅ [0.5h] [依赖 3.13.1, 3.13.2]
    扩展 src-tauri/src/commands/session.rs
    - 新增 search_messages command（调用 MessageRepo::search_fts）
    - 注册到 lib.rs invoke_handler

TODO-3.13.4 ✅ [1.0h] [依赖 3.13.3]
    创建 src/components/chat/MessageSearchResults.tsx
    - 搜索结果列表，按会话分组
    - 每条结果显示：会话标题 + 角色标签 + 高亮片段（dangerouslySetInnerHTML for <mark>）
    - 点击跳转到对应会话（切换 activeSessionId）
    - SessionPanel.tsx 搜索框：输入 → 防抖 300ms → 并行调用 search_sessions + search_messages → 展示结果
```

#### 3.14 会话导入/导出（3h） ✅ DONE

```
TODO-3.14.1 ✅ [1.0h] [无依赖]
    修改 src-tauri/src/db/models.rs
    - 新增 ExportData { version, exported_at, app, sessions: Vec<ExportSession> }
    - ExportSession { session + messages }
    - ImportResult { imported_count, skipped_count, errors }

TODO-3.14.2 ✅ [1.0h] [依赖 3.14.1]
    扩展 src-tauri/src/commands/session.rs
    - export_sessions(session_ids, file_path)：
      - 查询会话和关联消息 → 组装 ExportData → serde_json::to_string_pretty 写入文件
    - import_sessions(file_path)：
      - 读取 JSON → 反序列化 ExportData → 逐条插入（跳过 ID 冲突）→ 同步 FTS 索引 → 返回 ImportResult
    - 注册到 lib.rs invoke_handler

TODO-3.14.3 ✅ [1.0h] [依赖 3.14.2]
    前端 UI
    - SessionItem 右键菜单新增"导出"：弹出保存文件对话框（@tauri-apps/plugin-dialog save）→ 调用 export_sessions
    - Settings → About 页面新增 DataManagementCard：
      - "导出所有会话"按钮：调用 list_sessions → export_sessions（全部 ID）
      - "导入会话"按钮：弹出打开文件对话框（@tauri-apps/plugin-dialog open）→ 调用 import_sessions → Toast 显示结果
    - i18n 完整覆盖：zh-CN/en 的 settings.about 新增 dataManagement 相关键
```

### Sprint 5：Nuitka 打包验证（第 10 周）

#### 3.15 Nuitka 打包脚本（4h）🟡 部分完成

```
TODO-3.15.1 [0.5h] [无依赖] ✅ DONE
    创建 agent/run.py
    - Nuitka 独立入口点
    - multiprocessing.freeze_support()
    - uvicorn.run(app, host, port, log_level) — 直接传 app 对象而非字符串

TODO-3.15.2 [1.0h] [依赖 3.15.1] ✅ DONE
    创建 agent/build_nuitka.py
    - Nuitka 命令参数组装
    - --standalone --onefile
    - --include-package=app / --include-module=uvicorn.workers 等必要模块
    - --output-filename=misaka-agent
    - 打包前清理旧产物

TODO-3.15.3 [1.5h] [依赖 3.15.2] ❌ 未完成
    执行打包并验证
    - pip install nuitka（开发环境）⚪
    - python build_nuitka.py ⚪
    - 验证产物：运行 misaka-agent.exe → curl /health ❌（仓库无 agent/dist/）
    - 记录：体积 / 启动时间 / 内存占用 ❌

TODO-3.15.4 [1.0h] [依赖 3.15.3] ❌ 未完成
    问题修复与优化
    - 打包参数已初步调优（build_nuitka.py）✅
    - SidecarManager 仍 spawn `python -m uvicorn`，未集成 exe 路径 ❌
    - 更新文档记录打包流程 🟡（见 PROJECT_STRUCTURE / DEVELOPMENT_STATUS）
```

### 收尾：Schema 迁移 + 全量验证 ✅ COMPLETED

```
TODO-SCHEMA [1.0h] [在 Sprint 2 开始前完成] ✅ DONE
    修改 src-tauri/src/db/migrations.rs
    - migrate_v3：创建 mcp_servers 表
    - migrate_v4：创建 tool_permissions 表
    - migrate_v5：messages 表新增 tool_calls TEXT 字段
    - run_migrations() 中添加 if current_version < 3/4/5 分支

TODO-MODELS [0.5h] [依赖 TODO-SCHEMA] ✅ DONE
    修改 src-tauri/src/db/models.rs
    - McpServerRecord 在 mcp_server_repo.rs 中定义
    - ToolPermission 在 tool_permission_repo.rs 中定义
    - Message 结构体新增 tool_calls: Option<String> 字段
    - message_repo.rs 中 map_row / SELECT / import INSERT 同步更新

TODO-I18N  [1.0h] [Sprint 3 结束时] ✅ DONE
    更新 i18n 文件
    - en/chat.json：新增 session.* (pin/unpin/archive/group/search/export/delete)、toolCall、toolApproval key
    - zh-CN/chat.json：对应中文翻译
    - en/settings.json：MCP 管理页面 + 数据管理 key
    - zh-CN/settings.json：对应中文翻译
    - SessionPanel.tsx / SessionItem.tsx 硬编码中文全部替换为 t() 调用
    - formatRelativeTime 使用 Intl.RelativeTimeFormat 国际化

TODO-FINAL [2.0h] [全部完成后] ❌ 未完成
    Phase 3 完整验证
    - 逐一执行第 11 节 V1-V30 验证清单 ❌
    - 修复发现的问题（见 §1.3）
    - 更新文档
    - 确认 M2 里程碑后可进入 Phase 4
```

### TODO 统计

| 类别 | TODO 数量 | 预估时间 |
|------|----------|---------|
| Sprint 1: Sidecar 预热 | 15 个 | ~16h |
| Sprint 2: MCP 核心 | 15 个 | ~22h |
| Sprint 3: MCP 前端 | 12 个 | ~13h |
| Sprint 4: 会话管理 | 10 个 | ~9h |
| Sprint 5: Nuitka 打包 | 4 个 | ~4h |
| 收尾 | 4 个 | ~4.5h |
| **合计** | **~60 个** | **~64h** |

---

> **文档结束**
>
> 本文档是 Phase 3 的详细执行指南，覆盖了 Sidecar 预热增强、MCP 全链路集成、会话高级管理和 Nuitka 初步打包。
>
> **当前状态（2026-07-06）：** Phase 3 **未完成**。请按 **[`PHASE_3_REMAINING_TODO.md`](./PHASE_3_REMAINING_TODO.md)** 逐项完成后再进入 Phase 4。
>
> **Phase 4 衔接要点（Phase 3 完成后）：**
> 1. Python Sidecar 上实现 `create_deep_agent()`（替换 501 占位端点）
> 2. Rust `chat.rs` 从 Rig → SidecarClient 转发
> 3. 前端 Chat UI 层尽量零改动（ToolCallBlock 已兼容两种模式）
