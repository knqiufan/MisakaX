# MisakaX 前端 UI 设计与编写规范

本文档定义了 MisakaX 桌面端 AI Agent 客户端的前端 UI 编写规范。在进行任何前端组件开发、页面重构或样式调整时，**必须严格遵守**本规范，以确保整体视觉风格和交互体验的高度一致性。

## 文档关系

- **壳层布局、主导航收起语义、会话列表工具区、对话页工作目录顶栏**等专项约定：见 [shell-and-workspace-ui-spec.md](./shell-and-workspace-ui-spec.md)。  
- **按钮、下拉菜单、Popover、Select、Dialog、Tooltip 等控件的细节与变体**：编写或调整时须同时对照 [button-menu-design-spec.md](./button-menu-design-spec.md)。

**最后审阅 / Last reviewed:** 2026-05-19（v2）

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

### 4.3.x Composer 行内对齐与 Mention Pill（新增）

- Composer 顶层 `<div>` 必须使用 **`flex items-center gap-2`**：之前曾用 `items-end` 导致圆形附件按钮与输入框的中线错位（按钮整体下沉一格）。**任何**在 Composer 同一行放置「附件按钮 + 输入容器 + 发送按钮」的实现都必须遵守 `items-center`。
- 文件树「@ 引用」加入的工作区文件必须在 Composer 的输入容器内、`AttachmentPreview` **之上**以 chip（pill）的形式展示，组件为 `MentionPills`：
  - 视觉：`h-6` 高度、`rounded-full`、`border border-[color:var(--border-muted)] bg-[color:var(--surface-card-strong)]`，左侧 `AtSign` 图标使用 `text-primary/80`；
  - 行为：hover 显示删除 `X`，删除 = `composer-store.removeMention(id)`；
  - 数据：用 `mention.relPath` 作为 `title` 显示完整相对路径，`mention.name` 作为主标题；
  - 发送：在 `handleSend` 中通过 `composeMessageContent(trimmed, mentions)` 把 `@rel/path` 前置到正文（多个引用以空格连接，加两个换行隔开正文），发送成功后 `clearMentions()`；
  - 状态：mention 列表持久化作用域仅在当前 Composer 生命周期内，**禁止**与 attachments 混淆为同一数组。
- `canSendComposerMessage` 的可发送判定必须把 `mentions.length` 计入「有 payload」，否则仅引用文件无文本时按钮会异常禁用。

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

## 5. 总结

在编写 Tailwind 类名时，时刻问自己：**“这个样式在 macOS/Windows 原生应用中会出现吗？”** 如果答案是否定的（比如鼠标放上去按钮会跳一下），请坚决将其移除。保持 UI 的专业、冷静与克制。