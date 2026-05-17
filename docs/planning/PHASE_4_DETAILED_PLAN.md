# Phase 4：DeepAgents 全对话迁移 — 详细实施方案

> **所属项目：** MisakaX
> **阶段：** Phase 4（第 11-13 周）
> **总预估：** ~45 小时（含 Vibe Coding 加速）
> **前置条件：** Phase 3 已完成（Sidecar 预热就绪、MCP 全链路打通、SidecarClient 通信协议已定义）
> **前置文档：** [PHASE_3_DETAILED_PLAN.md](./PHASE_3_DETAILED_PLAN.md)、[MISAKAX_IMPLEMENTATION_PLAN.md](./MISAKAX_IMPLEMENTATION_PLAN%20-%20Opus4.6.md)
> **里程碑：** M3: Agent 平台 — DeepAgents 接管所有对话，Rig 退居非对话角色，MisakaX 从"对话客户端"进化为"Agent 平台"。
>
> **⚠️ Phase 4 定位说明：**
> Phase 4 是 MisakaX 架构的**关键拐点**——对话后端从 Rig 直调切换为 DeepAgents Harness。三大核心目标：
> 1. **DeepAgents Agent 实现**：在 Python Sidecar 中实现 `create_deep_agent()` 组装，替换 Phase 3 的 501 占位端点。
> 2. **Rust chat Command 迁移**：`chat.rs` 从 Rig 直调改为 HTTP 转发 Sidecar，保留 Rig 路径为降级开关。
> 3. **PowerMem 智能记忆集成**：通过自定义 Tool 将 PowerMem 接入 Agent 决策链路。
>
> **关键约束：前端 UI 层零改动。** 所有流式事件协议在 Phase 3 已定义（`SidecarClient` + `AgentStreamEvent`），Phase 4 仅需实现服务端逻辑。

---

## 目录

1. [阶段目标与验收标准](#1-阶段目标与验收标准)
2. [代码库现状基线](#2-代码库现状基线)
3. [任务依赖关系与执行顺序](#3-任务依赖关系与执行顺序)
4. [Sprint 1：DeepAgents 核心实现 (4.1–4.2)](#4-sprint-1deepagents-核心实现)
5. [Sprint 2：自定义 Tool 与 SubAgent (4.3–4.4)](#5-sprint-2自定义-tool-与-subagent)
6. [Sprint 3：Rust 侧对话迁移 (4.5–4.6)](#6-sprint-3rust-侧对话迁移)
7. [Sprint 4：状态持久化与记忆系统 (4.7–4.8)](#7-sprint-4状态持久化与记忆系统)
8. [Sprint 5：记忆 UI + HITL + 打包 + 集成测试 (4.9–4.12)](#8-sprint-5记忆-ui--hitl--打包--集成测试)
9. [新增依赖清单](#9-新增依赖清单)
10. [文件创建/修改清单](#10-文件创建修改清单)
11. [Phase 4 完整验证清单](#11-phase-4-完整验证清单)
12. [详细 TODO 列表](#12-详细-todo-列表)

---

## 1. 阶段目标与验收标准

### 1.1 核心目标

将 MisakaX 从"Rig 直调对话客户端"升级为"DeepAgents Agent 平台"——所有用户对话通过 Python Sidecar 中的 DeepAgents Harness 处理，Agent 内部自主决策使用哪些 Middleware 能力。

**Phase 4 的五条执行主线：**

| 主线 | 目标 | 交付物 |
|------|------|--------|
| **DeepAgents Agent 组装** | `create_deep_agent()` 一行组装含完整 Middleware 的 Agent | `agent/app/agent.py` |
| **自定义 Tool 开发** | PowerMem 检索/存储 + MCP Bridge + 文件操作 | `agent/app/tools.py` |
| **Rust 对话迁移** | `chat.rs` 从 Rig→Sidecar 转发，保留降级开关 | `chat.rs` 重构 |
| **状态持久化** | LangGraph Checkpointer + PowerMem 初始化 | 对话状态跨重启保留 |
| **记忆管理 UI** | 查看/搜索/删除长期记忆 | React 记忆管理页面 |

### 1.2 验收标准

| # | 验收项 | 验收方式 |
|---|--------|---------|
| AC-1 | 用户发送消息 → DeepAgents 处理 → 流式回复（与 Phase 3 体验一致） | 正常对话，观察流式输出 |
| AC-2 | Agent 自主决定是否调用 Tool（PowerMem / MCP / 文件操作） | 发送需要记忆的问题 → 触发 powermem_save |
| AC-3 | SubAgent 可被正确派生并返回结果（研究/编码） | 发送研究类问题 → 观察 researcher SubAgent 调用 |
| AC-4 | FilesystemMiddleware 基于会话 working_dir 操作文件 | 绑定目录后 Agent 可读写文件 |
| AC-5 | Rig 降级开关可用（config 中 `use_sidecar: false` 时回退 Rig 直调） | 关闭开关 → 验证 Rig 对话正常 |
| AC-6 | Rig 非对话功能正常（Embedding、自动标题、摘要生成） | 新会话自动生成标题 |
| AC-7 | LangGraph 状态持久化有效（重启 Sidecar 后恢复上下文） | 重启后发送后续问题 → 上下文连续 |
| AC-8 | PowerMem 记忆跨会话持久化 | 会话 A 存储记忆 → 会话 B 检索到 |
| AC-9 | 记忆管理页面可查看/搜索/删除记忆 | UI 操作验证 |
| AC-10 | Human-in-the-loop 审批在危险操作时触发 | Agent 尝试写文件 → 弹出审批对话框 |
| AC-11 | TodoListMiddleware 生成任务计划可视化 | 复杂问题 → Agent 生成 todo → UI 展示 |
| AC-12 | 端到端全链路（对话→Agent 决策→工具调用→记忆读写→流式输出） | 完整场景测试 |
| AC-13 | Nuitka 打包含 DeepAgents + PowerMem 依赖成功 | 打包产物独立运行 + health check |
| AC-14 | SummarizationMiddleware 长对话自动摘要压缩 | 50+ 轮对话后观察上下文窗口 |

---

## 2. 代码库现状基线

### 2.1 Phase 3 已交付的基础设施

**Python Sidecar (`agent/`)：**
- `app/main.py` — FastAPI 入口，已注册 health、info、agent 三组 router
- `app/routers/agent.py` — `/agent/chat` 和 `/agent/stream` **501 占位端点**（Phase 4 需实现）
- `app/routers/health.py` — 增强型 `/health`（返回 uptime、capabilities、agent_ready）
- `app/routers/info.py` — `/info`（版本、Python 版本、依赖可用性检测）
- `app/models.py` — Pydantic v2 模型（ChatRequest/ChatResponse/ChatMessage/ToolCall/TokenUsage）
- `app/config.py` — pydantic-settings 配置
- `pyproject.toml` — 依赖已声明 `langgraph>=0.3`（optional agent group）、`powermem>=1.1`（optional memory group）

**Rust 侧 Sidecar 通信基础设施：**
- `src-tauri/src/sidecar.rs` — 异步 `SidecarManager`（preheat + health_loop + auto-restart + status 事件）
- `src-tauri/src/services/sidecar_client.rs` — `SidecarClient`（health/info/chat/stream 方法已定义）
- `src-tauri/src/commands/sidecar.rs` — Tauri Command（get_sidecar_status / restart_sidecar）

**Rust 侧当前对话链路（Phase 4 需迁移）：**
- `src-tauri/src/commands/chat.rs` — `send_message` / `stop_generation` / `regenerate_message` / `get_messages`
- 当前路径：`send_message` → Rig `RigBackend::send_and_stream()` → 直接流式调用 LLM API
- Phase 4 目标路径：`send_message` → `SidecarClient::stream()` → Python DeepAgents → 流式返回

**Rust 侧 MCP 基础设施（Phase 4 复用）：**
- `src-tauri/src/services/mcp/manager.rs` — `McpManager`（连接管理、工具列表）
- `src-tauri/src/services/mcp_bridge.rs` — `McpToolBridge`（tool_descriptions + call_tool）
- Phase 4 的 DeepAgents 通过 `mcp_bridge_tool`（自定义 Tool）调用此桥接器

**关键数据模型：**
- `AgentChatRequest` / `AgentChatResponse` / `AgentStreamEvent` — Rust 侧已定义，与 Python `models.py` 对齐
- `ChatRequest` / `ChatResponse` — Python 侧已定义
- 两端模型已互相对齐，Phase 4 仅需实现端点逻辑

### 2.2 关键文件数量快照

| 层 | 文件数 | 说明 |
|----|--------|------|
| Rust (`.rs`) | 45 | 含 MCP 模块（4 文件）、Sidecar 模块（2 文件） |
| React (`.tsx`) | ~48+ | 含完整 chat 组件族 |
| Python (`.py`) | 8 | FastAPI 骨架 + 占位端点 |
| 已注册 Tauri Commands | 27+ | settings/router_configs/models/chat/workspace/session/sidecar/mcp |

### 2.3 Phase 4 可直接复用的模块

| 模块 | 当前状态 | Phase 4 复用方式 |
|------|----------|----------------|
| `SidecarClient` | ✅ 已实现 `chat()` / `stream()` | 直接调用，不修改 |
| `SidecarManager` | ✅ 异步预热 + 健康检查 | 直接复用，不修改 |
| `AppState.sidecar` | ✅ `Arc<SidecarManager>` 注入 | `chat.rs` 内通过 State 获取 |
| `AgentStreamEvent` | ✅ 已定义 `event` + `data` | 前端 SSE 解析逻辑已就绪 |
| `McpToolBridge` | ✅ 工具描述 + 调用 | 通过 HTTP 暴露给 Python 调用 |
| 前端流式渲染 | ✅ 已基于 Tauri Event 渲染 | 零改动 |
| ToolCallBlock 组件 | ✅ 已实现 | 兼容 Rig/DeepAgents 模式 |

---

## 3. 任务依赖关系与执行顺序

### 3.1 依赖图

```
Phase 3 产出（Sidecar 预热 / SidecarClient / MCP / 通信协议）
    │
    ├── Sprint 1: DeepAgents 核心实现 ──────────────────────────────────
    │   ├── 4.1 DeepAgents Agent 组装 (agent.py + config.py)
    │   │       └── 依赖：pip install deepagents 环境准备
    │   └── 4.2 FilesystemMiddleware 工作目录绑定
    │           └── 依赖：4.1（需要 agent 实例）
    │
    │   （Sprint 1 完成后 Sprint 2 + Sprint 3 可并行启动）
    │
    ├── Sprint 2: 自定义 Tool + SubAgent（与 Sprint 3 并行）─────────────
    │   ├── 4.3 自定义 Tool 开发 (powermem_search / powermem_save / mcp_bridge)
    │   │       └── 依赖：4.1（Tool 注入到 Agent）
    │   └── 4.4 SubAgent 配置 (researcher / coder / analyst)
    │           └── 依赖：4.1（SubAgent 注入到 Agent）
    │
    ├── Sprint 3: Rust 侧对话迁移（与 Sprint 2 并行）──────────────────
    │   ├── 4.5 Rust chat Command 改写 (Rig → Sidecar 转发 + 降级开关)
    │   │       └── 依赖：4.1（Sidecar 端点已实现）
    │   └── 4.6 Rig 对话功能退役 (保留 Embedding/标题/摘要)
    │           └── 依赖：4.5（迁移完成后清理）
    │
    │   （Sprint 2+3 完成后 Sprint 4 可启动）
    │
    ├── Sprint 4: 状态持久化 + 记忆系统 ─────────────────────────────────
    │   ├── 4.7 LangGraph Checkpointer (langgraph-checkpoint-sqlite)
    │   │       └── 依赖：4.1（Checkpointer 传入 create_deep_agent）
    │   └── 4.8 PowerMem 集成 (初始化 + 环境配置 + Tool 封装)
    │           └── 依赖：4.3（Tool 已定义）
    │
    └── Sprint 5: 记忆 UI + HITL + 打包 + 集成测试 ──────────────────────
        ├── 4.9  记忆管理 UI (查看/搜索/删除)
        │        └── 依赖：4.8（PowerMem 已集成）
        ├── 4.10 Human-in-the-loop 配置 + 前端审批 UI
        │        └── 依赖：4.1（interrupt_on 配置）+ Phase 3 权限审批组件
        ├── 4.11 Nuitka 打包脚本完善 (含 deepagents + powermem)
        │        └── 依赖：4.8（所有 Python 依赖确定）
        └── 4.12 端到端集成测试
                 └── 依赖：所有前序任务
```

### 3.2 推荐执行顺序（按周分配）

| 周 | 主要任务 | 预估时间 | 说明 |
|----|---------|---------|------|
| **第 11 周** | Sprint 1 全部 (4.1→4.2) + Sprint 2 (4.3, 4.4) | ~18h | DeepAgents 核心组装 + Tool/SubAgent 是整个 Phase 的地基 |
| **第 12 周** | Sprint 3 全部 (4.5→4.6) + Sprint 4 (4.7, 4.8) | ~15h | Rust 侧切换 + 状态持久化是端到端打通的关键 |
| **第 13 周** | Sprint 5 全部 (4.9→4.12) | ~12h | 记忆 UI + HITL + 打包 + 最终验证 |

### 3.3 为什么这个顺序

1. **DeepAgents Agent 先行**（Sprint 1）：这是整个 Phase 的核心——`create_deep_agent()` 组装完成后，Sidecar 才能提供真正的对话能力。所有后续任务都依赖此步。

2. **Tool + SubAgent 紧随**（Sprint 2）：自定义 Tool 定义了 Agent 的能力边界（能做什么）；SubAgent 定义了任务委派策略。两者独立于 Rust 侧迁移，可并行开发。

3. **Rust 迁移并行**（Sprint 3）：一旦 Sidecar 端点从 501 变为可用，就可以开始改写 `chat.rs`。这是最关键的"切换点"——用户对话路径从 Rig→DeepAgents。

4. **状态持久化跟进**（Sprint 4）：Checkpointer 和 PowerMem 确保对话状态和记忆的持久化。在基础对话流程打通后，加入持久化层。

5. **UI + 打包收尾**（Sprint 5）：记忆管理 UI、HITL 审批 UI 是功能补全；Nuitka 打包是发布准备；集成测试是质量保障。

---

## 4. Sprint 1：DeepAgents 核心实现

### 4.1 任务 4.1：DeepAgents 核心集成（6h）

#### 4.1.1 目标

在 Python Sidecar 中实现 `create_deep_agent()` 工厂调用，组装包含完整 Middleware 链的 Agent，并将 `/agent/chat` 和 `/agent/stream` 从 501 占位替换为真正的对话处理逻辑。

#### 4.1.2 前置条件

- `pip install deepagents[all]>=0.5.4` 安装成功
- `pyproject.toml` 中 `agent` optional-dependencies 已声明 `deepagents>=0.5`
- Phase 3 的 `app/models.py` 中 `ChatRequest` / `ChatResponse` 已定义

#### 4.1.3 实现方案

**依赖安装（更新 `pyproject.toml`）：**

```toml
[project.optional-dependencies]
agent = [
    "deepagents>=0.5.4",
    "langgraph>=0.3",
    "langchain-anthropic>=0.3",
    "langchain-openai>=0.3",
    "langgraph-checkpoint-sqlite>=2.0",
]
```

**核心 Agent 组装 (`agent/app/agent.py`)：**

```python
"""MisakaX DeepAgent 核心组装模块。

通过 create_deep_agent() 一行代码组装含完整 Middleware 的 Agent，
替代手动 StateGraph + Node + Edge 定义（节省 ~80% 代码量）。
"""
from __future__ import annotations

from typing import TYPE_CHECKING

from deepagents import SubAgent, create_deep_agent
from deepagents.backends import CompositeBackend, FilesystemBackend, StateBackend

from .config import get_settings
from .prompts import SYSTEM_PROMPT
from .tools import get_tools

if TYPE_CHECKING:
    from langgraph.graph.state import CompiledStateGraph


def build_agent(
    session_id: str | None = None,
    working_dir: str | None = None,
    checkpointer=None,
    store=None,
) -> CompiledStateGraph:
    """创建 MisakaX DeepAgent，返回 CompiledStateGraph。

    Args:
        session_id: 会话 ID（用于 Checkpointer thread_id）。
        working_dir: 会话绑定的工作目录路径（FilesystemMiddleware 基于此运行）。
        checkpointer: LangGraph Checkpointer（状态持久化）。
        store: LangGraph Store（跨会话共享数据）。

    Returns:
        编译后的 StateGraph，可直接 invoke/stream。
    """
    settings = get_settings()

    # 构建可插拔后端路由
    routes: dict[str, object] = {}
    if working_dir:
        routes["/workspace/"] = FilesystemBackend(root_dir=working_dir)

    # SubAgent 配置
    subagents = _build_subagents(settings)

    return create_deep_agent(
        model=settings.agent_model,
        tools=get_tools(),
        system_prompt=SYSTEM_PROMPT,
        subagents=subagents,
        skills=[str(settings.skills_dir)],
        memory=[str(settings.memories_dir)],
        backend=lambda rt: CompositeBackend(
            default=StateBackend(rt),
            routes=routes,
        ),
        interrupt_on=settings.interrupt_config,
        checkpointer=checkpointer,
        store=store,
        temperature=settings.temperature,
        max_tokens=settings.max_tokens,
    )


def _build_subagents(settings) -> list[SubAgent]:
    """构建 SubAgent 列表（研究/编码/分析）。"""
    from .tools import get_research_tools

    return [
        SubAgent(
            name="researcher",
            description="Deep research specialist for web search, document analysis, and information synthesis",
            system_prompt=(
                "You are a research specialist. Your task is to thoroughly research topics, "
                "find relevant information, and synthesize findings into clear, actionable summaries. "
                "Use available tools to search memory and external sources."
            ),
            tools=get_research_tools(),
        ),
        SubAgent(
            name="coder",
            description="Code generation, review, and refactoring agent with filesystem access",
            system_prompt=(
                "You are a coding specialist. Your task is to write, review, and refactor code. "
                "You have access to the workspace filesystem. Follow best practices, write clean code, "
                "and include proper error handling."
            ),
        ),
        SubAgent(
            name="analyst",
            description="Data analysis and problem decomposition agent",
            system_prompt=(
                "You are an analytical specialist. Your task is to break down complex problems, "
                "analyze data, identify patterns, and provide structured insights."
            ),
        ),
    ]
```

**Agent 配置 (`agent/app/config.py` 扩展)：**

```python
"""MisakaX Agent 配置模块。"""
from __future__ import annotations

from pathlib import Path
from functools import lru_cache

from pydantic import Field
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """Agent Sidecar 配置（环境变量 / .env 加载）。"""

    # Server
    host: str = "127.0.0.1"
    port: int = 9527
    debug: bool = False

    # Agent 模型
    agent_model: str = "claude-sonnet-4-6"
    temperature: float = 0.7
    max_tokens: int | None = None

    # API Keys（从环境变量或 Rust 传递）
    anthropic_api_key: str = ""
    openai_api_key: str = ""

    # 目录
    skills_dir: Path = Path.home() / ".misakax" / "skills"
    memories_dir: Path = Path.home() / ".misakax" / "memories"
    data_dir: Path = Path.home() / ".misakax" / "data"

    # PowerMem
    powermem_enabled: bool = True
    powermem_db_path: Path = Field(
        default_factory=lambda: Path.home() / ".misakax" / "data" / "powermem.db"
    )

    # Checkpointer
    checkpointer_db_path: Path = Field(
        default_factory=lambda: Path.home() / ".misakax" / "data" / "checkpoints.db"
    )

    # HITL 中断配置
    interrupt_config: dict = Field(default_factory=lambda: {
        "file_write": True,
        "file_delete": True,
        "shell_execute": True,
    })

    # MCP Bridge
    mcp_bridge_url: str = "http://127.0.0.1:9528"

    model_config = {"env_prefix": "MISAKA_", "env_file": ".env", "extra": "ignore"}


@lru_cache
def get_settings() -> Settings:
    return Settings()
```

**流式端点实现 (`agent/app/routers/agent.py` 重写)：**

```python
"""Agent 对话端点 — DeepAgents 全对话入口。"""
from __future__ import annotations

import json
import time
from typing import AsyncGenerator

from fastapi import APIRouter, HTTPException
from fastapi.responses import StreamingResponse

from app.agent import build_agent
from app.config import get_settings
from app.dependencies import get_checkpointer, get_store
from app.models import ChatMessage, ChatRequest, ChatResponse, TokenUsage, ToolCall

router = APIRouter(prefix="/agent", tags=["agent"])


@router.post("/chat")
async def agent_chat(request: ChatRequest) -> ChatResponse:
    """同步 Agent 对话 — 等待完整响应后返回。"""
    try:
        agent = build_agent(
            session_id=request.session_id,
            working_dir=request.working_dir,
            checkpointer=get_checkpointer(),
            store=get_store(),
        )

        config = {"configurable": {"thread_id": request.session_id or "default"}}
        if request.config.model:
            config["configurable"]["model"] = request.config.model

        messages = [
            {"role": msg.role.value, "content": msg.content}
            for msg in request.messages
        ]

        result = await agent.ainvoke({"messages": messages}, config=config)

        last_msg = result["messages"][-1]
        return ChatResponse(
            message=ChatMessage(role="assistant", content=last_msg.content),
            tool_calls=[
                ToolCall(id=tc["id"], name=tc["name"], arguments=tc.get("args", {}))
                for tc in getattr(last_msg, "tool_calls", [])
            ],
            usage=_extract_usage(result),
            model=get_settings().agent_model,
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/stream")
async def agent_stream(request: ChatRequest):
    """流式 Agent 对话 — SSE 格式逐 token 推送。"""
    return StreamingResponse(
        _stream_agent(request),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        },
    )


async def _stream_agent(request: ChatRequest) -> AsyncGenerator[str, None]:
    """流式 Agent 执行生成器。"""
    try:
        agent = build_agent(
            session_id=request.session_id,
            working_dir=request.working_dir,
            checkpointer=get_checkpointer(),
            store=get_store(),
        )

        config = {"configurable": {"thread_id": request.session_id or "default"}}
        messages = [
            {"role": msg.role.value, "content": msg.content}
            for msg in request.messages
        ]

        async for event in agent.astream_events(
            {"messages": messages}, config=config, version="v2"
        ):
            sse_data = _format_sse_event(event)
            if sse_data:
                yield sse_data

        yield _sse("done", {"finished": True})

    except Exception as e:
        yield _sse("error", {"message": str(e)})


def _format_sse_event(event: dict) -> str | None:
    """将 LangGraph 事件转为 SSE 格式。"""
    kind = event.get("event")

    if kind == "on_chat_model_stream":
        chunk = event.get("data", {}).get("chunk")
        if chunk and hasattr(chunk, "content") and chunk.content:
            return _sse("token", {"content": chunk.content})

    elif kind == "on_tool_start":
        return _sse("tool_start", {
            "name": event.get("name", ""),
            "input": event.get("data", {}).get("input", {}),
        })

    elif kind == "on_tool_end":
        output = event.get("data", {}).get("output", "")
        return _sse("tool_end", {
            "name": event.get("name", ""),
            "output": str(output)[:2000],
        })

    elif kind == "on_chat_model_start":
        return _sse("thinking_start", {})

    return None


def _sse(event: str, data: dict) -> str:
    """格式化 SSE 事件字符串。"""
    return f"event: {event}\ndata: {json.dumps(data, ensure_ascii=False)}\n\n"


def _extract_usage(result: dict) -> TokenUsage:
    """从 Agent 结果中提取 token 用量。"""
    metadata = result.get("__metadata__", {})
    return TokenUsage(
        prompt_tokens=metadata.get("prompt_tokens", 0),
        completion_tokens=metadata.get("completion_tokens", 0),
        total_tokens=metadata.get("total_tokens", 0),
    )
```

#### 4.1.4 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `agent/app/agent.py` | **新增** | DeepAgents 核心组装（build_agent + _build_subagents） |
| `agent/app/prompts.py` | **新增** | SYSTEM_PROMPT 常量 + prompt 模板 |
| `agent/app/dependencies.py` | **新增** | FastAPI 依赖注入（get_checkpointer / get_store） |
| `agent/app/routers/agent.py` | **重写** | 从 501 占位替换为完整 /agent/chat 和 /agent/stream |
| `agent/app/config.py` | **修改** | 扩展 Settings 类（agent_model/skills_dir/memories_dir/interrupt_config 等） |
| `agent/app/models.py` | **修改** | 可能需要新增 StreamEvent Pydantic 模型 |
| `agent/pyproject.toml` | **修改** | agent group 新增 `deepagents>=0.5.4` |

---

### 4.2 任务 4.2：FilesystemMiddleware 工作目录绑定（3h）

#### 4.2.1 目标

根据每个会话的 `working_dir` 字段动态构建 `CompositeBackend`，使 DeepAgents 的 `FilesystemMiddleware` 基于该目录执行文件操作（ls/read/write/edit/glob/grep）。

#### 4.2.2 实现方案

**工作目录传递链路：**

```
前端 ChatRequest { session_id, working_dir }
  → Rust chat.rs（从 Session 表读取 working_dir_local）
    → SidecarClient::stream(AgentChatRequest { working_dir: Some(path) })
      → Python /agent/stream (request.working_dir)
        → build_agent(working_dir=request.working_dir)
          → FilesystemBackend(root_dir=working_dir)
```

**工作目录验证（agent/app/utils.py）：**

```python
"""工作目录安全验证。"""
from pathlib import Path

FORBIDDEN_DIRS = {"/", "/etc", "/usr", "/bin", "/sbin", "/var", "C:\\Windows", "C:\\Program Files"}


def validate_working_dir(working_dir: str | None) -> Path | None:
    """验证工作目录路径的安全性和有效性。"""
    if not working_dir:
        return None

    path = Path(working_dir).resolve()

    if not path.exists():
        raise ValueError(f"Working directory does not exist: {path}")
    if not path.is_dir():
        raise ValueError(f"Path is not a directory: {path}")
    if str(path) in FORBIDDEN_DIRS:
        raise ValueError(f"Forbidden directory: {path}")

    return path
```

**Backend 动态构建（已在 4.1 的 build_agent 中实现）：**

```python
routes: dict[str, object] = {}
if working_dir:
    validated = validate_working_dir(working_dir)
    if validated:
        routes["/workspace/"] = FilesystemBackend(root_dir=str(validated))
```

#### 4.2.3 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `agent/app/utils.py` | **新增** | 工作目录验证、安全检查 |
| `agent/app/agent.py` | **修改** | build_agent 中集成 validate_working_dir |

---

## 5. Sprint 2：自定义 Tool 与 SubAgent

### 4.3 任务 4.3：自定义 Tool 开发（6h）

#### 4.3.1 目标

开发三个核心自定义 Tool，赋予 Agent 长期记忆存储/检索能力和 MCP 工具调用能力。

#### 4.3.2 Tool 清单

| Tool | 用途 | 触发场景 |
|------|------|---------|
| `powermem_search` | 从长期记忆中检索相关信息 | Agent 判断需要回忆之前的信息 |
| `powermem_save` | 保存重要信息到长期记忆 | Agent 判断当前信息值得记住 |
| `mcp_bridge` | 调用 Rust 侧管理的 MCP 工具 | Agent 需要调用外部工具（文件系统/搜索等） |

#### 4.3.3 实现方案

```python
# agent/app/tools.py
"""MisakaX 自定义 Tool 集合。

包含 PowerMem 记忆工具和 MCP Bridge 工具。
所有 Tool 通过 @tool 装饰器注册，遵循 LangChain Tool 协议。
"""
from __future__ import annotations

from typing import Any

from langchain_core.tools import tool

from .config import get_settings


# --------------------------------------------------------------------------- #
#  PowerMem Tools
# --------------------------------------------------------------------------- #

@tool
def powermem_search(query: str, limit: int = 5) -> list[dict[str, Any]]:
    """Search long-term memory for information relevant to the query.

    Use this tool when you need to recall previously stored information,
    user preferences, past conversations context, or any long-term knowledge.

    Args:
        query: Natural language search query describing what to recall.
        limit: Maximum number of results to return (default 5).

    Returns:
        List of memory entries with content and relevance score.
    """
    from .memory import get_memory_engine

    engine = get_memory_engine()
    if engine is None:
        return [{"content": "Memory system is not available.", "score": 0.0}]

    results = engine.search(query, limit=limit)
    return [
        {"content": r.content, "score": r.score, "created_at": str(r.created_at)}
        for r in results
    ]


@tool
def powermem_save(content: str, importance: str = "normal") -> str:
    """Save important information to long-term memory for future recall.

    Use this tool when the conversation contains information worth remembering:
    - User preferences or personal details they share
    - Important decisions or conclusions reached
    - Technical knowledge or solutions discovered
    - Context that would be useful in future conversations

    Args:
        content: The information to remember (be concise but complete).
        importance: Priority level - "low", "normal", or "high".

    Returns:
        Confirmation message.
    """
    from .memory import get_memory_engine

    engine = get_memory_engine()
    if engine is None:
        return "Memory system is not available. Information was not saved."

    engine.add(content, metadata={"importance": importance}, smart_process=True)
    return f"Memory saved successfully. Content: {content[:50]}..."


# --------------------------------------------------------------------------- #
#  MCP Bridge Tool
# --------------------------------------------------------------------------- #

@tool
async def mcp_bridge(server_id: str, tool_name: str, arguments: dict[str, Any] | None = None) -> str:
    """Call an MCP (Model Context Protocol) tool managed by the Rust backend.

    Use this tool to invoke external capabilities provided by MCP Servers,
    such as filesystem operations, web search, database queries, etc.

    Args:
        server_id: The ID of the MCP server to call.
        tool_name: The name of the tool to invoke on that server.
        arguments: Optional dictionary of arguments to pass to the tool.

    Returns:
        The tool execution result as a string.
    """
    import httpx

    settings = get_settings()
    url = f"{settings.mcp_bridge_url}/mcp/call_tool"

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(url, json={
            "server_id": server_id,
            "tool_name": tool_name,
            "arguments": arguments or {},
        })

    if response.status_code != 200:
        return f"MCP call failed: {response.text}"

    return response.json().get("result", "No result returned.")


# --------------------------------------------------------------------------- #
#  Tool Registry
# --------------------------------------------------------------------------- #

def get_tools() -> list:
    """获取所有可用的自定义 Tool。"""
    tools = [powermem_search, powermem_save, mcp_bridge]
    return tools


def get_research_tools() -> list:
    """获取研究类 SubAgent 专用 Tool。"""
    return [powermem_search]
```

#### 4.3.4 MCP Bridge HTTP 端点

需要在 Rust 侧暴露一个本地 HTTP 端点供 Python Sidecar 调用 MCP 工具（避免直接进程通信的复杂性）。

**方案 A（推荐）：Rust 侧新增 HTTP 服务**

在 Tauri 应用内启动一个轻量级 HTTP Server（仅绑定 127.0.0.1:9528），暴露 `/mcp/call_tool` 和 `/mcp/list_tools` 端点，供 Python Sidecar 调用。

**方案 B（备选）：Python 通过 Tauri Command 间接调用**

Python 发送 HTTP 请求到 Sidecar 自身的一个端点，该端点通过某种 IPC 机制调用 Rust。此方案复杂度高，不推荐。

**选择方案 A**，在 `src-tauri/src/services/` 下新增 `mcp_http_bridge.rs`。

#### 4.3.5 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `agent/app/tools.py` | **新增** | 自定义 Tool 定义（powermem_search/save + mcp_bridge） |
| `agent/app/memory.py` | **新增** | PowerMem 引擎初始化和管理 |
| `src-tauri/src/services/mcp_http_bridge.rs` | **新增** | 本地 HTTP 端点暴露 MCP 调用能力 |
| `src-tauri/src/services/mod.rs` | **修改** | 注册 mcp_http_bridge 模块 |

---

### 4.4 任务 4.4：SubAgent 配置（3h）

#### 4.4.1 目标

配置三个 SubAgent（researcher/coder/analyst），每个拥有独立上下文窗口和工具集。

#### 4.4.2 SubAgent 设计

| SubAgent | 职责 | 工具集 | 触发条件 |
|----------|------|--------|---------|
| **researcher** | 深度研究、信息搜索与综合 | powermem_search、web_search（如果有 MCP Server） | 用户询问需要深度研究的问题 |
| **coder** | 代码生成、审查、重构 | FilesystemMiddleware 工具（ls/read/write/edit） | 用户请求编写/修改代码 |
| **analyst** | 数据分析、问题分解 | powermem_search | 用户需要分析复杂问题 |

#### 4.4.3 实现

SubAgent 配置已在 4.1 的 `build_agent()` → `_build_subagents()` 中实现。此任务重点是：

1. 编写每个 SubAgent 的 system prompt（针对角色特化的详细指引）
2. 验证 SubAgent 上下文隔离（子代理不共享父代理的完整对话历史）
3. 验证异步 SubAgent 委托模式

**SubAgent Prompt 文件 (`agent/app/prompts.py` 扩展)：**

```python
RESEARCHER_PROMPT = """You are a research specialist working for MisakaX AI Agent.

## Core Capabilities
- Deep web research and information synthesis
- Academic paper and documentation analysis
- Memory-augmented research (recall past findings)

## Guidelines
- Always verify information from multiple sources
- Provide citations and source references when possible
- Use powermem_search to recall relevant past research
- Synthesize findings into clear, actionable summaries
- If unsure, acknowledge uncertainty explicitly
"""

CODER_PROMPT = """You are a coding specialist working for MisakaX AI Agent.

## Core Capabilities
- Code generation (Python, Rust, TypeScript, and more)
- Code review and quality analysis
- Refactoring and optimization
- Bug identification and fixing

## Guidelines
- Follow language-specific best practices and idioms
- Write clean, readable code with proper error handling
- Use the filesystem tools to read existing code context
- Explain complex logic with inline comments only when non-obvious
- Suggest tests for critical code paths
"""

ANALYST_PROMPT = """You are an analytical specialist working for MisakaX AI Agent.

## Core Capabilities
- Complex problem decomposition
- Data pattern recognition
- Structured reasoning and analysis
- Decision framework construction

## Guidelines
- Break complex problems into manageable sub-problems
- Identify assumptions and constraints explicitly
- Use structured formats (tables, lists) for clarity
- Provide confidence levels for conclusions
- Cross-reference with memory for historical context
"""
```

#### 4.4.4 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `agent/app/prompts.py` | **新增/扩展** | SubAgent 专用 system prompt |
| `agent/app/agent.py` | **修改** | 引用 prompts.py 中的常量 |

---

## 6. Sprint 3：Rust 侧对话迁移

### 4.5 任务 4.5：Rust chat Command 改写（4h）

#### 4.5.1 目标

将 `send_message` Command 的后端从 Rig 直调切换为 HTTP 转发到 Sidecar，同时保留 Rig 路径作为降级开关（通过配置控制）。

#### 4.5.2 迁移策略

```
当前 (Phase 2-3):
  send_message → resolve_model → RigBackend::send_and_stream → 流式推送

目标 (Phase 4):
  send_message
    ├── if config.use_sidecar == true (默认):
    │   → SidecarClient::stream → 解析 SSE → 转发 Tauri Event → 前端
    └── if config.use_sidecar == false (降级):
        → RigBackend::send_and_stream → 流式推送（原逻辑保留）
```

#### 4.5.3 核心变更

**新增降级开关配置 (`config.rs`)：**

在 `AppConfig` 中新增：
```rust
/// 是否使用 Sidecar（DeepAgents）处理对话。设为 false 回退到 Rig 直调。
pub use_sidecar: bool,  // default: true
```

**`chat.rs` 核心改写逻辑：**

```rust
// src-tauri/src/commands/chat.rs — send_message 改写核心逻辑

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    state: State<'_, AppState>,
    request: SendMessageRequest,
) -> Result<SendMessageResult, String> {
    let user_msg_id = uuid::Uuid::new_v4().to_string();
    let assistant_msg_id = uuid::Uuid::new_v4().to_string();

    // Step 1: 保存用户消息（不变）
    save_user_message(&state, &user_msg_id, &request)?;

    // Step 2: 读取配置，决定对话路径
    let use_sidecar = state.config.lock().unwrap().use_sidecar;

    if use_sidecar {
        // ─── DeepAgents 路径 ───
        send_via_sidecar(&app, &state, &request, &user_msg_id, &assistant_msg_id).await
    } else {
        // ─── Rig 降级路径（保留原逻辑） ───
        send_via_rig(&app, &state, &request, &user_msg_id, &assistant_msg_id).await
    }
}

/// DeepAgents 路径：转发 Sidecar + 解析 SSE + 转发 Tauri Event
async fn send_via_sidecar(
    app: &AppHandle,
    state: &AppState,
    request: &SendMessageRequest,
    user_msg_id: &str,
    assistant_msg_id: &str,
) -> Result<SendMessageResult, String> {
    // 1. 加载会话上下文（获取 working_dir）
    let (session, history) = load_session_context(state, &request.session_id)?;

    // 2. 创建 assistant 占位
    let model_spec = resolve_model_spec(
        request.model_override.as_deref(),
        session.model.as_deref(),
    )?;
    create_assistant_placeholder(state, assistant_msg_id, &request.session_id, &model_spec)?;

    // 3. 构建 AgentChatRequest
    let messages = build_agent_messages(&history, &request.content);
    let agent_request = AgentChatRequest {
        messages,
        config: AgentChatConfig {
            model: Some(model_spec.model_id.clone()),
            temperature: 0.7,
            max_tokens: None,
            stream: true,
        },
        session_id: Some(request.session_id.clone()),
        working_dir: session.working_dir_local.clone(),
    };

    // 4. 调用 SidecarClient::stream，解析 SSE，逐 token 转发
    let sidecar_client = SidecarClient::new(state.config.lock().unwrap().sidecar_port);
    let response = sidecar_client.stream(&agent_request).await
        .map_err(|e| format!("Sidecar stream failed: {e}"))?;

    let stream_result = parse_sidecar_sse(app, response, assistant_msg_id).await?;

    // 5. 更新 assistant 消息
    update_assistant_message(state, assistant_msg_id, &stream_result)?;
    update_session_stats(state, &request.session_id, &stream_result)?;

    Ok(SendMessageResult {
        user_message_id: user_msg_id.to_string(),
        assistant_message_id: assistant_msg_id.to_string(),
    })
}
```

**SSE 解析与 Tauri Event 转发：**

```rust
/// 解析 Sidecar 返回的 SSE 流，逐 token 通过 Tauri Event 推送给前端
async fn parse_sidecar_sse(
    app: &AppHandle,
    response: reqwest::Response,
    assistant_msg_id: &str,
) -> Result<StreamResult, String> {
    use futures_util::StreamExt;

    let mut content = String::new();
    let mut thinking = String::new();
    let mut stream = response.bytes_stream();

    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream read error: {e}"))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // 逐行解析 SSE
        while let Some(pos) = buffer.find("\n\n") {
            let event_block = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            if let Some(sse) = parse_sse_block(&event_block) {
                match sse.event.as_str() {
                    "token" => {
                        if let Some(token) = sse.data.get("content").and_then(|v| v.as_str()) {
                            content.push_str(token);
                            // 推送给前端（复用现有 stream_token 事件格式）
                            let _ = app.emit("stream_token", serde_json::json!({
                                "message_id": assistant_msg_id,
                                "token": token,
                            }));
                        }
                    }
                    "tool_start" => {
                        let _ = app.emit("tool_call_start", &sse.data);
                    }
                    "tool_end" => {
                        let _ = app.emit("tool_call_end", &sse.data);
                    }
                    "done" => break,
                    "error" => {
                        let msg = sse.data.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown error");
                        return Err(format!("Agent error: {msg}"));
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(StreamResult {
        content,
        thinking,
        usage: None, // TODO: 从 SSE 最终事件提取
        was_aborted: false,
    })
}
```

#### 4.5.4 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/src/commands/chat.rs` | **重大修改** | send_message 拆分为 sidecar/rig 双路径 |
| `src-tauri/src/config.rs` | **修改** | AppConfig 新增 `use_sidecar: bool` |
| `src-tauri/src/services/sidecar_client.rs` | **修改** | 可能需要 SSE 解析辅助函数 |

---

### 4.6 任务 4.6：Rig 对话功能退役 + 非对话功能保留（3h）

#### 4.6.1 目标

清理 Rig 对话相关代码路径，同时保留 Rig 在以下非对话场景的使用：
- **Embedding 生成**：知识库向量化（Phase 5 使用）
- **自动标题生成**：新会话首条消息后自动生成标题
- **摘要生成**：长对话摘要（可选）

#### 4.6.2 清理范围

| 保留 | 清理 | 说明 |
|------|------|------|
| ✅ `RigBackend::send_and_stream` | ❌ 不删除 | 保留为降级路径（`use_sidecar=false`） |
| ✅ `RigBackend::generate_embedding` | - | 知识库向量化 |
| ✅ `services/llm/` 模块 | - | Provider 抽象保留 |
| - | ⚠️ `chat.rs` 中 Rig 为默认路径 | 改为 Sidecar 为默认 |
| - | ⚠️ MCP Tool 注入 system_prompt 逻辑 | 迁移到 Python 侧 |

#### 4.6.3 具体操作

1. **`chat.rs`**：Rig 直调逻辑抽取为 `send_via_rig()` 独立函数（保留但非默认）
2. **MCP Tool 描述注入**：Phase 3 中 Rust 侧通过 `McpToolBridge::tool_descriptions()` 注入 system prompt 的逻辑，在 Sidecar 路径下不再需要（DeepAgents 通过 `mcp_bridge_tool` 直接调用），仅在 Rig 降级路径中保留
3. **新增自动标题 Command**：利用 Rig 生成会话标题（不经过 Sidecar，纯本地快速调用）

```rust
/// 使用 Rig 快速生成会话标题（非对话功能，不经过 Sidecar）
#[tauri::command]
pub async fn generate_session_title(
    state: State<'_, AppState>,
    session_id: String,
    first_message: String,
) -> Result<String, String> {
    // 使用 Rig 轻量调用生成标题
    // ...
}
```

#### 4.6.4 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/src/commands/chat.rs` | **修改** | 重构为双路径，标记 Rig 为降级 |
| `src-tauri/src/commands/session.rs` | **修改** | 新增 generate_session_title command |
| `src-tauri/src/lib.rs` | **修改** | 注册新 command |

---

## 7. Sprint 4：状态持久化与记忆系统

### 4.7 任务 4.7：LangGraph Checkpointer（2h）

#### 4.7.1 目标

配置 `langgraph-checkpoint-sqlite` 实现 Agent 对话状态持久化。Sidecar 重启后，通过 `thread_id`（= session_id）恢复对话上下文。

#### 4.7.2 实现方案

```python
# agent/app/dependencies.py
"""FastAPI 依赖注入 — Checkpointer 和 Store 单例。"""
from __future__ import annotations

from functools import lru_cache

from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver
from langgraph.store.memory import InMemoryStore

from .config import get_settings


@lru_cache
def get_checkpointer() -> AsyncSqliteSaver:
    """获取 SQLite Checkpointer 单例。"""
    settings = get_settings()
    settings.data_dir.mkdir(parents=True, exist_ok=True)
    db_path = str(settings.checkpointer_db_path)
    return AsyncSqliteSaver.from_conn_string(db_path)


@lru_cache
def get_store() -> InMemoryStore:
    """获取 LangGraph Store 单例（跨线程共享数据）。"""
    return InMemoryStore()
```

**生命周期管理 (`app/main.py`)：**

```python
@app.on_event("startup")
async def startup():
    checkpointer = get_checkpointer()
    await checkpointer.setup()  # 创建 schema

@app.on_event("shutdown")
async def shutdown():
    checkpointer = get_checkpointer()
    await checkpointer.conn.close()
```

#### 4.7.3 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `agent/app/dependencies.py` | **新增** | Checkpointer + Store 单例管理 |
| `agent/app/main.py` | **修改** | 添加 startup/shutdown 事件 |

---

### 4.8 任务 4.8：PowerMem 集成（3h）

#### 4.8.1 目标

初始化 PowerMem 记忆引擎，提供全局 `get_memory_engine()` 访问接口，被 `powermem_search` 和 `powermem_save` Tool 调用。

#### 4.8.2 实现方案

```python
# agent/app/memory.py
"""PowerMem 记忆引擎管理模块。"""
from __future__ import annotations

import logging
from functools import lru_cache
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from powermem import PowerMem

from .config import get_settings

logger = logging.getLogger(__name__)


@lru_cache
def get_memory_engine() -> "PowerMem | None":
    """获取 PowerMem 引擎单例（延迟初始化）。"""
    settings = get_settings()

    if not settings.powermem_enabled:
        logger.info("PowerMem is disabled via config.")
        return None

    try:
        from powermem import PowerMem

        settings.data_dir.mkdir(parents=True, exist_ok=True)
        engine = PowerMem(
            db_path=str(settings.powermem_db_path),
            embedding_model="text-embedding-3-small",
            openai_api_key=settings.openai_api_key or None,
        )
        logger.info("PowerMem engine initialized at %s", settings.powermem_db_path)
        return engine

    except ImportError:
        logger.warning("PowerMem package not installed. Memory features disabled.")
        return None
    except Exception as e:
        logger.error("Failed to initialize PowerMem: %s", e)
        return None
```

**环境变量传递：**

PowerMem 需要 Embedding API Key。方案：
1. Rust 启动 Sidecar 时将解密后的 API Key 通过环境变量传递
2. 或者 Python 侧从 `~/.misakax/config.yaml` 读取（已加密，需要对应解密逻辑）

推荐方案 1（更简洁）：

```rust
// sidecar.rs start_process() 中
let mut cmd = Command::new(&python_path);
cmd.env("MISAKA_OPENAI_API_KEY", &decrypted_openai_key);
cmd.env("MISAKA_ANTHROPIC_API_KEY", &decrypted_anthropic_key);
```

#### 4.8.3 记忆管理 HTTP 端点

```python
# agent/app/routers/memory.py
"""记忆管理端点 — 供前端查看/搜索/删除。"""
from fastapi import APIRouter, HTTPException, Query

from app.memory import get_memory_engine

router = APIRouter(prefix="/memory", tags=["memory"])


@router.get("/search")
async def search_memories(
    query: str = Query(..., min_length=1),
    limit: int = Query(default=20, ge=1, le=100),
):
    """搜索记忆库。"""
    engine = get_memory_engine()
    if engine is None:
        raise HTTPException(status_code=503, detail="Memory engine not available")

    results = engine.search(query, limit=limit)
    return {
        "results": [
            {
                "id": r.id,
                "content": r.content,
                "score": r.score,
                "created_at": str(r.created_at),
                "metadata": r.metadata,
            }
            for r in results
        ]
    }


@router.get("/list")
async def list_memories(
    offset: int = Query(default=0, ge=0),
    limit: int = Query(default=50, ge=1, le=200),
):
    """分页列出所有记忆。"""
    engine = get_memory_engine()
    if engine is None:
        raise HTTPException(status_code=503, detail="Memory engine not available")

    memories = engine.list(offset=offset, limit=limit)
    return {
        "memories": [
            {
                "id": m.id,
                "content": m.content,
                "created_at": str(m.created_at),
                "importance": m.metadata.get("importance", "normal"),
            }
            for m in memories
        ],
        "total": engine.count(),
    }


@router.delete("/{memory_id}")
async def delete_memory(memory_id: str):
    """删除指定记忆。"""
    engine = get_memory_engine()
    if engine is None:
        raise HTTPException(status_code=503, detail="Memory engine not available")

    engine.delete(memory_id)
    return {"deleted": True, "id": memory_id}
```

#### 4.8.4 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `agent/app/memory.py` | **新增** | PowerMem 引擎初始化和管理 |
| `agent/app/routers/memory.py` | **新增** | 记忆管理 HTTP 端点 |
| `agent/app/main.py` | **修改** | 注册 memory router |
| `src-tauri/src/sidecar.rs` | **修改** | 启动时传递 API Key 环境变量 |

---

## 8. Sprint 5：记忆 UI + HITL + 打包 + 集成测试

### 4.9 任务 4.9：记忆管理 UI（4h）

#### 4.9.1 目标

在前端新增记忆管理页面，支持查看、搜索、删除长期记忆。

#### 4.9.2 UI 设计

```
┌─────────────────────────────────────────────────────┐
│ 记忆管理                                    [搜索框] │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌── 记忆条目 ────────────────────────────────────┐ │
│  │ 🧠 用户偏好使用 TypeScript 和 React...          │ │
│  │ 创建时间: 2026-05-10  重要性: high   [🗑️ 删除] │ │
│  └──────────────────────────────────────────────────┘ │
│                                                      │
│  ┌── 记忆条目 ────────────────────────────────────┐ │
│  │ 🧠 项目使用 Tauri 2.x + Rust 后端...           │ │
│  │ 创建时间: 2026-05-09  重要性: normal  [🗑️ 删除] │ │
│  └──────────────────────────────────────────────────┘ │
│                                                      │
│  [加载更多]                                          │
└─────────────────────────────────────────────────────┘
```

#### 4.9.3 实现

- 新增 `src/pages/MemoryPage.tsx`
- 通过 Sidecar HTTP 端点 `/memory/search` 和 `/memory/list` 获取数据
- 使用 shadcn/ui Card + Input + Button 组件
- 支持搜索防抖（300ms）
- 删除操作需确认弹窗

#### 4.9.4 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/pages/MemoryPage.tsx` | **新增** | 记忆管理页面 |
| `src/lib/ipc/memory.ts` | **新增** | 记忆 API 封装（HTTP 调用 Sidecar） |
| `src/components/memory/MemoryCard.tsx` | **新增** | 单条记忆展示组件 |
| `src/components/memory/MemorySearchBar.tsx` | **新增** | 搜索栏组件 |
| 侧边栏导航 | **修改** | 新增"记忆"导航项 |
| i18n 文件 | **修改** | 新增 memory 相关翻译 |

---

### 4.10 任务 4.10：Human-in-the-loop 配置（3h）

#### 4.10.1 目标

配置 DeepAgents `interrupt_on` 参数，在危险操作（文件写入/删除/Shell 执行）时中断执行，等待用户审批。复用 Phase 3 已实现的 Tool Call 权限审批 UI。

#### 4.10.2 实现方案

**Python 侧 HITL 配置：**

```python
# agent.py 中 create_deep_agent 的 interrupt_on 参数
interrupt_on={
    "file_write": True,    # 写文件前中断
    "file_delete": True,   # 删文件前中断
    "shell_execute": True, # 执行命令前中断
}
```

**HITL 事件流：**

```
Agent → 决定写文件 → 触发 interrupt
  → SSE 发送 event: "interrupt"
    → Rust 转发 Tauri Event "agent_interrupt"
      → 前端弹出审批对话框
        → 用户点击 Approve/Deny
          → 前端调用 Rust Command "resume_agent" / "reject_agent"
            → Rust 调用 Sidecar POST /agent/resume 或 /agent/reject
              → Agent 继续/取消操作
```

**Sidecar 端点：**

```python
# agent/app/routers/agent.py 新增
@router.post("/resume")
async def resume_agent(session_id: str, approved: bool):
    """恢复被中断的 Agent 执行。"""
    # 通过 Checkpointer 获取中断状态
    # 调用 agent.ainvoke(None, config) 恢复
    ...
```

#### 4.10.3 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `agent/app/routers/agent.py` | **修改** | 新增 /agent/resume 端点 |
| `src-tauri/src/commands/chat.rs` | **修改** | 新增 resume_agent / reject_agent Command |
| 前端组件 | **修改** | 复用 Phase 3 ToolApprovalDialog，适配 HITL 事件 |

---

### 4.11 任务 4.11：Nuitka 打包脚本完善（4h）

#### 4.11.1 目标

在 Phase 3 的 Nuitka 打包基础上，确保 `deepagents` + `powermem` + `langgraph` 依赖正确打包。

#### 4.11.2 新增打包配置

```python
# agent/build_nuitka.py 更新
ADDITIONAL_INCLUDES = [
    "--include-package=deepagents",
    "--include-package=langgraph",
    "--include-package=langchain_core",
    "--include-package=langchain_anthropic",
    "--include-package=langchain_openai",
    "--include-package=powermem",
    "--include-package=sqlite3",
    "--include-module=langgraph.checkpoint.sqlite",
]
```

#### 4.11.3 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `agent/build_nuitka.py` | **修改** | 新增 DeepAgents/PowerMem 打包参数 |
| `agent/requirements.txt` | **修改** | 更新依赖版本锁定（用于打包环境） |

---

### 4.12 任务 4.12：端到端集成测试（4h）

#### 4.12.1 测试场景矩阵

| # | 场景 | 覆盖组件 | 预期结果 |
|---|------|---------|---------|
| T1 | 简单对话 | Sidecar → Agent → LLM → Stream | 流式回复正常 |
| T2 | 记忆存储 | Agent → powermem_save Tool → PowerMem | 记忆成功存储 |
| T3 | 记忆检索 | Agent → powermem_search Tool → PowerMem | 召回相关记忆 |
| T4 | MCP 工具调用 | Agent → mcp_bridge Tool → Rust MCP → Server | 工具执行结果返回 |
| T5 | 文件操作 + HITL | Agent → FilesystemMiddleware → interrupt → 审批 | 审批后文件写入 |
| T6 | SubAgent 委派 | Agent → task → researcher SubAgent → 结果汇总 | 子代理结果返回 |
| T7 | 降级开关 | use_sidecar=false → Rig 直调 | Rig 对话正常 |
| T8 | Sidecar 重启恢复 | 重启 Sidecar → 同一 session_id 继续对话 | 上下文保持 |
| T9 | 长对话摘要 | 50+ 轮对话 → SummarizationMiddleware | 自动压缩上下文 |
| T10 | 多会话隔离 | 会话 A 和 B 各自独立上下文 | 互不干扰 |
| T11 | 自动标题（Rig） | 新会话首条消息 → Rig 生成标题 | 标题生成正常 |
| T12 | Nuitka 打包产物运行 | 打包 exe → 独立运行 → health check | 打包成功 |

#### 4.12.2 测试方式

- T1-T6: 手动测试 + 日志验证
- T7-T8: 手动配置切换测试
- T9: 自动化对话脚本（50 轮）
- T10-T11: 手动测试
- T12: CI 脚本验证

---

## 9. 新增依赖清单

### 9.1 Python 依赖

| 依赖 | 版本 | 说明 | optional-group |
|------|------|------|---------------|
| `deepagents` | >=0.5.4 | DeepAgents Harness | agent |
| `langgraph` | >=0.3 | Agent 编排引擎 | agent |
| `langchain-anthropic` | >=0.3 | Anthropic Provider | agent |
| `langchain-openai` | >=0.3 | OpenAI Provider | agent |
| `langgraph-checkpoint-sqlite` | >=2.0 | SQLite Checkpointer | agent |
| `powermem` | >=1.1 | 智能记忆引擎 | memory |

### 9.2 Rust 依赖

| 依赖 | 版本 | 说明 |
|------|------|------|
| `futures-util` | >=0.3 | SSE 流解析（bytes_stream） |
| `axum` 或 `warp` | - | MCP HTTP Bridge（本地 HTTP Server，方案待定） |

### 9.3 前端依赖

无新增。所有 UI 组件使用已有的 shadcn/ui + Tailwind 实现。

---

## 10. 文件创建/修改清单

### 10.1 Python Sidecar — 新增文件

| 文件 | 职责 |
|------|------|
| `agent/app/agent.py` | DeepAgents 核心组装 |
| `agent/app/tools.py` | 自定义 Tool（PowerMem + MCP Bridge） |
| `agent/app/prompts.py` | System Prompt + SubAgent Prompt |
| `agent/app/memory.py` | PowerMem 引擎管理 |
| `agent/app/dependencies.py` | Checkpointer + Store 依赖注入 |
| `agent/app/utils.py` | 工作目录验证等工具函数 |
| `agent/app/routers/memory.py` | 记忆管理 HTTP 端点 |

### 10.2 Python Sidecar — 修改文件

| 文件 | 变更内容 |
|------|---------|
| `agent/app/routers/agent.py` | 从 501 占位 → 完整对话实现 |
| `agent/app/config.py` | 扩展 Settings（model/dirs/powermem/interrupt） |
| `agent/app/main.py` | 注册 memory router + startup/shutdown 事件 |
| `agent/pyproject.toml` | deepagents 依赖 |

### 10.3 Rust — 修改文件

| 文件 | 变更内容 |
|------|---------|
| `src-tauri/src/commands/chat.rs` | 双路径（Sidecar/Rig）+ SSE 解析 |
| `src-tauri/src/config.rs` | `use_sidecar` 配置项 |
| `src-tauri/src/sidecar.rs` | API Key 环境变量传递 |
| `src-tauri/src/services/mod.rs` | 注册 mcp_http_bridge |
| `src-tauri/src/commands/session.rs` | generate_session_title |
| `src-tauri/src/lib.rs` | 注册新 Command |

### 10.4 Rust — 新增文件

| 文件 | 职责 |
|------|------|
| `src-tauri/src/services/mcp_http_bridge.rs` | 本地 HTTP 暴露 MCP 工具调用 |

### 10.5 前端 — 新增文件

| 文件 | 职责 |
|------|------|
| `src/pages/MemoryPage.tsx` | 记忆管理页面 |
| `src/lib/ipc/memory.ts` | 记忆 API 封装 |
| `src/components/memory/MemoryCard.tsx` | 记忆卡片组件 |
| `src/components/memory/MemorySearchBar.tsx` | 搜索栏 |

### 10.6 前端 — 修改文件

| 文件 | 变更内容 |
|------|---------|
| 侧边栏导航组件 | 新增"记忆"入口 |
| i18n `zh-CN/*.json` | memory 命名空间 |
| i18n `en/*.json` | memory 命名空间 |

---

## 11. Phase 4 完整验证清单

| # | 验证项 | 操作 | 预期结果 |
|---|--------|------|---------|
| V1 | Sidecar 健康检查 | `curl http://127.0.0.1:9527/health` | `agent_ready: true` |
| V2 | Info 端点 | `curl http://127.0.0.1:9527/info` | `langgraph_available: true, powermem_available: true` |
| V3 | 同步对话 | `POST /agent/chat` 含简单问题 | 返回 ChatResponse |
| V4 | 流式对话 | `POST /agent/stream` | SSE 逐 token 返回 |
| V5 | 前端流式渲染 | UI 中发送消息 | 流式文字逐字显示 |
| V6 | 工具调用展示 | 触发 MCP 工具 | ToolCallBlock 正确展示 |
| V7 | 记忆存储 | 告诉 Agent 一个偏好 | powermem_save 被调用 |
| V8 | 记忆检索 | 在新会话询问之前的偏好 | powermem_search 返回结果 |
| V9 | 记忆管理 UI | 打开记忆页面 | 列表正确展示 |
| V10 | 记忆搜索 | 搜索关键词 | 相关记忆高亮 |
| V11 | 记忆删除 | 点击删除 → 确认 | 记忆消失 |
| V12 | FilesystemMiddleware | Agent 读取工作目录文件 | 返回文件内容 |
| V13 | HITL 中断 | Agent 尝试写文件 | 弹出审批对话框 |
| V14 | HITL 恢复 | 点击 Approve | 文件成功写入 |
| V15 | HITL 拒绝 | 点击 Deny | Agent 收到拒绝通知 |
| V16 | SubAgent 研究 | 发送研究类问题 | researcher 被调用 |
| V17 | SubAgent 编码 | 发送编码请求 | coder 被调用 |
| V18 | 降级开关 | 设置 use_sidecar=false | Rig 直调正常 |
| V19 | 自动标题 | 新会话首条消息 | 标题自动生成 |
| V20 | Checkpointer 持久化 | 重启 Sidecar → 同 session 继续 | 上下文连续 |
| V21 | 多会话隔离 | 两个会话分别对话 | 互不影响 |
| V22 | 摘要压缩 | 长对话（50+ 轮） | 上下文被压缩 |
| V23 | Todo 规划 | 复杂任务 | Agent 使用 write_todos |
| V24 | MCP Bridge | Agent 通过 mcp_bridge 调用工具 | 工具结果返回 |
| V25 | Nuitka 打包 | 执行 build_nuitka.py | 产物可运行 |
| V26 | 错误恢复 | Sidecar 崩溃 → 自动重启 → 重试对话 | 恢复正常 |
| V27 | Token 统计 | 对话后检查 session 统计 | 数据正确 |
| V28 | 停止生成 | 对话中点击停止 | 流中断，内容保存 |

---

## 12. 详细 TODO 列表

### Sprint 1：DeepAgents 核心实现（第 11 周前半）

#### 4.1 DeepAgents Agent 组装（6h）

```
TODO-4.1.1 [0.5h] [无依赖]
    修改 agent/pyproject.toml
    - [project.optional-dependencies].agent 新增 "deepagents>=0.5.4"
    - 确认所有 LangChain Provider 依赖版本对齐
    - 运行 pip install -e ".[agent,memory]" 验证安装

TODO-4.1.2 [1.0h] [依赖 4.1.1]
    创建 agent/app/prompts.py
    - SYSTEM_PROMPT 常量（MisakaX Agent 身份、能力描述、行为规范）
    - RESEARCHER_PROMPT / CODER_PROMPT / ANALYST_PROMPT（SubAgent 专用）
    - 模板格式：plain str，不使用 Jinja（保持简洁）

TODO-4.1.3 [1.5h] [依赖 4.1.1]
    创建 agent/app/agent.py
    - build_agent(session_id, working_dir, checkpointer, store) → CompiledStateGraph
    - _build_subagents(settings) → list[SubAgent]
    - 导入 create_deep_agent, SubAgent, CompositeBackend, FilesystemBackend, StateBackend
    - Middleware 配置：skills + memory + interrupt_on
    - 暂时 tools=[]（Sprint 2 实现后补全）

TODO-4.1.4 [0.5h] [依赖 4.1.1]
    创建 agent/app/dependencies.py
    - get_checkpointer() → AsyncSqliteSaver（lru_cache 单例）
    - get_store() → InMemoryStore（lru_cache 单例）
    - 确保 data_dir 目录自动创建

TODO-4.1.5 [1.5h] [依赖 4.1.3, 4.1.4]
    重写 agent/app/routers/agent.py
    - POST /agent/chat：同步调用 agent.ainvoke → 返回 ChatResponse
    - POST /agent/stream：SSE 流式调用 agent.astream_events → 逐事件推送
    - _stream_agent() 异步生成器：处理 on_chat_model_stream / on_tool_start / on_tool_end / done / error
    - _format_sse_event()：LangGraph 事件 → SSE 格式转换
    - _sse() 辅助：格式化 "event: {event}\ndata: {json}\n\n"
    - 异常处理：500 HTTPException + error SSE 事件

TODO-4.1.6 [0.5h] [依赖 4.1.5]
    修改 agent/app/config.py
    - Settings 新增：agent_model, temperature, max_tokens
    - Settings 新增：skills_dir, memories_dir, data_dir (Path)
    - Settings 新增：powermem_enabled, powermem_db_path, checkpointer_db_path
    - Settings 新增：interrupt_config (dict), mcp_bridge_url
    - Settings 新增：anthropic_api_key, openai_api_key

TODO-4.1.7 [0.5h] [依赖 4.1.5]
    修改 agent/app/main.py
    - @app.on_event("startup")：await get_checkpointer().setup()
    - @app.on_event("shutdown")：关闭 checkpointer 连接
    - 注册 memory_router（Sprint 4 创建后）
    - 更新 /health 返回 agent_ready: True（当 agent 模块可导入时）
```

#### 4.2 FilesystemMiddleware 工作目录绑定（3h）

```
TODO-4.2.1 [1.0h] [无依赖]
    创建 agent/app/utils.py
    - validate_working_dir(path: str | None) → Path | None
    - FORBIDDEN_DIRS 集合（系统关键目录黑名单）
    - 验证逻辑：exists + is_dir + not in forbidden
    - 异常：ValueError（明确错误信息）

TODO-4.2.2 [1.0h] [依赖 4.1.3, 4.2.1]
    修改 agent/app/agent.py — build_agent
    - 导入 validate_working_dir
    - working_dir 参数通过验证后构建 FilesystemBackend
    - routes["/workspace/"] = FilesystemBackend(root_dir=validated_path)
    - 验证失败时 log warning 但不阻断 Agent 创建（无工作目录也能对话）

TODO-4.2.3 [1.0h] [依赖 4.2.2]
    验证 FilesystemMiddleware 功能
    - 启动 Sidecar → 带 working_dir 调用 /agent/chat
    - 请求 Agent 列出目录文件 → 验证 ls 返回正确
    - 请求 Agent 读取文件 → 验证 read_file 返回内容
    - 请求 Agent 写文件 → 验证 interrupt 触发（HITL）
    - 无 working_dir 时 → 验证 Filesystem 工具不可用但对话正常
```

### Sprint 2：自定义 Tool + SubAgent（第 11 周后半）

#### 4.3 自定义 Tool 开发（6h）

```
TODO-4.3.1 [2.0h] [依赖 4.1.3]
    创建 agent/app/tools.py
    - @tool powermem_search(query, limit) → list[dict]
      - 延迟导入 get_memory_engine（避免启动时强依赖）
      - None 引擎时返回友好消息（不抛异常）
    - @tool powermem_save(content, importance) → str
      - 同样延迟导入 + None 保护
      - smart_process=True 启用智能处理
    - @tool async mcp_bridge(server_id, tool_name, arguments) → str
      - httpx.AsyncClient 调用 Rust MCP HTTP Bridge
      - 超时 30s + 错误处理
    - get_tools() → list：返回全部 Tool
    - get_research_tools() → list：researcher 专用子集

TODO-4.3.2 [2.0h] [无依赖，与 4.3.1 可并行]
    创建 src-tauri/src/services/mcp_http_bridge.rs
    - 启动本地 HTTP Server（127.0.0.1:9528，仅本地访问）
    - POST /mcp/call_tool { server_id, tool_name, arguments }
      - 从 AppState 获取 McpManager
      - 调用 McpManager::call_tool
      - 返回结果 JSON
    - GET /mcp/list_tools
      - 返回所有已连接 Server 的工具列表
    - 使用 axum（轻量级）或 warp

TODO-4.3.3 [1.0h] [依赖 4.3.2]
    修改 src-tauri/src/services/mod.rs
    - 新增 pub mod mcp_http_bridge
    修改 src-tauri/src/lib.rs（或 main.rs）
    - 在 Tauri setup hook 中 spawn MCP HTTP Bridge Server
    - 绑定到 AppState.config.mcp_bridge_port (默认 9528)

TODO-4.3.4 [1.0h] [依赖 4.3.1, 4.1.3]
    修改 agent/app/agent.py
    - build_agent 中 tools=get_tools()（从空列表替换）
    - _build_subagents 中 researcher.tools=get_research_tools()
    - 验证 Tool 注入到 Agent 后可被正确调用
```

#### 4.4 SubAgent 配置（3h）

```
TODO-4.4.1 [1.0h] [依赖 4.1.2]
    完善 agent/app/prompts.py — SubAgent Prompt
    - RESEARCHER_PROMPT：研究能力、信息综合、引用格式规范
    - CODER_PROMPT：编码标准、文件操作规范、测试建议
    - ANALYST_PROMPT：问题分解方法、结构化输出格式
    - 每个 prompt 控制在 500 token 以内

TODO-4.4.2 [1.0h] [依赖 4.4.1, 4.1.3]
    修改 agent/app/agent.py — _build_subagents
    - researcher：导入 RESEARCHER_PROMPT，绑定 get_research_tools()
    - coder：导入 CODER_PROMPT，无额外 Tool（使用 FilesystemMiddleware 内建工具）
    - analyst：导入 ANALYST_PROMPT，绑定 powermem_search（分析时可检索历史数据）

TODO-4.4.3 [1.0h] [依赖 4.4.2]
    验证 SubAgent 功能
    - 发送"请帮我研究 X" → 确认 researcher 被调用
    - 发送"请帮我写一个 Y" → 确认 coder 被调用
    - 检查 SubAgent 上下文隔离（子代理不看到完整父对话历史）
    - 检查 SubAgent 结果正确汇总返回父 Agent
```

### Sprint 3：Rust 侧对话迁移（第 12 周前半）

#### 4.5 Rust chat Command 改写（4h）

```
TODO-4.5.1 [0.5h] [无依赖]
    修改 src-tauri/src/config.rs
    - AppConfig 新增 use_sidecar: bool (default: true)
    - 对应 config.yaml 字段：use_sidecar: true
    - 确保 Config 反序列化正确

TODO-4.5.2 [2.0h] [依赖 4.5.1, 4.1.5]
    重写 src-tauri/src/commands/chat.rs — send_message
    - 拆分为 send_via_sidecar() + send_via_rig()
    - send_message 开头读取 use_sidecar 决定路径
    - send_via_sidecar：
      - 加载 session（获取 working_dir_local）
      - 构建 AgentChatRequest（messages + config + session_id + working_dir）
      - SidecarClient::stream() → 获取 reqwest::Response
      - parse_sidecar_sse() → 逐 SSE 事件转发 Tauri Event
      - 最终 update_assistant_message
    - send_via_rig：现有逻辑原封不动抽取为独立函数
    - 同时修改 regenerate_message 适配双路径

TODO-4.5.3 [1.0h] [依赖 4.5.2]
    实现 parse_sidecar_sse 函数
    - 使用 futures_util::StreamExt 读取 bytes_stream
    - 逐行缓冲解析 SSE（event: + data: + 空行分隔）
    - token 事件 → app.emit("stream_token", ...)
    - tool_start/tool_end 事件 → app.emit("tool_call_start"/"tool_call_end", ...)
    - done 事件 → 结束循环
    - error 事件 → 返回 Err
    - 返回 StreamResult { content, thinking, usage, was_aborted }

TODO-4.5.4 [0.5h] [依赖 4.5.2]
    修改 Cargo.toml
    - 确保 futures-util 依赖存在（SSE 流解析需要）
    修改 src-tauri/src/lib.rs
    - 确保新 command 注册（如有新增）
```

#### 4.6 Rig 对话功能退役（3h）

```
TODO-4.6.1 [1.0h] [依赖 4.5.2]
    重构 src-tauri/src/commands/chat.rs
    - send_via_rig() 明确标注为"降级路径"（注释说明）
    - MCP Tool 描述注入逻辑仅在 Rig 路径生效
    - Sidecar 路径不注入 MCP（Agent 通过 mcp_bridge_tool 直接调用）

TODO-4.6.2 [1.0h] [依赖 4.6.1]
    新增 generate_session_title Command
    - 使用 Rig 快速调用（短 prompt + 低 max_tokens）
    - 不经过 Sidecar（标题生成是轻量非对话操作）
    - prompt："Generate a concise title (5-10 words) for: {first_message}"
    - 注册到 lib.rs invoke_handler

TODO-4.6.3 [1.0h] [依赖 4.6.1, 4.6.2]
    验证 Rig 非对话功能完整
    - 测试 generate_session_title 正常工作
    - 测试降级开关（use_sidecar=false）→ Rig 对话正常
    - 确认 services/llm/ 模块无破坏性变更
    - 确认 Embedding 接口仍可用（Phase 5 知识库需要）
```

### Sprint 4：状态持久化 + 记忆系统（第 12 周后半）

#### 4.7 LangGraph Checkpointer（2h）

```
TODO-4.7.1 [1.0h] [依赖 4.1.4]
    完善 agent/app/dependencies.py
    - get_checkpointer()：创建 AsyncSqliteSaver，WAL 模式
    - 确保 ~/.misakax/data/ 目录存在
    - 连接字符串使用 settings.checkpointer_db_path

TODO-4.7.2 [1.0h] [依赖 4.7.1]
    修改 agent/app/main.py — startup/shutdown
    - startup：await get_checkpointer().setup()（创建 schema）
    - shutdown：close checkpointer connection
    - 验证：重启后同 thread_id 恢复对话状态
    - 验证：不同 thread_id 互相隔离
```

#### 4.8 PowerMem 集成（3h）

```
TODO-4.8.1 [1.0h] [无依赖]
    创建 agent/app/memory.py
    - get_memory_engine() → PowerMem | None（lru_cache）
    - 延迟初始化：首次调用时创建
    - ImportError 保护（powermem 未安装时返回 None）
    - Exception 保护（初始化失败时 log + 返回 None）
    - 配置：db_path 从 settings 读取

TODO-4.8.2 [1.0h] [依赖 4.8.1]
    创建 agent/app/routers/memory.py
    - GET /memory/search?query=&limit= → 搜索结果
    - GET /memory/list?offset=&limit= → 分页列表
    - DELETE /memory/{memory_id} → 删除
    - 503 返回：引擎不可用时
    注册到 agent/app/main.py

TODO-4.8.3 [1.0h] [依赖 4.8.1]
    修改 src-tauri/src/sidecar.rs — start_process
    - 启动进程时注入环境变量：
      - MISAKA_OPENAI_API_KEY（从加密存储解密）
      - MISAKA_ANTHROPIC_API_KEY（同上）
    - 从 AppConfig + RouterConfigRepo 读取并解密 key
    - 安全保障：仅传递给子进程，不 log 明文
```

### Sprint 5：收尾与集成测试（第 13 周）

#### 4.9 记忆管理 UI（4h）

```
TODO-4.9.1 [1.0h] [依赖 4.8.2]
    创建 src/lib/ipc/memory.ts
    - searchMemories(query, limit) → fetch Sidecar /memory/search
    - listMemories(offset, limit) → fetch Sidecar /memory/list
    - deleteMemory(id) → fetch Sidecar /memory/{id} DELETE
    - 通过 Sidecar HTTP 端点而非 Tauri IPC（记忆在 Python 侧管理）

TODO-4.9.2 [1.5h] [依赖 4.9.1]
    创建 src/pages/MemoryPage.tsx
    - 布局：标题栏 + 搜索栏 + 记忆卡片列表 + 加载更多
    - 状态管理：search query + memories list + loading + total count
    - 搜索防抖 300ms（useDebounce）
    - 分页加载（每次 50 条）
    - 空状态提示（无记忆时引导说明）

TODO-4.9.3 [1.0h] [依赖 4.9.2]
    创建 src/components/memory/MemoryCard.tsx
    - 展示：content（截断 200 字符）+ created_at + importance badge
    - 操作：展开全文 + 复制 + 删除
    - 删除确认弹窗（AlertDialog）
    创建 src/components/memory/MemorySearchBar.tsx
    - Input + 搜索图标 + 清除按钮
    - controlled input，值由父组件管理

TODO-4.9.4 [0.5h] [依赖 4.9.2]
    修改侧边栏导航
    - 新增"记忆"图标入口（Brain icon）
    - 路由连接到 MemoryPage
    修改 i18n 文件
    - zh-CN: memory.title / memory.search / memory.empty / memory.delete / ...
    - en: 对应英文
```

#### 4.10 Human-in-the-loop 配置（3h）

```
TODO-4.10.1 [1.5h] [依赖 4.1.5]
    修改 agent/app/routers/agent.py
    - 新增 POST /agent/resume { session_id, thread_id, approved }
    - approved=true：恢复中断（agent.ainvoke(None, config)）
    - approved=false：取消操作并通知 Agent
    - _stream_agent 中处理 interrupt 事件 → 发送 SSE event: "interrupt"

TODO-4.10.2 [1.0h] [依赖 4.10.1]
    修改 src-tauri/src/commands/chat.rs
    - 新增 resume_agent Command
    - 新增 reject_agent Command
    - 调用 SidecarClient POST /agent/resume
    - parse_sidecar_sse 中处理 "interrupt" 事件：
      - 发送 Tauri Event "agent_interrupt" 给前端
      - 等待前端响应（通过 oneshot channel 或轮询）

TODO-4.10.3 [0.5h] [依赖 4.10.2]
    前端适配
    - 复用 Phase 3 ToolApprovalDialog 组件
    - 监听 "agent_interrupt" 事件 → 弹出审批对话框
    - Approve → invoke resume_agent
    - Deny → invoke reject_agent
```

#### 4.11 Nuitka 打包完善（4h）

```
TODO-4.11.1 [2.0h] [依赖 4.8.1]
    修改 agent/build_nuitka.py
    - ADDITIONAL_INCLUDES 新增：
      - --include-package=deepagents
      - --include-package=langgraph
      - --include-package=langchain_core
      - --include-package=langchain_anthropic
      - --include-package=langchain_openai
      - --include-package=powermem
      - --include-module=langgraph.checkpoint.sqlite
    - 排除不需要的包（减小体积）
    - 更新 requirements.txt（锁定版本用于打包环境）

TODO-4.11.2 [2.0h] [依赖 4.11.1]
    执行打包并验证
    - pip install 所有依赖（agent + memory group）
    - python build_nuitka.py
    - 验证产物：运行 exe → curl /health → agent_ready=true
    - 验证：curl /agent/chat（简单问题）→ 返回响应
    - 记录：体积 / 启动时间 / 内存占用
    - 修复打包问题（如有）
```

#### 4.12 端到端集成测试（4h）

```
TODO-4.12.1 [2.0h] [依赖所有前序]
    手动端到端测试 — 核心链路
    - T1: UI 发送消息 → DeepAgents → 流式回复（token by token）
    - T2: 告诉 Agent "记住我喜欢 Python" → powermem_save 被调用 → 确认存储
    - T3: 新会话问 "我喜欢什么语言" → powermem_search 召回 → 回答 Python
    - T4: 连接 MCP Server → 问需要工具的问题 → mcp_bridge 调用 → 结果正确
    - T5: 绑定工作目录 → 问 Agent 列出文件 → FilesystemMiddleware 工作正常
    - T6: 设置 use_sidecar=false → 对话 → 验证 Rig 降级正常
    - T7: 新会话首条消息 → 自动标题生成（Rig）

TODO-4.12.2 [1.0h] [依赖 4.12.1]
    手动端到端测试 — 高级功能
    - T8: HITL → 请求 Agent 写文件 → 审批弹窗 → Approve → 文件写入
    - T9: SubAgent → 复杂研究问题 → researcher 被调用 → 结果汇总
    - T10: 重启 Sidecar → 同 session 继续对话 → 上下文连续（Checkpointer）
    - T11: 50+ 轮对话 → 观察 SummarizationMiddleware 压缩行为
    - T12: 两个会话分别对话 → 验证互不影响

TODO-4.12.3 [1.0h] [依赖 4.12.2]
    问题修复 + 文档更新
    - 修复测试中发现的问题
    - 更新 CLAUDE.md（如有架构/命令变化）
    - 更新 /info 端点返回准确信息
    - 确认验证清单 V1-V28 全部通过
```

---

## TODO 统计

| Sprint | TODO 数量 | 预估时间 |
|--------|----------|---------|
| Sprint 1: DeepAgents 核心 (4.1, 4.2) | 10 个 | ~9h |
| Sprint 2: Tool + SubAgent (4.3, 4.4) | 7 个 | ~9h |
| Sprint 3: Rust 迁移 (4.5, 4.6) | 7 个 | ~7h |
| Sprint 4: 持久化 + 记忆 (4.7, 4.8) | 5 个 | ~5h |
| Sprint 5: UI + HITL + 打包 + 测试 (4.9-4.12) | 10 个 | ~15h |
| **合计** | **~39 个** | **~45h** |

---

## 附录 A：关键风险与应对

| 风险 | 概率 | 应对 |
|------|------|------|
| DeepAgents v0.5 API 与文档不一致 | 中 | 固定版本 0.5.4；实际 API 以源码为准 |
| PowerMem 初始化慢影响 Sidecar 启动 | 低 | 延迟初始化（首次 Tool 调用时） |
| SSE 流解析丢失事件 | 中 | 完善缓冲区逻辑 + 重试机制 |
| MCP HTTP Bridge 端口冲突 | 低 | 配置化端口 + 自动寻找可用端口 |
| Nuitka 打包后 deepagents 运行异常 | 中 | 逐模块排查 + 必要时回退 PyInstaller |
| HITL 恢复状态不一致 | 中 | Checkpointer 保证中断/恢复原子性 |

---

## 附录 B：Phase 4 → Phase 5 衔接点

Phase 4 完成后，以下模块可直接被 Phase 5（Skills + 知识库）复用：

| Phase 4 产出 | Phase 5 复用方式 |
|-------------|----------------|
| `DeepAgents SkillsMiddleware` | Phase 5 Rust 扫描 SKILL.md → 同步至 `agent/skills/` → SkillsMiddleware 自动加载 |
| `Rig Embedding` | Phase 5 知识库向量化（Rig 非对话功能） |
| `PowerMem` | Phase 5 知识库 RAG 可复用记忆引擎的向量搜索能力 |
| `MCP Bridge` | Phase 5 Skills 中的 MCP 工具由 Agent 通过 mcp_bridge 调用 |
| `FilesystemMiddleware` | Phase 5 Skills 文件操作 |

---

> **文档结束**
>
> 本文档是 Phase 4 的详细执行指南，覆盖了 DeepAgents 核心组装、自定义 Tool 开发、Rust 侧对话迁移、
> 状态持久化与记忆系统、记忆管理 UI、HITL 配置和 Nuitka 打包完善。
>
> 所有 TODO 共 **~39 项**（Python Sidecar ~17 项 + Rust 后端 ~12 项 + React 前端 ~7 项 + 集成测试 3 项），
> 按 Sprint 顺序完成后即可交付 M3 里程碑：Agent 平台。
>
> **Provider 配置兼容提醒（2026-05-17）：**
> Provider Dialog 重构后，对话页可用模型语义改为“仅来自已启用的 `custom_models`”，并额外携带 `vendor`、
> `advanced_json`、`temperature`、`max_tokens` 等配置。当前 Python Sidecar 仍是 501 占位端点，Pydantic
> 模型 `extra="ignore"` 不会被这些字段破坏；Phase 4 实现 DeepAgents 端点时，`SidecarClient` 转发层需要继续
> 传递最终解析出的 `model_id`，并按需把高级配置映射到 `ChatConfig`，不要在 sidecar 内重新读取未启用模型。
>
> **代码库对齐说明：**
> 本文档基于 Phase 3 完成后的实际代码库状态编写（45 个 Rust 文件、~48+ TSX 组件、8 个 Python 文件、
> 27+ 个已注册 Tauri Command、Schema v5），所有路径、模块引用、模型字段均已与现有代码精确对齐。
>
> **核心架构不变式：**
> - 前端 UI 层零改动（所有流式事件协议在 Phase 3 已定义）
> - `SidecarClient` / `SidecarManager` / `AgentStreamEvent` 直接复用
> - Rig 保留为降级开关 + 非对话功能（Embedding/标题/摘要）
> - DeepAgents 成为所有用户对话的唯一入口
