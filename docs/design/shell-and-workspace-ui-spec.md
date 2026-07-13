# MisakaX 壳层布局与工作区 UI 规范

| 属性 | 说明 |
|------|------|
| **用途** | 定义主窗口混合壳结构、会话侧栏、对话页顶栏、设置页与工作区布局语义。 |
| **受众** | 负责 `AppShell`、`UnifiedTopBar`、`SessionPanel`、`SettingsSidebar`、`ChatPage`、`WorkspaceBar`、`SettingsPage` 及相关布局的前端开发者。 |
| **最后审阅** | 2026-07-14（v16） |

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
   - `chat`：会话列表 + Quick actions + 底栏（通知 / 用户菜单 / Settings）。
   - `settings`：整列换成 `SettingsSidebar`（五分区导航）。
   - `skills` / `knowledge` / `dashboard` / `notifications`：**无左栏**，仅 TopBar + 主内容。
3. **主内容区**：当前页面；无会话时为居中 hero（见 §6）。
4. **Sidecar / Agent 状态**：不在会话底栏展示；放在设置 → 关于 → 系统信息（`SidecarStatusBadge`，异常时可点重启）。

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
- Chat 页内仅保留「消息 ↔ Explorer」的 `react-resizable-panels`；**不再**用百分比 Panel 承载会话列表。
- Chat ↔ Explorer 分隔：命中区约 8px（`w-2`），默认细线不可见，hover/drag 显线。

### 1.3 视觉连续性

Session / Settings 左栏 / Main 使用 `--sidebar`、`--border`、`--background` 等暖色 charcoal token；选中态用 `bg-sidebar-accent`，禁止冷蓝描边。

---

## 2. 会话列表（SessionPanel）

### 2.1 Quick actions

- 自上而下：`新建会话` 全宽 ghost 行（`h-9`、`rounded-xl`、`text-[13px]`）+ 搜索输入（同高、同圆角）。

### 2.2 底栏（SessionPanelFooter，必须遵守）

固定在会话列表底部（`shrink-0`），行形与 Quick actions 一致（`h-9 rounded-xl text-[13px]`）：

1. **通知** → `{ page: "notifications" }`
2. **用户菜单** → 技能 / 知识库 / 仪表盘（进现有页）；个人中心 / 退出保持 disabled；**不**在菜单内重复 Settings
3. **Settings** 全宽行 → `{ page: "settings" }`

Sidecar 状态**不**放在底栏（见 §1 / 关于页）。

### 2.3 会话行（SessionItem）

- 行高 `h-8`、`rounded-xl`、`px-3`、标题 `text-[13px]` 单行截断。
- 右侧时间固定宽约 `38px`、`text-[11px] text-muted-foreground/40`；hover 时让位给 ⋯ 菜单。
- 激活：`bg-sidebar-accent`；过渡 `duration-150`。

### 2.4 分区标题

- `text-[13px] font-semibold text-sidebar-foreground/55`；可折叠分组 caret 12px。

---

## 3. 对话页顶栏 — 工作目录（Workspace bar）

### 3.1 信息层级

工作目录条是 Agent 场景的**关键上下文**。

- **未设置目录**：标题（`text-sm font-medium`）+ 短说明（`text-xs muted`）+ 主 CTA「选择目录」。
- **已设置目录**：文件夹显示名作标题；完整路径 `font-mono text-[10px] text-muted-foreground/60` 单行截断 + Tooltip。

### 3.2 布局与令牌

- 高度约 TopBar 量级：`min-h-10`，`px-3 py-1.5`，底边 `border-border/40`，表面 `bg-background`（无重 blur / 大图标槽）。
- 图标操作：`ghost` + `size-7`；Explorer / Tool Logs **打开态**用 `variant="secondary"`（可再点关闭 = **toggle**，不强制 disabled）。
- Explorer：`onToggleExplorer` 必须真正开/关；无工作目录时可不传回调（按钮 disabled）。
- Tool Logs：与 Explorer **语义解耦**；本期挂载为**聊天列内抽屉**（WorkspaceBar 下方，`max-h-[min(40vh,320px)]`），头 `h-10` + 11px uppercase + 关闭 X；开态传 `toolLogsOpen`。
- 控件细节见 [button-menu-design-spec.md](./button-menu-design-spec.md)。

---

## 4. 设置页

参考视觉：[`docs/ui/04-settings.md`](../ui/04-settings.md)。**本期 IA** 仍为五分区：`general` / `models` / `mcp` / `appearance` / `about`（不扩 overview / runtime / health 等）。

### 4.1 导航壳

- 进入 settings 时，**壳层左栏整列**换成 `SettingsSidebar`（与会话列表共用 `sessionListWidth` / gutter）。
- 项形：`h-9 px-3 rounded-xl text-[13px]`（与会话 Quick actions 同形）。
- `SettingsPage` **只渲染内容槽**；不再内嵌桌面左导航或窄屏横条 pill（避免双导航）。
- Back 在 `UnifiedTopBar`（ghost sm、`h-7`、ArrowLeft）；**所有非 chat 页**（含 settings 与技能/知识库/仪表盘/通知）均提供返回 chat。

导航唯一源：`src/components/settings/nav-config.ts`。路由仍为 Zustand `route.page === "settings"`（无 React Router）。

### 4.2 内容槽与页宽

- 内容：`p-6 lg:p-10`；必须 `mx-auto`。
- 多数分区：`max-w-4xl` + `space-y-10`；Appearance / About：`max-w-3xl` + `space-y-6`。

### 4.3 卡片与表单

| Primitive | 规格 |
|-----------|------|
| `SettingsCard` | `rounded-lg border-border/50 bg-card p-5`，**无阴影** |
| `SettingsSubCard` | `rounded-md bg-muted/40` + inset `divide-y divide-border/50` |
| `FieldRow` | label/description 左、Switch/Select 右；禁止手写 `flex justify-between + Switch` |
| `StatusPill` | `text-[10px]` + `size-1.5` 色点（Provider / MCP 运行态） |

Provider 目录网格仅 `md:grid-cols-2`。Appearance 主题分段：`rounded-md bg-muted p-0.5`；**无**强调色选择器；**无**「侧边栏默认展开」折叠开关（左栏已无折叠态）。

模型选择器 / Provider Dialog 业务规则不变；控件跟 [button-menu-design-spec.md](./button-menu-design-spec.md)。

---

## 5. 对话页右侧 Workspace Explorer

参考视觉：[`docs/ui/03-workspace.md`](../ui/03-workspace.md)（**仅 chrome**；Misaka **不**复刻其多轨 / 480px 像素宽模型）。**产品模型**仍为 Chat 内单一 Explorer（文件树 + Monaco），不做 Git/Widget/Assistant 多轨。

宽度：Chat↔Explorer 用 `react-resizable-panels` **百分比**（默认约 70/30，Explorer `min 18%` / `max 55%`）；**不以** CodePilot `SIDEBAR_DEFAULT_WIDTH=480` 为 Misaka 目标。分隔命中区 `w-2`，默认细线不可见，hover/drag 显线（与会话栏 gutter 气质一致；类名集中在 `panelResizeHandle.ts`）。

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

- 流式：自动展开 + Shimmer 文案
- 结束：1s 后自动折叠一次；显示「Thought for N seconds」
- 正文：走 `MessageResponse`（可 Markdown）

### 6.5 工具

- 主路径：`ToolActionsGroup` — 左色线 + 紧凑行 + 状态绿/红/转圈
- Tool Logs 复用同一组件；MCP 审批仍走 `ToolApprovalDialog`

### 6.6 历史

| 项 | 值 |
|----|-----|
| 首屏 | `limit=50` |
| 更早 | `limit=100` + `beforeId` |
| 虚拟列表 | `@tanstack/react-virtual`；estimate 220；overscan 6 |
| Prepend | 保位 `scrollToIndex(align:'start')`；仅尾追加才自动置底 |

Composer 外壳：`rounded-2xl` 输入组 + `shadow-[var(--shadow-diffuse)]`（**会话页与 Hero 共用同一阴影**，禁止 Hero 外包再套一层 diffuse）。单行输入行高约 **32px 文本域**（`leading-8` + `py-0`，与发送 `size-8` 对齐）+ 外壳 `py-1.5`；输入行 **`items-center`**。发送 / 附件外置钮均为 `size-8`（`icon-sm`）`rounded-full`，图标约 `size-3.5`；可发送态**不得**用 ghost（避免 hover 图标变黑）。附件与发送 Tooltip：`delayDuration={2000}` + fade 约 150ms。底部 Model/MCP/Skill 行 `mt-2` + `pl-10`。附件预览胶囊：`rounded-full border-border/40 bg-muted`。行内文件引用 chip（`InlineMentionChip`）：`rounded-md border-primary/25 bg-primary/8`，与附件胶囊可区分。不引入 CodePilot Hood vibrancy / ActionBar 产品控件。

---

## 7. 首页 Hero（无会话）

- 组件：`NewChatWelcome`（`ChatPage` 在无 `activeSession` 时渲染）。
- 布局：垂直居中，`max-w-3xl`，`px-4 py-8`。
- 品牌：`MisakaLogo` `h-9 w-9`（浅色圆体 + 双闪电）+ 时段问候 `text-3xl font-medium` + 短提示。
- Composer：复用 `MessageInput`；首发经 `pendingOutbound` 创建会话后由 `ChatView` 发送。
- 下方引导：可选「选择工作目录」。

---

## 8. 明确不在本期壳层范围

- macOS 14px CardFrame 悬浮卡 / vibrancy / traffic-light 避让  
- React Router URL（`/chat/[id]`、`/settings/*`）  
- Settings 扩展 IA（overview / runtime / health / usage / assistant / tasks / bridge）  
- Workspace 多轨（独立 FileTree 轨、Assistant 轨、Git / Widget 固定 Tab）  
- Rewind / Checkpoint / RuntimeSwitch / SplitChat / Composer ActionBar（Runtime·Permission·Cockpit）  
