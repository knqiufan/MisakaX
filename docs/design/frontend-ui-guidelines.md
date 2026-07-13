# MisakaX 前端 UI 设计与编写规范

本文档定义了 MisakaX 桌面端 AI Agent 客户端的前端 UI 编写规范。在进行任何前端组件开发、页面重构或样式调整时，**必须严格遵守**本规范，以确保整体视觉风格和交互体验的高度一致性。

## 文档关系

- **壳层布局、主导航收起语义、会话列表工具区、对话页工作目录顶栏、设置页卡片系统、Workspace Explorer chrome、对话消息/Markdown/思考与工具**等专项约定：见 [shell-and-workspace-ui-spec.md](./shell-and-workspace-ui-spec.md)。  
- **按钮、下拉菜单、Popover、Select、Dialog、Tooltip 等控件的细节与变体**：编写或调整时须同时对照 [button-menu-design-spec.md](./button-menu-design-spec.md)。
- **可复刻参考（CodePilot）**：[`docs/ui/02-chat.md`](../ui/02-chat.md)、[`docs/ui/03-workspace.md`](../ui/03-workspace.md)、[`docs/ui/04-settings.md`](../ui/04-settings.md)、[`docs/ui/06-markdown-message-tools.md`](../ui/06-markdown-message-tools.md)（视觉与能力对齐；IA 以 shell 规范本期边界为准）。

**最后审阅 / Last reviewed:** 2026-07-14（v11）

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

### 3.3 首页 Hero
- 无会话时主区为居中 hero：`max-w-3xl` + `MonolithIcon`（36px）+ 时段问候标题（`text-3xl font-medium`）+ Composer。
- 空会话态与 hero 共用品牌标视觉，避免大尺寸冷色图标占位。

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
- `textarea` 单行态高度与发送钮对齐：`leading-5`（20px）+ `py-1.5` → **32px**（`min-h-8` / `COMPOSER_SINGLE_LINE_HEIGHT_PX`），与 `icon-sm` 同高；占位符与发送钮视觉居中。
- **禁止**用过大 `min-h`（如 CodePilot 参考的 `min-h-16`）或「先设高再读 `scrollHeight`」把空态撑高，导致占位符贴左上角、下方大块留白、外壳呈瘦高胶囊。`SegmentTextarea` 须先锁 32px，仅在硬换行或 `scrollHeight` 明确溢出时再增高（上限 200px）。
- 输入组内行：`py-1.5 pl-3 pr-1.5` + `items-end`（单行时与 32px 发送钮齐平；多行时发送贴底）。外壳 `rounded-2xl border-input` + `--shadow-diffuse`。
- 底部 Model / MCP / Skill 胶囊行：与输入壳间距 `mt-2`；`pl-10` 对齐输入壳左缘（越过 32px 附件 + `gap-2`）。
- 附件预览支持图片缩略图与文本文件卡片两类；非图片附件不得伪装成图片缩略图，应使用文件图标、文件名与大小信息表达。
- **附件入口与 Radix asChild 嵌套（必读，新增）**：
  - `AttachButton` 作为 `DropdownMenuTrigger asChild` 的 child 时，**必须**用 `forwardRef` 实现，并把 trigger 注入的 `ref` 与 `...rest` props 透传到底层 `<button>`，否则 Dropdown 的 click/keyboard handler 与定位 anchor 都无法生效，会出现「按钮点击无反应」的回归。
  - **禁止**给 trigger 的 child 元素再传 `onClick={() => undefined}` 等占位回调；Radix Slot 会保留 child 已声明的事件，且空回调可能掩盖真正的 trigger 行为。
  - 文案使用 `chat.composer.attach` 等 i18n key，桌面端中文 UI 优先使用「添加附件」等动名词组合，避免单字「附加」造成歧义。

### 4.3.x Composer 行内文件引用 Chip

- Composer 顶层 `<div>` 必须使用 **`flex items-center gap-2`**；输入行内 `ComposerInlineField` 必须带 **`flex-1 min-w-0`**，发送按钮 `shrink-0` 贴右，禁止仅按内容宽度收缩导致按钮悬空中部。
- 数据模型为 **`ComposerSegment[]`**（`text` 与 `mention` 交错），引用插入当前 `composerCursor` 位置（`insertMentionAtCursor`），而非固定堆在输入框最前。`normalizeSegments` 会 **prune 掉 mention 之间的空 text 段**，避免空 `textarea` 默认宽度把 chip 撑开；仅保留末尾 text 段作为输入区。
- `ComposerInlineField` 按 segment 渲染多个 `SegmentTextarea` 与 `InlineMentionChip` 交错；**仅最后一个 text segment** 使用 `flex-1 min-w-[2rem]` 撑满剩余宽度，前面的 text segment 用 **`scrollWidth` 动态测宽**（`shrink-0`、`whitespace-nowrap`、`wrap="off"`），**禁止** `cols={1}` 或 `Nch` 固定宽度（中文会竖排）。
- `InlineMentionChip` 视觉（中性 / primary 低彩，对齐 charcoal 体系）：
  - ready：`h-6`、`rounded-md`、`border border-primary/25`、`bg-primary/8`，`FileTypeIcon` + 文件名同色族；
  - loading：`opacity-60` + `Loader2`；
  - error：`bg-destructive/8` + `AlertTriangle` + `text-destructive`；
  - **禁止**显示删除 `X` 按钮；删除通过 **Backspace**（光标在 text 段 offset=0 时删前一个 mention）或 **Delete**（光标在 text 段末尾时删后一个 mention）完成。
- 引用触发后通过 `focusRequestId` + `composerCursor` 自动 focus 到插入点后的 text segment；**不**弹出「已引用」成功 toast（失败仍 `toast.error`）。
- **跨段全选**：多 `textarea` 无法原生跨 chip 选区；`Ctrl/Cmd+A` 由 `useComposerTextSelection` 选中全部 text segment（chip 不参与），各段 `bg-primary/15` 高亮；`Ctrl+C` 复制纯文本；`Backspace/Delete` 清空全部文本但保留 chip。
- 发送：`ready` mentions 走 `mentionsToWorkspaceAttachments(segments)`；展示正文用 `buildOutgoingFromSegments` 按 segment 顺序交错 `@relPath` 与文本；发送成功后 `clearMentions()` 重置为 `createEmptyDocument()`。
- `canSendComposerMessage` 使用 `getDocumentPlainText(segments)` 作为 `content`，`countSendableFromSegments` + `attachments.length` 作为 `attachmentCount`。
- `AttachmentPreview` 仍在 segment 行**之上**，与 workspace 引用语义分离。

### 4.3.y Composer 附件下拉菜单（新增）

- `AttachmentMenu` 仅渲染**实际可用**的入口；禁止保留「即将支持」「Coming soon」之类的临时占位项，避免给用户希望落空。
- 菜单内**不得**出现 `DropdownMenuLabel`（分组标题）；菜单内容只承载选项 item，统一 `图标 size-4 + 8px gap + 文案` 排版。
- 图片入口（`onPickImages`）的 `<input type="file">` 必须使用 `ACCEPTED_IMAGE_TYPES`（仅 IMAGE MIME）；Markdown / 文本入口必须使用 `ACCEPTED_TEXT_TYPES`（.md/.markdown/.txt + 对应 MIME），**不得**把 PDF/Office 等当前不支持的扩展名也加入 accept，否则用户能选中却又被前端拒绝。

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
- 展开动画仅 `fade-in` / 极小 `slide-in-from-top-*`，不使用缩放。

## 5. 总结

在编写 Tailwind 类名时，时刻问自己：**“这个样式在 macOS/Windows 原生应用中会出现吗？”** 如果答案是否定的（比如鼠标放上去按钮会跳一下），请坚决将其移除。保持 UI 的专业、冷静与克制。