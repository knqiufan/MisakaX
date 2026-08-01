# Skills、工作区终端与 Sandbox 实施进度

> **用途：** 作为本轮 Skills/安全检查/Git 标识/终端/Sandbox/最终架构审查的单一进度台账。
> **受众：** 项目负责人、开发、测试、安全和后续接手者。
> **最后审阅 / Last reviewed：** 2026-08-01
> **代码基线：** `main@14a048a`。
> **重要说明：** 本文按实际代码审计记录，不把“已有 UI 外壳”计作完整功能；`DEVELOPMENT_STATUS.md` 中关于 Phase 5 Skills 尚未开始的描述已落后于当前代码，最终架构阶段需统一修订。

---

## 1. 状态图例

| 标记 | 含义 |
|---|---|
| ✅ | 已实现且有基本验证证据 |
| 🟡 | 部分实现或存在关键闭环缺口，不能视为交付 |
| ⬜ | 尚未实现 |
| ⛔ | 已知安全/架构阻断项；完成前不能上线相关能力 |
| 📄 | 仅完成调研/设计文档，尚无代码 |

## 2. 总览

| 能力 | 当前状态 | 判断 |
|---|---:|---|
| Skills 基础 inventory/本地与远端安装 | 🟡 | 主流程已存在；仅有归档级风险检查，未经过完整 Security Gate |
| 受管 Skill 启用/禁用 | ✅ | stable ID、统一 Gate、activation generation 与 Python 只读逐项挂载已闭环 |
| 外部 Skill 启用/禁用 | 🟡 | 来源与开关已持久化、默认禁用且不改原目录；S3 扫描裁决仍待实现 |
| composer Skill 选择过滤 | ✅ | 仅展示 `effective_active` 来源，发送 stable ID；失效事件会移除 chip 并非阻塞提示 |
| 详情按需文件浏览 | ⬜ | 当前直接读取并在文件树前展示完整 `SKILL.md` |
| Skills 迁入 Settings/MCP 下方 | ⬜ | 当前仍为独立 `skills` route |
| Skills 安全检查 | 🟡 | 有安全解包和脚本/二进制布尔标注；无 quarantine、多引擎、policy、finding、rescan |
| Git/本地项目 badge | ⬜ | 无 Git service/DTO/UI |
| 嵌入式终端 | ⬜ | 现有 Terminal 图标实际打开 Tool Logs；无 xterm/PTY |
| Tauri 终端安全收口 | ⛔ | 主 WebView shell/fs/http 权限偏宽且 CSP 为空，终端上线前必须修复 |
| Agent OS Sandbox | ⛔ | 当前为逻辑路径 guard；Shell 在宿主直接执行并继承环境 |
| 最终架构审查/重构 | ⬜ | 必须等其他功能和回归基线完成 |
| 本轮调研与规划文档 | ✅ | 已在本基线生成并完成交叉链接 |

## 3. 当前代码证据

### 3.1 Skills 已有能力

- `src-tauri/src/services/skills/installer.rs`
  - 可列出数据库受管 Skills，并发现 Codex/Claude/Cursor 外部目录。
  - 受管 Skill 可 `set_enabled`；外部记录不是 DB 实体，当前总是构造为 enabled。
  - `get_detail` 会读取完整 `SKILL.md` 并列举文件。
  - `installed_selection` 对受管 Skill 再检查 enabled/healthy。
- `src-tauri/src/services/skills/archive.rs`
  - 已限制 ZIP/解压大小、文件数、压缩比、路径穿越、绝对/反斜杠路径、重复项和符号链接。
  - 已计算 hash，并识别常见脚本/二进制扩展；尚非语义/行为安全扫描。
- `src-tauri/src/services/skills/manifest.rs`
  - 已校验 YAML frontmatter、名称/描述等基础规范。
- `src-tauri/src/commands/skills.rs`
  - list/detail/install/export/download/set_enabled/uninstall 等 commands 已注册。
- `src/components/chat/composer/MessageInput.tsx` 与 `SkillSelectorPopover.tsx`
  - 已按 enabled/healthy 过滤并支持 Skill 选择。
- `src/__tests__/composer-skills.test.ts`、`skills-components.test.tsx`、`skills-page-selection.test.tsx`
  - 已存在部分 composer/Skill UI 测试，可作为新特征测试起点。

### 3.2 详情和路由差距

- `src/components/skills/SkillDetailPanel.tsx` 当前在文件结构之前渲染整份 `skill_markdown`，文件行不可按需打开预览。
- `src/lib/ipc/skills.ts` 的详情类型携带全文，没有独立 summary/list-files/read-file API。
- `src/pages/SkillsPage.tsx` 是独立页面；`src/stores/app-store.ts` 存在独立 `{ page: "skills" }`。
- `src/pages/settings/SettingsPage.tsx` 的 tab 目前不含 Skills，导航顺序没有 MCP -> Skills。

### 3.3 禁用闭环差距

- 受管 Skill 的 DB `enabled=false` 能影响前端和 Rust selection。
- `agent/app/agent.py` 仍把全局 `settings.skills_dir` 作为 `/skills/` 挂载并传给 Agent。
- `agent/app/workspace_backend.py` 管理挂载和逻辑路径，但没有消费 per-session activation view。
- 因而当前不能证明“disabled Skill 不会被自动注入/发现”；这是 P1 的第一优先级阻断项。

### 3.4 Workspace 与 Terminal 差距

- `src/components/chat/workspace/WorkspaceBar.tsx` 已使用 Terminal 图标，但 action 打开的是 Tool Logs，不是 PTY。
- `src/components/chat/ChatView.tsx` 持有 Tool Logs drawer 状态。
- `src/pages/ChatPage.tsx` 右侧只渲染 `WorkspaceExplorer`。
- `src/components/chat/workspace/WorkspaceExplorer.tsx` 已有文件树和 Monaco，可复用 panel shell。
- `package.json` 无 xterm 依赖，`src-tauri/Cargo.toml` 无 PTY/Git/Sandbox 依赖。

### 3.5 Sandbox 与 Tauri 安全差距

- `agent/app/workspace_backend.py` 的逻辑根可降低误操作，但不是 OS 安全边界。
- 生产 Agent 当前通过 `LocalShellBackend(..., virtual_mode=True, inherit_env=True)` 执行宿主命令。
- `src-tauri/capabilities/default.json` 授予主 WebView 通用 Shell spawn/execute/stdin/kill 和较宽 FS/HTTP 能力。
- `src-tauri/tauri.conf.json` 的 CSP 为 `null`。
- 没有 Bubblewrap/Seatbelt/Windows restricted token、网络 Broker、资源限制或进程树审计。

## 4. 文档交付进度

| 文档 | 状态 | 用途 |
|---|---:|---|
| [`CROSS_PLATFORM_DESKTOP_SANDBOX_RESEARCH.md`](../research/CROSS_PLATFORM_DESKTOP_SANDBOX_RESEARCH.md) | ✅ | 三平台与容器/Wasm 候选调研 |
| [`SKILL_SECURITY_SCANNING_RESEARCH.md`](../research/SKILL_SECURITY_SCANNING_RESEARCH.md) | ✅ | 主流扫描方法、工具和推荐流水线 |
| [`SANDBOX_TECH_SELECTION.md`](../architecture/SANDBOX_TECH_SELECTION.md) | ✅ / Proposed | Sandbox ADR，待三平台 Spike 转 Accepted |
| [`WORKSPACE_SKILLS_SECURITY_ARCHITECTURE.md`](../architecture/WORKSPACE_SKILLS_SECURITY_ARCHITECTURE.md) | ✅ | 总体模块、数据、状态和阶段架构 |
| [`SKILLS_WORKSPACE_TERMINAL_UI_DESIGN.md`](../design/SKILLS_WORKSPACE_TERMINAL_UI_DESIGN.md) | ✅ | Settings/Skills/Git badge/Terminal UI 规范 |
| [`SKILLS_REPOSITORY_AND_SECURITY_PLAN.md`](../planning/SKILLS_REPOSITORY_AND_SECURITY_PLAN.md) | ✅ | Skills 分阶段 TODO |
| [`WORKSPACE_CONTEXT_AND_TERMINAL_PLAN.md`](../planning/WORKSPACE_CONTEXT_AND_TERMINAL_PLAN.md) | ✅ | Git badge/PTY/CSP 分阶段 TODO |
| [`SANDBOX_IMPLEMENTATION_PLAN.md`](../planning/SANDBOX_IMPLEMENTATION_PLAN.md) | ✅ | 三平台 Sandbox 分阶段 TODO/Gate |
| [`ARCHITECTURE_REVIEW_AND_REFACTOR_PLAN.md`](../planning/ARCHITECTURE_REVIEW_AND_REFACTOR_PLAN.md) | ✅ | 功能完成后的无回退重构 TODO |
| [`MASTER_IMPLEMENTATION_EXECUTION_GUIDE.md`](../planning/MASTER_IMPLEMENTATION_EXECUTION_GUIDE.md) | ✅ | AI Coding 文档路由、阶段闭环、测试、进度与 main push 总指导 |
| [`tauri-capability-csp-audit.md`](../guides/tauri-capability-csp-audit.md) | ✅ | W0 主 WebView 权限调用点、CSP 目标与 Tool Logs 迁移决定 |
| 本进度文档 | ✅ | 单一事实台账 |

## 5. 里程碑进度

| 里程碑 | 状态 | 已完成 | 下一退出条件 |
|---|---:|---|---|
| M0 调研与方案 | ✅ | 仓库审计、官方资料调研、总体架构、UI、分项计划、总执行指导、进度台账 | 文档 review 通过 |
| M1 契约与回归基线 | ✅ | S0/W0 特征测试、稳定 DTO/error/event、默认关闭 feature flags、capability/CSP 审计 | 进入 S1/W1 前保持基线测试绿色 |
| M2 Skills 闭环 | 🟡 | S0/S1 完成：稳定来源、全来源持久开关、activation view、只读 Sidecar 挂载 | S2 lazy files/Settings；S3 强制扫描 Gate |
| M3 工作区体验 | 🟡 | W0 行为/安全基线完成；Explorer/resize 基础可复用 | Git/local badge、WorkspacePanel、三平台 PTY、CSP/IPC 收窄 |
| M4 Sandbox Spike | 📄 | 调研和 ADR | 三平台 filesystem/network/process attack fixtures 通过 |
| M5 Sandbox 默认化 | ⬜ | 逻辑 guard/审批可复用 | Agent/Skill/MCP 无 host Shell fallback，严格模式发布 gate |
| M6 安全强化 | ⬜ | 基础 archive checks | 深度扫描、存量 rescan、恶意样本、独立安全 review |
| M7 架构审查 | ⬜ | 已有计划 | 热点拆分、旧路径清理、功能/性能/安全无回退 |

## 6. 建议执行顺序

1. **M1 先行：** 不先改 UI；先用测试固定现有 selection/mount/route/panel 行为，定义 stable IDs 和 events。
2. **M2 与 M3 可部分并行：** Skills 团队处理 Registry/Gate/Settings；Workspace 团队处理 Git Context/Panel/PTy，但 Tauri capability 收窄需要统一 review。
3. **M4 平台 Spike 尽早开始：** Windows 是最高不确定性，必须先证明强网络边界，不等所有 UI 完成。
4. **M5 在 provider gate 后切生产执行链：** Python Brokered Backend、Skills activation、MCP spawn 一起关闭旁路。
5. **M6 后再默认启用高保证：** 完成深度扫描、存量迁移和 release attack suite。
6. **M7 严格最后：** 只在所有行为和性能有基线后做结构清理。

## 7. 当前阻断和决策点

| 编号 | 类型 | 内容 | 处理阶段 |
|---|---|---|---|
| B-01 | 安全 | ✅ S1 已解除：Python 不再挂载整目录，只读取 generation-stamped activation view | 已关闭 |
| B-02 | 安全 | 外部 Skill 已有稳定身份和持久开关；quarantine/扫描裁决仍缺失 | M2 / S3 |
| B-03 | UX/API | 详情 DTO 自动携带全文，长内容挤压文件树 | M2 / S2 |
| B-04 | 安全 | 所有安装入口缺统一 Security Gate/quarantine | M2 / S3 |
| B-05 | 安全 | 主 WebView 通用 Shell 权限 + CSP null | M3 上线门 |
| B-06 | 安全 | Agent Shell 宿主执行和环境继承 | M4-M5 |
| B-07 | 选型 | Windows strong sandbox 需 UAC setup/专用用户/firewall 的产品接受度 | M4 Spike review |
| B-08 | 依赖 | Cisco scanner 是否并入 Sidecar 或独立 helper，需三平台打包 PoC | M6 / S4 |

## 8. 进度更新规则

每个 PR/里程碑完成后更新本文：

1. 将状态改为 ✅ 前，附代码、测试、平台和安全证据；“代码写完但未实机”仍是 🟡。
2. 更新 `代码基线` 和 Last reviewed。
3. 在对应计划勾选 TODO，并在这里记录里程碑级结果，不复制所有 PR 细节。
4. 新阻断必须写编号、影响、owner/阶段和解除条件。
5. 安全功能若降级，立即把状态从 ✅ 调回 🟡/⛔，不能只写在 release note。
6. 文档与代码冲突时以测试过的实际代码为事实，并在同一变更修正文档。

## 9. 下一步（尚未执行）

- [ ] 对本轮文档完成团队 review，确认 Windows UAC setup、存量未扫描 Skill 默认禁用和云扫描隐私三项产品决策。
- [x] 从 [`SKILLS_REPOSITORY_AND_SECURITY_PLAN.md`](../planning/SKILLS_REPOSITORY_AND_SECURITY_PLAN.md) 的 S0 与 [`WORKSPACE_CONTEXT_AND_TERMINAL_PLAN.md`](../planning/WORKSPACE_CONTEXT_AND_TERMINAL_PLAN.md) 的 W0 建立回归基线。
- [x] 执行 Skills S1，关闭 disabled Skill 仍被全目录挂载的 B-01。
- [ ] 执行 Workspace W1/W2，交付只读工作区标识与 panel 容器。
- [ ] Sandbox B0–B7：按用户当前范围暂不实施；未获得新指令前不启动 Spike 或生产执行链改造。

## 10. 实施记录

### 2026-08-01 S0/W0 契约与回归基线

- 状态：已完成并推送。
- 基线：`main@fa24bd7`；推送：`10b9f6c`（`origin/main`）。
- 完成 TODO：Skills S0 全部 9 项；Workspace Terminal W0 全部 7 项。
- 代码证据：`src-tauri/src/contracts.rs`、`src/lib/ipc/contracts.ts`、`src/lib/feature-flags.ts`、Skills installer 特征测试、Workspace/Tool Logs/Explorer 特征测试、Tauri security config baseline 测试。
- 测试证据：前端 full 232 passed；Rust `cargo test --features test-private -j 1` 全量通过（并行 `cargo test` 曾因 Windows 页文件不足 OOM，串行回退绿色）；Python full 116 passed + 1 strict xfail；`npm run build`、`cargo check`、`ruff` 通过。
- 环境差异：Python 实际解释器为 3.13.9，非项目目标 3.11.x；本阶段测试通过，但发布验证仍须在 3.11.x 重跑。
- 安全证据：严格 xfail 明确复现 legacy `/skills` 全目录挂载暴露 disabled Skill；主 WebView 通用 Shell/FS/HTTP 与 CSP null 已由静态测试固定为 W5 待消除基线。
- 文档同步：两份专项 Plan、Tauri capability/CSP 审计、本进度台账。
- 风险/阻断：B-01、B-02、B-05 均未解除；本阶段只建立可回归证据。
- 下一步：Skills S1，先实现 stable identity、全来源持久开关和 activation generation/view，关闭 B-01。

### 2026-08-01 Skills S1 稳定身份与激活视图

- 状态：已完成；实现提交 `14a048a`。
- 完成 TODO：Skills S1 全部 13 项；Sandbox 未实施。
- 数据证据：v11 `skill_sources`、稳定 `skill_id`、slug/hash 消息快照、单调 activation generation；升级前用 SQLite `VACUUM INTO` 自动生成可恢复 v10 备份。
- 运行时证据：managed/Codex/Claude/Cursor 来源持久化并按 rank 冲突消解；外部首次发现默认禁用；制品 hash 改变会失效旧激活；enable/selection/mount 共用 `SkillSecurityGate`。
- Sidecar 证据：Rust 每次请求发送 activation view；Python 已删除 slug 推导宿主路径和全局 `settings.skills_dir` 挂载，只读虚拟 `/skills/<slug>/` 拒绝 write/edit/upload。
- UI 证据：Composer 只选择 `effective_active` 来源并发送 stable ID；禁用、缺失、hash 变化或冲突会清理 chip，提示已补中英文 i18n。
- 测试证据：前端 full 233 passed；Python full 121 passed，新增 workspace targeted 10 passed；Rust `cargo test --features test-private -j 1` 全量通过；`npm run build`、`cargo check`、`ruff` 通过。
- 兼容与限制：旧 slug 查询/显式 path-bearing Sidecar 协议保留一个版本；`legacy_allowed` 只描述迁移兼容，不等同扫描通过。S3 将以 scan/policy 状态替换该过渡裁决。
- 风险/阻断：B-01 已解除；B-02 缩小为扫描与 quarantine 缺口；B-05 仍待 Workspace W5。
- 回滚：代码回滚到 `663c8bb`；数据库用同目录 `.pre-v11.sqlite3` 备份恢复，外部 Skill 原目录从未移动或删除。
- 下一步：Skills S2，迁入 Settings 并实现 summary/tree/read-file 按需详情；同时可启动 Workspace W1/W2。
