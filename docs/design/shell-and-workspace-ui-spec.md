# MisakaX 壳层布局与工作区 UI 规范

| 属性 | 说明 |
|------|------|
| **用途** | 定义主窗口混合壳结构、任务侧栏、对话页顶栏、设置页与工作区布局语义。 |
| **受众** | 负责 `AppShell`、`UnifiedTopBar`、`SessionPanel`、`SettingsSidebar`、`ChatPage`、`WorkspaceBar`、`SettingsPage` 及相关布局的前端开发者。 |
| **最后审阅** | 2026-08-01（v29） |

## 相关文档

- **全局动效、色彩、组件原则（桌面 Agent 气质、charcoal 主色）**  
  → [frontend-ui-guidelines.md](./frontend-ui-guidelines.md)

- **按钮、图标按钮、下拉菜单、Popover、Select、Dialog、Tooltip、Toast 等控件细节**  
  → 凡实现或调整上述控件，**须对照** [button-menu-design-spec.md](./button-menu-design-spec.md)。

- **可复刻参考（CodePilot 逆向）** → [`docs/ui/01-overall-and-home.md`](../ui/01-overall-and-home.md)、[`docs/ui/03-workspace.md`](../ui/03-workspace.md)、[`docs/ui/04-settings.md`](../ui/04-settings.md)

---

## 1. 混合壳信息架构

MisakaX 主界面（本期 Win / 跨平台**实心底**，无 macOS 悬浮卡）为**单列左栏**（对齐 `docs/ui` ChatList 模型；**无**独立 NavRail）：

```
AppShell (flex-col h-screen)
  [非 chat] UnifiedTopBar   <!-- h-10；返回 + 标题；chat 不渲染，避免空顶条 -->
  ContentRow (flex-1)
    [chat | settings] LeftColumn (默认 240px，可拖 180–300)
      chat     → SessionPanel（含底栏）
      settings → SettingsSidebar
    [chat | settings] ResizeGutter (8px)
    Main → ContentArea
```

1. **顶栏 `UnifiedTopBar`**：**仅非 chat** 渲染（返回 → chat + 页面标题）；chat 页不渲染，对话 chrome 由 WorkspaceBar / Hero 承担。
2. **左栏（单列）**：
   - `chat`：任务列表 + Quick actions + 仅含用户菜单的底栏。
   - `settings`：整列换成 `SettingsSidebar`（六分区导航）。
   - `skills` / `knowledge` / `dashboard` / `notifications`：**无左栏**，仅 TopBar + 主内容。
3. **主内容区**：当前页面；无任务时为居中 hero（见 §6）。
4. **Sidecar / Agent 状态**：不在任务底栏展示；放在设置 → 关于 → 系统信息（`SidecarStatusBadge`，异常时可点重启）。

### 1.1 左栏显隐与窄屏（必须遵守）

- chat / settings 时左栏**常显**，禁止顶栏整栏折叠开关。
- **无** `sessionListOpen` / `sidebarCollapsed`；宽度由 `sessionListWidth` 持久化。
- `LG_BREAKPOINT = 1024px`：窄于该宽度时，展示宽度夹到 `min(偏好宽, 180)`（`resolveLeftColumnWidth`），**不**隐藏左栏、**不**写回偏好；≥1024 恢复用户偏好宽。

### 1.2 左栏宽度（必须遵守）

| 常量 | 值 |
|------|-----|
| 默认宽 | `240px` |
| min / max | `180` / `300` |
| 持久化 | `localStorage.misakax_chatlist_width` |
| 分隔 | `ResizeGutter` 宽 8px；默认线不可见，hover/drag 显线；双击重置 240 |

- Gutter 必须是左栏的**兄弟节点**，禁止放进 SessionPanel / SettingsSidebar 内部。
- Chat 页内仅保留「消息 ↔ Explorer」的 `react-resizable-panels`；**不再**用百分比 Panel 承载任务列表。
- Chat ↔ Explorer 分隔：命中区约 8px（`w-2`），默认细线不可见，hover/drag 显线。

### 1.3 视觉连续性

Session / Settings 左栏 / Main 使用 `--sidebar`、`--border`、`--background` 等暖色 charcoal token；选中态用 `bg-sidebar-accent`，禁止冷蓝描边。

---

### 1.4 系统托盘与窗口关闭

- 系统托盘属于桌面壳层，不以 React Popover 替代原生菜单。启动后保留品牌图标；左键恢复并聚焦主窗口，右键使用系统原生菜单。
- 托盘菜单的项目上下文以当前活动任务的 `working_directory` 为唯一来源，名称可使用 `project_name` 作为展示值。目录不存在或无活动任务时，项目说明项保持禁用，“打开当前项目”禁用；“新建任务”改为打开现有工作目录选择器。
- 原生菜单顺序固定为：打开应用、禁用的当前项目说明、新建任务、打开当前项目文件夹、设置、退出；菜单文本随应用语言更新。项目级置顶和移除仍属于任务栏内项目右键菜单，不重复到托盘。
- 主窗口关闭策略必须由持久化的 `close_behavior` 控制：`ask` 弹出应用内 Dialog，`minimize_to_tray` 仅隐藏窗口，`quit` 才结束进程和 Sidecar。询问 Dialog 复用标准 Dialog、Switch 与按钮焦点/动效规范，不添加缩放或弹跳动效。
- 通用设置使用 `FieldRow` 右侧 Select 暴露三种关闭策略。用户在询问 Dialog 勾选“记住”后，所选最小化或退出策略应立即持久化，并在下次启动时生效。

---

## 2. 任务列表（SessionPanel）

### 2.1 Quick actions

- 自上而下：`新建任务` 全宽 ghost 行（`h-9`、`rounded-xl`、`text-[13px]`）+ 搜索输入（同高、同圆角）。

### 2.2 底栏（SessionPanelFooter，必须遵守）

固定在任务列表底部（`shrink-0`），仅保留一个 `UserMenu` 触发器，行形与 Quick actions 一致（`h-9 rounded-xl text-[13px]`）。

- 用户菜单顺序：禁用的个人中心 → 通知（保留未读徽标）→ Settings → 分隔线 → 技能 / 知识库 / 仪表盘 → 分隔线 → 禁用的退出。
- 通知与 Settings **不得**作为底栏独立行重复出现。

Sidecar 状态**不**放在底栏（见 §1 / 关于页）。

### 2.3 任务行（SessionItem）

- 行高 `h-8`、`rounded-xl`、`px-3`、标题 `text-[13px]` 单行截断。
- 右侧时间与 ⋯ 菜单共用固定宽 `52px` 的尾部容器（需容纳中文日期如「12月31日」与英文 `Dec 31`）；时间文案必须 `whitespace-nowrap` + `leading-none`，禁止换行。
- hover、选中或菜单打开时仅显示菜单按钮，其他状态仅显示时间，二者不得重叠；菜单按钮仍右对齐于尾部容器。
- 激活：`bg-sidebar-accent`；过渡 `duration-150`。

### 2.4 分区标题

- `text-[13px] font-semibold text-sidebar-foreground/55`；可折叠分组 caret 12px。

### 2.5 工作目录优先分组（新增）

- **一级键**必须是规范化后的 `working_directory`（完整路径），**禁止**仅按 `project_name` 合并——同名不同路径的项目不得混在一组。
- 分组头显示项目名 / 默认工作区本地化名；完整路径放 Tooltip；旁注任务数量。
- 手工 `group_name` 作为该工作区内的**二级**可折叠分组；无手工组的任务直接列在该工作区下。
- **置顶任务**只影响同工作区内排序，不得把 session 抽离出其工作目录分组。
- 搜索结果与归档视图使用同一分组规则。
- collapse key 须带工作区命名空间（如 `workspaceKey::groupName`），避免相同手工组名跨目录联动折叠。

### 2.5.1 项目右键菜单与持久化偏好

- 每个工作目录分组（包括默认工作区）的标题均可通过 `ContextMenu` 打开右键菜单；菜单项目按「置顶项目／取消置顶项目 → 新建任务 → 在文件资源管理器中打开 → 分隔线 → 移除项目」排序，并为每项使用 `size-4` Lucide 图标。
- 置顶与移除状态使用本机 SQLite 的 `workspace_preferences` 保存，键必须与规范化 `working_directory` 分组键一致；置顶项目在其他项目之前排序，任务自身置顶规则不变。
- 「移除项目」仅隐藏该项目，不删除任务或消息；在同一目录新建任务时必须自动恢复显示。资源管理器打开失败时使用现有 `workspace.explorer.openInExplorerFailed` toast。

### 2.6 新建 / 修改工作区选择器 intent（新增）

- `chat-store` 使用显式 `workspaceSelectorIntent`：`new-session` | `change-session`（外加目标 session id）。
- **禁止**用「当前是否已有 activeSession」推断选择器意图——否则在已有任务时点「新建」会误改当前目录。
- `new-session`：确认后创建新 session 并 `upsertSession` 激活；创建失败时当前任务不变。选择器打开期间不提前清空当前任务。
- `change-session`：只更新捕获的目标 session id；打开期间切换任务不得把目录写到另一个 session。
- 「跳过 / 使用默认工作区」绑定应用管理目录 `~/.misakax/workspace`，`workspace_kind=default`；显式选目录为 `custom`。

---

## 3. 对话页顶栏 — 工作目录（Workspace bar）

### 3.1 信息层级

工作目录条是 Agent 场景的**关键上下文**。

- **未设置目录**：标题（`text-sm font-medium`）+ 短说明（`text-xs muted`）+ 主 CTA「选择目录」。
- **已设置目录**：文件夹显示名作标题；完整路径 `font-mono text-[10px] text-muted-foreground/60` 单行截断 + Tooltip。
- **`workspace_kind === "default"`**：标题使用本地化「默认工作区」名称，Tooltip / 副标题仍展示受控路径 `~/.misakax/workspace`（平台实际 home）。

### 3.2 布局与令牌

- 高度约 TopBar 量级：`min-h-10`，`px-3 py-1.5`，底边 `border-border/40`，表面 `bg-background`（无重 blur / 大图标槽）。
- 图标操作：`ghost` + `size-7`；Explorer / Terminal **当前打开 mode** 用 `variant="secondary"`，再次点击同 mode 关闭，点击另一 mode 打开并切换。
- Explorer/Terminal 统一传递 `WorkspacePanelMode` action，禁止分别维护两套 open boolean。无工作目录时不执行 toggle；Terminal 在 W3/W4 完成前由默认关闭的 feature flag 隐藏。
- Tooltip 使用准确动词与快捷键：Explorer 为 `Ctrl/Cmd+Shift+E`，Terminal 为 `Ctrl/Cmd+反引号`；两者必须有 `aria-pressed`、`aria-keyshortcuts` 与 focus-visible ring。
- Tool Logs：与 WorkspacePanel **语义解耦**；入口固定为消息内工具组的日志按钮，挂载仍是 WorkspaceBar 下方聊天列抽屉（`max-h-[min(40vh,320px)]`），头 `h-10` + 11px uppercase + 关闭 X。
- 控件细节见 [button-menu-design-spec.md](./button-menu-design-spec.md)。

### 3.3 Composer footer 工作区标识

- `WorkspaceContextBadge` 位于 composer footer 左侧，和右侧 Model/MCP/Skill 控件共享一行但视觉优先级更低；使用 `h-6`、紧凑圆角表面、`text-[11px]` 与 `GitBranch`/硬盘语义图标。
- 正常分支显示 ref；detached 显示 `detached:<short-sha>`；长 ref 中间省略。非 Git或查询退化均显示“本地项目”，路径仍只在既有 WorkspaceBar 展示，不复制到 badge tooltip。
- badge 是状态文本而非操作按钮：无 hover 位移、无 chevron/menu、无 stage/commit/checkout 行为；小窗口优先隐藏文字、保留图标和可访问名称。
- 首载 skeleton 保持稳定宽度；后台 focus/watcher 刷新不改变 composer 可输入性。具体 generation、诊断脱敏与 IPC 约束见 [Skills/Workspace/Terminal UI 设计 §6](./SKILLS_WORKSPACE_TERMINAL_UI_DESIGN.md#6-输入框下方工作区标识)。

---

## 4. 设置页

参考视觉：[`docs/ui/04-settings.md`](../ui/04-settings.md)。**本期 IA** 为六分区：`general` / `models` / `mcp` / `skills` / `appearance` / `about`（不扩 overview / runtime / health 等）。

### 4.1 导航壳

- 进入 settings 时，**壳层左栏整列**换成 `SettingsSidebar`（与任务列表共用 `sessionListWidth` / gutter）。
- 项形：`h-9 px-3 rounded-xl text-[13px]`（与任务 Quick actions 同形）。
- `SettingsPage` **只渲染内容槽**；不再内嵌桌面左导航或窄屏横条 pill（避免双导航）。
- Back 在 `UnifiedTopBar`（ghost sm、`h-7`、ArrowLeft）；**所有非 chat 页**（含 settings 与技能/知识库/仪表盘/通知）均提供返回 chat。

导航唯一源：`src/components/settings/nav-config.ts`。路由仍为 Zustand `route.page === "settings"`（无 React Router）。

### 4.2 内容槽与页宽

- 内容：`p-6 lg:p-10`；必须 `mx-auto`。
- 多数分区：`max-w-4xl` + `space-y-10`；Appearance / About：`max-w-3xl` + `space-y-6`。
- tab descriptor 使用 `contentWidth: normal | wide`；normal 继续保留上述 `3xl/4xl` 阅读宽，只有 Skills 使用 `wide`（`max-w-6xl`）。禁止由 feature 用负 margin 或绝对定位逃出内容槽。

### 4.3 卡片与表单

| Primitive | 规格 |
|-----------|------|
| `SettingsCard` | `rounded-lg border-border/50 bg-card p-5`，**无阴影** |
| `SettingsSubCard` | `rounded-md bg-muted/40` + inset `divide-y divide-border/50` |
| `FieldRow` | label/description 左、Switch/Select 右；禁止手写 `flex justify-between + Switch` |
| `StatusPill` | `text-[10px]` + `size-1.5` 色点（Provider / MCP 运行态） |

Provider 目录网格仅 `md:grid-cols-2`。Appearance 主题分段：`rounded-md bg-muted p-0.5`；**无**强调色选择器；**无**「侧边栏默认展开」折叠开关（左栏已无折叠态）。

模型选择器 / 供应商弹窗遵循以下规则；控件跟 [button-menu-design-spec.md](./button-menu-design-spec.md)。

- 中文界面将 Provider 统一表述为「供应商」；选择供应商品牌时，将其本地化名称同步写入供应商名称字段，用户仍可随后改写。
- 模型类型以紧凑图标标识，并始终提供 Tooltip 文字；获取结果与已保存模型都显示同一类型标识。自动识别只作为初始值，用户可在模型行内通过可多选的 checkbox 图标标签改为文本、多模态、语音、嵌入、重排或图像类型，禁止为此使用下拉框。
- MCP 添加服务器弹窗提供「表单填写 / JSON 输入」两个互斥入口。JSON 入口一次只解析一个服务器；HTTP 表单在 URL 下方提供 JSON 请求头，错误必须就近显示且不得静默丢弃配置。
- Max tokens 使用固定预设（200K、220K、273K、300K、1M）或自定义模式；预设以 radio 标签组呈现，禁止使用下拉框。仅自定义模式展示数值输入框，新建供应商默认 220K。

### 4.4 Settings > Skills 仓库

- Skills 是 Settings 第四个分区，固定置于 MCP 与 Appearance 之间，沿用 `SettingsSidebar`、`UnifiedTopBar` 与 resize gutter；独立 Skills route/title/nav/page wrapper 已移除，所有入口直接导航到 `{ page: "settings", tab: "skills" }`。
- Settings 内容槽为 Skills 使用 `wide` + `max-w-6xl`，同时保持 `h-full min-h-0 overflow-hidden`；禁止新增第二套侧边栏、网页式 Hero 或 feature 内负 margin。
- 顶部固定信息层级为标题/短说明、已安装/在线发现切换、搜索、来源筛选与「上传安装」主操作；列表与详情在宽屏为紧凑双列，在窄宽度自然纵向排列。
- 在线与已安装详情共用固定头部和 `文件 / 安全 / 概览` tabs。普通远端详情只取仓库元数据，不自动下载或解包制品；本地文件也只在用户点击后按相对路径读取。
- 上传、导入、卸载等风险操作必须使用现有 Dialog，并说明 ZIP 预检或删除影响；导出/下载必须先使用系统文件保存对话框。

#### 4.4.1 Skills 双栏面板与长列表

- 宽屏双栏必须在页面、网格与两侧面板链路上同时保留 `min-h-0` 与 `min-w-0`（含 `SkillsContent` 网格列、`SkillsList` / `SkillDetailPanel` 根节点）；页面外层不承担 Skills 列表或详情正文的滚动，左侧列表和右侧详情分别使用自己的滚动容器。窄屏自然改为页面纵向滚动，不得把固定高度带到移动布局。
- 列表和详情使用相同的卡片表面、边框与标题栏层级；列表卡片只承载选择和快捷操作。列表项标题单行截断；描述使用最多两行（`line-clamp-2` + `break-words`），禁止单行硬裁切导致卡片内文字贴边消失。
- 详情头部与 tabs 固定；Files 内文件树和预览是两个独立 `ScrollArea`，其他 tab 各自滚动。正文容器与预览块必须 `min-w-0` / `max-w-full`；长 YAML/URL 行用 `whitespace-pre-wrap` + `break-words` / `overflow-wrap: anywhere` 换行，不得横向溢出后被父级 `overflow: hidden` 吃掉。
- 长列表先渲染 10 项，接近列表底部时再追加下一批；追加期间使用就地、非阻塞的旋转加载反馈。已安装 Skills 按来源分组，组标题展示来源和总数量，卡片内不重复来源文字。
- 切换「已安装 / 在线发现」或在线来源时，必须清除当前选中项和详情，并忽略较早详情请求的迟到结果，避免跨上下文保留旧 Skill 内容。
- 文件树默认可见但不默认选中文件，展示相对路径、类型和大小，目录按需展开并分页；未点击文件时正文读取次数必须为 0。文本源码按 200 KiB 分段显式继续加载；二进制/不支持编码显示 metadata 空态。远端未缓存制品必须解释“安装或显式扫描后检查”，不能暗中下载。
- Security tab 的摘要、隐私契约、findings 和审批历史只在首次打开该 tab 后加载；findings 使用后端 cursor 分页，筛选变化必须丢弃旧页。JSON/SARIF 导出先走系统保存对话框，不在 WebView 中拼接下载链接。
- `review_required` 就地显示原因输入与批准/拒绝；少于 3 个非空字符时动作 disabled。批准后的受管 Skill 仍为 disabled，已有有效批准跨详情重开/应用重启可恢复并可撤销；blocked 结论不能显示批准捷径。
- 首次升级扫描状态条位于 toolbar 与双栏内容之间，宽度跟随 Skills 内容槽；pending/running 展示 `completed / total`，`completed_with_errors` 展示失败数与显式重试，completed 不占布局空间。

---

## 5. 对话页右侧 WorkspacePanel

参考视觉：[`docs/ui/03-workspace.md`](../ui/03-workspace.md)（**仅 chrome**；Misaka **不**复刻其多轨 / 480px 像素宽模型）。**产品模型**为单一 `WorkspacePanel` 容器，mode 仅有 Explorer（文件树 + Monaco）与 Terminal；不做 Git/Widget/Assistant 多轨。

宽度：Chat↔WorkspacePanel 用 `react-resizable-panels` **百分比**（默认约 70/30，Panel `min 18%` / `max 55%`）；**不以** CodePilot `SIDEBAR_DEFAULT_WIDTH=480` 为 Misaka 目标。分隔命中区 `w-2`，默认细线不可见，hover/drag 显线（与任务栏 gutter 气质一致；类名集中在 `panelResizeHandle.ts`）。视口 `<= 960px` 时右栏改为右侧 overlay（`width: min(88%, 520px)`），不压缩聊天列并支持 Escape 关闭。

### 5.0 状态边界

- Panel store 只管理 `open/mode/size/sessionId/workspaceGeneration`，且只持久化 `open/mode/size`；Explorer tabs/activePath 与未来 Terminal session/output 分属各自 store/manager。
- 旧 Explorer store 的 `open` 只迁移一次到新 panel key；迁移不得带走或清除 tabs。任务/工作区切换立即更新 runtime session/generation，旧 generation 不得覆盖新绑定。
- Explorer 与访问过的 Terminal 内容槽在 mode 隐藏时保留；切 mode 只改变可见性。Terminal 未完整交付时必须保持 rollout flag 默认关闭，不能把准备占位文案当成功能入口。

### 5.1 表面与头

- 表面：`bg-background` 不透明；左边框 `border-border/40`。
- 面板头：`h-10`；标题 `text-[11px] font-semibold uppercase tracking-wider text-muted-foreground`；右：刷新 + 收起（X 14 / `size-3.5`），`ghost` icon。
- 有打开文件时：左侧树列「FILES」小头与总头同高（`h-10`）+ 同款 11px uppercase；右侧编辑列用 Tab + 文件信息双行 header。

### 5.2 编辑器 chrome

| 元素 | 规格 |
|------|------|
| Tab 条 | **无** `border-b`；`px-2 pt-1.5 pb-3`；胶囊 `rounded-full`；激活 `bg-muted`；动态宽 40–160px；标签 `text-sm font-medium` |
| 关 Tab | 图标区与文字激活区**拆分**为两个 button；hover 时前导图标 ↔ X 交叉淡入；点图标关、点文字激活 |
| 键盘 | ArrowLeft/Right 循环、Home/End；`role="tab"` + `aria-controls`；内容区 `role="tabpanel"` |
| 文件信息行 | `h-12 border-b border-border/40`；文件名 `text-xs font-medium`；路径 `text-[10px] mono muted/60`；**脏点在右侧控件族**（Save 左侧）；保存钮 `size-7` |

### 5.3 Tool Logs（聊天列抽屉）

- 与 Explorer **解耦**：不占右轨、不进 Explorer Tab。
- 入口位于每个消息 `ToolActionsGroup` 汇总行右侧，使用日志图标与“工具日志”可访问名称；只打开聚合抽屉，不改变该消息分组的展开状态。
- 挂载：WorkspaceBar 下方；`border-b border-border/40`；头 `h-10` + uppercase 标题 + 关闭 X；内容区可滚动，高度上限约 `min(40vh, 320px)`。
- 空态：居中 `text-sm muted`（i18n `chat.toolLogs.empty`）。

---

## 6. 对话消息 / Markdown / 思考与工具

参考：[`docs/ui/02-chat.md`](../ui/02-chat.md)、[`docs/ui/06-markdown-message-tools.md`](../ui/06-markdown-message-tools.md)。

### 6.1 宽度契约

聊天主列可读内容统一：`mx-auto w-full max-w-3xl px-4`（列表、Composer、空态）。

### 6.2 气泡

| 角色 | 规格 |
|------|------|
| User | `max-w-[95%] ml-auto`；`rounded-2xl bg-muted px-4 py-3 text-sm text-foreground`（非 primary 实心） |
| Assistant | **无**卡片底/圆角外壳；正文直接铺画布 |
| Error | `rounded-xl` + destructive 边框/字色 |

### 6.3 Markdown

- 管线：`MessageResponse` → `streamdown` + `@streamdown/code|math|mermaid|cjk`
- 覆盖：`src/components/chat/markdown/markdown-components.tsx`（标题/leading-7、围栏 `rounded-xl bg-muted/20`、表格顶栏）
- 样式：`streamdown/styles.css` + `katex` + `src/styles/chat-markdown.css`（终端 fence、`search-highlight-flash`）

### 6.4 思考

- 流式：自动展开 + Shimmer 文案（用户中途手动折叠则尊重）
- 结束：1s 后自动折叠一次；显示「Thought for N seconds」
- 历史回放：默认收起
- 正文：走 `MessageResponse`（可 Markdown）；仅展示供应商真实 reasoning，不伪造
- 累积型 reasoning 字段须做后缀差分，避免同一段思考重复拼接

### 6.5 工具

- 主路径：`ToolActionsGroup` — 左色线 + 紧凑行 + 状态绿/红/转圈
- **运行中自动展开；全部完成后约 1s 默认收起**；历史默认收起；用户手动切换后不再自动覆盖
- 实时与历史以相同 `tool_call_id` 集合为准（前端 upsert + 后端幂等登记）
- Tool Logs 复用同一组件（可 `defaultOpen`）；MCP 审批仍走 `ToolApprovalDialog`

### 6.5.1 流式正文

- 禁止 Streamdown 逐字 `isAnimating` 打字机效果
- Token 按动画帧合并提交；内容连续增长，完成后用权威全文校正
- CSS 类：`misaka-chat-md--streaming`；遵守 `prefers-reduced-motion`

### 6.6 历史

| 项 | 值 |
|----|-----|
| 首屏 | `limit=50` |
| 更早 | `limit=100` + `beforeId` |
| 虚拟列表 | `@tanstack/react-virtual`；estimate 220；overscan 6 |
| Prepend | 保位 `scrollToIndex(align:'start')`；仅尾追加才自动置底 |

Composer 外壳：`rounded-2xl` 输入组 + `shadow-[var(--shadow-diffuse)]`（**任务页与 Hero 共用同一阴影**，禁止 Hero 外包再套一层 diffuse）。单行输入行高约 **32px 文本域**（`leading-8` + `py-0`，与发送 `size-8` 对齐）+ 外壳 `py-1.5`；输入行 **`items-center`**。发送 / 附件外置钮均为 `size-8`（`icon-sm`）`rounded-full`，图标约 `size-3.5`；可发送态**不得**用 ghost（避免 hover 图标变黑）。附件与发送 Tooltip：`delayDuration={2000}` + fade 约 150ms。底部 Model/MCP/Skill 行 `mt-2` + `pl-10`。附件预览胶囊：`rounded-full border-border/40 bg-muted`。行内文件引用 chip（`InlineMentionChip`）：`rounded-md border-primary/25 bg-primary/8`，与附件胶囊可区分。不引入 CodePilot Hood vibrancy / ActionBar 产品控件。

---

## 7. 首页 Hero（无任务）

- 组件：`NewChatWelcome`（`ChatPage` 在无 `activeSession` 时渲染）。
- 布局：垂直居中，`max-w-3xl`，`px-4 py-8`。
- 品牌：`MisakaLogo` `h-9 w-9`（浅色圆体 + 双闪电）+ 时段问候 `text-3xl font-medium` + 短提示。
- Composer：复用 `MessageInput`；首发经 `pendingOutbound` 创建任务后由 `ChatView` 发送。
- 下方引导：可选「选择工作目录」。

---

## 8. 明确不在本期壳层范围

- macOS 14px CardFrame 悬浮卡 / vibrancy / traffic-light 避让  
- React Router URL（`/chat/[id]`、`/settings/*`）  
- Settings 扩展 IA（overview / runtime / health / usage / assistant / tasks / bridge）  
- Workspace 多轨（独立 FileTree 轨、Assistant 轨、Git / Widget 固定 Tab）  
- Rewind / Checkpoint / RuntimeSwitch / SplitChat / Composer ActionBar（Runtime·Permission·Cockpit）  
