# MisakaX 壳层布局与工作区 UI 规范

| 属性 | 说明 |
|------|------|
| **用途** | 定义主窗口混合壳结构、会话侧栏、对话页顶栏、设置页与工作区布局语义。 |
| **受众** | 负责 `AppShell`、`UnifiedTopBar`、`Sidebar`、`SessionPanel`、`ChatPage`、`WorkspaceBar` 及相关布局的前端开发者。 |
| **最后审阅** | 2026-07-13（v6） |

## 相关文档

- **全局动效、色彩、组件原则（桌面 Agent 气质、charcoal 主色）**  
  → [frontend-ui-guidelines.md](./frontend-ui-guidelines.md)

- **按钮、图标按钮、下拉菜单、Popover、Select、Dialog、Tooltip、Toast 等控件细节**  
  → 凡实现或调整上述控件，**须对照** [button-menu-design-spec.md](./button-menu-design-spec.md)。

- **可复刻参考（CodePilot 逆向）** → [`docs/ui/01-overall-and-home.md`](../ui/01-overall-and-home.md)

---

## 1. 混合壳信息架构

MisakaX 主界面（本期 Win / 跨平台**实心底**，无 macOS 悬浮卡）：

```
AppShell (flex-col h-screen)
  UnifiedTopBar          <!-- h-10 / 40px -->
  ContentRow (flex-1)
    NavRail (Sidebar)    <!-- 展开 160px / 折叠 w-14 -->
    [chat 且开启时] SessionPanel (默认 240px，可拖 180–300)
    [chat 且开启时] ResizeGutter (8px)
    Main → ContentArea
```

1. **顶栏 `UnifiedTopBar`**：全局 40px；chat 页提供会话列表开关；非 chat 页显示页面标题；settings 提供返回。
2. **主导航栏（NavRail）**：对话 / 技能 / 知识库 / 仪表盘等入口；底栏 Settings / 用户入口。
3. **会话列表栏（SessionPanel）**：**仅** `route.page === "chat"` 且 `sessionListOpen` 时显示；固定像素宽，非百分比。
4. **主内容区**：当前页面；无会话时为居中 hero（见 §6）。

### 1.1 收起行为（必须遵守）

- **TopBar 会话列表开关**只控制 SessionPanel（第 2 列）显隐，**不**折叠 NavRail。
- **NavRail 折叠**（设置项 / 窄屏）只收窄主导航为图标轨，**不**强制隐藏会话栏。
- 两列状态解耦：`sidebarCollapsed` 与 `sessionListOpen` 独立存储。

### 1.2 断点

- `LG_BREAKPOINT = 1024px`：窄于该宽度时，默认折叠 NavRail，并关闭会话列表（用户可再手动打开）。

### 1.3 会话栏宽度（必须遵守）

| 常量 | 值 |
|------|-----|
| 默认宽 | `240px` |
| min / max | `180` / `300` |
| 持久化 | `localStorage.misakax_chatlist_width` |
| 分隔 | `ResizeGutter` 宽 8px；默认线不可见，hover/drag 显线；双击重置 240 |

- Gutter 必须是会话栏的**兄弟节点**，禁止放进 SessionPanel 内部。
- Chat 页内仅保留「消息 ↔ Explorer」的 `react-resizable-panels`；**不再**用百分比 Panel 承载会话列表。

### 1.4 视觉连续性

Nav / Session / Main 使用 `--sidebar`、`--border`、`--background` 等暖色 charcoal token；选中态用 `bg-sidebar-accent`，禁止冷蓝描边。

---

## 2. 会话列表顶部工具区

### 2.1 Quick actions

- 自上而下：`新建会话` 全宽 ghost 行（`h-9`、`rounded-xl`、`text-[13px]`）+ 搜索输入（同高、同圆角）。
- Settings **留在 NavFooter**，会话栏不重复底栏 Settings。

### 2.2 会话行（SessionItem）

- 行高 `h-8`、`rounded-xl`、`px-3`、标题 `text-[13px]` 单行截断。
- 右侧时间固定宽约 `38px`、`text-[11px] text-muted-foreground/40`；hover 时让位给 ⋯ 菜单。
- 激活：`bg-sidebar-accent`；过渡 `duration-150`。

### 2.3 分区标题

- `text-[13px] font-semibold text-sidebar-foreground/55`；可折叠分组 caret 12px。

---

## 3. 对话页顶栏 — 工作目录（Workspace bar）

### 3.1 信息层级

工作目录条是 Agent 场景的**关键上下文**，不可使用过小字号、松散纯文本凑合。

- **未设置目录**：须有明确**标题**、**简短说明**、以及醒目的**主操作**（如「选择目录」）。  
- **已设置目录**：**文件夹显示名**作为标题层级；完整路径次要展示（可单行截断 + Tooltip）。

### 3.2 布局与令牌

- 足够垂直内边距与可选最小高度；底部分割线使用 `border-border` / `--border-muted`。
- 后续可将高度向 TopBar 的 40px 量级收敛；本期优先首页 hero。

### 3.3–3.5

按钮 / Tooltip / Tool Logs / Workspace Explorer 语义沿用既有约定，控件细节见 [button-menu-design-spec.md](./button-menu-design-spec.md)。

---

## 4. 设置页与模型选择器

（Provider 弹窗、对话页模型选择器规则不变；Appearance **仅** light / dark / system，无强调色选择器。）

---

## 5. 对话页右侧 Workspace Explorer

（文件树 / 编辑器分栏语义不变；Explorer 与 Tool Logs 解耦。）

---

## 6. 首页 Hero（无会话）

- 组件：`NewChatWelcome`（`ChatPage` 在无 `activeSession` 时渲染）。
- 布局：垂直居中，`max-w-3xl`，`px-4 py-8`。
- 品牌：`MonolithIcon` `h-9 w-9` + 时段问候 `text-3xl font-medium` + 短提示。
- Composer：复用 `MessageInput`；首发经 `pendingOutbound` 创建会话后由 `ChatView` 发送。
- 下方引导：可选「选择工作目录」。

---

## 7. 明确不在本期壳层范围

- macOS 14px CardFrame 悬浮卡 / vibrancy / traffic-light 避让  
- React Router URL（`/chat/[id]`）  
- 完整聊天气泡 / Markdown / Settings 卡片系统深改  
