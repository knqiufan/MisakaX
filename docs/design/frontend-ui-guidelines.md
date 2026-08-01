# MisakaX 前端 UI 设计与编写规范

本文档定义了 MisakaX 桌面端 AI Agent 客户端的前端 UI 编写规范。在进行任何前端组件开发、页面重构或样式调整时，**必须严格遵守**本规范，以确保整体视觉风格和交互体验的高度一致性。

## 文档关系

- **壳层布局、单列左栏、会话列表工具区与底栏、对话页工作目录顶栏、设置页卡片系统、Workspace Explorer chrome、对话消息/Markdown/思考与工具**等专项约定：见 [shell-and-workspace-ui-spec.md](./shell-and-workspace-ui-spec.md)。  
- **按钮、下拉菜单、Popover、Select、Dialog、Tooltip 等控件的细节与变体**：编写或调整时须同时对照 [button-menu-design-spec.md](./button-menu-design-spec.md)。
- **可复刻参考（CodePilot）**：[`docs/ui/02-chat.md`](../ui/02-chat.md)、[`docs/ui/03-workspace.md`](../ui/03-workspace.md)、[`docs/ui/04-settings.md`](../ui/04-settings.md)、[`docs/ui/06-markdown-message-tools.md`](../ui/06-markdown-message-tools.md)（视觉与能力对齐；IA 以 shell 规范本期边界为准）。

**最后审阅 / Last reviewed:** 2026-08-01（v24）

## 1. 设计理念 (Design Philosophy)

MisakaX 的目标是打造一个**现代化、专业、克制的桌面端 Agent 客户端**。
- **桌面端直觉优先**：交互应该干脆利落，符合桌面软件的使用直觉，而不是移动端或网页端的体验。
- **克制的动效**：避免过度设计，去除花哨的过渡和缩放，让用户的注意力集中在内容和 AI 交互上。
- **高信息密度**：UI 元素应紧凑合理，留白适中。
- **视觉关键词**：charcoal 单色主色、暖灰边框、Geist 字体；**禁止**蓝紫品牌主色与可切换强调色。

## 2. 动效与交互规范 (Motion & Interaction)

这是最容易出现“网页感”的地方，必须严格控制：

### 2.1 禁止使用的网页感动效
- **禁止悬停位移**：不要在按钮、卡片或列表项上使用 `hover:-translate-y-*` 等悬停上浮效果。
- **禁止点击缩放**：不要在可点击元素上使用 `active:scale-*`（如 `active:scale-95`）等点击缩小的弹簧效果。
- **禁止弹出层缩放**：在 Dialog、DropdownMenu、Tooltip、Select 等弹出组件中，**严禁使用** `zoom-in` 和 `zoom-out` 动画。

### 2.2 推荐的交互方式
- **颜色与背景过渡**：交互反馈主要通过 `background-color`、`color`、`border-color` 的变化来实现。
- **弹出层动效**：仅使用淡入淡出（`fade-in` / `fade-out`）配合极小范围的滑入（`slide-in-from-*`）。
- **过渡时间**：保持快速响应。使用 Tailwind 的 `duration-150` 或系统定义的 `duration-[var(--ds-dur-fast)]`，缓动函数推荐使用 `ease-out`。

### 2.3 弹出层动效基础设施（必读）

- 项目依赖 **`tw-animate-css`**（在 `src/index.css` 中 `@import "tw-animate-css"`）。`animate-in` / `animate-out` / `fade-in-0` / `slide-in-from-*-1` 等类名由此提供；**禁止**删除该依赖或省略 import，否则所有菜单会瞬间出现/消失。
- 共享类字符串：`src/lib/overlay-motion.ts` 的 `OVERLAY_MOTION` + `OVERLAY_SIDE_SLIDE`（菜单 / Select / Tooltip）与 `DIALOG_MOTION`（Dialog）。新建 Radix 弹出层 Content **必须**复用这些常量，不要手写另一套时长或 `slide-*-2`（位移 >4px）。
- 非 Radix、用条件渲染挂载的自定义面板（如 ModelSelector）必须用 `usePresence`（`src/hooks/usePresence.ts`）保留关闭退场帧，**禁止** `{open && <Panel />}` 直接卸载导致收回闪断。

## 3. 视觉与色彩规范 (Visual & Color)

项目基于 **Tailwind CSS v4** 和 **shadcn/ui (New York)**，主色为 **charcoal**（非 Zinc 蓝灰冷色）。

### 3.0 主题模式

- 仅支持 **`light` / `dark` / `system`** 三种；旧配置 `dim` 迁移为 `dark`。
- 主色固定 charcoal：Light `--primary: oklch(0.262 0 0)`；Dark 近白反转。
- **不提供**用户可选强调色（Accent）；Appearance 设置中不展示色板。

### 3.1 颜色变量 (CSS Variables)
- 强制使用 CSS 变量来定义颜色，以完美适配深色/浅色模式。
- 边框色温为**暖灰**（如 Light `--border: oklch(0.923 0.003 48.717)`），交互激活用 `sidebar-accent` / `primary` 低透明，**禁止**冷蓝 `--border-accent` / `--surface-active` 作为主交互色。
- 常用背景：`bg-background`、`bg-muted`、`bg-[color:var(--surface-card)]`、`bg-[color:var(--surface-hover)]`。
- 常用边框：`border-border`、`border-[color:var(--border-subtle)]`、`border-[color:var(--border-strong)]`。
- 常用文字：`text-foreground`、`text-muted-foreground`。
- 字体：Geist Variable + Geist Mono；`body` 使用 `antialiased`。
- 产品圆角：`--radius: 1rem`（16px）。

### 3.2 阴影与层级 (Shadows & Elevation)
- **扁平化为主**：基础按钮、输入框等控件尽量减少阴影，使用边框（Border）来区分边界。
- **Composer / Hero**：可用 `--shadow-diffuse` 弥散阴影。
- **弹出层阴影**：仅在下拉菜单、模态框、Tooltip 等悬浮层级（z-index 较高）的元素上使用 `shadow-lg` 或自定义的深阴影。
- 本期跨平台壳为**实心底**；不做 macOS 悬浮卡 / vibrancy。

### 3.3 品牌标与首页 Hero
- **品牌标**：`MisakaLogo`（`src/components/brand/MisakaLogo.tsx`）— 浅色暖圆底 + 双圆角闪电；源图为 `src/assets/brand/misakax-logo.png`，圆外必须保留透明通道，禁止把编辑器棋盘格写入像素。
- **单一视觉源**：`misakax-logo.svg` 必须与 PNG 使用相同的 128 × 128 画布几何和固定暖白 / charcoal 色；`MisakaLogo` 必须直接渲染该 SVG 资源，禁止再次复制或变形闪电路径。
- **桌面图标**：修改 PNG 后，运行 `npx tauri icon src/assets/brand/misakax-logo.png` 更新 `src-tauri/icons/`；`src-tauri/build.rs` 必须监听 `icons/icon.ico`，以便 `tauri dev` 重建 Windows 资源。
- **禁止**继续用旧的 5×5 diffusion 点阵作主品牌；`MonolithIcon` 仅作兼容 re-export。
- 无会话时主区为居中 hero：`max-w-3xl` + `MisakaLogo`（36px）+ 时段问候标题（`text-3xl font-medium`）+ Composer。
- 空会话态与 hero / 设置「关于」页共用同一品牌标。

## 4. 组件编写原则 (Component Guidelines)

### 4.1 按钮 (Buttons)
- 默认造型为 **胶囊**：`Button` 基类 `rounded-full`（见 [button-menu-design-spec.md](./button-menu-design-spec.md) 文首现行实现说明）。
- 主要按钮（Primary）使用实色背景，次要按钮（Secondary/Outline/Ghost）使用透明或半透明背景。
- 悬停（Hover）时仅改变背景色亮度或透明度，不改变物理尺寸和位置；允许按下时 `translate-y-px` 微按压，禁止 `scale`。
- Size 优先用组件 API：`icon-sm`（32px）用于顶栏 / Composer 圆钮，避免到处手写 `size-8`。

### 4.2 菜单与导航 (Navigation & Menus)
- 侧边栏导航项（NavItem）选中时应有明显的视觉区分（如强调色边框或背景），未选中时保持低调。
- 避免在菜单项切换时加入复杂的宽度/高度动画。

### 4.3 对话输入区（Composer 单行）

- 外层（附件 + 输入壳）：`flex items-center gap-2`；附件 `size-8` 外置左侧，与输入壳垂直居中。
- 同行圆形图标按钮宜统一触控尺寸（均为 `size-8` / `icon-sm`），附件与发送图标约 `size-3.5`，避免 + 钮视觉大于发送钮。
- 新版 Composer 中，附件入口使用外置左侧圆形 `+` 按钮，不放入输入框内部；按钮与输入容器同属一行，输入容器内部只承载附件预览、文本域与发送/停止按钮。
- `textarea` 单行态高度与发送钮对齐：`leading-8` + `py-0` → **32px**（`h-8` / `COMPOSER_SINGLE_LINE_HEIGHT_PX`），与 `icon-sm` 同高。`wrap="off"` 时**仅硬换行**才增高；**禁止**用 `scrollHeight > 32` 判定单行（leading-8 下 scrollHeight 会虚高，框被撑高后文字贴顶、下方留白）。
- 输入组内行：`py-1.5 pl-3 pr-1.5` + **`items-center`**（文本域与发送钮相对外壳垂直居中；多行增高时仍保持居中）。外壳 `rounded-2xl border-input` + `--shadow-diffuse`。
- **发送钮颜色**：可发时用 `variant="default"`（或显式 `text-primary-foreground hover:text-primary-foreground [&_svg]:text-current`）；**禁止**可发态套 `variant="ghost"`——其 `hover:text-accent-foreground` 会在深色 primary 底上把箭头染成近黑而「消失」。
- **附件 / 发送 Tooltip**：`delayDuration={2000}`（悬停约 2s 再出）；气泡沿用统一 Tooltip 的 fade 入/出（约 150ms），禁止零延迟闪现。
- 底部 Model / MCP / Skill 胶囊行：与输入壳间距 `mt-2`；`pl-10` 对齐输入壳左缘（越过 32px 附件 + `gap-2`）。
- **思考开关（Brain）**：
  - 位于 Model / MCP / Skill 同行，使用 `Brain` 图标的 `icon-sm` 圆形按钮 + Tooltip；须设置 `aria-pressed`。
  - **开启态**（默认）：低强度 primary 高亮（如 `bg-primary/10 text-primary`），表示请求供应商可展示的思考/reasoning 流，并写入历史 `thinking_content`。
  - **关闭态**：中性 ghost；关闭时后端使用原生关闭参数或同 Provider 的 `thinking_off_model_id`，二者皆不可用则拒绝发送并提示配置。
  - 状态持久化于 localStorage（`misakax:thinkingEnabled`），发送时快照进 `llm_config.thinking_enabled`；**禁止**仅做前端隐藏、而后端仍请求思考。
  - **不伪造思考内容**：只渲染供应商实际返回、可安全展示的 reasoning；历史回放仅用于 UI，不把纯文本 reasoning 当作带签名上下文回灌模型。
- **深度研究开关（Telescope，新增）**：
  - 与思考开关同一 Composer 底栏行，使用 `Telescope` 图标的 `icon-sm` 圆形按钮 + Tooltip；须设置 `aria-pressed`。
  - **关闭态**（默认）：中性 ghost；普通聊天仅允许直接文件工具 / PowerMem / MCP，**禁止** DeepAgents `task` 与同步子代理。
  - **开启态**：低强度 primary 高亮；该回合以 `llm_config.agent_mode: "research"` 快照发送，启用 researcher/coder/analyst 与 general-purpose 子代理编排。
  - 状态持久化于 localStorage（`misakax:researchEnabled`）；发送与重新生成都必须携带当前快照，**禁止**仅改前端展示。
- 附件预览支持图片缩略图与文本文件卡片两类；非图片附件不得伪装成图片缩略图，应使用文件图标、文件名与大小信息表达。
- **附件入口与 Radix asChild 嵌套（必读，新增）**：
  - `AttachButton` 作为 `DropdownMenuTrigger asChild` 的 child 时，**必须**用 `forwardRef` 实现，并把 trigger 注入的 `ref` 与 `...rest` props 透传到底层 `<button>`，否则 Dropdown 的 click/keyboard handler 与定位 anchor 都无法生效，会出现「按钮点击无反应」的回归。
  - **禁止**给 trigger 的 child 元素再传 `onClick={() => undefined}` 等占位回调；Radix Slot 会保留 child 已声明的事件，且空回调可能掩盖真正的 trigger 行为。
  - 文案使用 `chat.composer.attach` 等 i18n key，桌面端中文 UI 优先使用「添加附件」等动名词组合，避免单字「附加」造成歧义。

### 4.3.x Composer 行内文件引用 Chip

- Composer 顶层 `<div>` 必须使用 **`flex items-center gap-2`**；输入行内 `ComposerInlineField` 必须带 **`flex-1 min-w-0`**，发送按钮 `shrink-0` 贴右，禁止仅按内容宽度收缩导致按钮悬空中部。
- 数据模型为 **`ComposerSegment[]`**（`text`、文件 `mention` 与 `skill` 交错）；两类引用均插入当前 `composerCursor` 位置，而非固定堆在输入框最前。`normalizeSegments` 会 **prune 掉引用之间的空 text 段**，避免空 `textarea` 默认宽度把 chip 撑开；仅保留末尾 text 段作为输入区。
- `ComposerInlineField` 按 segment 渲染多个 `SegmentTextarea`、`InlineMentionChip` 与 `InlineSkillChip` 交错；**仅最后一个 text segment** 使用 `flex-1 min-w-[2rem]` 撑满剩余宽度，前面的 text segment 用 **`scrollWidth` 动态测宽**（`shrink-0`、`whitespace-nowrap`、`wrap="off"`），**禁止** `cols={1}` 或 `Nch` 固定宽度（中文会竖排）。
- `InlineMentionChip` 视觉（中性 / primary 低彩，对齐 charcoal 体系）：
  - ready：`h-6`、`rounded-md`、`border border-primary/25`、`bg-primary/8`，`FileTypeIcon` + 文件名同色族；
  - loading：`opacity-60` + `Loader2`；
  - error：`bg-destructive/8` + `AlertTriangle` + `text-destructive`；
- `InlineSkillChip` 沿用相同 `h-6`、`rounded-md`、`border-primary/25`、`bg-primary/8` 的低彩 primary 语义，使用 `Sparkles` 作为稳定图标；**禁止**使用高饱和独立品牌色。
- **禁止**显示删除 `X` 按钮；删除通过 **Backspace**（光标在 text 段 offset=0 时删前一个引用）或 **Delete**（光标在 text 段末尾时删后一个引用）完成。
- 引用触发后通过 `focusRequestId` + `composerCursor` 自动 focus 到插入点后的 text segment；**不**弹出「已引用」成功 toast（失败仍 `toast.error`）。
- **跨段全选**：多 `textarea` 无法原生跨 chip 选区；`Ctrl/Cmd+A` 由 `useComposerTextSelection` 选中全部 text segment（chip 不参与），各段 `bg-primary/15` 高亮；`Ctrl+C` 复制纯文本；`Backspace/Delete` 清空全部文本但保留 chip。
- 发送：`ready` mentions 走 `mentionsToWorkspaceAttachments(segments)`；Skill segment 只发送 Registry 的稳定 `skill_id`，slug 仅作显示快照，**不得**把完整 `SKILL.md` 或宿主绝对路径拼入用户正文；展示正文用 `buildOutgoingFromSegments` 按 segment 顺序交错 `@relPath` 与文本；发送成功后重置整个引用文档。
- `canSendComposerMessage` 使用 `getDocumentPlainText(segments)` 作为 `content`，`countSendableFromSegments` + `attachments.length` 作为 `attachmentCount`。
- `AttachmentPreview` 仍在 segment 行**之上**，与 workspace 引用语义分离。

### 4.3.x.1 Composer Skill 选择与 Slash 菜单

- 底栏 Skill 多选器只展示后端判定为 **`effective_active`** 的具体来源；`enabled`、`healthy` 或 slug 相同均不能替代该判定。选择与取消必须按稳定 `skill_id` 直接增删同一份行内 `skill` segment，禁止维护第二套选择状态。
- 已选来源因禁用、缺失、制品变化或来源冲突而退出激活视图时，Composer 必须自动移除对应 chip，并用一次非阻塞、本地化提示说明；不得继续发送旧 slug，也不得在库存尚未加载时误删 chip。
- `/` 仅在空文本段或空白边界触发；过滤范围为最后一个 `/` 后的片段。IME 合成期间不触发或截获键盘操作。
- Slash 浮层使用 `usePresence` 和 fade + `slide-in-from-bottom-1`，不得直接条件卸载或缩放；`↑/↓` 移动、`Enter/Tab` 插入、`Esc` 关闭，列表项提供 `role="option"` 与可见焦点。

### 4.3.y Composer 附件下拉菜单（新增）

- `AttachmentMenu` 仅渲染**实际可用**的入口；禁止保留「即将支持」「Coming soon」之类的临时占位项，避免给用户希望落空。
- 菜单内**不得**出现 `DropdownMenuLabel`（分组标题）；菜单内容只承载选项 item，统一 `图标 size-4 + 8px gap + 文案` 排版。
- 图片入口（`onPickImages`）的 `<input type="file">` 必须使用 `ACCEPTED_IMAGE_TYPES`（仅 IMAGE MIME）；Markdown / 文本入口必须使用 `ACCEPTED_TEXT_TYPES`（.md/.markdown/.txt + 对应 MIME），**不得**把 PDF/Office 等当前不支持的扩展名也加入 accept，否则用户能选中却又被前端拒绝。

### 4.3.z ScrollArea（纵向面板）

- 共享 `ScrollArea`（`src/components/ui/scroll-area.tsx`）的 Viewport 必须把 Radix 默认内容包装层约束为 **`block` + `min-w-0` + `w-full`**，避免 `display: table` 被长行撑开后被根节点 `overflow: hidden` 横向裁切。
- 放进 flex / grid 双栏的滚动面板，链路本身也要带 `min-w-0`；长文本预览用换行（`break-words` / `overflow-wrap: anywhere`），不要依赖嵌套横向裁切。

### 4.3.z.1 Skills 设置与按需文件预览

- Skills 的稳定入口是 `Settings > Skills`；历史 `{ page: "skills" }` 只做兼容重定向。领域 UI 必须由 `SkillsSettingsFeature` 承载，不重新依赖顶层 route。
- 选中已安装 Skill 的首屏只允许请求 summary 与目录页，`body_bytes_transferred` 必须为 0；没有明确选择文件前不得调用 `skills_read_file`，也不得默认读取或渲染 `SKILL.md`。
- 详情固定使用 `文件 / 安全 / 概览` 顺序并默认文件；安全摘要只在首次打开安全 tab 时按需加载。文件树与预览各自滚动，窄宽度改为上下两区，不让长文件撑高 Settings 根页面。
- 文件树使用标准 tree ARIA 与 roving focus；支持方向键、Home/End、Enter/Space。所有异步目录页、预览段和扫描摘要都必须核对稳定 `skill_id` 与 generation，迟到结果直接丢弃。
- 文本仅以转义后的源码 `<pre>` 展示；单段不超过 200 KiB、总预览不超过后端预算，并由用户显式继续加载。二进制或不支持编码只显示大小、编码和可用 hash 元数据，禁止 Base64/HTML/iframe/远端资源内联。
- 具体布局、安全空态与 Switch 语义以 [Skills/Workspace/Terminal UI 设计](./SKILLS_WORKSPACE_TERMINAL_UI_DESIGN.md) 为准；本节只记录跨页面必须复用的实现约束。

### 4.4 空页面与占位符 (Empty States)
- 空页面设计应具有**引导性**。
- 避免“提示文本”与“操作按钮”在逻辑上产生冲突（例如：提示“即将推出”却又提供“新建”按钮）。
- 居中对齐，使用柔和的图标（透明度降低）和简明的文案。

### 4.5 助手消息错误状态（流式 / IPC 失败）

- 当 `message.role === "assistant"` 且 `message.status === "error"` 时，**不使用 Markdown 渲染**，以纯文本展示 `message.content`（存放后端或 IPC 错误文案）。
- 容器使用 **低强度破坏性语义色**：`border-destructive/45`、`bg-destructive/8`、`text-destructive`，附 `AlertTriangle` 图标，并设 `role="alert"`。
- 禁止在此状态使用悬停位移动效；与 [按钮/菜单规范](./button-menu-design-spec.md) 一致，仅颜色/背景过渡。
- 用户可见的错误还应通过 **Sonner `toast.error`**（`ChatView` 中 IPC `catch`）补充提示；`stream_error` 事件仅负责在助手仍为 `streaming` 时写入 `updateMessageError`，避免与 `catch` 重复 Toast 或覆盖已由 `catch` 写入的错误文案。

### 4.6 对话消息列表布局（MessageList / MessageItem）

权威约定与 [shell-and-workspace-ui-spec.md §6](./shell-and-workspace-ui-spec.md) 一致；本节省略重复，仅列编写要点。

- **列表容器**（`MessageList`）：主列可读宽 `mx-auto w-full max-w-3xl px-4`（与 Composer / 空态同一契约）。
- **无头像**：`MessageItem` 不渲染 User / Assistant Avatar，仅显示内容。
- **User 消息**：
  - 整条 `flex justify-end`；内容区 `max-w-[95%] ml-auto`。
  - 气泡：`rounded-2xl bg-muted px-4 py-3 text-sm text-foreground`（**非** primary 实心）。
  - 气泡下方悬停显示 **复制** / **重新生成**（`size-6`，`opacity-0 → group-hover/msg:opacity-100`）。
- **Assistant 消息**：
  - 容器 `w-full`，**无**气泡底、无圆角外壳，正文直接铺在画布上（Markdown / 纯文本）。
  - 错误态用 `border-destructive/45 bg-destructive/8` 等（见 4.5）。
  - 底部悬停仍显示 Copy / Regenerate + TokenBadge。

### 4.6.x Assistant 可读宽度

- 主列已由 `MessageList` 的 `max-w-3xl` 约束；Assistant **不再**套 `surface-card` 底色卡片。
- 禁止用卡片底与 User 气泡混淆。

### 4.6.y 工具调用状态行（ToolActionsGroup）

- 主路径为左色线分组 + 紧凑行；pending 用静态圆点（`size-2 rounded-full bg-muted-foreground/40`），running 用 **中性** spinner（`text-muted-foreground`，禁止蓝色品牌 spinner）。
- **执行中自动展开**；**全部完成后约 1s 自动折叠一次**（与思考块同节奏）。历史回放默认收起；用户手动展开/收起后不再被自动逻辑覆盖。
- Tool Logs 抽屉可传 `defaultOpen` 强制展开，与消息内工具组语义解耦。
- 展开动画仅 `fade-in` / 极小 `slide-in-from-top-*`，不使用缩放。
- 实时 `stream:tool_call` 必须按 `tool_call_id` **幂等 upsert**，禁止同 id 追加重复行（与历史持久化条数一致）。

### 4.6.z 流式正文呈现（MessageResponse）

- 助手正文走 Streamdown `mode="streaming"` + `parseIncompleteMarkdown`，但 **`isAnimating={false}`**——禁止逐字打字机动画。
- Token 增量在 `useStreamListener` 中按 **requestAnimationFrame** 合并后再写入 store，避免每个 token 触发一次完整 Markdown 重渲染。
- 流式结束以后端 `stream_complete.full_content` 为权威覆盖；尊重 `prefers-reduced-motion`。
- 仅渲染供应商真实返回的 `thinking_content`；历史回放不伪造思考。

## 5. 总结

在编写 Tailwind 类名时，时刻问自己：**“这个样式在 macOS/Windows 原生应用中会出现吗？”** 如果答案是否定的（比如鼠标放上去按钮会跳一下），请坚决将其移除。保持 UI 的专业、冷静与克制。
