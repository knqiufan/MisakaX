# 富内容与产物交付实施过程记录

> **用途：** 记录实际实施、验证、决策变更、风险与下一步，保证人类和 AI Agent 接手时可追溯。
> **受众：** 所有实施者与评审者。
> **最后审阅 / Last reviewed：** 2026-08-09
> **状态：** R0、R1、R2 已通过远程全量 CI。R3 图表已完成本地验证，待阶段提交、推送与远程 CI；R4 尚未开始阶段提交。

---

## 使用规则

1. 每次实际代码、依赖、迁移、CSP/capability、行为或设计文档变更后，追加一条记录；不要重写旧记录。
2. 每条记录必须包含范围、修改文件、代码审查、本地验证、commit/push/远程 CI 状态、风险/回滚和下一步。没有执行验证时明确写“未执行”及原因。
3. 安全决策、scope/CSP/Sandbox 策略、格式 allowlist、保留策略和 schema version 变更必须有独立 ADR 链接或本文件的决策条目。
4. 不记录 API key、绝对用户路径、完整私有文件内容、会话内容或未脱敏命令参数。
5. 完成某一阶段时，先满足 [AI Coding 执行说明 §7](./07-ai-coding-execution-guide.md#7-阶段完成门禁代码审查本地验证git-与远程-ci)：代码审查和本地测试通过 → commit → 非强制 push → required CI 全绿（及所需 reviewer 批准）。之后才可更新 [01-phased-module-practice-plan.md](./01-phased-module-practice-plan.md) 的状态为完成并启动下一阶段；CI 失败/不可查询时必须保持当前阶段未完成。
6. 涉及 UI 时同步相应 `docs/design/` 文档。

## 阶段看板

| 阶段 | 状态 | 负责人 | 开始 | 完成 | 证据/备注 |
|---|---|---|---|---|---|
| R0 契约/安全基线 | 已完成 | 当前实施者 | 2026-08-09 | 2026-08-09 | `b1ed884`；[CI #31271639940](https://github.com/knqiufan/MisakaX/actions/runs/31271639940) 的 8 项检查全绿 |
| R1 ArtifactService/图片/下载 | 已完成 | 当前实施者 | 2026-08-09 | 2026-08-09 | `53a079d`；[CI #31272842590](https://github.com/knqiufan/MisakaX/actions/runs/31272842590) 的 8 项检查全绿 |
| R2 文件预览 | 已完成 | 当前实施者 | 2026-08-09 | 2026-08-09 | `2e979e1`；[CI #31285793427](https://github.com/knqiufan/MisakaX/actions/runs/31285793427) 的 8 项检查全绿 |
| R3 图表 | 本地验证完成，待门禁 | 当前实施者 | 2026-08-09 | — | 受限 ChartSpec、ECharts richText、可访问数据表和 ArtifactService CSV 导出；待 commit/push/CI |
| R4 地图 | 未开始阶段门禁 | 待分配 | — | — | 候选工作区改动未提交、未验证、未推送；R4b 不在范围内 |
| R5 Agent/Sidecar/MCP | 未开始 | 待分配 | — | — | 依赖 Phase 4 真正对话链路；通过阶段门禁后完成 |
| R6 加固/发布 | 未开始 | 待分配 | — | — | 三平台/沙箱 gate；通过阶段门禁后完成 |

## 决策记录

| 日期 | 决策 | 理由 | 影响 |
|---|---|---|---|
| 2026-08-08 | 采用版本化 ContentBlock + Renderer Registry，不采用 Markdown 内嵌脚本或 renderer middleware | 富内容需要独立持久化、顺序、状态、无障碍和下载生命周期；middleware 不适合作为块调度模型 | R0 schema、R3/R4 renderer、R5 adapter |
| 2026-08-08 | 所有输出文件/图片进入 Rust ArtifactService，二进制不进入 `messages.content` | 控制 I/O、哈希、配额、导出、清理和 URI 授权，避免 Base64 膨胀 | R1 DB/IPC/存储迁移 |
| 2026-08-08 | 不为本功能添加 WebView 通用 FS/HTTP/Shell 权限 | 现有 Tauri capability/CSP 已是最小权限；安全下载/预览可经 Rust 窄接口实现 | R1 URI/export、R4 网络设计 |
| 2026-08-08 | OS Sandbox 不阻塞静态富内容 MVP，但外部执行/高风险转换必须接 SandboxBroker | 内容渲染与 OS 执行隔离是不同安全问题；不能用任何一者替代另一者 | R2、R5、R6 与 Sandbox B0–B7 对齐 |
| 2026-08-08 | 地图先支持本地 GeoJSON，远程瓦片后置且必须使用 provider registry/broker | 避免模型驱动任意网络请求和 CSP 扩张 | R4a/R4b 切分 |
| 2026-08-09 | 每个 R 阶段必须在代码审查和本地测试通过后 commit/push，并等待 required CI 全绿后才能进入下一阶段 | 防止未验证阶段叠加、保留可回滚检查点，并使远程仓库成为阶段完成的权威证据 | R0–R6 执行顺序、阶段看板和交接流程 |

## 实施记录

### 2026-08-08 — 规划与调研基线建立

- **范围：** 建立本目录的总体方案、分阶段实践、架构、功能、UI/UX、文件系统/Sandbox 调研、执行指南和资料索引。
- **修改：** 新增 `docs/planning/rich-content-delivery/` 下的实施文档；未修改 `src/`、`src-tauri/`、`agent/`、依赖或 Tauri 配置。
- **本地现状核验：** 阅读了聊天消息 DTO/Store/Event、Markdown renderer、图片输入附件、Rust LLM backend、SQLite message schema/repository、workspace FS commands、Tauri capability/CSP 和现有 Sandbox/开发状态文档。
- **外部调研：** Tauri FS/permissions/CSP/asset protocol、ECharts accessibility/dataset、MapLibre CSP/worker、PDF.js、SheetJS、MDN iframe sandbox。完整链接见 [08-research-sources.md](./08-research-sources.md)。
- **验证：** 文档路径与链接尚待本轮最终链接/文件检查；未运行构建或测试，因为未改业务代码。
- **风险/待定：** R1 必须完成 asset protocol vs custom URI Spike；R2 必须在选 parser 前完成许可证、资源上限和恶意文件 fixture 评审；R4b 需产品确认瓦片服务与隐私规则。
- **下一步：** 由实施者按 [07-ai-coding-execution-guide.md](./07-ai-coding-execution-guide.md) 执行 R0，先创建契约测试和迁移设计评审。

### 2026-08-09 — 阶段 Git/CI 门禁补充

- **范围：** 为后续 AI Coding 执行增加逐阶段审查、本地验证、commit/push 和远程 CI 门禁。
- **修改：** 更新执行说明、分阶段计划与本实施记录模板；未修改产品代码、依赖、迁移或 Tauri 配置。
- **代码审查：** 文档交叉引用与阶段顺序已核对；新增规则禁止在 CI 失败、等待、不可查询或所需 reviewer 未批准时启动下一阶段。
- **验证：** 本目录 Markdown 本地链接检查 → pass；`git diff --check` → pass。
- **未验证：** 未运行构建/测试，因为本次只改文档。
- **Git：** 未提交；由当前文档维护任务的提交策略决定。
- **远程 CI：** 不适用（仅文档编辑，尚未创建提交）。
- **风险/回滚：** 若仓库未配置可查询的 required CI，阶段将按规则阻塞并请求维护者明确验证方式。
- **文档同步：** [01-phased-module-practice-plan.md](./01-phased-module-practice-plan.md)、[07-ai-coding-execution-guide.md](./07-ai-coding-execution-guide.md)。
- **下一步：** 开始 R0 前遵循新增 §7 门禁。

### 2026-08-09 — R0–R4：本地实现与验证

- **范围：** 落地版本化 `ContentBlock`/`ArtifactRecord`、SQLite 迁移与兼容导入导出；建立应用专属 artifact store、窄 IPC/原生保存、图片与文件预览；交付受限图表和仅本地 GeoJSON 地图 renderer。
- **修改：** `src-tauri/src/services/content/`、`services/artifacts/`、`db/migrations.rs`、repository/commands/contracts/config；`src/features/chat-content/`、消息渲染接线、IPC DTO、双语 i18n 与预览依赖；具体清单见 [R0–R4 交接](./09-r0-r4-handoff.md)。
- **迁移/兼容性：** 新增迁移 v14 的 `message_blocks` 与 `artifacts` 表；`messages.content` 保留，旧消息不含 blocks 时继续 Markdown 渲染；导出包含 block 与 artifact manifest，导入 manifest 以 `expired` 记录保留而不假装带有二进制。
- **安全影响：** 新块写入和读取均由默认关闭的 `MISAKAX_RICH_CONTENT_WRITE` / `MISAKAX_RICH_CONTENT_RENDER` 控制；artifact ingress 校验 session、魔数/MIME、SHA-256、原子写入、配额、路径边界、像素上限与 Office 宏标记；SVG/HTML/未知二进制不进入预览。图表只编译 allowlist spec，地图不接受 tile URL/远程资源。
- **代码审查：** 完成 self-review，修复了受条件调用 Hook、PDF.js 6 canvas 参数、MapLibre 严格类型、Mammoth 浏览器声明、CSV 公式导出与 feature flag 双读接缝。
- **验证：** `cargo fmt --check`、`cargo check` → pass；`cargo test --lib content` → 5 passed；`cargo test --lib artifact` → 8 passed；`npm test -- --run` → 38 files / 271 passed；`npm run build` → pass（Vite 对大懒加载 chunk 给出性能 warning）。
- **未验证：** 未进行三平台手工预览、恶意压缩包/加密 Office 压测或远程 CI；未运行完整 Rust suite。`npm install` 报告 9 个 audit 项，未自动执行可能改变依赖树的修复。
- **Git：** 未提交、未推送（本次请求未授权外部 Git 写入）。
- **远程 CI：** 未执行；因此按 §7 门禁，R0–R4 不能标为正式阶段完成，也不得据此启动 R5 实现。
- **风险/回滚：** 关闭两个 rich-content flag 即回退旧消息路径；迁移保留旧字段。删除迁移/新表前须先导出或备份。R4b 的瓦片/网络/CSP 扩张明确未实施。
- **文档同步：** [frontend-ui-guidelines.md](../../design/frontend-ui-guidelines.md) §4.6.x.1 与本实施记录；新增 [R0–R4 交接](./09-r0-r4-handoff.md)。
- **下一步：** 审查 diff，按阶段拆分 commit/push 并取得 required CI/reviewer 证据；之后再决定是否启动 R5。

### 2026-08-09 — R0：提交、推送与远程 CI 门禁

- **范围：** 仅提交 R0 的 ContentBlock/Artifact 契约、迁移、兼容性、默认关闭 feature flag、Repository 与契约测试；未纳入 R1–R4 的服务、renderer、依赖、UI 或交接文档改动。
- **代码审查：** 已检查暂存差异与 `git diff --cached --check`；确认 migration v14 保留 legacy `messages.content`，未知块退回非执行 fallback，且未添加 CSP/capability/通用文件权限。未发现本阶段需修复项。
- **验证：** `cargo fmt --check` → pass；`cargo check` → pass；`cargo test --lib content` → 5 passed / 0 failed。
- **未验证：** R0 不涉及前端 renderer；完整 Rust suite、人工 reviewer 与三平台手工测试未执行。
- **Git：** `6012b1db73fc19a3be39f76fc24e3aee56ba0d41`（`feat(rich-content): establish r0 contracts and migrations`）已非强制推送至 `origin/codex/rich-content-r0-r4`。
- **远程 CI：** GitHub REST API 查询该 commit 的 `check-runs` 和 Actions runs，均返回 0。远程未配置或无法查询 required CI；按执行说明 §7.3，此阶段为“阻塞”，不能开始/推送 R1–R4。
- **风险/回滚：** 该 R0 commit 可独立回滚；保留的 uncommitted R1–R4 本地改动不属于已通过门禁的交付。
- **下一步：** 请仓库维护者配置或指定 R0 的 required CI 验证方式（以及需要的人工审批）；验证全绿后，更新本记录并再开始 R1 的阶段审查、测试、提交和推送。

### 2026-08-09 — R0：启用分支 CI 并修复远程 Clippy

- **范围：** 扩大现有 CI 的 push 触发范围至全部分支；修复 R0 两个 repository 的 Clippy 兼容性。未纳入 R1–R4 的服务、renderer、依赖或 UI 改动。
- **代码审查：** CI 仍保留 `pull_request` 对 `main`/`master` 的限制，只有 `push` 触发扩展为 `"**"`，以便阶段分支得到同一套远程门禁。远程失败日志定位到 `artifact_repo.rs` 与 `message_block_repo.rs` 的 `repeat().take()`；替换为等价且不分配额外数据的 `std::iter::repeat_n()`。
- **验证：** `cargo fmt --check` → pass；`cargo clippy --all-targets --all-features -- -D warnings` → pass；`cargo test --lib content` → 5 passed / 0 failed。
- **Git：** `3064b81`（`ci: run phase checks on feature branches`）已推送并实际触发 CI；本条记录随 R0 Clippy 修复提交推送。
- **远程 CI：** [CI #31268907752](https://github.com/knqiufan/MisakaX/actions/runs/31268907752) 证明新分支触发已生效。前端及三平台 Terminal Runtime 均成功；Rust 作业在 Clippy 阶段因上述两处 lint 失败，Tauri Build 因依赖失败被跳过。本地修复后等待下一次完整远程运行。
- **风险/回滚：** 该 lint 修复不改变 SQL placeholder 数量或顺序；若需要回退，可单独还原 CI 触发和两个 iterator 表达式。
- **下一步：** 推送 R0 修复并等待所有远程 required checks 全绿，再开始 R1。

### 2026-08-09 — R0：迁移夹具回退与全量回归

- **范围：** 修复 R0 migration v14 对既有迁移夹具和备份测试的影响；未改变生产 schema、R1–R4 服务、renderer、依赖或 UI。
- **代码审查：** 第二轮 CI 的 Rust 作业通过 Clippy 后，在 `test_migration_idempotent` 暴露硬编码的 v13 版本断言。进一步全量回归确认测试辅助函数在构造 v5/v10/v11/v12 fixture 时没有移除 v14 表和 schema 记录，导致重复建表、跳过前移和备份未生成。新增 `revert_v14`，在各旧版本辅助路径先删除 v14 的表、索引和版本记录；该函数仅用于测试夹具，真实迁移仍为前向单向执行。
- **验证：** 隔离 target directory 中 `cargo test --all-features --test db_migrations_tests` → 19 passed / 0 failed；`cargo nextest run --all-features --profile ci` → 439 passed / 0 skipped；`cargo fmt --check` → pass；`cargo clippy --all-targets --all-features -- -D warnings` → pass。
- **Git：** 本条记录随 R0 migration fixture 修复提交推送。
- **远程 CI：** [CI #31269092419](https://github.com/knqiufan/MisakaX/actions/runs/31269092419) 的前端和三平台 Terminal Runtime 成功；Rust nextest 在 v14 fixture 兼容性失败，Tauri Build 被依赖关系跳过。已完成本地全量修复，等待下一次远程运行。
- **风险/回滚：** 只改变测试辅助代码和版本断言；可独立回退，不影响用户数据库。
- **下一步：** 推送并等待 R0 的完整远程 CI 全绿。

### 2026-08-09 — R0：Windows ConPTY CI 启动竞态修复

- **范围：** 仅稳定现有 Windows Terminal Runtime 集成测试的首条握手时机，以完成 R0 的远程门禁；未改变终端服务或 R0 生产功能。
- **代码审查：** [CI #31271436559](https://github.com/knqiufan/MisakaX/actions/runs/31271436559) 中 Rust、前端、macOS 与 Ubuntu 作业均成功，Windows 的 `windows_profiles_and_long_unicode_workspace_round_trip` 未收到首条 PowerShell 命令。日志表明 ConPTY 已启动但冷启动 PowerShell 尚未就绪。测试辅助函数只在 Windows 将既有 300ms 等待提升到 1 秒；命令、断言和超时覆盖均未放宽。
- **验证：** `cargo test --all-features --test terminal_manager_tests -- --nocapture` → 10 passed / 0 failed；`cargo fmt --check` → pass；`cargo clippy --all-targets --all-features -- -D warnings` → pass。
- **Git：** 本条记录随 CI 竞态修复提交推送。
- **远程 CI：** 上述运行的 Windows 作业失败使 Tauri Build 跳过；本地已复现该测试集通过，等待下一次远程全矩阵验证。
- **风险/回滚：** 只增加 Windows 测试初始化等待 700ms；不改变运行时代码、用户终端行为或安全边界。
- **下一步：** 推送并等待 R0 全部远程检查通过。

### 2026-08-09 — R1：ArtifactService 后端与窄 IPC

- **范围：** 提供应用私有 content-addressed artifact store、magic/MIME/尺寸/配额校验、原子写入、会话归属、过期清理、预览元数据、原生保存对话框与窄 Tauri IPC。重型预览 renderer、图表和地图仍留待 R2–R4。
- **代码审查：** Artifact 路径只由 Rust 从 application data 目录解析；写入在 session 存在性校验后才执行；导出使用原生 dialog 并验证写出哈希；未添加 WebView FS/HTTP/Shell permission 或 URL/path 入口。会话删除前过期所属 artifact，保留数据库审计记录。
- **验证：** `cargo fmt --check` → pass；`cargo check` → pass；`cargo test --lib artifact` → 7 passed / 0 failed。
- **未验证：** R1 不接入重型前端 renderer；图像放大/通用文件 preview 将由 R2 覆盖。
- **Git：** `ab7f1ee`（后端/IPC）、`1181618`（Clippy）、`53a079d`（命令 manifest 与权限白名单）均已非强制推送至 `origin/codex/rich-content-r0-r4`。
- **远程 CI：** [CI #31272842590](https://github.com/knqiufan/MisakaX/actions/runs/31272842590) 的 Rust、Frontend、三平台 Terminal Runtime 与三平台 Tauri Build 共 8 项检查全绿。
- **风险/回滚：** 关闭 rich-content feature flags 可保持旧消息路径；删除 artifact 数据前会先将记录标为 expired，并只删除无 active 引用的字节文件。
- **下一步：** R1 已完成；可开始 R2 的独立阶段审查与实现收口。

### 2026-08-09 — R1：远程 Clippy 兼容性修复

- **范围：** 修复 [CI #31272376136](https://github.com/knqiufan/MisakaX/actions/runs/31272376136) 暴露的 R1 lint；未改变 ArtifactService 的输入、存储、访问控制或 IPC 行为。
- **代码审查：** `limit_text` 改用 `enumerate` 保持相同的零起始行数上限；`register_base64` 与已有 `register_bytes` 一样声明窄入口的多参数例外，避免为迎合 lint 而弱化 IPC 的显式字段。初始 R1 commit 的前端与三平台 Terminal Runtime 均通过，Rust 仅在 Clippy 阶段失败。
- **验证：** `cargo fmt --check` → pass；`cargo clippy --all-targets --all-features -- -D warnings` → pass；`cargo test --lib artifact` → 7 passed / 0 failed。
- **Git：** `ab7f1ee` 已推送；本条记录随 R1 Clippy 修复提交推送。
- **远程 CI：** 修复后的运行先在 [CI #31272566088](https://github.com/knqiufan/MisakaX/actions/runs/31272566088) 通过 Clippy，但安全基线发现注册命令未同步至 Tauri AppManifest/权限白名单；补充修复后，最终 [CI #31272842590](https://github.com/knqiufan/MisakaX/actions/runs/31272842590) 8/8 成功。
- **风险/回滚：** 仅 lint 等价改动，可单独回退。
- **下一步：** R1 required CI 已全绿，可进入 R2。

### 2026-08-09 — R1：命令清单/权限同步与阶段完成

- **范围：** 将 R1 新注册的 artifact 与 content-block 命令同步至受控 AppManifest、`main-commands` 最小权限白名单及生成 schema；未引入通用 FS、HTTP 或 Shell 权限，也未混入 R2–R4 UI/依赖。
- **代码审查：** 对照 `invoke_handler`、`build.rs` 和 `permissions/main.toml` 三处命令集合，确认 8 个 artifact/block 命令均为 Rust 窄接口，终端权限集不变。安全基线的集合一致性测试修复后通过。
- **验证：** `cargo fmt --check` → pass；`cargo test --all-features --test security_config_baseline_tests` → 5 passed / 0 failed。
- **未验证：** 未进行 R2–R4 的 preview/renderer 手工场景；这些内容未纳入 R1 提交。
- **Git：** `53a079d`（`fix(rich-content): authorize r1 artifact commands`）已非强制推送至 `origin/codex/rich-content-r0-r4`。
- **远程 CI：** [CI #31272842590](https://github.com/knqiufan/MisakaX/actions/runs/31272842590) completed/success；8 个 required 作业均成功，R1 阶段门禁已满足。
- **风险/回滚：** 回滚 `53a079d` 会恢复安全基线拒绝行为，且不能保留 R1 的注册命令；正常回滚 R1 时应将服务和该权限提交一并回退。
- **文档同步：** 本实施记录的阶段看板和 R1 证据更新。
- **下一步：** 开始 R2；先把候选工作区改动收口为仅文件预览、图片展示与下载回退，再执行独立审查、测试、commit/push/CI。

### 2026-08-09 — R2：本地文件预览、图片查看与下载回退

- **范围：** 在 Assistant 消息中接入有序内容块 dispatcher、Artifact/图片卡和只读预览 Dialog。文本、CSV、PDF、XLSX 与 DOCX 仅在用户打开预览后动态加载；未支持或解析失败的文件保留原件下载。图表/地图 renderer 与其导出 IPC 不在本阶段提交。
- **修改：** 新增 `src/features/chat-content/` 的 R2 renderer、payload reader 与 block ErrorBoundary；`MessageItem` 在有 blocks 时走有序 renderer、无 blocks 时保持 Markdown；新增 PDF.js、SheetJS、Mammoth 依赖及浏览器声明，补齐双语 `chat.richContent.*` 文案和 UI 规范。
- **代码审查：** 确认 Renderer Registry 对 chart/map 仍映射到非执行 notice；文件 bytes 只能经 artifact 窄 IPC 获取，未接受路径、`file:`、HTML、SVG 或远端 URL；DOCX 仅抽取 DOM `textContent`，不注入转换出的 HTML。每个块由 ErrorBoundary 隔离，图片/文档失败不会中断相邻 Markdown。发现阶段拆分后未提交的 R3/R4 renderer 仍需其导出 IPC 类型，已保留在未暂存候选切片中，未混入 R2。
- **验证：** `npm run build` → pass（Vite 提示部分动态依赖 chunk 大于 500 kB，未阻塞）；`npm test -- --run` → 38 files / 271 passed。Vitest/JSDOM 输出 `HTMLCanvasElement.getContext` 未实现诊断，但测试进程 exit 0。
- **未验证：** 未执行三平台人工 PDF/XLSX/DOCX/图片预览、恶意加密 Office/压缩炸弹压测或屏幕阅读器实测；不把这些未执行项当作通过。R3/R4 renderer 尚未进行阶段审查、提交或推送。
- **Git：** `2e979e1`（`feat(rich-content): complete r2 file previews`）已非强制推送至 `origin/codex/rich-content-r0-r4`。
- **远程 CI：** [CI #31285793427](https://github.com/knqiufan/MisakaX/actions/runs/31285793427) completed/success；Rust、Frontend、三平台 Terminal Runtime 与三平台 Tauri Build 共 8 项检查全绿。
- **风险/回滚：** 移除 R2 renderer/依赖即可恢复 R1 的后端 artifact 能力；关闭 `MISAKAX_RICH_CONTENT_RENDER` 继续显示 legacy Markdown。预览始终为只读，任何失败保留下载路径。
- **文档同步：** `docs/design/frontend-ui-guidelines.md` §4.6.x.1；本实施记录。
- **下一步：** R2 已完成；可开始 R3 的独立阶段审查与实现收口。

### 2026-08-09 — R3：受限图表、数据表与 CSV 导出

- **范围：** 启用 `chart` 块的前端 renderer，动态加载 ECharts，提供 metric/data-table fallback、ARIA 标注和通过 ArtifactService 的 CSV 导出。地图保持未注册 fallback，未纳入本阶段。
- **修改：** 新增前端 ChartSpec reader（24 series、5000 points、512 字符标签上限）和 `ChartBlockRenderer`；新增 `chart_export_csv` Rust command、AppManifest/最小权限/schema 同步与窄 IPC。CSV 逐字段转义，且为 `= + - @` 前缀加 apostrophe，避免表格公式注入；导出前验证消息归属当前会话。
- **代码审查：** ChartSpec 仅映射固定图类型，未接收原始 ECharts option/HTML/function/外部 URL；ECharts tooltip 固定为 `richText`，不使用 HTML renderer；加载失败降级到同一份数据表，`role="img"`/ARIA 与键盘可达的数据表操作并存。未发现会扩大 CSP、FS/HTTP/Shell capability 的变更。
- **验证：** `cargo fmt --check` → pass；`cargo clippy --all-targets --all-features -- -D warnings` → pass；`cargo test --all-features --lib chart` → 2 passed / 0 failed；`cargo test --all-features --test security_config_baseline_tests` → 5 passed / 0 failed；`npm run build` → pass（动态 chunk 大小 warning）；`npm test -- --run` → 38 files / 271 passed（JSDOM canvas diagnostic，exit 0）。
- **未验证：** 未进行真实屏幕阅读器、手工深浅主题/窗口缩放或大量 series 的交互回归；R4 地图代码、MapLibre 依赖和 GeoJSON 导出未纳入暂存。
- **Git：** 待提交；暂存范围为 R3 图表、CSV 导出、受控命令权限、ECharts 依赖与本记录。
- **远程 CI：** 待 R3 commit 推送后运行。
- **风险/回滚：** 回滚本阶段可恢复 chart→notice fallback；CSV 导出临时 artifact 会在客户端导出流程完成后 expire，不产生通用文件写入权限。
- **文档同步：** R2 已同步的 `frontend-ui-guidelines.md` §4.6.x.1 对图表的通用规范继续适用；本实施记录补充实现证据。
- **下一步：** 审查 R3 暂存差异、提交并等待 remote CI 全绿；之后才开始 R4。

## 后续记录模板

```markdown
### YYYY-MM-DD — R?：<短标题>

- **范围：**
- **修改：**
- **迁移/兼容性：**
- **安全影响：**
- **代码审查：** <self-review/reviewer、结论、发现与修复>
- **验证：** `<command>` → <结果>；手工场景 → <结果>
- **未验证：** <原因或“无”>
- **Git：** `<commit SHA>` / `<branch>`；push → <远程/结果>
- **远程 CI：** <运行链接/required checks/状态；未通过或不可查询时说明阻塞原因>
- **风险/回滚：**
- **文档同步：**
- **下一步：**
```
