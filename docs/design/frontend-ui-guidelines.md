# MisakaX 前端 UI 设计与编写规范

本文档定义了 MisakaX 桌面端 AI Agent 客户端的前端 UI 编写规范。在进行任何前端组件开发、页面重构或样式调整时，**必须严格遵守**本规范，以确保整体视觉风格和交互体验的高度一致性。

## 文档关系

- **壳层布局、主导航收起语义、会话列表工具区、对话页工作目录顶栏**等专项约定：见 [shell-and-workspace-ui-spec.md](./shell-and-workspace-ui-spec.md)。  
- **按钮、下拉菜单、Popover、Select、Dialog、Tooltip 等控件的细节与变体**：编写或调整时须同时对照 [button-menu-design-spec.md](./button-menu-design-spec.md)。

**最后审阅 / Last reviewed:** 2026-06-07（v5）

## 1. 设计理念 (Design Philosophy)

MisakaX 的目标是打造一个**现代化、专业、克制的桌面端 Agent 客户端**。
- **桌面端直觉优先**：交互应该干脆利落，符合桌面软件的使用直觉，而不是移动端或网页端的体验。
- **克制的动效**：避免过度设计，去除花哨的过渡和缩放，让用户的注意力集中在内容和 AI 交互上。
- **高信息密度**：UI 元素应紧凑合理，留白适中。

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

项目基于 **Tailwind CSS v4** 和 **shadcn/ui (New York 风格 / Zinc 基础色)** 构建。

### 3.1 颜色变量 (CSS Variables)
- 强制使用 CSS 变量来定义颜色，以完美适配深色/浅色模式。
- 常用背景：`bg-background`、`bg-muted`、`bg-[color:var(--surface-card)]`、`bg-[color:var(--surface-hover)]`。
- 常用边框：`border-border`、`border-[color:var(--border-subtle)]`、`border-[color:var(--border-strong)]`。
- 常用文字：`text-foreground`、`text-muted-foreground`。

### 3.2 阴影与层级 (Shadows & Elevation)
- **扁平化为主**：基础按钮、输入框等控件尽量减少阴影，使用边框（Border）来区分边界。
- **弹出层阴影**：仅在下拉菜单、模态框、Tooltip 等悬浮层级（z-index 较高）的元素上使用 `shadow-lg` 或自定义的深阴影，并配合 `backdrop-blur` 增加质感。

## 4. 组件编写原则 (Component Guidelines)

### 4.1 按钮 (Buttons)
- 按钮应具有明确的边界。
- 主要按钮（Primary）使用实色背景，次要按钮（Secondary/Outline/Ghost）使用透明或半透明背景。
- 悬停（Hover）时仅改变背景色亮度或透明度，不改变物理尺寸和位置。

### 4.2 菜单与导航 (Navigation & Menus)
- 侧边栏导航项（NavItem）选中时应有明显的视觉区分（如强调色边框或背景），未选中时保持低调。
- 避免在菜单项切换时加入复杂的宽度/高度动画。

### 4.3 对话输入区（Composer 单行）

- 附件按钮、文本域、发送/停止按钮放在同一 `flex` 行时，使用 **`items-center`** 做垂直居中；避免在单行默认高度下配合过高的 `min-height` 使用 `items-end`，否则圆形图标按钮容易视觉上“沉底”。
- 同行圆形图标按钮宜统一触控尺寸（例如均为 `size-8`），附件与发送样式对齐。
- 新版 Composer 中，附件入口使用外置左侧圆形 `+` 按钮，不放入输入框内部；按钮与输入容器同属一行，输入容器内部只承载附件预览、文本域与发送/停止按钮。
- `textarea` 单行态必须通过 `leading-[20px]` 与 `py-2` 保证文本视觉垂直居中；禁止只用 `min-h-[36px]` 撑高文本域，否则占位符会贴近左上角。
- 附件预览支持图片缩略图与文本文件卡片两类；非图片附件不得伪装成图片缩略图，应使用文件图标、文件名与大小信息表达。
- **附件入口与 Radix asChild 嵌套（必读，新增）**：
  - `AttachButton` 作为 `DropdownMenuTrigger asChild` 的 child 时，**必须**用 `forwardRef` 实现，并把 trigger 注入的 `ref` 与 `...rest` props 透传到底层 `<button>`，否则 Dropdown 的 click/keyboard handler 与定位 anchor 都无法生效，会出现「按钮点击无反应」的回归。
  - **禁止**给 trigger 的 child 元素再传 `onClick={() => undefined}` 等占位回调；Radix Slot 会保留 child 已声明的事件，且空回调可能掩盖真正的 trigger 行为。
  - 文案使用 `chat.composer.attach` 等 i18n key，桌面端中文 UI 优先使用「添加附件」等动名词组合，避免单字「附加」造成歧义。

### 4.3.x Composer 行内文件引用 Chip

- Composer 顶层 `<div>` 必须使用 **`flex items-center gap-2`**；输入行内 `ComposerInlineField` 必须带 **`flex-1 min-w-0`**，发送按钮 `shrink-0` 贴右，禁止仅按内容宽度收缩导致按钮悬空中部。
- 数据模型为 **`ComposerSegment[]`**（`text` 与 `mention` 交错），引用插入当前 `composerCursor` 位置（`insertMentionAtCursor`），而非固定堆在输入框最前。`normalizeSegments` 会 **prune 掉 mention 之间的空 text 段**，避免空 `textarea` 默认宽度把 chip 撑开；仅保留末尾 text 段作为输入区。
- `ComposerInlineField` 按 segment 渲染多个 `SegmentTextarea` 与 `InlineMentionChip` 交错；**仅最后一个 text segment** 使用 `flex-1 min-w-[2rem]` 撑满剩余宽度，前面的 text segment 用 **`scrollWidth` 动态测宽**（`shrink-0`、`whitespace-nowrap`、`wrap="off"`），**禁止** `cols={1}` 或 `Nch` 固定宽度（中文会竖排）。
- `InlineMentionChip` 视觉（透明蓝底，对齐 Cursor 行内引用）：
  - ready：`h-6`、`rounded-md`、`border border-sky-400/30`、`bg-sky-500/10`，`FileTypeIcon` + 文件名同色（如 `.tsx` → `text-sky-400`）；
  - loading：`opacity-60` + `Loader2`；
  - error：`bg-destructive/8` + `AlertTriangle` + `text-destructive`；
  - **禁止**显示删除 `X` 按钮；删除通过 **Backspace**（光标在 text 段 offset=0 时删前一个 mention）或 **Delete**（光标在 text 段末尾时删后一个 mention）完成。
- 引用触发后通过 `focusRequestId` + `composerCursor` 自动 focus 到插入点后的 text segment；**不**弹出「已引用」成功 toast（失败仍 `toast.error`）。
- **跨段全选**：多 `textarea` 无法原生跨 chip 选区；`Ctrl/Cmd+A` 由 `useComposerTextSelection` 选中全部 text segment（chip 不参与），各段 `bg-sky-500/20` 高亮；`Ctrl+C` 复制纯文本；`Backspace/Delete` 清空全部文本但保留 chip。
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

- **列表容器**（`MessageList`）不设 `max-w-*` 居中限制，消息撑满父容器可用宽度；仅保留少量水平 padding（`px-2`）提供呼吸空间。
- **无头像**：`MessageItem` 不渲染任何 Avatar 图标（User / Assistant），仅显示对话内容。
- **User 消息**：
  - 整条消息 `flex justify-end` 靠右停靠。
  - 气泡容器 `max-w-[80%]`，文字较短时自动收窄，文字超长时最大不超过父容器 80%。
  - 保留圆角背景气泡样式（`bg-primary text-primary-foreground`，`rounded-[var(--radius-ui-lg)]`）。
  - 气泡下方悬停显示 **复制** 和 **重新生成** 小图标按钮（`size-6`，hover 出现，`opacity-0 → group-hover/msg:opacity-100`）。
- **Assistant 消息**：
  - 容器 `w-full`，内容 100% 宽度平铺，不设气泡背景与边框，以纯文本/Markdown 形式直接输出。
  - 错误态仍使用 `border-destructive/45 bg-destructive/8` 等语义色容器（见 4.5）。
  - 底部悬停仍显示 Copy / Regenerate + TokenBadge。

## 5. 总结

在编写 Tailwind 类名时，时刻问自己：**“这个样式在 macOS/Windows 原生应用中会出现吗？”** 如果答案是否定的（比如鼠标放上去按钮会跳一下），请坚决将其移除。保持 UI 的专业、冷静与克制。