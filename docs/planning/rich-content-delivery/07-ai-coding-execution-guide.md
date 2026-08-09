# 富内容与产物交付：AI Coding 执行说明

> **用途：** 让后续 AI Coding Agent 能在不重新猜测项目状态和安全边界的情况下继续实施。
> **受众：** AI Coding Agent、开发者与代码评审者。
> **最后审阅 / Last reviewed：** 2026-08-09
> **状态：** R0–R4 已依本指南完成；R5、R6 尚未启动。阶段实现和门禁证据见 [实施过程记录](./06-implementation-log.md) 与 [R0–R4 交接](./09-r0-r4-handoff.md)。

---

## 1. 当前事实与目标

MisakaX 是 Tauri 2 + React 19 + TypeScript + Rust 2021 + Python 3.11 Sidecar 的桌面 Agent 客户端。当前聊天已支持 Markdown/Streamdown、Mermaid、数学公式、工具调用、图片**输入**附件和流式文本，但 assistant 消息仍以单一 `content: string` 为主，没有图表、地图、生成文件预览/下载或模型**输出**图片的类型化协议。

本工作流的目标是把 assistant 回复变成有序的、版本化 `ContentBlock` 列表，并通过 Rust `ArtifactService` 处理图片和文件。完整定义见 [00-overall-implementation-plan.md](./00-overall-implementation-plan.md) 与 [02-architecture-design.md](./02-architecture-design.md)。

## 2. 开始任何代码任务前的阅读顺序

1. 本目录的 [README.md](./README.md)、[00-overall-implementation-plan.md](./00-overall-implementation-plan.md)、[01-phased-module-practice-plan.md](./01-phased-module-practice-plan.md)。
2. 本目录中与本次任务对应的架构、功能、UI、安全文档；不要只读任务标题就开始写代码。
3. 项目根 `AGENTS.md`，以及要修改路径范围内的规则/规范。
4. [开发状态与续做指南](../../project/DEVELOPMENT_STATUS.md)，确认 Phase 3/4 当前真实前置条件。
5. 若涉及安全、文件或执行：
   - [Tauri Capability/CSP 审计](../../guides/tauri-capability-csp-audit.md)
   - [Sandbox ADR](../../architecture/SANDBOX_TECH_SELECTION.md)
   - [Sandbox 实施计划](../SANDBOX_IMPLEMENTATION_PLAN.md)
6. 若涉及 UI：
   - [frontend-ui-guidelines.md](../../design/frontend-ui-guidelines.md)
   - [shell-and-workspace-ui-spec.md](../../design/shell-and-workspace-ui-spec.md)
   - [button-menu-design-spec.md](../../design/button-menu-design-spec.md)
   - [Chat UI](../../ui/02-chat.md)、[Markdown/消息规范](../../ui/06-markdown-message-tools.md)

实施前用 `git status --short` 检查工作树。不要覆盖用户已有的无关修改；必要时在实施记录中标记冲突和决策。

## 3. 不可违反的约束

### 3.1 安全

- **禁止**添加 `@tauri-apps/plugin-fs` 或在 capability 中给予通用 `fs:*`、`http:*`、Shell execute/spawn 权限来快速实现预览/下载。
- **禁止**让 React 接受/拼接绝对文件路径、`file:` URL、任意 artifact URI、任意远程 URL 或模型指定的保存路径。
- **禁止**让模型/Agent 传入原生 ECharts option function、HTML/JS/CSS、地图 tile URL、SVG/HTML inline 内容、shell argv 或 Tauri scope。
- **禁止**为地图或预览把生产 CSP 宽化为任意 `https:`、`*`、`unsafe-eval`、任意 `frame-src`。所有 CSP/capability 改动均需专门测试与文档更新。
- **禁止**在 Sandbox 不可用时，将 Agent/Skill/MCP 的外部 converter 或 shell 自动降级到宿主直接执行。
- **禁止**把二进制/大 Base64 写进 `messages.content` 或无限 JSON DTO；用 artifact store + ID 引用。
- **禁止**把在线 Office/第三方网页嵌入当默认预览方案，避免数据外流和 iframe/CSP 风险。

### 3.2 架构与兼容

- Rust 是 artifact 身份、权限、路径、MIME、哈希、配额、导出、清理和审计的唯一权威；React/Python 不能复制安全判断。
- 新消息块必须有 `schema_version`、稳定 `block_id`、消息内 `position`、状态和 safe fallback。
- 旧 `messages.content`、`attachments`、`tool_calls`、流事件、导入/导出必须双读或有兼容 adapter；不得一次性删除旧路径。
- renderer/previewer/provider normalizer 使用 Registry + Strategy + Adapter，不做跨层巨型 `switch` 或“一个万能 RichMessage 组件”。
- 所有新异步事件至少携带 session/message/block identity 和 generation/revision；UI 必须忽略迟到事件。
- UI 中没有硬编码文案，新增 key 同时写入 `src/locales/zh-CN/` 和 `src/locales/en/`。

## 4. 推荐代码阅读地图

| 任务 | 先读的当前文件 |
|---|---|
| 消息 DTO/存储 | `src/lib/ipc/types.ts`、`src/lib/ipc/chat.ts`、`src/stores/chat-store.ts` |
| 流式投影 | `src/hooks/use-stream-listener.ts`、`src-tauri/src/services/llm/streaming.rs`、`services/sidecar_sse.rs` |
| 消息 UI/Markdown | `src/components/chat/message/MessageItem.tsx`、`src/components/chat/markdown/MessageResponse.tsx`、`markdown-components.tsx` |
| 图片/附件兼容 | `src/components/chat/composer/attachmentUtils.ts`、`AttachmentPreview.tsx`、`services/llm/backend.rs` |
| 数据库/会话导入 | `src-tauri/src/db/migrations.rs`、`db/models.rs`、`db/repository/message_repo.rs` |
| Tauri 权限/资源 | `src-tauri/capabilities/default.json`、`src-tauri/tauri.conf.json`、`src-tauri/src/lib.rs` |
| 现有文件边界 | `src-tauri/src/commands/fs_explorer.rs`、`src/lib/ipc/fs.ts` |
| Agent/Sidecar | `agent/app/models.py`、`routers/agent.py`、`stream_content.py`、`src-tauri/src/services/sidecar_*` |

不要仅凭旧文档假定 Sidecar 已接管主对话。当前开发状态明确它是后续 Phase 4 的主线；先以现有 Rig + Tauri event 链路建立兼容合同，再接入 Sidecar adapter。

## 5. 分阶段执行模板

### R0：先建契约和测试

1. 写/更新 TypeScript 与 Rust DTO fixture 测试，覆盖 legacy message、未知 block、非法 schema、顺序与状态。
2. 设计 SQLite migration 和 rollback/compat 行为；审查索引、导入/导出、会话删除。
3. 新增 feature flag，默认关闭 UI 切换；不要在本阶段引入重型 renderer 依赖。
4. 跑精准测试，再跑受影响的前端/Rust 检查；记录结果。

### R1：Artifact Store Spike 后再做实现

1. 在不改 capability 宽度的条件下实现小型 URI Spike；比较 scoped `asset:` 与 custom protocol。
2. 用攻击样本验证路径、MIME、哈希、会话归属与导出取消。
3. 选择方案写入 [06-implementation-log.md](./06-implementation-log.md) 的决策记录，再实现 ArtifactService、metadata、图片和下载 UI。

### R2–R4：按 renderer 独立垂直切片交付

- 每增加一种 previewer/renderer，先建立安全 schema 和 fallback，再增加 UI。
- 重型库动态 import；不存在时与加载失败时可阅读消息、可下载原件。
- R4 远程瓦片是单独子阶段，不得随静态地图一起悄悄修改 CSP/network。

### R5–R6：后接真实 Agent、MCP 与 Sandbox

- 将 provider/Sidecar/MCP 输出收敛到同一 `RichOutputAdapter` 和窄工具契约。
- 对外部命令/转换器，先验证相应 Sandbox provider；没有 strict guarantee 就 disabled + diagnostic。
- 完成三平台、离线、资源/恶意样本、安全回归，再标记发布就绪。

## 6. 验证命令与最低测试矩阵

根据变更范围选择，而不是无差别执行所有慢任务。项目不应在日常工作前运行 `cargo clean`。

```powershell
# 前端类型与打包
npm run build

# 前端测试（优先运行相关 test；必要时全量）
npm test

# Rust：在 src-tauri/ 下，优先增量精准验证
cargo check
cargo test --test <targeted_test>
cargo nextest run --all-features --profile ci  # 提交前，若已安装

# Sidecar：仅改动 Python 或协议时
cd agent
python -m pytest
```

每个富内容阶段的最低验收还包括：

- 旧 Markdown、现有用户图片输入、工具调用、流式停止/重新生成、消息分页、导入/导出。
- 深色/浅色、窄窗口、键盘、屏幕阅读器摘要、减少动态偏好。
- URI 越权/路径 traversal/MIME confusion/超限/取消/文件缺失。
- 若引入网络或外部执行，CSP/capability diff 与 Sandbox policy 回归。

## 7. 阶段完成门禁：代码审查、本地验证、Git 与远程 CI

**R0–R6 的每个阶段都是一个独立交付单元。未完整通过本节门禁前，严禁启动下一阶段的代码开发、依赖接入或迁移。** 下一阶段可以做不改代码的调研和计划细化，但不得开始实现。

### 7.1 固定顺序

阶段任务看似完成后，严格按以下顺序执行：

1. **收口阶段范围。** 确认变更只覆盖当前 R 阶段；将临时调试、无关格式化、无用依赖和未完成实验从提交中移除或拆开。
2. **代码审查通过。** 完成变更差异审查和当前阶段的安全/兼容性检查：查看 `git diff`、调用链、迁移、错误处理、i18n、测试、CSP/capability、日志脱敏及本目录的验收项。仓库/团队要求人工 reviewer 时，必须取得实际批准；AI 不得虚构或自行替代人工审批。没有人工审批要求时，在实施记录中写明已完成的 self-review 清单和发现/修复结果。
3. **本地测试通过。** 运行第 6 节中所有与改动相关的精准测试、构建及安全回归；若当前阶段改变跨层协议、迁移、CSP/capability 或共享聊天渲染，必须补跑相应全量测试。所有结果必须为 pass；不可把已知失败、跳过、超时或“理论上应通过”记作通过。
4. **更新实施证据。** 在 [06-implementation-log.md](./06-implementation-log.md) 记录实际修改、代码审查结果、本地命令及输出摘要、未验证项、风险和下一步；涉及 UI 时同步规定的 `docs/design/` 文档。
5. **提交当前阶段。** 用一个可审阅、可回滚的 Conventional Commit 提交当前阶段已验证变更。提交前再次检查 `git status --short`，不得把用户无关改动、凭据、构建产物、缓存或本地测试数据纳入提交。
6. **推送提交。** 以非强制方式推送当前分支到已配置的远程仓库；首次推送使用 upstream。记录 commit SHA、分支和远程 URL/CI 运行链接（如可获取）。
7. **等待远程 CI 全绿。** 只以远程仓库对该 commit/分支报告的 required CI 状态为准。所有必需检查都成功、没有 queued/running/pending/neutral/未配置项，并且所需人工审查已满足后，才可把当前阶段标记为“完成”并启动下一阶段。

### 7.2 Git 操作约束

```powershell
# 收口和审查当前阶段差异
git status --short
git diff --check
git diff --staged

# 阶段通过本地门禁并更新记录后
git add <only-current-phase-files>
git commit -m "feat(rich-content): complete r1 artifact delivery"
git push -u origin <current-branch>  # 仅首次；后续使用 git push
```

- 分支遵循仓库 `codex/` 前缀和项目既有分支/PR 规则；未经明确授权不得推送到受保护主分支，也不得使用 `--force`、`--force-with-lease` 或改写已推送历史。
- 一个阶段有多次 CI 修复时，修复仍归属**同一阶段**：每次修复都要重新执行代码审查和受影响本地测试，再以新的、清晰的 fix commit 推送。不要通过 `commit --amend` 改写已被远程 CI 检查的提交。
- 提交说明应体现阶段、实际交付和风险边界，例如 `feat(rich-content): complete r3 chart renderer`、`fix(rich-content): handle expired artifact previews`；不要使用含糊的 `update`、`wip` 或把多个阶段混在一个提交中。

### 7.3 远程 CI 失败、不可用或等待中的处理

| 远程状态 | 当前阶段状态 | 允许动作 | 禁止动作 |
|---|---|---|---|
| queued/running/pending | 待 CI | 监控运行、整理证据、修复 CI 明确暴露的当前阶段问题 | 启动下一阶段实现 |
| failed | CI 修复中 | 定位失败，修复当前阶段，重复 7.1 的审查/本地测试/提交/推送 | 以本地 pass 忽略远程失败；跳到下一阶段 |
| required review 未批准 | 待审查 | 等待/请求实际 reviewer；补充说明 | 虚构批准或自行绕过分支保护 |
| CI 未配置/无法查询 | 阻塞 | 在实施记录标明远程、commit、原因并请求用户/仓库维护者提供 CI 验证方式 | 自行视为 CI 已通过或进入下一阶段 |
| all required checks passed | 已完成 | 更新阶段看板和实施记录，启动下一阶段 | — |

远程 CI 的验证应使用仓库实际提供的状态页、PR checks 或已配置的连接器/CLI；“`git push` 返回成功”仅代表上传成功，**不是** CI 通过。等待期间若用户要求停止、切换任务或 CI 需要人工权限，记录当前 commit/状态后安全停下。

## 8. 依赖选择与研究要求

所有新增依赖先看本项目锁定版本和许可证，再用官方文档验证当前 major/minor API。建议评估而不是在未验证前承诺：

- 图表：Apache ECharts，显式 ARIA 组件、dataset/encode 方案。
- 地图：MapLibre GL JS，本地 bundle、CSP worker 与 WebGL 回退。
- PDF：PDF.js，本地 viewer/worker，不走 `file:` 或在线服务。
- XLSX：可从 `ArrayBuffer` 读取的本地解析器；禁止公式/宏执行。

详细的官方来源和结论见 [08-research-sources.md](./08-research-sources.md)。若候选依赖改变 CSP、WebWorker、WASM、许可证或 native binary 边界，必须先做小型 Spike 和安全评审。

## 9. UI 文档同步与交付

只要实施了 UI，除本目录记录外，必须按项目规则把可复用的规范同步到合适的 `docs/design/` 文件并更新 Last reviewed 日期。不能只把规则留在代码注释或 PR 描述中。

每次完成一个工作包，更新 [06-implementation-log.md](./06-implementation-log.md)：修改范围、审查结果、本地验证、commit/push/远程 CI 证据、未验证项、迁移/回滚、安全影响、下一步。只有远程 CI 全绿后才可把该阶段更新为完成。最终交付前核对 [00-overall-implementation-plan.md](./00-overall-implementation-plan.md) 第 7 节成功标准。

## 10. 交接提示

如任务只要求文档、调研或设计，不要擅自修改生产代码/依赖/CSP。若任务要求实施而关键产品选择缺失（例如保留期、地图数据提供商、在线网络同意、文件格式范围），先从已有设置、需求或本目录决策门中查找；仍无法确定且选择会扩大权限或影响用户数据时，停下并请求产品决策。
