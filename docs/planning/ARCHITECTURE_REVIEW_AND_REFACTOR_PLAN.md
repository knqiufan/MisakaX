# 功能完成后的架构审查与安全重构计划

> **用途：** 在 Skills、工作区终端和 Sandbox 功能稳定后，审查并渐进拆解不合理架构，同时严格保持功能行为。
> **受众：** Tech Lead、全栈维护者、测试、安全和发布工程师。
> **最后审阅 / Last reviewed：** 2026-08-01
> **执行前提：** 本轮其他功能达到各自完成定义；没有回归基线时不得开始结构性重构。
> **原则：** 先测量、再抽象、后迁移；一批只改变一个依赖边界。

---

## 1. 目标

- 消除 Skills/扫描/激活、Workspace/Terminal、Sandbox/审批之间的重复策略和反向依赖。
- 让 Tauri commands 变薄、领域逻辑可单测、platform adapter 可替换。
- 降低巨型组件、巨型 service/store、跨层 DTO 和事件竞态。
- 清除迁移期 facade、feature flag、dual-write 和 deprecated API。
- 统一可观测性、错误、配置、生命周期和测试 fixture。
- 保持所有用户功能、数据、快捷键、性能和三平台安全保证不退化。

## 2. 禁止事项

- 不更换 React、Tauri、SQLite、LangGraph/DeepAgents 等核心技术栈。
- 不把“顺便新增功能”混入纯重构 PR。
- 不一次移动整个 `src/`、`src-tauri/src/` 或 `agent/app/`。
- 不在测试失败时用放宽扫描/沙箱策略换取通过。
- 不删除数据库旧字段/feature flag，除非迁移遥测和回滚窗口已满足。
- 不用大范围格式化掩盖语义 diff。

## 3. 启动门

架构审查开始前必须有：

- Skills、Terminal、Sandbox 各自的完成定义和三平台 gate 记录。
- 主用户流程 E2E、数据库 migration/rollback、恶意样本和 sandbox attack suite。
- 性能基线：启动、详情、扫描、composer、terminal、Agent command。
- 错误率、取消/超时、资源和事件 backlog 的可观测数据。
- 可恢复的 release tag/branch 和数据库备份/fixture。

任一缺失时只允许做分析和补基线，不进入结构迁移。

## 4. 审查方法

### 4.1 静态视图

- 模块依赖图和循环依赖。
- 文件/类型/函数复杂度、重复、扇入/扇出。
- React props/store 跨域耦合、effects 和事件订阅清理。
- Rust commands -> services -> repos/platform API 的依赖方向。
- Python orchestration 与 filesystem/shell/bridge 的边界。
- TS/Rust/Python DTO 重复、enum 漂移和自然语言错误解析。

### 4.2 运行时视图

- 一次消息从 composer -> Rust -> Sidecar -> Skill activation -> Sandbox -> stream 的 trace。
- Scan download/quarantine/analyzer/policy/install 的 trace。
- Workspace change -> Git refresh -> panel/terminal lifecycle 的 trace。
- 应用退出/崩溃时 Sidecar、PTY、sandbox child、scanner 的资源清理。

### 4.3 变更分类

| 类别 | 示例 | 处理 |
|---|---|---|
| 安全缺陷 | bypass、权限过宽、fallback host shell | 立即单独修复，不等重构 |
| 架构债务 | command 过厚、重复 policy、平台代码泄漏 | branch-by-abstraction |
| 机械债务 | 文件位置、命名、dead code | 语义稳定后单独移动 |
| 性能债务 | 全文 DTO、事件洪水、重复 hash/Git query | 先 profile，再优化 |
| 产品改动 | 新 Git 功能、多终端 | 移出本计划 |

## 5. 重点审查对象

以下是基于 `fa24bd7` 和目标架构的预期热点，执行时必须重新测量，不能把规划判断当作最终事实：

1. `services/skills/installer.rs`：当前同时处理 inventory、外部发现、detail、install、enable、selection，适合拆为 application use cases 和 adapters。
2. `services/skills/archive.rs`：归档安全、风险检测和安装准备混合，需按 validator/analyzer/artifact store 拆分。
3. `SkillDetailPanel.tsx`/`SkillsPage.tsx`：数据获取、布局、动作和全文渲染耦合；目标按 summary/files/security/actions 拆分。
4. `ChatView`/`ChatPage`/workspace explorer store：Tool Logs、右栏和会话状态可能形成交叉所有权。
5. Python `workspace_backend.py`/`agent.py`：编排、挂载、路径和 Shell 执行需要清晰分层。
6. Tauri capability/default commands：检查是否仍有通用 fs/http/shell 暴露和无 scope custom command。
7. Rust/TS/Python 的 Skill/scan/sandbox DTO：需要单一 schema 来源或契约测试。
8. event names/listeners：检查 listener 泄漏、旧 generation、重复 emit 和错误吞噬。

## 6. Phase R0：建立架构快照和重构预算

### TODO

- [ ] 固定审查 commit/tag，确认工作树和数据库 fixture。
- [ ] 生成 frontend/Rust/Python 模块依赖图，列出循环和跨域 import。
- [ ] 采集文件大小、复杂度、重复、test coverage 和 compile/bundle 指标。
- [ ] 对三个关键流程加 correlation ID/trace，采集 P50/P95 和失败分布。
- [ ] 盘点所有 feature flags、deprecated commands、dual-read/write 和 compatibility facade。
- [ ] 盘点所有后台 task、listener、watcher、PTY、runner、Sidecar 生命周期所有者。
- [ ] 盘点 Tauri capabilities/CSP、bridge tokens、Secrets、日志和数据库敏感字段。
- [ ] 创建 Architecture Decision Log：问题、证据、选择、非选择、迁移/回滚。
- [ ] 按安全/可靠性/维护/性能价值和迁移风险排序，不以“文件太长”单一排序。
- [ ] 为每批重构设定最大 diff/模块范围和可回滚 checkpoint。

## 7. Phase R1：契约和错误统一

先统一边界，再移动实现。

### TODO

- [ ] 为 Rust/TS/Python DTO 建 schema/golden contract tests，选择生成或人工镜像策略。
- [ ] 统一 SkillId、generation、timestamps、severity、scan decision、sandbox mode enum。
- [ ] 统一 `AppError { code, params, retryable, action, correlation_id }`，移除 UI 字符串解析。
- [ ] 统一 domain event envelope、命名、seq/generation 和 subscription cleanup helper。
- [ ] 区分 query DTO、command DTO、event DTO，删除万能 `SkillDetail`/巨型状态对象。
- [ ] 为 terminal bytes、scan progress、Agent stream 定义背压/取消契约。
- [ ] 对旧 command/DTO 增 adapter 和使用计数，确认无调用后再删除。
- [ ] 契约变更逐项跑 E2E，不与目录移动同批提交。

## 8. Phase R2：Rust 分层与服务拆解

目标依赖方向：commands -> application -> domain ports -> infrastructure。

### TODO

- [ ] 把 Tauri commands 收敛为参数校验、scope/ownership 和 service 调用。
- [ ] 从 `SkillInstaller` 提取 Inventory、ArtifactInstall、Activation、FileQuery use cases。
- [ ] 从 `archive.rs` 提取 ArchiveValidator、ContentInventory 和 analyzers。
- [ ] 把 SQLite row/JSON 与 domain types 通过 repo adapter 映射，领域层不依赖 rusqlite。
- [ ] 把 Git CLI、PTY、catalog HTTP、scanner helper、sandbox OS API 放入 infrastructure adapters。
- [ ] 把 Approval/Audit/Network Broker 建为横切 application services，不复制到每个 provider。
- [ ] 确保 TerminalManager 和 SandboxBroker 的 session/process 类型不可互换。
- [ ] 为 application services 使用 fake ports 做 deterministic unit tests。
- [ ] 每抽取一个 seam 运行全套受影响集成测试和基准；无行为变化后再删除旧实现。
- [ ] 最后进行目录机械移动，使用 `rg` 清理旧 import 和 dead code。

## 9. Phase R3：React feature 边界和状态所有权

### TODO

- [ ] 将 Skills 按 inventory/detail/files/security/actions 组织，数据 hook 与纯视图分离。
- [ ] Settings shell 只管理导航/宽度，Skills feature 不修改全局 layout。
- [ ] WorkspacePanel 只管理 open/mode/size；Explorer tabs、Terminal session、Tool Logs 分属自己的 store/hook。
- [ ] WorkspaceContext 使用 query cache/generation，不写入 composer 业务 store。
- [ ] 把 IPC/error/event mapping 集中在 `lib/ipc` 或等价边界，组件不散落 `invoke/listen`。
- [ ] 审计所有 `useEffect` listener、AbortController 和 StrictMode 双调用。
- [ ] 拆分超过约定复杂度的组件，但保持 DOM/ARIA/testid 与视觉快照。
- [ ] 删除迁移期旧 SkillsPage、全文 detail、旧 Tool Logs action 和无调用 hooks。
- [ ] 运行视觉、键盘、i18n、窄窗口和性能回归后再合并每批改动。

## 10. Phase R4：Python Sidecar 边界

### TODO

- [ ] 将 orchestration、model provider、memory、skills activation 和 execution bridge 分包。
- [ ] `BrokeredWorkspaceBackend` 只负责 DeepAgents protocol 适配，不复制 Rust policy。
- [ ] 清除生产 `LocalShellBackend` fallback 和 `inherit_env=true`。
- [ ] 统一 bridge client 的 token rotation、timeout、cancel、retry 和错误映射。
- [ ] 检查 Sidecar 线程/task 生命周期、应用 shutdown 和旧 session token。
- [ ] 评估执行 worker 进程隔离；若引入，先通过 anti-corruption interface，不重写 FastAPI 控制面。
- [ ] 固定 pyproject/Nuitka 依赖，补三平台打包/import/startup tests。
- [ ] 对 Agent streaming、MCP approval、Skill mount 和 Sandbox execution 做端到端 trace 对比。

## 11. Phase R5：数据、迁移和配置清理

### TODO

- [ ] 验证 stable SkillId、scan tables、approvals、message snapshots 的外键和索引。
- [ ] 删除结束 dual-write 前运行一致性查询并保存报告。
- [ ] 清理旧 `risk_json`、slug-only selection、全文 detail 等字段/接口，按版本化 migration 执行。
- [ ] 对 content-addressed artifact/quarantine/audit 制定 retention、vacuum 和配额。
- [ ] 将 sandbox/scanner/settings config 版本化，未知字段和降级行为明确。
- [ ] 测试 upgrade N-1 -> N、失败 rollback、备份恢复和部分 migration crash。
- [ ] 不删除用户外部 Skill 原目录、工作区、终端历史或审计，除非有明确用户动作。

## 12. Phase R6：安全与性能复审

### TODO

- [ ] 重跑 Skill 恶意样本、archive/path、scan bypass 和 activation generation suite。
- [ ] 重跑 sandbox 三平台 attack gate、network bypass、process tree 和 setup/uninstall。
- [ ] 重跑 terminal XSS/OSC/link/ownership/output flood/CSP/capability tests。
- [ ] 对 custom Tauri commands 做 scope/authorization 和 fuzz review。
- [ ] 对 runner/bridge/scanner protocols 做 schema、size、timeout、replay 和 confused-deputy review。
- [ ] 比较重构前后启动、bundle、memory、detail、scan、terminal 和 Agent execution P50/P95。
- [ ] 任何安全保证或 P95 超出预算，回滚该批并分析，不能用扩大权限修复。
- [ ] 安排独立 reviewer 检查平台 sandbox 和 migration，不由原作者单独签字。

## 13. Phase R7：文档与旧路径收尾

### TODO

- [ ] 更新 `PROJECT_STRUCTURE.md`、`DEVELOPMENT_STATUS.md` 和总架构图到真实代码。
- [ ] 更新 Skills/Workspace/Sandbox 设计和用户指南，删除历史错误声明。
- [ ] 更新三份 UI 规范的 Last reviewed 和最终交互。
- [ ] 关闭/删除已完成 feature flags、compatibility adapters 和 deprecated warnings。
- [ ] 用 `rg` 检查旧 route、commands、DTO、event、LocalShellBackend 和 broad permissions 无生产引用。
- [ ] 记录未解决债务、风险、负责人、优先级和不会在本轮解决的原因。
- [ ] 更新实施进度文档，附测试、性能、安全和发布证据链接。

## 14. 每批重构的合并 Gate

- [ ] PR 说明“行为不变”的证据和边界。
- [ ] 受影响单元/集成/E2E/安全测试通过。
- [ ] migration 有 forward/rollback 或明确不可逆审批。
- [ ] 性能没有超预算；bundle/Sidecar 体积变化已解释。
- [ ] capabilities/CSP/policy 没有放宽。
- [ ] 日志不新增 Secret/文件内容泄露。
- [ ] 能以一次 revert 恢复，不依赖手工修数据库。
- [ ] 文档/ADR 在结构或决策变化时同步。

## 15. 完成定义

- 目标依赖方向无已知循环，platform/SQLite/Tauri 细节不泄漏到 domain/UI。
- 生产路径无迁移期双实现、旧 route、全文 detail、整目录 Skills mount 或 host Shell fallback。
- 三平台功能、安全、升级/回滚和性能回归全部通过。
- 架构文档、项目结构和实际代码一致；遗留债务有明确台账。
- 用户行为、数据和可访问性没有回退。
