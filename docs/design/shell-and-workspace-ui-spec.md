# MisakaX 壳层布局与工作区 UI 规范

| 属性 | 说明 |
|------|------|
| **用途** | 定义主窗口三栏结构、会话侧栏工具区、对话页顶栏、设置页 Provider 弹窗与对话页模型选择器的布局语义和样式约定。 |
| **受众** | 负责 `AppShell`、`Sidebar`、`ChatPage`、`SessionPanel`、`WorkspaceBar`、`ModelSettings`、`ProviderDialog` 及相关布局的前端开发者。 |
| **最后审阅** | 2026-06-05（v3） |

## 相关文档

- **全局动效、色彩、组件原则（桌面 Agent 气质、禁止网页感缩放等）**  
  → [frontend-ui-guidelines.md](./frontend-ui-guidelines.md)

- **按钮、图标按钮、下拉菜单、Popover、Select、Dialog、Tooltip、Toast 等控件细节**  
  → 凡实现或调整上述控件，**须对照** [button-menu-design-spec.md](./button-menu-design-spec.md)，与全局规范一并遵守。

---

## 1. 三栏壳层与侧边栏收起语义

MisakaX 主界面在逻辑上划分为：

1. **主导航栏（Nav rail）**：`Sidebar` — 对话 / 技能 / 知识库 / 仪表盘等入口。  
2. **会话列表栏（Session sidebar）**：`SessionPanel` — 搜索会话、会话列表、新建会话。  
3. **主内容区（Main workspace）**：当前路由页面（如对话中的 `WorkspaceBar` + 消息区等）。

### 1.1 收起行为（必须遵守）

- **「收起侧边栏」仅作用于主导航栏（第 1 列）**：收窄或隐藏标签文字、保留图标型导航等，以实现更宽的主工作区。  
- **不得**将会话列表栏（第 2 列）与主导导航栏共用同一「全局收起」开关而一并隐藏 — 会话列表应保持可见（除非产品有单独的「会话栏」折叠需求并另设独立状态）。

实现上：主导航宽度/折叠状态与会话面板是否挂载/显示 **应解耦**，避免单个 `collapsed` 同时控制两列。

### 1.2 视觉连续性

会话栏与主导航同属左侧信息架构，分割线、背景可使用 `--border-muted`、`--surface-sidebar` 等令牌保持与整体设计系统一致。

### 1.3 会话栏与主内容区可拖拽分栏（必须遵守）

- `SessionPanel`（第 2 列）与「主内容区」（第 3 列）应被同一个 `react-resizable-panels` 的 `PanelGroup`（`orientation="horizontal"`）包裹，使用户可拖拽改变两者宽度比。
- `SessionPanel` Panel 约束：`defaultSize="18%"`、`minSize="14%"`、`maxSize="32%"`；超过 32% 时主内容区可读性下降，低于 14% 时会话列表会出现搜索/列表挤压，禁止突破。
- 分隔条（`PanelResizeHandle`）必须使用 `w-px bg-[color:var(--border-muted)] hover:bg-[color:var(--border-strong)]` 的极细样式，**不得**使用粗线或带阴影的拖拽条，与桌面 Agent 克制原则保持一致。
- 由于 react-resizable-panels v4 已**移除** `autoSaveId`，本项目暂不持久化两栏比例，刷新后回归默认值；后续若需要持久化，使用 `defaultLayout + onLayoutChanged + localStorage` 自行实现，不要回退到旧版 prop。
- `SessionPanel` 自身**禁止**再设固定宽度（如 `w-[260px]`），宽度完全由父 Panel 控制；其根容器使用 `flex h-full w-full min-w-0 flex-col`。

---

## 2. 会话列表顶部工具区（搜索 + 新建）

### 2.1 比例与宽度

- 会话栏应有**稳定最小宽度**，使「搜索输入框」与下方列表项在视觉上**同列对齐**，避免搜索条过窄、像独立小胶囊悬空。  
- 搜索框与「新建会话」按钮宜处于**同一行工具栏**：输入框横向 `flex-1 min-w-0` 占满剩余空间，高度与次要按钮对齐（例如统一偏大的触控/桌面友好的高度，如 `h-10` 量级），文字不低于 `text-sm`。

### 2.2 样式要点

- 使用设计系统表面与边框令牌（如 `--surface-card`、`--border-muted`、`--radius-ui-md`）使工具区与会话列表有清晰分区（例如工具区底部分割线）。  
- 搜索图标置于输入框左侧，预留足够 `padding-left`，避免文字与图标挤压。  
- 「新建」推荐与输入框等高的 **outline / secondary 系**按钮，具体变体、尺寸、hover 规范见 [button-menu-design-spec.md](./button-menu-design-spec.md)。  
- 文案与无障碍：`placeholder`、按钮 `aria-label` / `title` 应走 i18n，勿硬编码单一语言。

---

## 3. 对话页顶栏 — 工作目录（Workspace bar）

### 3.1 信息层级

工作目录条是 Agent 场景的**关键上下文**，不可使用过小字号、松散纯文本凑合。

- **未设置目录**：须有明确**标题**（如「未设置工作目录」）、**简短说明**（可选用 `workspace` 命名空间下的说明文案，并限制行数避免顶栏过高）、以及醒目的**主操作**（如「选择目录」）。  
- **已设置目录**：**文件夹显示名**作为标题层级；完整路径次要展示（可单行截断 + Tooltip），切换目录的操作使用清晰可见的控件（图标按钮须有 Tooltip、`aria-label`）。

### 3.2 布局与令牌

- 使用足够 **垂直内边距** 与可选 **最小高度**，使顶栏与「一小块文字」区分开。  
- 左侧可用带边框的图标容器承载文件夹图标，背景使用 `--surface-card` / `--surface-card-strong` 等，与 `surface-topbar` 或消息区背景形成层级。  
- 底部分割线使用 `--border-muted`（或与壳层分割线一致）。

### 3.3 按钮与 Tooltip

此处涉及 `Button`、`Tooltip` 的实现细节，须符合 [button-menu-design-spec.md](./button-menu-design-spec.md) 中的变体、尺寸与动效约束，并与 [frontend-ui-guidelines.md](./frontend-ui-guidelines.md) 中禁止的网页感动效一致。

---

## 4. 设置页与模型选择器

### 4.1 Provider 配置弹窗

- Provider 配置属于高信息密度设置流程，Dialog 应使用扩展尺寸规范：`max-w-[96rem]`、`max-h-[85vh]`，内容区内部滚动；尺寸细节见 [button-menu-design-spec.md](./button-menu-design-spec.md) 的“扩展尺寸弹窗”。
- 基础信息以“接口类型”和“供应商品牌”两个维度分开选择。接口类型 radio group 宜保持三列紧凑布局；供应商品牌 radio group 可在桌面宽度下扩展为五列，保持卡片高度一致。
- 不支持的接口类型 × 供应商品牌组合必须禁用，并提供 Tooltip 说明，不允许用户保存一个 UI 已知不可用的组合。
- API Key 默认保持隐藏；编辑态只有用户主动点击显示或调用 reveal 时才暴露明文。明文不应作为设置页列表、日志或 toast 内容出现。
- 高级配置默认折叠。当前只展示 `temperature` 与 `max_tokens` 两项，使用 `Accordion/Collapsible` 收纳，展开后字段使用两列网格并在字段右上角提供 Tooltip 解释。

### 4.2 对话页模型选择器

- 对话页模型选择器只展示已启用的 `custom_models`，不混入未选择的内置模型；Provider 删除或模型禁用后，当前选择失效时必须回退到占位文案。
- Popover 顶部固定搜索框，支持按模型名称、router 别名、品牌名过滤；清空搜索后恢复全量列表。
- 列表按供应商品牌分组，分组标题使用 sticky 样式；每个模型项主标题显示模型名，副标题显示 router 名，选中态只通过背景、边框或图标表达，不使用位移或缩放。
- 列表高度约束为 `max-h-[360px]` 并内部滚动，模型数量超过 50 时也不应造成外层布局抖动。
- 空状态需区分“没有任何可用模型”和“搜索无结果”。前者应提供“去设置”入口，后者只提示修改搜索条件。

---

## 5. 对话页右侧 Workspace Explorer

### 5.1 布局语义

- 对话页可在主对话区右侧展示 `WorkspaceExplorer`，用于查看当前会话工作目录的文件树与文本文件内容。
- 右侧 Explorer 与中间 `ChatView` 使用可拖拽的水平分栏；聊天区必须保留稳定最小宽度，Explorer 可由 `WorkspaceBar` 右侧图标按钮展开，并在 Explorer 顶栏中收起。
- 未设置工作目录时，不应打开 Explorer；相关入口须禁用或引导用户先选择工作目录。

### 5.2 文件树与编辑器

- 文件树按需懒加载目录，目录优先、名称升序；隐藏文件默认不展示，避免干扰普通对话工作流。
- 点击文本文件后在 Explorer 内打开 tab，tab 标题显示文件名，完整路径通过 `title` 或 Tooltip 暴露。
- 编辑器使用 Monaco；脏文件 tab 必须显示圆点，`Ctrl/Cmd + S` 保存成功后圆点消失。
- 关闭脏 tab 时必须提供保存、丢弃、取消三种选择；不得静默丢弃用户修改。
- **Tabs + Editor 必须按需挂载（必须遵守）**：
  - 当 `tabs.length === 0` 时，Explorer **仅渲染** `FileTreeView`，让文件树占满整个面板高度；禁止保留「No file open + 空 Monaco 占位」造成的双层视觉冗余。
  - 当用户从文件树点击文件、打开第一个 tab 时，再挂载 `react-resizable-panels`（`orientation="horizontal"`）把面板拆分为**左树右编**两栏：树 Panel 默认约 `32%`、`minSize` `22%`、`maxSize` `50%`；编辑器 Panel `minSize` `35%`。
  - 当用户关闭最后一个 tab、`tabs.length` 回到 0 时，立即移除编辑器 Panel，回到文件树独占布局。
  - 该条件渲染应在 `WorkspaceExplorer` 组件内完成，禁止把该决策上提到 `ChatPage`。
- **Explorer 顶栏唯一标题（必须遵守）**：
  - 顶栏仅展示 `explorer.title`（工作区）一层 section 标题；**禁止**在 `FileTreeView` 内再渲染 `treeTitle` 等第二套标题。
  - 文件树刷新按钮并入顶栏（`RefreshCw` + `explorer.refreshTree`），与收起按钮并列；`FileTreeView` 通过 `refreshKey` 响应刷新，不向子树重复暴露标题行。

### 5.3 安全与边界

- 前端只能通过后端 `fs_*` IPC 读取和写入文件；后端必须校验目标路径位于当前工作目录下。
- 大文件与二进制文件应被拒绝并给出用户可理解的错误反馈，避免卡死编辑器或将二进制误送入文本编辑流程。
- 「在文件资源管理器中打开」必须由后端命令（`fs_reveal_in_explorer`）实现，统一进行 `validate_under_root` 校验后调用系统进程（Windows `explorer /select,`，macOS `open -R`，Linux `xdg-open` 目录）。前端不得直接拼接 shell 命令打开任意路径。
- **Windows `/select,` 必须用 `CommandExt::raw_arg` 自行拼接命令行**：Rust 标准的 `Command::arg` 会对含空格的参数加引号，把整段 `/select,"D:\path with space\foo.txt"` 一起引起来变成 `"/select,..."`，explorer 解析失败时**会回退到打开「文档」目录**。必须使用 `raw_arg(format!("/select,\"{}\"", path.replace('/', "\\")))`，并把分隔符规范化为 `\`。

### 5.4 入口图标与文案

- `WorkspaceBar` 右侧打开 Explorer 的按钮**禁止**使用 `PanelRight*` 等通用「侧栏开关」图标，因其指向性不足；应使用具备工作区/文件树语义的图标（推荐 `FolderTree`）。Tooltip 与 `aria-label` 必须走 `workspace.openExplorer` 这类 i18n key，不得硬编码英文。
- Explorer 面板自身的标题、收起按钮、顶栏刷新、空状态提示、tab 关闭、未保存对话框等**全部文案都必须 i18n**；新增子区域时优先在 `workspace.explorer.*` 命名空间下扩展，避免 chat / common 命名空间被工作区强耦合。

### 5.5 Explorer 入场动画

- Explorer 由 `WorkspaceBar` 切换打开时，**整个 `<aside>` 容器**使用 `animate-in fade-in slide-in-from-right-4 ease-out duration-[220ms]` 作为出场动画，时长贴齐 `--ds-dur-slow`；不得使用桌面 Agent 风格忌讳的缩放/位移幅度大的入场。
- 由于 `react-resizable-panels` 的 Panel 宽度切换是即时的，入场动画必须挂在 Explorer 内层容器上（而非 Panel 外层），让用户视觉焦点落在内容渐入上，从而弱化宽度硬切的突兀感。
- **打开第一个 tab 时**：编辑器列（`EditorColumn`）使用 `animate-in fade-in slide-in-from-right-2 ease-out duration-[var(--ds-dur-slow)]`；`EditorPane` 在 tab 切换时对 `tab.path` 使用 `key` + `fade-in duration-[var(--ds-dur-fast)]` 做轻量内容过渡；禁止缩放。
- 关闭最后一个 tab 或收起 Explorer **无需**逆向动画，与桌面应用关闭侧栏的直觉一致（瞬时让位给主内容区）。
- `prefers-reduced-motion` 下依赖全局 motion token 缩短时长（见 `tokens-motion.css`）。

### 5.6 文件树右键菜单

- 文件树节点（无论文件或目录）必须支持原生右键菜单（基于 `ContextMenu` 组件）。
- 文件节点的菜单（自上而下，按业务重要性排序）：
  1. **在当前对话中引用**（`explorer.mentionInChat`，`AtSign` 图标）— **仅文件**；触发后把该文件加入 Composer 的 mention 列表（见 [frontend-ui-guidelines.md §4.3.x Mention Pill](./frontend-ui-guidelines.md)），并以 toast 反馈「已引用 {name}」。
  2. **在文件资源管理器中打开**（`explorer.openInExplorer`，`FolderSearch` 图标）— 调用 `fs_reveal_in_explorer`；失败走 `workspace.explorer.openInExplorerFailed` 的 `toast.error`。
  3. **复制完整路径**（`explorer.copyPath`，`Copy` 图标）— 使用 `@tauri-apps/plugin-clipboard-manager` 的 `writeText`，禁止用浏览器 `navigator.clipboard` 绕过 Tauri 权限。
- 目录节点菜单不展示「在当前对话中引用」（避免引用整目录带来的歧义与 token 浪费）。
- 菜单分组使用 `ContextMenuSeparator`：mention 与其它项之间一条；reveal 与 copy 之间一条。
- 菜单触发的 Tauri IPC 失败必须给出 `toast.error` + i18n key；禁止仅在 console 打印吞错误。

### 5.7 Monaco Tab 与编辑面板美化

- `EditorTabs` 容器使用 `h-9`、与编辑器同背景 `bg-[color:var(--surface-card)]`、底部分割线；Tab 采用 **VS Code 扁平风格**：无 `rounded-t` 浏览器叠层，激活态用 `border-b-2 border-primary`，未选中 `border-b-2 border-transparent` + hover 背景。
- 未选中 Tab 默认隐藏关闭按钮（`opacity-0 group-hover:opacity-100`），激活 Tab 常显关闭按钮，桌面端避免视觉噪音。
- 脏标记使用 `size-1.5` 圆点（`bg-primary/80`），禁止使用大号 bullet 或文字「•」。
- `EditorBreadcrumb`：`h-7`、`text-[11px]`、`text-muted-foreground`，展示工作区相对路径，`truncate` + `title` 完整路径；位于 Tab 栏与 Monaco 之间。
- `EditorPane` Monaco 区域**贴边铺满**，禁止 `rounded-tl` 等与 Tab 叠层的圆角衔接；Monaco 选项强制开启 `smoothScrolling`、`cursorBlinking: "smooth"`、`overviewRulerBorder: false`、`padding: { top: 10, bottom: 10 }`，滚动条尺寸 10px。
- 「未选择文件」占位区禁止只放一行文字；使用低饱和图标容器（`size-12` 圆角卡片 + 弱化图标）+ 短句提示（含「左侧」语义），与 `ChatEmptyState` 保持同一调性。

## 6. 小结

| 区域 | 要点 |
|------|------|
| 主导航收起 | 只控制第 1 列，不误伤会话列表。 |
| 会话工具栏 | 全宽对齐、可读字号、工具栏分区清晰。 |
| 工作目录顶栏 | 标题/说明/主次操作层级分明，禁用「过小过素」的一次性排版。 |
| Provider 配置弹窗 | 扩展尺寸、内部滚动、双维度 radio、按需 reveal、高级项默认折叠。 |
| 对话页模型选择器 | 只列启用模型、品牌分组、顶部搜索、sticky 标题、内部滚动与明确空状态。 |
| Workspace Explorer | 右侧可拖拽/收起、懒加载文件树、Monaco tab 编辑、脏标记与保存确认。 |
| 按钮与菜单 | 一律交叉参考 `button-menu-design-spec.md`。 |

编写或修改前端时，请将本规范与 [frontend-ui-guidelines.md](./frontend-ui-guidelines.md)、[button-menu-design-spec.md](./button-menu-design-spec.md) **一并查阅**，避免只做局部改动而破坏壳层语义或桌面端观感。
