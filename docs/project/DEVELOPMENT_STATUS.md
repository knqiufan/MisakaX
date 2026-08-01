# MisakaX 开发状态与续做指南

> **用途：** 记录代码库真实进度，标明「从哪里继续开发」。  
> **受众：** 维护者、协作者、AI 辅助开发。  
> **最后审阅 / Last reviewed:** 2026-08-01

---

## 1. 一句话结论

**当前处于 Phase 3 代码关门阶段（约 95%），M2 里程碑等待最终 UI/实机复验记录。**

MisakaX 已是可用的**桌面 LLM 对话客户端**（流式对话、工作目录、MCP 管理、Provider 配置）。Phase 3 的 Sidecar watchdog、对话内 MCP 工具闭环、导入刷新、Nuitka 打包和 F1-F3 自动化测试已落地；进入 Phase 4 前请先按 Phase 3 §6 补齐最终 UI/实机复验记录。

**续做入口：** [`docs/planning/PHASE_3_REMAINING_TODO.md`](../planning/PHASE_3_REMAINING_TODO.md)（Epic A–F 详细 TODO + V1–V30 记录表）

---

## 2. 阶段完成度（对齐总体规划）

总体规划见 [`MISAKAX_IMPLEMENTATION_PLAN - Opus4.6.md`](../planning/MISAKAX_IMPLEMENTATION_PLAN%20-%20Opus4.6.md)（Phase 0–6 + Buddy 并行线）。

| 阶段 | 主题 | 状态 | 完成度 | 详细计划 |
|------|------|------|--------|----------|
| **Phase 0** | 环境搭建与项目初始化 | ✅ 完成 | ~100% | [PHASE_0_DETAILED_PLAN.md](../planning/PHASE_0_DETAILED_PLAN.md) |
| **Phase 1** | UI 骨架、设置、主题、i18n、Provider | ✅ 完成 | ~95% | [PHASE_1_DETAILED_PLAN.md](../planning/PHASE_1_DETAILED_PLAN.md) |
| **Phase 2** | Rig 过渡对话、流式管线、工作目录 | ✅ 完成 | ~95% | [PHASE_2_DETAILED_PLAN.md](../planning/PHASE_2_DETAILED_PLAN.md) |
| **Phase 3** | Sidecar 预热、MCP、会话高级管理 | 🟡 **代码关门，待实机复验** | ~95% | [PHASE_3_REMAINING_TODO.md](../planning/PHASE_3_REMAINING_TODO.md) |
| **Phase 4** | DeepAgents 全对话迁移 + PowerMem | ⏸️ 待 Phase 3 | ~5% | [PHASE_4_DETAILED_PLAN.md](../planning/PHASE_4_DETAILED_PLAN.md) |
| **Phase 5** | Skills + 知识库 RAG | 🟡 Skills 已交付，RAG 待实施 | ~50% | 总体规划 §8 |
| **Phase 6** | 打磨、Dashboard、打包、发布 | ❌ 未开始 | ~0% | 总体规划 §9 |
| **Phase B** | Buddy 桌面伴侣 | ❌ 未开始 | ~0% | 总体规划 §10 |

**综合主线进度：约 55%–60%**（Phase 0–2 已交付；**Phase 3 代码关门，待最终实机复验**，完成记录后进入 Phase 4）。

### 里程碑对照

| 里程碑 | 目标 | 状态 |
|--------|------|------|
| **M1** 核心可用 | Rig 对话 + 工作目录 + 会话管理 | ✅ 已达成 |
| **M2** 基础设施就绪 | Sidecar 预热 + MCP 全链路 | 🟡 代码路径就绪，待 UI/实机复验记录 |
| **M3** Agent 平台 | DeepAgents 接管对话 | ⏸️ Phase 4（Phase 3 完成后） |
| **M4** 知识增强 | Skills + 知识库 | 🟡 Skills 已交付，知识库待实施 |
| **M5** 发布就绪 | 跨平台打包 + 打磨 | ❌ Phase 6 |

---

## 3. 已交付能力（现在就能用）

配置 Provider API Key 后，以下功能可在 `npm run tauri dev` 中验证：

### 3.1 对话与会话

- 多 Provider LLM 对话（OpenAI / Anthropic / Gemini / OpenAI 兼容），Rust **Rig** 直调
- 流式输出、停止生成、重新生成
- 思维链（thinking）折叠展示、Token 统计、图片附件
- 会话 CRUD、分组 / 置顶 / 归档、FTS5 全文搜索、导入 / 导出

**关键路径：**

| 层 | 入口文件 |
|----|----------|
| 前端 | `src/pages/ChatPage.tsx` → `src/components/chat/ChatView.tsx` |
| IPC | `src/lib/ipc/chat.ts`、`src/lib/ipc/sessions.ts` |
| Rust | `src-tauri/src/commands/chat.rs`、`src-tauri/src/commands/session.rs` |
| LLM | `src-tauri/src/services/llm/`（`backend.rs`、`streaming.rs`、providers） |

### 3.2 工作目录与文件编辑

- 新建会话时选择工作目录；会话绑定 `working_directory`
- 工作区资源管理器、Monaco 编辑器、文件树、多 Tab 编辑
- 独立 `WorkspacePanel` 管理 open/mode/size/session generation，Explorer tabs 保持分离；宽窗 70/30 resize、窄窗 overlay、Explorer 快捷键和旧 open 偏好迁移已完成
- W3 已交付 owner-bound Rust PTY、窄 IPC、背压、事件顺序边界和 Windows Job Object / Unix process-group 回收；W4 已交付本地 xterm UI、独立 terminal store、严格 seq/generation 生命周期、resize/clipboard/workspace 选择与可访问错误态。capability/CSP 收口尚待 W5，入口继续受默认关闭的 feature flag 隔离
- Rust 文件读写命令（`fs_explorer`、`workspace`）
- Composer footer 只读显示 Git 分支、detached short SHA 或“本地项目”；支持 worktree/submodule、可信 Git CLI 退化、generation/cache 与 HEAD/ref 刷新，不提供 Git 写操作

**关键路径：** `src/components/chat/workspace/`、`src/components/chat/composer/WorkspaceContextBadge.tsx`、`src/lib/ipc/terminal.ts`、`src-tauri/src/commands/{workspace,terminal}.rs`、`src-tauri/src/services/{workspace,terminal}/`

### 3.3 设置与 Provider

- 设置页六 Tab：通用 / 模型 / MCP / Skills / 外观 / 关于
- Provider CRUD、API Key **AES-GCM 加密**、自定义模型、连接测试、拉取模型列表
- 主题（明 / 暗 / 跟随系统）、中 / 英文 i18n

**关键路径：** `src/pages/settings/`、`src-tauri/src/commands/router_configs.rs`

### 3.4 MCP 工具调用

- rmcp Client：stdio / HTTP transport
- MCP Server 配置（Settings + DB + `~/.misakax/mcp.json`）
- 工具列表、调用、权限审批（ask / approve / deny / always allow）
- 对话内 Tool Call 展示与 `ToolApprovalDialog`
- 对话内 MCP 工具循环：解析 LLM tool JSON → 审批 → 执行/拒绝 → stream event → `tool_calls` 持久化

**关键路径：**

| 层 | 入口文件 |
|----|----------|
| Rust | `src-tauri/src/services/mcp/manager.rs`、`src-tauri/src/commands/mcp.rs` |
| 前端 | `src/pages/settings/McpSettings.tsx`、`src/components/chat/ToolApprovalDialog.tsx` |

### 3.5 Skills 仓库与扫描 Gate

- Settings > Skills 是唯一入口；独立 route、title/nav 分支和页面 wrapper 已删除。
- 受管与外部 source 使用 stable SkillId、activation generation 和历史消息 hash snapshot；禁用/未扫描 source 不会进入 Sidecar mount。
- summary/tree/read-file 按需加载，全文 detail command 已删除；扫描结论来自 v12 scan/finding/approval 表，不再读取旧 `risk_json`。
- Schema v13 首次升级将旧 source 设为 `unscanned + disabled`，批量扫描进度持久化，应用中断可恢复，失败项可重试；升级前保留 `.pre-v13.sqlite3`。
- Codex / Claude / Cursor 外部目录只读；Sandbox 依赖的第三方深度 scanner（S4）按当前范围延后。

**关键路径：** `src/components/skills/`、`src-tauri/src/services/skills/`、`agent/app/agent.py`

### 3.6 Sidecar 预热（尚未参与对话）

- 应用启动可自动预热 Python Sidecar（`auto_start_sidecar` in config）
- 健康检查、`SidecarStatusBadge` 状态展示
- 运行时 watchdog：就绪后检测子进程退出 / health 失败，最多 3 次自动重启
- Nuitka 二进制优先启动：存在 `agent/dist/misaka-agent.exe` 时优先 spawn，否则回退 uvicorn
- `/health`、`/info` 可用；`/agent/chat`、`/agent/stream` 返回 **501 占位**

**关键路径：** `src-tauri/src/sidecar.rs`、`src-tauri/src/services/sidecar_client.rs`、`agent/app/routers/`

### 3.7 测试覆盖

| 范围 | 数量 | 运行命令 |
|------|------|----------|
| 前端 Vitest | 37 个测试文件 / 267 tests | `npm test` |
| Rust 集成测试 | 37 个测试文件 | 日常：`cargo test --test <name>`；提交前：`cargo nextest run --all-features --profile ci`（或 `cargo test`） |

> Rust 日常构建/测试依赖增量编译，**不要**在每次 `cargo test` 前执行 `cargo clean`。日常改代码优先 `cargo check` + 精准 `--test`（映射表见优化指南 §4.2）；Cursor hook `.cursor/hooks/post-edit-test.sh` 已按映射自动选择测试。仅在链接异常、切分支后编译诡异失败等情况下按需 `cargo clean`。见 [`docs/guides/rust-build-test-optimization.md`](../guides/rust-build-test-optimization.md)。

---

## 4. 未完成 / 占位项

### 4.1 Phase 3 遗留（进入 Phase 4 前确认）

| 项 | 现状 | 参考 |
|----|------|------|
| Sidecar Agent 端点 | `agent/app/routers/agent.py` 返回 501 | Phase 3 AC-4 → Phase 4 替换 |
| UI/实机复验记录 | C/D/F 自动化与 CLI 验证已完成；V1-V26 中仍有若干 UI 操作需人工最终确认 | Phase 3 §6 |
| 发布捆绑 | Nuitka 本地产物已验收；`tauri.conf.json` `externalBin` 与安装包捆绑属发布阶段 | Phase 6 |

### 4.2 Phase 4 核心缺口（**主续做线**）

| 项 | 现状 | 首要文件 |
|----|------|----------|
| DeepAgents Agent 组装 | 未实现 | 新建 `agent/app/agent.py` |
| 自定义 Tool（PowerMem / MCP Bridge） | 未实现 | 新建 `agent/app/tools.py` |
| Rust 对话后端切换 | 仍走 Rig | `src-tauri/src/commands/chat.rs`、`services/llm/backend.rs` |
| PowerMem 长期记忆 | 依赖未安装（optional） | `agent/pyproject.toml` `[project.optional-dependencies]` |
| 记忆管理 UI | 无 | Phase 4 Sprint 5 |

### 4.3 其余占位页面

以下页面为 `EmptyState` + `comingSoon`，数据库表已预留但无业务逻辑：

- `src/pages/KnowledgePage.tsx`
- `src/pages/DashboardPage.tsx`
- `src/pages/NotificationsPage.tsx`

`knowledge_docs` / `knowledge_fts` 表已在 Schema v1 创建；**sqlite-vec 向量表与 RAG 检索未实现**。

---

## 5. 从哪里继续开发（推荐顺序）

### 当前：完成 Phase 3

详见 [`PHASE_3_REMAINING_TODO.md`](../planning/PHASE_3_REMAINING_TODO.md) — 当前重点是补齐 §6 中 UI/实机复验记录，然后进入 Phase 4。

| Epic | 首项 TODO | 入口文件 |
|------|-----------|----------|
| A Sidecar | P3-TODO-A1 watchdog | `src-tauri/src/sidecar.rs` |
| B MCP 对话 | P3-TODO-B1 审批抽取 → B4 tool loop | `commands/chat.rs`、`services/mcp_bridge.rs` |
| C Nuitka | P3-TODO-C1 打包 | `agent/build_nuitka.py` |

### 之后：Phase 4 DeepAgents 迁移

按 [`PHASE_4_DETAILED_PLAN.md`](../planning/PHASE_4_DETAILED_PLAN.md) Sprint 顺序：

```
Sprint 1  Python DeepAgents 核心
  └─ agent/app/agent.py          create_deep_agent()
  └─ agent/pyproject.toml        安装 deepagents / langgraph / powermem optional deps
  └─ agent/app/routers/agent.py  替换 501 → 真实 /chat /stream

Sprint 2  自定义 Tool
  └─ agent/app/tools.py          PowerMem + MCP Bridge + 文件 Tool

Sprint 3  Rust 对话迁移
  └─ src-tauri/src/commands/chat.rs       Sidecar 转发策略
  └─ src-tauri/src/services/sidecar_client.rs  流式事件对齐
  └─ src-tauri/src/services/llm/backend.rs      保留 Rig 降级开关

Sprint 4–5  Checkpointer + 记忆 UI + 集成测试
```

**设计约束（Phase 4 文档已强调）：** 前端 Chat UI **零改动**；流式 Event 协议沿用现有 `SidecarClient` + Tauri Event 管线。

**本地联调：**

```bash
# 终端 1：Sidecar（开发时也可依赖 Tauri 自动预热）
cd agent
pip install -e ".[agent,memory,dev]"
python -m uvicorn app.main:app --host 127.0.0.1 --port 9527

# 终端 2：桌面应用
npm run tauri dev
```

### 路线 B：Phase 3 收尾（可与 A 并行）

1. 验收 Sidecar 异常重启（Phase 3 AC-2）
2. 跑通 `agent/build_nuitka.py` 独立 health check（Phase 3 AC-15）
3. 用 curl 验证 Agent 占位端点改真实实现后的协议

### 路线 C：Phase 5 垂直切片（Phase 4 之后）

- **Skills：** 已完成多来源发现、Settings/Chat 选择、扫描 Gate 与存量迁移；S4 随 Sandbox 另行实施
- **知识库：** `knowledge_docs` + sqlite-vec 向量表 + `KnowledgePage` 最小 RAG

---

## 6. 代码库规模速查

| 维度 | 规模 |
|------|------|
| 前端 TS/TSX | ~160+ 文件 |
| Rust 源码 | 47 个 `.rs`（`src-tauri/src/`） |
| Tauri Commands | ~50+（见 `src-tauri/src/lib.rs` `invoke_handler`） |
| DB Schema | **v13**（`src-tauri/src/db/migrations.rs`） |
| UI 设计规范 | `docs/design/frontend-ui-guidelines.md` 等 3 份 |

---

## 7. 架构现状（简图）

```
React 前端 ✅
  Chat / Settings / Workspace / MCP UI
        │
        ▼
Tauri Rust ✅
  ├── chat ──▶ Rig ──▶ LLM API          ← 当前对话路径
  ├── MCP (rmcp) ──▶ MCP Servers
  ├── SQLite v13
  └── SidecarManager ──▶ Python :9527
                              ├── /health ✅
                              └── /agent/* ❌ 501（Phase 4）
```

Phase 4 完成后，对话路径变为：`chat.rs → SidecarClient → DeepAgents → LLM/MCP/PowerMem`。

---

## 8. 文档索引

| 需求 | 文档 |
|------|------|
| **续做入口（本文）** | `docs/project/DEVELOPMENT_STATUS.md` |
| 总体规划与里程碑 | `docs/planning/MISAKAX_IMPLEMENTATION_PLAN - Opus4.6.md` |
| 当前 Phase 详细任务 | `docs/planning/PHASE_4_DETAILED_PLAN.md` |
| 目录与模块说明 | `docs/project/PROJECT_STRUCTURE.md` |
| 架构选型 | `docs/architecture/MISAKAX_ARCHITECTURE_FINAL - DeepSeek-V4-Pro.md` |
| UI 规范（改前端必读） | `docs/design/frontend-ui-guidelines.md` |
| AI / 开发者环境 | 根目录 `CLAUDE.md`、`AGENTS.md` |

---

## 9. 维护说明

- 每完成一个 Phase 或里程碑，更新本文 **§2 阶段完成度** 与 **§4 未完成项**。
- 同步更新根目录 `README.md` 的路线图勾选状态。
- 勿在 `docs/` 根目录新增 loose 文件；状态类文档放在 `docs/project/`。
