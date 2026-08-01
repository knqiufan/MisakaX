# Skills 设置、工作区标识与终端 UI 设计

> **用途：** 定义 Skills 设置页、按需文件预览、安全报告、输入框下方工作区标识和右侧终端的交互规范。
> **受众：** 产品、UI/UX、React、Rust IPC 和测试维护者。
> **最后审阅 / Last reviewed：** 2026-08-01
> **状态：** 增量实施中；S1 稳定身份/激活闭环与 S2 Settings/按需文件预览已落地，安全 findings 由 S3、Workspace/Terminal 由 W1–W6 继续实施。
> **上位规范：** [`frontend-ui-guidelines.md`](./frontend-ui-guidelines.md)、[`shell-and-workspace-ui-spec.md`](./shell-and-workspace-ui-spec.md)、[`button-menu-design-spec.md`](./button-menu-design-spec.md)。

---

## 1. 设计原则

- **先结构，后内容。** Skill 详情默认展示文件树和安全状态，不自动读取 `SKILL.md` 全文。
- **状态可解释。** “启用”“健康”“扫描通过”是三个独立概念，不能只用一个绿色圆点表达。
- **桌面密度，不做网页卡片墙。** 使用现有 Zinc token、紧凑行高、轻边框和无缩放弹跳动效。
- **高风险动作就地说明。** 启用被阻止时在当前区域解释原因并给出“查看安全报告/重新扫描”，不只弹 Toast。
- **扩展入口不伪装成已完成能力。** Git badge 首期只读；终端首期单会话/单面板，不显示无功能的提交按钮或空标签。

## 2. 设置导航

设置左侧顺序固定为：

```text
通用
模型
MCP
Skills
外观
关于
```

Skills 紧邻 MCP 下方，使用与其他设置项相同的图标尺寸、选中背景、键盘焦点和 tooltip 规则。旧独立 Skills 入口移除；历史深链重定向后保持选中 Settings > Skills。

Skills tab 需要宽内容模式，但 Settings 的导航和顶部栏不变化。宽度只由 tab descriptor 声明，不能在组件内用负 margin 逃出容器。

## 3. Skills 主布局

桌面宽度充足时使用“仓库列表 + 详情”两栏；详情内部在 Files tab 使用“文件树 + 文件预览”分栏：

```text
┌ Settings ──────────────────────────────────────────────────────────────┐
│ MCP                                                                    │
│ Skills ●  ┌──────── Skill 仓库 ───────┬──────── Skill 详情 ──────────┐ │
│ 外观      │ [搜索] [来源] [状态]       │ skill-name   [开关] [···]    │ │
│ 关于      │                            │ 来源 · 版本 · 扫描状态         │ │
│           │ ● Skill A     已启用       │ [文件] [安全] [概览]           │ │
│           │ ○ Skill B     已禁用       │ ┌ 文件树 ─┬ 文件预览 ───────┐ │ │
│           │ ! Skill C     需复核       │ │SKILL.md │ 选择文件后加载    │ │ │
│           │                            │ │scripts/ │                   │ │ │
│           │                            │ └─────────┴──────────────────┘ │ │
│           └────────────────────────────┴────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

建议尺寸：

- 仓库列表 280–340 px，可调整但保存合理上下限。
- 详情最小 520 px；Files 内文件树 220–300 px。
- 列表、详情、文件树和文件预览各自滚动；页面根不要因为长 `SKILL.md` 产生超长滚动。
- 小于可用阈值时切成列表页 -> 详情页的单栏导航，保留返回按钮和焦点恢复。

## 4. Inventory 行与启用开关

### 4.1 行内容

每行最多两行主信息：名称；来源 + 简短描述/状态。右侧显示语义明确的 Switch 或状态 icon，不在 hover 时才出现关键开关。

来源 badge：`受管`、`Codex`、`Claude`、`Cursor`、在线仓库名。存在同名冲突时必须展示来源并在详情中解释生效优先级。

### 4.2 Switch 语义

- Switch 的 accessible name 为“启用 {skillName}”，并暴露 checked/disabled/busy。
- 点击后进入短暂 pending，完成后由后端事件确认；失败回滚并就地显示原因。
- 禁用立即从选择器和当前输入的 Skill chip 中移除；如消息已发送，历史消息保留快照。
- `scanning`、`blocked`、`stale`、`unhealthy` 时 Switch disabled。旁边原因可点击进入安全 tab，不用 disabled tooltip 承载全部信息。
- 外部 Skill 与受管 Skill 都允许禁用；外部文件仍留在原目录。
- 对 `review_required` 的人工批准是独立动作，不能通过反复点击 Switch 绕过。

## 5. Skill 详情

### 5.1 固定头部

详情头部保持在右栏顶部，包含：

- 名称、来源、版本/路径摘要。
- 启用 Switch、更多菜单（导出/卸载/重新扫描等按来源显示）。
- 健康状态、扫描结论、最后扫描时间；状态使用图标 + 文本，不只靠颜色。

### 5.2 Tabs

固定顺序：`文件`、`安全`、`概览`。

- **文件**默认选中，先返回文件树；没有自动选中文件，也不自动读取 `SKILL.md`。
- **安全**展示结论、严重度分布、策略/引擎版本、扫描时间和 findings。
- **概览**展示描述、兼容性、许可证、允许工具、来源和安装信息，不重复整份 Markdown。

### 5.3 文件树和预览

- 文件树按目录优先、名称排序；支持键盘方向键、Home/End、Enter 打开和正确 tree ARIA。
- 点击文件后显示 skeleton，再调用单文件 read API。切换文件取消旧请求或用 request generation 丢弃迟到结果。
- 文本使用只读 Monaco/轻量代码视图，Markdown 默认显示源码；可提供显式“预览/源码”切换，但不能执行 HTML、脚本或远端资源。
- 大文件首段加载后显示“已加载 200 KiB / 总大小”，用户可继续加载；不可一次把多 MB 放入 DOM。
- 后端单段硬上限 200 KiB、单文件总预览预算 2 MiB、并发读取上限 8；到达总预算时只保留 metadata/已加载源码，不绕过限制继续读取。
- 二进制显示类型、大小、hash 和“不支持内联预览”；图片若未来预览，必须走受控 blob URL 和大小限制。
- 路径 breadcrumb 可复制，但不能通过编辑路径读取 Skill 根外内容。
- 空状态文案：“选择左侧文件以查看内容”，而不是默认塞入 `SKILL.md`。

### 5.4 安全报告

安全 tab 顶部使用中性结论卡，不使用营销式“安全认证”：

- `未发现阻断项`
- `有警告，需要确认`
- `需要人工复核`
- `已阻止安装/启用`
- `扫描失败，尚无结论`

Findings 支持严重度、类别、文件过滤；每行显示 severity、规则标题、文件:行和引擎。展开后显示证据片段、风险、修复建议、误报/例外入口。证据按文本渲染并转义。

重新扫描显示实时阶段和可取消状态。远端扫描服务、VirusTotal 或 LLM 会发送数据时，操作前明确说明数据范围和隐私影响。

## 6. 输入框下方工作区标识

工作区标识属于 composer footer 的上下文信息，与 Model/MCP/Skill 控件在同一底部区域，但视觉优先级更低：

```text
┌──────────────────────── 输入区域 ─────────────────────────┐
│ 输入消息…                                                   │
├─────────────────────────────────────────────────────────────┤
│ ⑂ main / detached:a1b2c3      [Model] [MCP 2] [Skill 1] [↑] │
└─────────────────────────────────────────────────────────────┘
```

非 Git 工作区显示硬盘/文件夹语义图标和“本地项目”。

规则：

- Git 正常分支：GitBranch icon + branch；分支过长中间省略，tooltip 显示完整值。
- Detached HEAD：`detached:<short-sha>`，不能伪装为分支。
- 查询中使用稳定宽度 skeleton，查询失败退化为“本地项目”并在 tooltip 提供诊断。
- 首期是只读 badge，不显示 chevron、不打开菜单、不响应提交/切分支。
- 保留 `WorkspaceContextAction` 扩展接口，但没有 provider 时不渲染空按钮。
- 窄宽度优先保留发送和模型控制；badge 可缩为图标 + tooltip，但不能与 Skill chip 混淆。

## 7. WorkspaceBar 与右侧终端

### 7.1 工具栏行为

- 原“工作日志/Tool Logs”快捷图标替换为真正的 Terminal 操作。
- Terminal 和 Explorer 图标表示右侧 `WorkspacePanel` 的 mode；当前 mode 有选中态，再次点击可关闭右栏。
- Tool Logs 如仍保留，应移动到消息内工具组或 `···` 菜单，并使用日志图标和准确名称。
- Tooltip 使用“打开终端”“打开文件浏览器”，包含快捷键时在右侧显示。

### 7.2 终端面板

```text
┌──────────── WorkspacePanel ─────────────┐
│ Terminal · 本机权限          [清屏] [×] │
│ D:\code\project>                        │
│ █                                       │
└─────────────────────────────────────────┘
```

- 打开时自动在当前会话工作区启动；标题显示 `Terminal · 本机权限`，避免和 Agent 沙箱混同。
- 首期一个会话对应一个终端；重复打开复用存活 session，不自动创建多个 shell。
- panel resize 后 debounce 发送 cols/rows；隐藏时不销毁，关闭时提示/终止仍在运行的 Shell 按产品策略处理。
- 工作区切换且旧终端存活时，提示“在新工作区重启”或“保留旧终端”，不静默 `cd`。
- Shell 退出后显示退出码和“重新启动”，不无限自动重启。
- 复制/粘贴遵循系统快捷键；粘贴多行/疑似危险命令的确认属于后续增强，首期至少不启用终端自动链接执行。

### 7.3 视觉与动效

- 右栏沿用 Explorer 背景、边框和 resize handle；Terminal 不使用独立网页风格标题栏。
- 打开/关闭只使用现有桌面面板位移/透明度时长，不缩放、不弹跳。
- 终端颜色从主题 token 映射，ANSI 色保持可读；高对比主题和中文等宽 fallback 必须实测。

## 8. Loading、空状态和错误

| 场景 | UI |
|---|---|
| Inventory 首次加载 | 固定数量 skeleton 行，避免布局跳动 |
| Summary 加载 | 详情头 skeleton；不请求文件正文 |
| 文件未选择 | 文件树可用 + 预览空状态 |
| 文件读取失败 | 预览区就地错误 + 重试，不清空整个详情 |
| 扫描中 | 状态、阶段、已扫描文件数和取消按钮 |
| 扫描阻止 | Switch disabled + 查看报告/移除操作 |
| Git 不可用 | 本地项目 badge；诊断仅在 tooltip/日志 |
| PTY 启动失败 | 终端面板内诊断、重试和复制错误 ID |
| Sandbox 未配置 | Agent 执行区域提示；不要把本机 Terminal 错误标成 Sandbox 错误 |

## 9. i18n、可访问性与测试 hooks

- 所有新文案进入 i18n；不要把状态 enum 直接显示给用户。
- 状态颜色至少满足 WCAG 对比，并始终配图标/文本。
- Split panel、tabs、tree、switch、menu、terminal toolbar 均有键盘路径和可见 focus ring。
- 建议 hooks：`data-testid=skills-settings`, `skill-enable-{id}`, `skill-file-tree`, `skill-file-preview`, `skill-security-report`, `workspace-context-badge`, `workspace-terminal-toggle`, `workspace-terminal`。
- 屏幕阅读器不逐字符朗读持续终端输出；终端区域按 xterm.js 可访问性建议配置，退出和错误通过独立 live region 通知。

## 10. 实现后必须同步的现有规范

本设计落地时，不得只修改代码；至少同步：

1. [`shell-and-workspace-ui-spec.md`](./shell-and-workspace-ui-spec.md)：Skills 从独立页迁入 Settings；右侧栏从 Explorer-only 变为 Explorer/Terminal；修订“Git/多轨不在范围”的表述为“仅只读 workspace badge，Git 操作仍不在范围”。
2. [`frontend-ui-guidelines.md`](./frontend-ui-guidelines.md)：composer footer 的 WorkspaceContextBadge、Skill 开关/扫描状态和 Terminal 主题约束。
3. [`button-menu-design-spec.md`](./button-menu-design-spec.md)：Skill Switch、扫描动作、WorkspaceBar Terminal toggle 和相关 menu/tooltip。

更新最小相关章节和 Last reviewed，使用交叉链接避免复制整段规则。
