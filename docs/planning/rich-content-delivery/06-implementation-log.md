# 富内容与产物交付实施过程记录

> **用途：** 记录实际实施、验证、决策变更、风险与下一步，保证人类和 AI Agent 接手时可追溯。
> **受众：** 所有实施者与评审者。
> **最后审阅 / Last reviewed：** 2026-08-09
> **状态：** R0 已提交并推送，但远程仓库未报告可查询的 CI；按阶段门禁阻塞。R1–R4 的本地改动不得推送或标记完成，直到维护者提供并通过 R0 的远程验证方式。

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
| R0 契约/安全基线 | 远程 CI 未配置，阻塞 | 当前实施者 | 2026-08-09 | — | `6012b1d` 已推送；迁移/双读/默认关闭 flag/安全测试已完成，本地 pass；GitHub API 未报告 check run 或 workflow run |
| R1 ArtifactService/图片/下载 | 本地实现完成，待门禁 | 当前实施者 | 2026-08-09 | 2026-08-09 | 窄 IPC + 原生保存对话框；未引入宽 URI scope；未 commit/push/CI |
| R2 文件预览 | 本地实现完成，待门禁 | 当前实施者 | 2026-08-09 | 2026-08-09 | 本地只读预览、资源上限和下载回退；未 commit/push/CI |
| R3 图表 | 本地实现完成，待门禁 | 当前实施者 | 2026-08-09 | 2026-08-09 | 受限 spec、ARIA、表格与 CSV 产物导出；未 commit/push/CI |
| R4 地图 | 本地实现完成，待门禁 | 当前实施者 | 2026-08-09 | 2026-08-09 | 仅本地 GeoJSON/no tiles；R4b 未开始；未 commit/push/CI |
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
