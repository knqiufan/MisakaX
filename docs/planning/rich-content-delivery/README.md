# MisakaX 富内容与产物交付实施文档集

> **用途：** 为聊天中的统计图、地图、生成文件预览与下载、以及多模态模型图片输出建立一条可分阶段实施、可审计且默认安全的路线。
> **受众：** 产品、React、Rust、Python Sidecar、安全、测试，以及后续参与实现的 AI Coding Agent。
> **最后审阅 / Last reviewed：** 2026-08-13
> **状态：** R0–R4 已完成并通过远程 CI；R5/R6 的执行隔离前置 S0 合同已实现，R5 producer integration、真实 Provider 与 R6 发布 Gate 仍待实施。

---

## 先读什么

| 目标 | 必读文档 |
|---|---|
| 了解总体范围、优先级和决策 | [00-overall-implementation-plan.md](./00-overall-implementation-plan.md) |
| 拆分任务并开始实施 | [01-phased-module-practice-plan.md](./01-phased-module-practice-plan.md) |
| 修改协议、Rust、数据库或渲染结构 | [02-architecture-design.md](./02-architecture-design.md) |
| 实现具体用户行为、配置和异常状态 | [03-functional-design.md](./03-functional-design.md) |
| 实现聊天 UI、组件样式和交互 | [04-ui-ux-design.md](./04-ui-ux-design.md) |
| 评估文件系统、网络、解析器或 Sandbox 影响 | [05-filesystem-and-sandbox-research.md](./05-filesystem-and-sandbox-research.md) |
| 接手任务、选择命令、验证与更新记录 | [07-ai-coding-execution-guide.md](./07-ai-coding-execution-guide.md) |
| 查找调研依据或本仓库现状证据 | [08-research-sources.md](./08-research-sources.md) |
| 记录每轮实施、决策、验证和遗留风险 | [06-implementation-log.md](./06-implementation-log.md) |
| 交接已完成的 R0–R4 及其门禁证据 | [09-r0-r4-handoff.md](./09-r0-r4-handoff.md) |

## 一句话决策

聊天消息从“只有 Markdown 字符串”演进为“有序、版本化的内容块”；图表、地图、文件和图片都通过 Rust 持有的 `ArtifactService` 交付。React 只接收可显示的元数据和受限 URI，不能获得任意文件路径或通用文件系统权限。

完整的跨平台执行隔离 **不是**展示图表、静态地图、图片或下载文件的前置条件；但由 Agent/Skill/MCP 运行外部程序、处理不可信复杂文件或访问网络时，必须进入 Rust `Execution Isolation Broker`（沿用 `SandboxBroker` 名称），采用 snapshot 交付、受验证 manifest/artifact 回收、默认断网和 capability evidence，不能退化为宿主进程直接执行。

## 文档边界

- 本目录只规定富内容与产物交付能力及其执行隔离挂接点；不替代工作区、Skills、MCP 和跨平台 Sandbox 总体 ADR。
- [Sandbox 方案复核](../../research/SANDBOX_STRATEGY_REASSESSMENT_2026-08.md) 已于 2026-08-13 纳入仍为 Proposed 的 Sandbox ADR；S0 合同已实现。涉及生产执行路径时必须继续完成 S0.5 和对应 Spike Gate；不得按旧 Windows 专用账户或 macOS Seatbelt 假设建设。
- R0–R4 已新增契约、数据库迁移、受控 artifact store、预览、图表和本地 GeoJSON 地图；每阶段的实现和远程 CI 证据见 [实施过程记录](./06-implementation-log.md) 与 [R0–R4 交接](./09-r0-r4-handoff.md)。
- 后续任何实现必须保持现有 Markdown、Mermaid、数学公式、会话导入导出和图片输入附件的兼容性。

## 与现有文档的关系

- 现有安全权威与修订输入：[Sandbox ADR（Proposed，S0 已实现）](../../architecture/SANDBOX_TECH_SELECTION.md)、[Sandbox 实施计划](../SANDBOX_IMPLEMENTATION_PLAN.md)、[2026-08 方案复核](../../research/SANDBOX_STRATEGY_REASSESSMENT_2026-08.md)、[Tauri Capability/CSP 审计](../../guides/tauri-capability-csp-audit.md)。
- 聊天视觉基线：[Chat UI](../../ui/02-chat.md)、[Markdown 与消息工具](../../ui/06-markdown-message-tools.md)、[设计总规范](../../design/frontend-ui-guidelines.md)。
- 代码真实进度与先后依赖：[开发状态与续做指南](../../project/DEVELOPMENT_STATUS.md)。

当本目录的方案与已接受的安全 ADR、Capability/CSP 审计或项目 `AGENTS.md` 冲突时，以后者为准。当前 ADR 仍为 Proposed；S0 只冻结执行合同与 fail-closed 门禁，不把任何本地/远程 Provider 宣称为已接受或可发布。生产实现继续以 S0.5 与对应 Spike Gate 为准。
