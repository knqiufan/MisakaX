# Skills、工作区终端与 Sandbox 实施进度

> **用途：** 作为本轮 Skills/安全检查/Git 标识/终端/Sandbox/最终架构审查的单一进度台账。
> **受众：** 项目负责人、开发、测试、安全和后续接手者。
> **最后审阅 / Last reviewed：** 2026-08-02
> **代码基线：** `main@ed82bfb`。
> **重要说明：** 本文按实际代码审计记录，不把“已有 UI 外壳”计作完整功能；Skills S0–S3、S5 与 Workspace W0–W5 实现已闭环，S4 因依赖 Sandbox helper 按用户范围明确延期，Sandbox 本身暂不实施；W6 已完成 Windows 11 当前机的长路径、shell、压力、崩溃恢复、真实 pwsh 7.6.3、进程级离线启动与 Release bundle 证据，Windows 10/macOS/Linux/原生 IME/签名发布仍待对应环境验证。

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
| Skills 基础 inventory/本地与远端安装 | ✅ | 本地、远端与外部发现统一经过 quarantine/扫描/策略 Gate；发布只接受不可伪造的 ApprovedArtifactId |
| 受管 Skill 启用/禁用 | ✅ | stable ID、统一 Gate、activation generation 与 Python 只读逐项挂载已闭环 |
| 外部 Skill 启用/禁用 | ✅ | 来源与开关持久化、默认禁用且不改原目录；发现/文件变化会扫描，hash/规则/策略失效时转 stale |
| composer Skill 选择过滤 | ✅ | 仅展示 `effective_active` 来源，发送 stable ID；失效事件会移除 chip 并非阻塞提示 |
| 详情按需文件浏览 | ✅ | summary/tree/read-file 已拆分；文件正文仅在用户选择后按 200 KiB 分段读取 |
| Skills 迁入 Settings/MCP 下方 | ✅ | Settings 导航顺序为 MCP → Skills；旧 `skills` route/title/nav/page wrapper 已删除 |
| Skills 安全检查 | ✅ | 内置离线引擎、quarantine、versioned policy、finding/审批/rescan/export、升级批量扫描与统一 Gate 已闭环；Sandbox deep scanner 按范围延后 |
| Git/本地项目 badge | ✅ | 只读 WorkspaceContext/Git provider、generation DTO/event、composer badge 与无路径诊断已闭环 |
| 嵌入式终端 | 🟡 | W3–W5 已交付 owner-bound PTY、xterm UI 与最小 capability/CSP；W6 已通过 Windows 11 当前机的 PowerShell 5/cmd、官方便携 pwsh 7.6.3、超长 Unicode cwd、10 MiB 突发、崩溃回收/重开、离线解析失败启动和 Release UI，Windows 10/macOS/Linux、原生 IME 与签名发布仍未闭环 |
| Tauri 终端安全收口 | ✅ | 通用 Shell execute/spawn/stdin/kill、FS/HTTP/Notification 权限已删除；生产 CSP、精确 custom-command manifest 与 Windows Release 审计通过 |
| Agent OS Sandbox | ⛔ | 当前为逻辑路径 guard；Shell 在宿主直接执行并继承环境 |
| 最终架构审查/重构 | ⬜ | 必须等其他功能和回归基线完成 |
| 本轮调研与规划文档 | ✅ | 已在本基线生成并完成交叉链接 |

## 3. 当前代码证据

### 3.1 Skills 已有能力

- `src-tauri/src/services/skills/installer.rs`
  - 可列出受管 Skills，并将 Codex/Claude/Cursor 外部目录同步为稳定 source；所有来源的开关状态均持久化。
  - 单一 `skill_sources` inventory 与 stable-ID 查询已经取代旧 `skills` dual-write/slug fallback；生产 Settings UI 只使用 summary/tree/read-file 按需接口。
  - 受管发布在文件替换与 SQLite transaction 任一步失败时回滚；发布入口要求安全模块签发的 `ApprovedArtifactId`。
- `src-tauri/src/services/skills/archive.rs`
  - 已限制 ZIP/解压大小、文件数、压缩比、路径穿越、绝对/反斜杠路径、重复项和符号链接。
  - 归档验证与语义分析已拆分；嵌套归档、MIME、Unicode/ADS/设备名和 ZIP bomb 均有回归测试。
- `src-tauri/src/services/skills/security/`
  - 扫描器按 archive/content/manifest/secret/command/permission/policy 职责拆分，只读且从不执行目标 Skill。
  - 2 并发队列、取消/超时/progress、启动恢复、90 天历史清理、500 MiB quarantine/7 天过期和文件 watcher debounce 已落地。
  - balanced-v1 将 Critical/High block、Medium review、Low/Info warnings；engine error/timeout fail closed。
  - v13 首次升级任务会持久化逐 source 状态、顺序扫描所有健康 `unscanned` source、恢复中断项并支持仅重试失败项；扫描前始终禁用注入。
- `src-tauri/src/services/skills/manifest.rs`
  - 已校验 YAML frontmatter、名称/描述等基础规范。
- `src-tauri/src/commands/skills.rs`
  - 除 inventory/file/install 外，已注册 findings、审批/拒绝/撤销、重新扫描/取消、JSON/SARIF 导出和隐私契约 commands；安全错误使用稳定 code/correlation ID。
- `src/components/chat/composer/MessageInput.tsx` 与 `SkillSelectorPopover.tsx`
  - 已按 enabled/healthy 过滤并支持 Skill 选择。
- `src/__tests__/composer-skills.test.ts`、`skills-components.test.tsx`、`skills-page-selection.test.tsx`
  - 已存在部分 composer/Skill UI 测试，可作为新特征测试起点。

### 3.2 S5 旧路径清理证据

- `skills_get_detail`、全文 `SkillDetail.skill_markdown`、旧 `SkillRepo`/`skills` table、`find_id_or_legacy_slug` 与全部 Skills 休眠 feature flags 已从生产代码删除。
- 独立 `SkillsPage`、`{ page: "skills" }`、标题和导航分支已删除，入口只有 Settings → Skills。
- v13 将旧 `legacy_allowed`/待审批来源统一转为禁用的 `unscanned`，建立可恢复的迁移扫描状态，删除旧 `skills` table 与 `skill_sources.risk_json`；scan summary 是安全事实来源。
- 历史消息的 stable ID/slug/hash 快照继续可读；外部目录只读且迁移/卸载不修改其内容。

### 3.3 禁用与会话闭环证据

- Rust 在每次 Agent 请求前生成并发送带 generation 的 activation view；过期 generation 在挂载前被拒绝。
- Python 只消费逐项只读 activation mount，不接收全局 Skills 目录、slug 推导宿主路径或 Skills 专用 `LocalShellBackend` fallback。
- enable、selection、composer chip、消息发送与 Sidecar mount 共用 stable ID、effective-active 与扫描 Gate；禁用/变化事件会使旧选择失效。

### 3.4 Workspace 与 Terminal 证据/差距

- `src-tauri/src/services/workspace/` 已提供只读 Git provider、可信可执行文件解析、结构化 argv、超时/取消/输出上限、single-flight/TTL/watcher/generation；普通分支、detached、worktree、submodule、bare 与非 Git 均有真实 Git fixture。
- `src/components/chat/composer/WorkspaceContextBadge.tsx` 与 `src/hooks/use-workspace-context.ts` 已交付 composer Git/本地标识，快速工作区切换会丢弃旧请求/旧 generation，诊断 tooltip 不暴露绝对路径。
- `src/stores/workspace-panel-store.ts` 独立管理 open/mode/size/session/generation，只持久化 UI 偏好；旧 Explorer open 偏好一次性迁移，Explorer tabs/activePath 仍归原 store。
- `src/pages/ChatPage.tsx` 已用单一 `WorkspacePanel` 包裹既有 `WorkspaceExplorer`；宽窗为 70/30 百分比分栏，窄窗为不挤压聊天列的右侧 overlay。
- `src/components/chat/workspace/WorkspaceBar.tsx` 的 Explorer/Terminal 使用同一 mode action、pressed/tooltip/快捷键；W5 后 Terminal 默认启用，前后端只保留 `false|0` 显式紧急 kill switch，不渲染不可用空动作。
- Tool Logs 已迁到消息 `ToolActionsGroup` 的明确日志按钮，仍打开 ChatView 内原聚合抽屉，不再借用 Terminal 图标或右轨。
- `src-tauri/src/services/terminal/` 已实现 `TerminalManager`、可信 shell 解析、window + Chat Session + workspace generation ownership、限额/背压、`terminal:output` / `terminal:exited` 事件和 Windows Job Object / Unix process-group 回收；spawn 不接受 cwd、任意 executable、argv 或 env。Windows 超长 canonical cwd 由 PowerShell 安全 `Set-Location -LiteralPath`，cmd 无法支持时以 `shell_workspace_path` 失败关闭。
- `src/lib/ipc/terminal.ts` 只暴露 spawn/write/resize/kill/get-state 窄接口；输出使用 base64 二进制载荷和递增 seq，W4 UI 已按 terminal/generation/seq 丢弃迟到事件并以 `last_seq` 排空退出。
- `WorkspacePanel` 会保持访问过的 Terminal 内容槽挂载；实际 process/output 生命周期归 Rust manager 与独立 `terminal-store`，不能写回 panel 偏好 store。
- manifests 已锁定 `portable-pty = 0.9.0`、`@xterm/xterm = 6.0.0` 与 `@xterm/addon-fit = 0.11.0`；本地 xterm UI 已落地，Sandbox 依赖仍未加入。W1 Git 上下文继续使用受控系统 Git CLI，不引入 Git crate。
- `src-tauri/capabilities/default.json` 只授予本地 `main` window 精确 event、HTTP(S) 外链 open、dialog、clipboard、`main-commands` 与五个 `terminal-runtime` commands；102 个 custom commands 的 handler/manifest/permission 集合由测试锁定。
- `src-tauri/tauri.conf.json` 已配置严格 production CSP；HMR origin 只在 `devCsp` 生效。Windows Release 默认入口已真实启动 PowerShell 并验证 prompt、中文/宽字符、ANSI、原生 clipboard paste 与关闭后进程树归零。

### 3.5 Sandbox 安全差距

- `agent/app/workspace_backend.py` 的逻辑根可降低误操作，但不是 OS 安全边界。
- 生产 Agent 当前通过 `LocalShellBackend(..., virtual_mode=True, inherit_env=True)` 执行宿主命令。
- W5 已关闭 WebView 通用 shell/fs/http 权限和 CSP 空缺；这只加固用户 Terminal/WebView 边界，不等于 Agent OS Sandbox。
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
| [`tauri-capability-csp-audit.md`](../guides/tauri-capability-csp-audit.md) | ✅ | W5 最小 capability、生产 CSP、Release bundle 与 Windows 实机证据 |
| 本进度文档 | ✅ | 单一事实台账 |

## 5. 里程碑进度

| 里程碑 | 状态 | 已完成 | 下一退出条件 |
|---|---:|---|---|
| M0 调研与方案 | ✅ | 仓库审计、官方资料调研、总体架构、UI、分项计划、总执行指导、进度台账 | 文档 review 通过 |
| M1 契约与回归基线 | ✅ | S0/W0 特征测试、稳定 DTO/error/event、默认关闭 feature flags、capability/CSP 审计 | 进入 S1/W1 前保持基线测试绿色 |
| M2 Skills 闭环 | ✅ | S0–S3、S5 完成：稳定来源、按需文件、只读挂载、quarantine/内置扫描/统一 Gate、存量迁移与旧路径清理 | S4 Sandbox deep scanner 由用户范围明确延期，不阻塞当前 Skills 交付 |
| M3 工作区体验 | 🟡 | W0–W5 完成；W6 Windows 11 当前机的 shell/长路径/突发输出/崩溃恢复/真实 pwsh/离线解析失败/Release bundle 已验证 | Windows 10、macOS/Linux、原生 IME、旧 Git、休眠恢复与签名/发布验证 |
| M4 Sandbox Spike | 📄 | 调研和 ADR | 三平台 filesystem/network/process attack fixtures 通过 |
| M5 Sandbox 默认化 | ⬜ | 逻辑 guard/审批可复用 | Agent/Skill/MCP 无 host Shell fallback，严格模式发布 gate |
| M6 安全强化 | 🟡 | S3 内置离线扫描、恶意/良性 corpus、审批与 stale 生命周期；S5 存量 rescan 完成 | S4 deep scanner（Sandbox 延后）、独立安全 review |
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
| B-02 | 安全 | ✅ S3 已解除：外部 Skill 发现/变更统一扫描，裁决、审批、stale 与持久开关闭环 | 已关闭 |
| B-03 | UX/API | ✅ S2 已解除：详情首载不再返回正文，文件树与预览独立滚动并按需分段读取 | 已关闭 |
| B-04 | 安全 | ✅ S3 已解除：本地/远端/外部入口统一 Security Gate；发布需要 `ApprovedArtifactId` | 已关闭 |
| B-05 | 安全 | ✅ W5 已解除：主 WebView 最小 capability + 严格生产 CSP + Release 验证 | 已关闭 |
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
- [x] 执行 Skills S2，迁入 Settings 并关闭详情首载全文的 B-03。
- [x] 执行 Skills S3，交付 quarantine、内置离线扫描、finding/policy/审批和强制安装 Gate，关闭 B-02/B-04。
- [ ] Skills S4 — DEFERRED(Sandbox)：第三方 deep scanner 必须在 read-only/offline Sandbox helper 中验证；按用户范围暂不实现，不以宿主直接运行替代。
- [x] 执行 Skills S5，完成存量扫描迁移和旧生产旁路清理。
- [x] 执行 Workspace W1，交付只读工作区标识、可信 Git provider 与缓存/刷新闭环。
- [x] 执行 Workspace W2，交付独立 WorkspacePanel 容器、统一 panel actions 与 Tool Logs 新入口。
- [x] 执行 Workspace W3，交付 owner-bound Rust PTY/TerminalManager 与窄 IPC；W5 前保持 Terminal rollout 默认关闭。
- [x] 执行 Workspace W4，交付 xterm UI、事件顺序/生命周期、复制粘贴、resize 与工作区切换选择。
- [x] 执行 Workspace W5，删除主 WebView 通用权限、配置生产 CSP 与 custom command authorization，完成 Windows Release 真实 shell 复验并解除 B-05。
- [ ] 执行 Workspace W6，完成压力/诊断/离线验证；Windows 当前机证据先行，macOS/Linux 与签名/发布矩阵不得用 Windows 结果代替。
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

### 2026-08-01 Skills S2 Settings 与按需文件详情

- 状态：已完成；实现提交 `e118d63`。
- 完成 TODO：Skills S2 全部 15 项；无数据库迁移；Sandbox 未实施。
- API 证据：新增 `skills_get_summary`、`skills_list_files`、`skills_read_file`、`skills_get_scan_summary`；普通远端详情只读取仓库元数据，不自动下载或解包归档；旧 `skills_get_detail` 仅保留一版兼容且新 UI 不再调用。
- 文件边界：后端只接受稳定 Skill ID 与相对路径；拒绝绝对路径、穿越、ADS、设备名、符号链接/junction/reparse point，并执行 canonical containment 与打开前后快照校验。单段 200 KiB、单文件预览总量 2 MiB、并发读取 8；二进制或不支持编码只返回元数据。
- UI 证据：Skills 已迁入 Settings 的 MCP 下方；宽屏为列表/详情双栏，窄屏为单列切换；固定详情头下提供 Files/Security/Overview，默认 Files。首选 Skill 只请求 summary 与根文件页，点击文件后才请求正文；树支持 roving focus 与方向键导航，晚到响应按 generation/path 丢弃。
- 测试证据：前端 full 33 files / 243 tests passed；Rust `cargo test --features test-private -j 1` 全量通过，file-provider 单元/安全/竞态/预算与 Windows junction 用例绿色；`npm run build`、`cargo check`、`cargo fmt --check`、`git diff --check` 通过。Vite 仅保留既有大 chunk 警告。
- 性能证据：测试固定 500 项树首屏预算 `< 1000 ms`；Rust 固定 500 项目录列举 `< 200 ms`，100 次 summary 读取 P95 `< 150 ms`；summary 响应正文传输量为 0。
- 文档同步：更新 Skills/Workspace Terminal UI 设计及三份全局前端规范，补充 Settings 顺序、双层滚动、惰性预览、Switch/Tab/Tree 键盘语义与禁止远端自动下载规则。
- 特性开关：S0 的 `skills_settings_tab_v2=false`、`skills_lazy_file_preview=false` 契约仍保留但当前未参与路由分支；S2 安全文件提供器与 Settings 路由为单一生产实现。S5 需明确删除这些休眠开关或补齐真正可验证的回滚路径，不能将其误报为可用回滚开关。
- 风险/阻断：B-03 已关闭；B-02/B-04 的 quarantine、多引擎扫描与统一安装 Security Gate 仍由 S3 负责；B-05 仍待 Workspace W5。
- 回滚：代码回滚到 `9f1c235`；本阶段无数据库或外部 Skill 目录变更。若回滚，新 UI/API 与安全文件读取提供器会整体退出。
- 下一步：Skills S3，建立 artifact/scan/finding/approval 数据模型、quarantine 与所有入口统一 Security Gate；继续排除 Sandbox 实施。

### 2026-08-01 Skills S3 隔离扫描与强制安装门

- 状态：已完成；实现提交 `f41d858`，Sandbox helper/deep scanner 未实施。
- 完成 TODO：Skills S3 全部 16 项；v12 新增 artifact/scan/finding/approval、索引和 `skill_sources.current_scan_id`，升级前自动生成可恢复 v11 备份。
- Gate 证据：本地 ZIP、SkillHub/ClawHub/ModelScope 远端 ZIP 与 Codex/Claude/Cursor external discover 均进入同一扫描服务；安装器发布只接受安全模块在 allow/审计批准后签发的 `ApprovedArtifactId`，旧 install command 不再接收任意暂存目录。
- 隔离/一致性：500 MiB quarantine、7 天过期、90 天非当前扫描历史清理；启动把 queued/scanning 恢复为 fail-closed error。发布使用原子目录替换 + SQLite transaction，任一步失败恢复旧目录和 DB。
- 内置扫描：只读、不执行目标 Skill；拆为 archive/content/manifest/secret/command/permission/policy。`balanced-v1` 固定 Critical/High block、Medium review、Low/Info warnings；2 并发、15 秒队列预算、8 秒扫描预算、取消/progress/restart recovery 已验证。
- 生命周期：artifact hash、内置引擎版本、policy 版本、审批到期/撤销和 source revocation 会转 stale 并禁用；500 ms watcher 自动 reconcile，启用/选择/挂载仍同步校验 hash 与 activation generation。
- UI/隐私：Security tab 按需加载 summary、cursor findings 与审批历史；支持严重度过滤、脱敏证据、修复建议、实时进度/取消、重新扫描、理由审批/拒绝/撤销、JSON/SARIF 保存。内置扫描网络、VT hash、文件上传和云 LLM 均为 false；不存在绕过同意的上传代码路径。
- 安全语料：Windows persistence、Linux download-exec、macOS persistence、Prompt Injection、credential access/exfiltration、obfuscation 和良性 fixture；另有 traversal/ADS/设备名/Unicode、symlink/junction、ZIP bomb、嵌套 archive/MIME、长行/文件数/超时/取消测试。三平台脚本是静态语料覆盖，不等同 macOS/Linux 实机验证。
- 测试证据：前端 full 33 files / 247 tests passed；Rust `cargo test --all-features -j 1` 全量通过，S3 定向 security 15、migration 18、corpus 4 均通过；Python 120 passed / 1 skipped；`npm run build`、`cargo check`、`cargo fmt --check`、`git diff --check` 通过。普通 `cargo test` 因仓库既有 `fs_explorer_tests` 需要 `test-private` 无法编译，按项目规定用 all-features 全量命令验证。
- 性能证据：500 文件/长行/扫描预算用例在 security lib suite 总执行 `< 1 s`；Windows `--all-features` 全量首次链接约 4–7.5 分钟，测试执行本身不足 1 秒；Vite 仍只有既有大 chunk 警告。
- 环境差异：本次仅 Windows 实机；Python 实际解释器 3.13.9，非目标 3.11.x。macOS/Linux 打包和实机扫描/terminal 验证不得据此标完成。
- 规范同步：`SKILLS_WORKSPACE_TERMINAL_UI_DESIGN.md` 与三份全局 UI 规范已加入 findings、审批恢复/撤销、进度取消、隐私与导出规则；`ui-ux-pro-max` skill 的本地脚本缺失，改按已读取规范人工落实键盘、ARIA 与性能要求。
- 特性开关：S0 Skills flags 仍默认 false 且未参与生产分支；S3 是单一生产 Gate。S5 必须删除休眠 flags 或实现真实可验证回滚，不能把当前 flags 记作可用 rollout。
- 回滚：代码回滚到 `3fa8f6c`；数据库使用同目录 `.pre-v12.sqlite3` 恢复。quarantine/外部源不被回滚删除，外部目录从未被修改。
- 下一步：S4 依赖 Sandbox helper 的第三方深度扫描按用户范围延后；继续 Skills S5，完成存量迁移/兼容清理，再进入 Workspace W1–W6。

### 2026-08-01 Skills S5 存量迁移与旧路径清理

- 状态：已完成并推送；实现提交 `789676d`。S4 仍因 Sandbox helper 依赖而按用户范围延期。
- 完成 TODO：Skills S5 全部 10 项；v13 新增持久化升级扫描 aggregate/item 状态，升级前自动生成可恢复 v12 备份。
- 迁移证据：旧 `legacy_allowed`/`pending_user`/`user_allowed` source 统一转为禁用 `unscanned`；启动后顺序扫描全部健康未扫描 source，持久化总数/当前项/失败项，重启恢复 running 项并可仅重试失败项；扫描中删除 source 会重算进度而不悬挂。
- 单一事实源：删除旧 `SkillRepo`、`skills` table、dual-write、slug fallback、`risk_json` 与兼容裁决；`skill_sources` + scan summary 是唯一 inventory/安全状态来源，stable-ID 消息快照继续可读。
- 生产清理：删除 `skills_get_detail`/全文 DTO、独立 `SkillsPage` 与 route/title/nav wrapper、5 个无生产分支的 Skills flags；Settings 是唯一入口。Python 保持逐项只读 activation view，整目录与 Skills 专用 shell fallback 均不存在。
- UI/规范：Settings 顶部显示紧凑的持久迁移进度/失败恢复提示和失败项重试；仅 passed/warnings/approved 可启用。Skills UI 设计与三份全局 UI 规范已同步入口、banner、按钮、i18n/a11y 规则。
- 测试证据：前端 full 33 files / 248 tests；Rust `cargo test --all-features -j1` 全量 0 失败，v13 migrations 19/19，最新迁移恢复/删除边界 2/2；Python full 121 passed；`npm run build`、TypeScript、`cargo check`、`cargo fmt --check`、`git diff --check` 通过。Vite 仅有既有大 chunk 警告。
- 环境差异：仅 Windows 实机；Python 实际解释器非目标 3.11.x。外部目录不可写/activation 行为有跨平台逻辑测试，但 macOS/Linux 实机与打包不据此标完成。
- 特性开关：S0 的 Skills 休眠 flags 已删除；安全 Gate 与 Settings/按需详情为单一生产实现，不存在可绕过 Gate 的旧回滚分支。
- 回滚：代码回滚到 `89dc716`；数据库用同目录 `.pre-v13.sqlite3` 恢复 v12。回滚不删除 quarantine 或用户外部目录，外部目录从未被迁移写入。
- 下一步：Workspace Terminal W1，交付只读 workspace identity/Git context；Sandbox 继续排除。

### 2026-08-01 Workspace W1 只读 WorkspaceContext 与 Git Provider

- 状态：已完成并推送；实现提交 `0675b24`，Sandbox 未实施。
- 完成 TODO：Workspace Terminal W1 全部 11 项；W0 的休眠 `workspace_context_badge` flag 已删除，当前只保留后续 terminal/narrow rollout flags。
- 后端证据：新增 `WorkspaceContextService`、`VcsProvider` 与 `GitCliProvider`；工作区由 chat session 后端解析并 canonicalize，前端不能传 cwd、命令或 executable。Git 只使用结构化 argv 和只读 `rev-parse`/`symbolic-ref`，执行前清理继承的 `GIT_*`，设置 `GIT_OPTIONAL_LOCKS=0`/`GIT_TERMINAL_PROMPT=0`，并具备可信路径解析、2 秒超时、显式取消、8 KiB stdout/stderr 上限与 UTF-8/branch/SHA 校验。
- 一致性证据：cache key 为 canonical path + 单调 workspace generation；3 秒 TTL、并发 single-flight、会话 rebind/unbind 取消、HEAD/common refs/packed-refs watcher 与 300 ms debounce 已落地；focus 和 terminal-exit refresh seam 统一发出 `workspace:context:changed`。
- UI/隐私证据：composer footer 新增纯展示 `WorkspaceContextBadge`，覆盖 branch、`detached:<sha>`、Local project、loading 和窄栏文字隐藏；长分支中部省略。tooltip 仅显示通用诊断和短 correlation ID，不显示工作区绝对路径；未渲染 chevron/menu 或 Git 写操作空壳。
- 测试证据：前端 full 34 files / 253 tests；Rust workspace unit 5/5、真实 Git integration 5/5；普通分支、detached、worktree、submodule、bare、非 Git、空格/中文路径、缺失 Git、超时/取消、快速切换、旧 generation 和 watcher 事件均有覆盖。`npm run build`、`npx tsc --noEmit`、`cargo check --all-features -j1`、`cargo fmt --check`、`git diff --check` 通过；Vite 仅有既有大 chunk 警告。
- 安全证据：生产 workspace context 源码静态审计未发现 add/commit/checkout/push/pull/merge/reset/restore/switch/stage 子命令；Git 可执行文件拒绝工作区、临时目录与相对 PATH 候选，诊断文案不携带敏感路径。
- 环境差异：本阶段仅在 Windows 实机验证系统 Git；worktree/submodule 等语义通过真实 Git 仓库测试，但不据此宣称 macOS/Linux 实机或打包完成，三平台发布证据留待 W6。
- 架构说明：service 通过事件/任务回调与 Tauri 解耦，避免 Windows 测试二进制被 GUI manifest/ComCtl v6 加载条件影响，同时生产 setup 仍由 AppHandle adapter 注入事件与异步任务。
- 回滚：代码回滚到 `4800a10`；本阶段无数据库迁移，也不修改工作区文件或 Git 元数据。
- 下一步：Workspace Terminal W2，建立独立 panel state/container，替换 Terminal/Tool Logs 入口并保持 Explorer 状态；Sandbox 继续排除。

### 2026-08-01 Workspace W2 WorkspacePanel 容器

- 状态：已完成并推送；实现提交 `4e30c10`，未引入 PTY/xterm，Sandbox 未实施。
- 完成 TODO：Workspace Terminal W2 全部 8 项。现有 `WorkspaceExplorer` 的 props/API 未改变，外层由 `WorkspacePanel` 统一协调 explorer/terminal mode。
- 状态证据：新增独立 `workspace-panel-store`，runtime 持有 open/mode/size/sessionId/workspaceGeneration，只持久化 open/mode/size；Explorer store 删除 open 并继续独立拥有 tabs/activePath/file state。旧 `misakax:workspace-explorer.state.open` 仅在新 key 不存在时迁移一次，成功后删除旧键。
- 生命周期证据：同 mode 快速 toggle 关闭/重开，跨 mode 自动打开并切换；切任务接受新 session/generation，同一绑定拒绝倒退 generation，工作目录切换用 generation 0 先清除旧所有权。访问过的 Terminal slot 切回 Explorer 后保持挂载，为 W4 避免重复 spawn 建立契约，实际 process/output 不进入 panel store。
- UI/入口证据：WorkspaceBar Explorer/Terminal 共用单一 mode action，具备 secondary pressed state、hover/focus、准确 Tooltip、`aria-pressed`、`aria-keyshortcuts`；快捷键为 `Ctrl/Cmd+Shift+E` 和 `Ctrl/Cmd+反引号`。Tool Logs 已迁入消息 `ToolActionsGroup` 的独立 `ScrollText` 按钮，继续打开聊天列抽屉且不联动分组折叠。
- 响应式证据：宽窗使用 `react-resizable-panels` 百分比布局，默认 70/30、panel 18%–55%，拖动结束后持久化；视口 `<= 960px` 改为右侧 `min(88%, 520px)` overlay，不挤压聊天列，并支持 Escape/标题关闭按钮。
- Rollout 证据：Terminal mode shell 已接线但 `workspaceTerminal` flag 默认关闭；W3/W4 完成前稳定入口不显示半成品 Terminal。内部 fallback 明确为 runtime disabled，不伪装成可用 PTY。
- 测试证据：前端 full 34 files / 257 tests；W2 定向与 W1 context/flags 合计 15/15。覆盖 legacy migration、resize clamp/persistence、Explorer tabs 分离、session/generation、rapid toggle、Terminal hidden mount、Tool Logs 新入口、active/ARIA/shortcut 与窄窗策略；`npm run build`、`npx tsc --noEmit`、`git diff --check` 通过，Vite 仅有既有大 chunk 警告。
- 环境差异：本阶段为前端容器/状态契约，只在 Windows WebView 开发环境验证；真实 PTY、ConPTY/Unix PTY、IME/TUI 与 macOS/Linux 实机不据此标完成。
- 回滚：代码回滚到 `fe7a2ad`；无数据库迁移。若需恢复旧 UI 偏好，可删除 `misakax:workspace-panel`，但回滚不会修改 Explorer 文件或工作区内容。
- 下一步：Workspace Terminal W3，先做 `portable-pty` 锁定版本 PoC，再实现 owner-bound TerminalManager、窄 IPC、背压与进程树 cleanup；Sandbox 继续排除。

### 2026-08-01 Workspace W3 owner-bound PTY 与窄 IPC

- 状态：已完成并推送；实现提交 `9249915`。Terminal feature flag 继续默认关闭，Sandbox 未实施。
- 依赖与 PoC：锁定 `portable-pty = 0.9.0`、`@xterm/xterm = 6.0.0`、`@xterm/addon-fit = 0.11.0`，并记录于 [`WORKSPACE_TERMINAL_PTY_POC.md`](../research/WORKSPACE_TERMINAL_PTY_POC.md)。Windows ConPTY 已实机；macOS/Linux 只完成跨平台实现和上游 API 核对，真实运行/打包留 W6。
- IPC/授权证据：新增 `terminal_spawn/write/resize/kill/get_state`；WebView 不能提交 cwd、任意 executable、argv 或 env。Rust 从 Chat Session 重读并 canonicalize 工作区，所有后续请求校验 window label + session ID + workspace generation ownership；session 更新/删除与 spawn 共享 mutation lock，避免检查后换目录。
- 进程与数据证据：随机 UUID terminal ID；Windows 以 `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` 回收 shell/grandchild，Unix 保留 PTY process group 并用 SIGHUP/SIGKILL 清理。输出通道以 64 × 8 KiB 形成 512 KiB 有界背压，按最多 32 KiB/16 ms 批处理；output 使用 base64 + seq，exited 携带 `last_seq`。
- 安全与限额：Windows 只解析可信绝对系统 shell，Unix 只接受可执行且 basename 在 zsh/bash/sh/fish allowlist 的绝对路径；清理 MisakaX/Sandbox/Tauri signing/内部 bridge 环境变量。已施加全局/窗口/会话 session 数、输入/输出/命令/resize 速率、64 KiB 单次写入和尺寸范围限制。
- 测试证据：`terminal_manager_tests` 5/5，覆盖真实 ConPTY DSR 握手、CR 输入、resize、Unicode、SGR 颜色、alternate-screen/TUI、非零退出、owner 篡改、缺失 cwd、输出洪水与 shell/grandchild cleanup；terminal unit 4/4。前端 full 35 files / 260 tests，`npm run build`、`cargo check --all-features -j1`、`cargo fmt --check` 和 `git diff --check` 通过。
- 全量回归：`cargo nextest run --all-features --profile ci` 在并发编译阶段受 Windows page file（OS 1455 / `0xc000012d`）限制；按仓库指南回退 `cargo test --all-features -j 1` 后 72 个 unit 及全部 integration/doc tests 通过，未执行 `cargo clean`。
- 回滚：代码回滚到 `f93bf61`；本阶段无数据库迁移，不修改用户工作区内容。显式 kill、window destroyed、会话删除与应用退出均会清理终端进程树。
- 下一步：Workspace Terminal W4，接入 xterm/FitAddon、严格 seq/generation 消费、resize/输入/退出/工作区切换 UI；随后 W5 才能解除 B-05。Sandbox 继续排除。

### 2026-08-01 Workspace W4 xterm UI 与生命周期

- 状态：实现已完成并推送；提交 `2a5b112`。Terminal feature flag 继续默认关闭，Sandbox 未实施。
- UI/生命周期：新增本地打包的 `TerminalPanel`、FitAddon、token 主题和独立 `terminal-store`；panel mode 隐藏保活，真正关闭延迟一个 StrictMode probe tick 后终止 shell。重复 start 按 chat session/generation 去重，显式支持 restart/stop。
- 协议证据：listener 在 spawn 前就绪；output 仅接收当前 aggregate/terminal/generation 与递增 seq，`exited.last_seq` 排空后进入退出态。base64 输出写入 xterm 字节 buffer；UTF-8 `onData` 与 raw `onBinary` 分片串行发送，ResizeObserver 100 ms debounce 后只传 cols/rows。
- UI/安全：固定显示“终端 · 本机权限”，只展示工作区 basename；清屏/复制/粘贴/收起使用 28px ghost 工具按钮。工作区切换必须选择重启或保留；exit/error 就地显示并提供显式重试。未引入 `innerHTML`、WebLinksAddon、link provider 或自动外链，持续输出不进入 live region。
- 测试证据：前端全量 37 files / 267 tests，`npx tsc --noEmit`、`npm run build`、`git diff --check` 通过；定向覆盖 StrictMode 去重、迟到 seq/generation、exit 排空、workspace 选择、二进制/UTF-8 输入、clipboard、resize 和链接禁用。Vite 仅有既有大 chunk 警告。
- Windows 桌面证据：feature flag 临时开启后，Terminal 入口、标题、toolbar、xterm focus 区和 PTY 启动失败/重试态均真实渲染；当前 `terminal_spawn` 被尚未实施的 W5 Tauri capability 拒绝，因此不把 prompt/Unicode/ANSI/保活记为已验证。该结果确认 B-05 是真实上线门，而非文档性清理。
- 环境差异：W3 已验证 Windows ConPTY Unicode/ANSI/alternate screen 与进程树；原生 IME、完整 Windows UI shell 路径和 macOS/Linux 实机仍未据此标完成。
- 回滚：代码回滚到 `2366ce6`；本阶段无数据库迁移，不修改工作区文件。回滚不影响 W3 manager 中已建立的应用退出/会话删除 cleanup。
- 下一步：Workspace W5，先配置最小 custom-command capability、移除通用 shell/fs/http 并建立生产 CSP；随后回到同一 Windows UI 路径补齐真实 shell、Unicode/ANSI、resize 和 mode 保活证据。Sandbox 继续排除。

### 2026-08-01 Workspace W5 Tauri capability、CSP 与 Release 上线门

- 状态：实现已完成、提交并推送；实现提交 `b96d0e3`。B-05 已关闭，Terminal 默认启用；Sandbox 未实施。
- 权限证据：移除 WebView fs/http/notification 插件、Cargo 依赖与初始化；Shell 只保留经统一 helper 校验的 HTTP(S) 外链 open。`default` capability 仅绑定本地 `main` window，包含 event listen/unlisten、shell open、dialog open/save、clipboard read/write、`main-commands` 和精确五项 `terminal-runtime`。
- custom command 证据：`build.rs` 将全部 102 个注册 commands 写入 Tauri AppManifest；`permissions/main.toml` 与 `permissions/terminal.toml` 拆分普通命令和 Terminal runtime，静态测试锁定 handler/manifest/permission 集合并拒绝通配或越权 window。
- CSP/外链证据：production CSP 禁止远端 script、`unsafe-eval`、任意 frame 与远端 connect；HMR origin 只在 `devCsp` 生效，Release `index.html` 只引用本地 `/assets`。外链 helper 拒绝非 HTTP(S)、控制字符、credentials 与超长 URL；Markdown/About/provider 字段均不使用 `window.open` 或 WebView 内导航。
- 终端协议加固：事件名修正为 Tauri 2 合法的 `workspace:context:changed`、`terminal:output`、`terminal:exited`；有界 pre-spawn queue 消除 listener/spawn 竞态。粘贴改用原生 clipboard 并规范化为 CR；恶意 title/OSC/link/output DOM 回归通过。Windows canonical `\\?\` 前缀只在完成安全校验后移除再交给 child shell。
- Rollout：前端 `VITE_MISAKAX_WORKSPACE_TERMINAL` 与后端 `MISAKAX_WORKSPACE_TERMINAL` 默认启用，只有 `false|0` 显式关闭；两侧 gate 必须同时通过。
- 自动化证据：前端全量 38 files / 271 tests；Rust `cargo check --all-features` 与串行 `cargo test --all-features -j1` 通过（73 lib tests 及全部 integration/doc tests，security 5/5、Windows Terminal 5/5）；`npm run build`、`npm run tauri build`、`git diff --check` 通过。Vite 仅有既有大 chunk 警告。
- Windows Release 实机：未设置 feature flag 的 EXE 默认显示 Terminal；首次打开显示真实 PowerShell 工作区 prompt，原生粘贴成功执行 `RELEASE-W5-OK`、中文宽字符与 ANSI 颜色；关闭后验证进程树归零。开发态也通过普通 cwd、中文和 ANSI 路径。
- Release 产物：`misaka-x.exe` SHA-256 `61C8827977488ECFEC8F5351E66E38F684957F61B351D619634B4C59CEC83F36`；MSI `8731A966E10AD275909B5F0501B9D08316E87956109868EB0C1208B7E32A21C8`；NSIS `89B2EBE69A35CF4768B0E08E29C18D41E465C3A05DFC06B71460668FFD0D450E`。
- 审计边界：Tauri 会把同一配置中的非活动 `devCsp` 字面量编译进 Windows EXE，所以字符串扫描仍可看到 localhost；Release 运行时使用严格 production CSP，前端 bundle 无远端资源。不得把“二进制完全没有 dev origin 字符串”误记为已验证。
- 环境差异：本阶段仅证明 Windows 11 当前机与本地未签名 installer；不宣称 Windows 10、macOS/Linux、原生 IME、签名/notarization 或压力矩阵完成。
- 回滚：代码回滚到 `38506da` 会重新关闭入口并恢复旧权限/CSP 基线，不会修改用户工作区内容或数据库；不建议在生产安全边界上部分回滚。
- 下一步：Workspace W6，补充 Windows shell/长路径/压力/离线/诊断证据，并在可用设备上分别验证 macOS/Linux 与签名/发布矩阵。Sandbox 继续排除。

### 2026-08-01–02 Workspace W6 Windows 当前机验证

- 状态：实现提交 `b2521f0`，持续输出测试提交 `7a1f30e`，pwsh 真实命令测试提交 `ed82bfb`；Windows 11 当前机验证已完成，W6 总体仍为 🟡。Sandbox 未实施。
- 环境：Windows 11 Pro 10.0.26200 x64、Windows PowerShell 5、cmd、Git 2.52.0.windows.1；系统未持久安装 `pwsh`，WSL 未安装；已确认 `zh-Hans-CN` Microsoft IME 存在，但原生组合输入受桌面自动化安全边界限制，仍需人工实机确认。
- shell/长路径：普通 profile 与 ConPTY resize/Unicode/ANSI/TUI 回归继续通过；PowerShell 5 可进入超过 260 字符且含中文的 canonical cwd。请求 cmd 进入该 cwd 返回稳定 `shell_workspace_path`，请求缺失的 pwsh 安全回退到 Windows PowerShell 并记录 fallback reason。官方 PowerShell 7.6.3 x64 便携包经 SHA-256 核对后，在隔离临时目录以进程级 `ProgramFiles` 覆盖执行真实 `W6_PWSH_MAJOR=7` / `W6_PWSH_OK`，未安装软件或修改持久 PATH。
- 压力/崩溃：10 MiB 单条突发长行在降低后的 256 KiB/s 测试阈值下触发 `OutputLimit`、有界输出并回收进程；名义 10 MiB/s 节流源在 ConPTY 背压下于 20 秒/32 MiB 边界内正常退出或限流并归零。独立测试 helper 被强制终止后，`KILL_ON_JOB_CLOSE` 回收 grandchild，新 manager 可执行 `W6_REOPEN_OK`；不把源速率误写成 manager 实收吞吐。
- 自动化：`terminal_manager_tests` 10/10（本轮 5.81 s）；增强后的 Windows profile 用例在系统 PowerShell 5 fallback 与官方便携 pwsh 7.6.3 两种进程环境下均通过；前端 38 files / 271 tests；`cargo check --all-features`、`cargo test --all-features -j1`（73 lib tests 及全部 integration/doc tests）、`npm run build`、`npm run tauri build`、`git diff --check` 全部通过。仅保留既有 jsdom Canvas 与 Vite 大 chunk 警告。
- Release 实机：未设置 feature flag 的 `misaka-x.exe` 显示真实工作区 PowerShell prompt，并输出 `W6_RELEASE_OK`、`W6_SUSTAINED_OK`；验证实例的已核对绝对路径进程树已归零。进程级 WebView2 DNS 失败规则 `MAP * ~NOTFOUND, EXCLUDE localhost` 下，Release 仍渲染本地资源、恢复终端并显示真实 prompt，整个进程树远端 TCP 连接为 0；未修改主机网卡或防火墙。
- 新产物：EXE 37,701,632 bytes / SHA-256 `94413D63F73E2DFFE25260D7581F5F8CC91FCA7569A504A0D86F19E974BE8E24`；MSI 16,744,448 bytes / `0CF296194A71A1A66D452DE33D1BCCF902D3A7114060FA2E77B4977A44755FDA`；NSIS 13,088,974 bytes / `17F6F3F094224F9F87743B75ED433F008103F04FF9783A44432D196F4A74DC02`。
- 未覆盖：Windows 10、macOS/Linux、原生 IME 组合输入、旧 Git、物理网卡断开、系统休眠恢复、代码签名/notarization 与三平台正式 installer；Windows 结果不得外推。真实 pwsh 与应用级离线依赖已通过，不以更强的物理断网措辞替代已记录的进程级证据。
- 下一步：在对应 runner/设备上补齐上述矩阵；当前代码无需为未验证平台伪造完成标记。Sandbox 与依赖它的 S4 deep scanner 继续排除。
