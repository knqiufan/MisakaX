# Phase 1：基础框架与核心 UI — 详细实施方案

> **所属项目：** MisakaX
> **阶段：** Phase 1（第 2-4 周）
> **总预估：** ~35 小时（含 Vibe Coding 加速）
> **前置条件：** Phase 0 已完成（Tauri + React + Python 环境就绪，SQLite 数据库初始化，`cargo check` 和 `npm run build` 通过）
> **前置文档：** [PHASE_0_DETAILED_PLAN.md](./PHASE_0_DETAILED_PLAN.md)、[MISAKAX_IMPLEMENTATION_PLAN.md](./MISAKAX_IMPLEMENTATION_PLAN%20-%20Opus4.6.md)

---

## 目录

1. [阶段目标与验收标准](#1-阶段目标与验收标准)
2. [UI 设计方案](#2-ui-设计方案)
3. [任务 1.1：AppShell 主布局](#3-任务-11appshell-主布局)
4. [任务 1.2：路由系统](#4-任务-12路由系统)
5. [任务 1.3：侧边栏导航](#5-任务-13侧边栏导航)
6. [任务 1.4：设置页面](#6-任务-14设置页面)
7. [任务 1.5：Rust 端设置 CRUD Commands](#7-任务-15rust-端设置-crud-commands)
8. [任务 1.6：路由配置 UI + Rust 后端（多 Provider API Key 管理）](#8-任务-16路由配置-ui--rust-后端)
9. [任务 1.7：主题系统](#9-任务-17主题系统)
10. [任务 1.8：i18n 国际化](#10-任务-18i18n-国际化)
11. [任务 1.9：Zustand Stores 搭建](#11-任务-19zustand-stores-搭建)
12. [任务 1.10：Tauri IPC 封装层](#12-任务-110tauri-ipc-封装层)
13. [Phase 1 完整验证清单](#13-phase-1-完整验证清单)
14. [详细 TODO 列表](#14-详细-todo-列表)

---

## 1. 阶段目标与验收标准

### 1.1 核心目标

搭建完整的 UI 骨架 + 设置系统 + 主题/国际化。Phase 结束时，用户可以看到完整的界面布局，可以配置 API Key 并切换主题和语言。

### 1.2 任务依赖关系

```
1.9 Zustand ────────────────────────────────────────────────┐
1.10 IPC 封装 ──────────────────────────────────────────────┤
                                                             │
1.1 AppShell ──→ 1.2 路由 ──→ 1.3 侧边栏 ──┐               │
                                              │               │
1.7 主题系统 ─────────────────────────────────┤               │
1.8 i18n ─────────────────────────────────────┤               │
                                              ↓               │
                                    1.4 设置页面 ←─────────────┘
                                         │
                                         ↓
                              1.5 Rust 设置 CRUD
                                         │
                                         ↓
                              1.6 Provider API Key 管理
```

> **关键路径：** 1.9/1.10（基础设施）→ 1.1 → 1.2 → 1.3 → 1.4 → 1.5 → 1.6
> **可并行：** 1.7/1.8 与 1.1~1.3 并行推进；1.9/1.10 可最先独立完成

### 1.3 验收标准（Done Definition）

| # | 验收条件 | 验证方式 |
|---|---------|---------|
| AC-1 | 应用启动后显示完整布局（侧边栏 + 顶栏 + 内容区），侧边栏底部有通知和用户头像 | 视觉检查 |
| AC-2 | 侧边栏可导航至 Chat/Skills/Knowledge/Dashboard/Notifications 页面，底部用户菜单可跳转 Settings | 点击导航切换 |
| AC-3 | 设置页面包含 通用/模型/MCP/外观/关于 五个子面板 | 点击 Tab 切换 |
| AC-4 | 可在 UI 中添加/编辑/删除 API Key，保存后重启仍存在 | CRUD 操作 |
| AC-5 | 明/暗/跟随系统 三种主题可切换，切换后即时生效 | 切换主题 |
| AC-6 | 可切换中/英文界面，切换后所有文字更新 | 切换语言 |
| AC-7 | Zustand stores 数据变化能驱动 UI 更新 | 开发者工具检查 |
| AC-8 | Tauri IPC 调用有统一错误处理和 Loading 状态 | 网络异常模拟 |
| AC-9 | 侧边栏可折叠/展开，移动端响应式适配 | 窗口缩放 |
| AC-10 | API Key 采用加密存储（不明文写入磁盘） | 检查 SQLite/config |

### 1.4 交付物一览

- 完整的应用骨架 UI（可导航各页面）
- 设置页面可保存/读取配置
- API Key 加密存储
- 主题切换即时生效
- 中英文切换
- 完整的 Zustand 状态管理
- 类型安全的 Tauri IPC 封装

---

## 2. UI 设计方案

### 2.1 设计理念

| 原则 | 说明 |
|------|------|
| **简洁** | 去除一切不必要的视觉干扰，内容优先 |
| **克制** | 色彩使用极度克制，仅强调色点缀关键交互 |
| **呼吸感** | 大量留白，元素间距宽松，让界面"呼吸" |
| **一致性** | 统一的圆角、阴影、动效语言 |
| **无障碍** | 高对比度文字、键盘可达、语义化标签 |

### 2.2 设计风格

**整体风格：** 参考 Claude Desktop / Linear / Raycast 的极简工作台美学。

- **色彩体系：** Zinc 中性灰为底 + Indigo(#6366f1) 强调色，避免花哨配色
- **排版：** Inter 为主字体（西文），系统默认为中文回退；JetBrains Mono 为代码字体
- **圆角：** 统一 `radius: 0.625rem`（10px），输入框稍小 8px
- **阴影：** 仅 Popover/Modal/DropMenu 使用柔和阴影，主体内容无阴影
- **动效：** 200ms ease-out 过渡，侧边栏折叠使用 300ms spring 动画

### 2.3 全局布局架构

```
┌─────────────────────────────────────────────────────────────────────┐
│ Window Chrome (Tauri native titlebar)                                │
├────────────┬────────────────────────────────────────────────────────┤
│            │  TopBar (48px)                                          │
│            │  ┌──────────────────────────────────────────────────┐  │
│  Sidebar   │  │  页面标题              [全局搜索 ⌘K]               │  │
│  (240px)   │  └──────────────────────────────────────────────────┘  │
│            │                                                         │
│ ┌────────┐ │  Content Area                                          │
│ │ Logo   │ │  ┌──────────────────────────────────────────────────┐  │
│ │        │ │  │                                                    │  │
│ ├────────┤ │  │                                                    │  │
│ │ Chat   │ │  │                                                    │  │
│ │ Skills │ │  │           根据路由渲染不同页面内容                    │  │
│ │ Know.  │ │  │                                                    │  │
│ │ Dash.  │ │  │                                                    │  │
│ │        │ │  │                                                    │  │
│ ├────────┤ │  │                                                    │  │
│ │🔔通知   │ │  │                                                    │  │
│ │👤头像   │ │  │                                                    │  │
│ │◁ 折叠  │ │  │                                                    │  │
│ └────────┘ │  └──────────────────────────────────────────────────┘  │
├────────────┴────────────────────────────────────────────────────────┤
│ StatusBar (24px, optional)                                           │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.4 侧边栏设计

| 状态 | 宽度 | 内容 |
|------|------|------|
| 展开 | 240px | 图标 + 文字标签 |
| 折叠 | 56px | 仅图标，Tooltip 提示 |

**分区布局：**
1. **顶部** — Logo + 应用名（折叠时仅 Logo icon）
2. **主导航** — Chat / Skills / Knowledge / Dashboard（图标 + 文字）
3. **底部固定** — 通知入口 / 用户头像（含下拉菜单）/ 折叠按钮

**用户头像下拉菜单项：**
- 个人中心（跳转 Profile 页面，Phase 后续实现）
- 设置（跳转 Settings 页面）
- 分隔线
- 退出登录（清除本地 session，Phase 后续实现）

**通知入口：**
- 图标右上角显示未读数量 Badge（红色圆点或数字）
- 点击后跳转通知页面（`{ page: "notifications" }`）
- 展开时显示"通知"文字 + Badge；折叠时仅显示图标 + Badge

**导航项视觉：**
- 默认：`text-muted-foreground`，无背景
- Hover：`bg-accent` 浅色背景 + `text-accent-foreground`
- Active：`bg-primary/10` + `text-primary` + 左侧 3px 竖线指示条
- 过渡：150ms ease-out

### 2.5 TopBar 设计

- 高度固定 48px
- 左侧显示当前页面标题（可带面包屑）
- 右侧功能区：全局搜索入口 (⌘K)
- 背景色 `bg-background/80 backdrop-blur-md` — 半透明模糊效果
- 底部 1px `border-border` 分隔线
- TopBar 保持轻量，不承载用户操作入口（通知和用户菜单已移至侧边栏底部）

### 2.6 设置页面设计

**采用左侧 Tab 列表 + 右侧内容区的双栏布局：**

```
┌─────────────────────────────────────────────────────────────┐
│ Settings                                                     │
├──────────────┬──────────────────────────────────────────────┤
│  ⚙ General   │  General Settings                             │
│  🤖 Models   │  ┌──────────────────────────────────────────┐ │
│  🔌 MCP     │  │  Language    [English ▼]                  │ │
│  🎨 Appearance│  │  Startup     [☑] Start on login         │ │
│  ℹ About    │  │  Data Dir    ~/.misakax/    [Browse]      │ │
│              │  │  ...                                      │ │
│              │  └──────────────────────────────────────────┘ │
└──────────────┴──────────────────────────────────────────────┘
```

**各 Tab 内容概览：**

| Tab | 内容 |
|-----|------|
| General | 语言、开机启动、数据目录、代理设置、日志级别 |
| Models | Provider 列表（可添加/编辑/删除）、默认模型选择、API Key 管理 |
| MCP | MCP Server 配置列表、连接状态、工具权限 |
| Appearance | 主题模式（明/暗/系统）、强调色选择器、字体大小、紧凑模式 |
| About | 版本号、更新检查、开源许可、系统信息 |

### 2.7 Chat 页面预留设计（Phase 2 实现）

Phase 1 仅放置占位内容，但布局已规划：

```
┌─────────────────────────────────────────────────┐
│ Chat 页面                                        │
├──────────────┬──────────────────────────────────┤
│ 会话列表面板  │  消息流区域                         │
│ (260px)      │  ┌──────────────────────────────┐ │
│ ┌──────────┐ │  │                              │ │
│ │ 搜索框    │ │  │    空状态 / 欢迎画面           │ │
│ │ + 新会话  │ │  │                              │ │
│ ├──────────┤ │  │                              │ │
│ │ 会话 1   │ │  │                              │ │
│ │ 会话 2   │ │  └──────────────────────────────┘ │
│ │ ...      │ │  ┌──────────────────────────────┐ │
│ └──────────┘ │  │  输入框 (Phase 2)              │ │
│              │  └──────────────────────────────┘ │
└──────────────┴──────────────────────────────────┘
```

### 2.8 配色方案详细定义

**亮色模式 (Light)：**

| 角色 | 色值 | 用途 |
|------|------|------|
| Background | `#FFFFFF` | 主背景 |
| Card | `#FAFAFA` | 卡片/面板背景 |
| Sidebar | `#FAFAFA` | 侧边栏背景 |
| Border | `#E4E4E7` | 分隔线/边框 |
| Muted Text | `#71717A` | 次要文字 |
| Primary Text | `#18181B` | 主文字 |
| Accent | `#6366F1` | 强调色（Indigo-500） |
| Accent Hover | `#4F46E5` | 强调色悬停态 |
| Destructive | `#EF4444` | 删除/危险操作 |

**暗色模式 (Dark)：**

| 角色 | 色值 | 用途 |
|------|------|------|
| Background | `#09090B` | 主背景 |
| Card | `#18181B` | 卡片/面板背景 |
| Sidebar | `#0F0F11` | 侧边栏背景（比主背景更深） |
| Border | `#27272A` | 分隔线/边框 |
| Muted Text | `#A1A1AA` | 次要文字 |
| Primary Text | `#FAFAFA` | 主文字 |
| Accent | `#818CF8` | 强调色（Indigo-400，暗色下提亮） |
| Accent Hover | `#6366F1` | 强调色悬停态 |
| Destructive | `#F87171` | 删除/危险操作 |

### 2.9 组件设计规范

| 组件 | 规范 |
|------|------|
| **Button** | 高度 36px(sm)/40px(default)/44px(lg)，圆角 `radius-md`，禁用态 opacity-50 |
| **Input** | 高度 40px，1px border，focus 时 ring-2 ring-accent/50 |
| **Select** | 使用 shadcn/ui Select，下拉菜单圆角 `radius-lg` |
| **Card** | 1px border，无阴影（减少视觉噪音），padding 24px |
| **Tabs** | 使用下划线风格（非卡片风格），active 项 2px 底线 |
| **Toggle** | Switch 组件，16px 圆形滑块，36x20px 轨道 |
| **Toast** | 右下角弹出，自动消失 5s，支持 action 按钮 |
| **Dialog** | 居中弹出，backdrop blur，max-width 480px |

### 2.10 响应式断点

| 断点 | 宽度 | 行为 |
|------|------|------|
| Desktop | ≥1280px | 完整三栏 |
| Tablet | 900-1279px | 侧边栏自动折叠为 56px |
| Minimum | 900px (minWidth) | Tauri 窗口最小尺寸限制 |

---

## 3. 任务 1.1：AppShell 主布局

> **预估时间：** 4h
> **产出：** 完整的三栏布局框架组件

### 3.1 文件结构

```
src/
├── components/
│   ├── layout/
│   │   ├── AppShell.tsx          # 主布局容器
│   │   ├── Sidebar.tsx           # 侧边栏
│   │   ├── SidebarHeader.tsx     # 侧边栏顶部（Logo）
│   │   ├── SidebarFooter.tsx     # 侧边栏底部（通知 + 用户头像 + 折叠）
│   │   ├── NavItem.tsx           # 导航项组件
│   │   ├── UserMenu.tsx          # 用户头像下拉菜单
│   │   ├── TopBar.tsx            # 顶栏（标题 + 搜索）
│   │   └── ContentArea.tsx       # 内容区包装
│   └── ui/                       # shadcn/ui 组件
├── App.tsx                        # 根组件（使用 AppShell）
└── ...
```

### 3.2 AppShell 组件设计

```tsx
// src/components/layout/AppShell.tsx
interface AppShellProps {
  children: React.ReactNode;
}

export function AppShell({ children }: AppShellProps) {
  // 从 Zustand store 获取侧边栏状态
  // 响应式处理
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background">
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <TopBar />
        <main className="flex-1 overflow-auto">
          {children}
        </main>
      </div>
    </div>
  );
}
```

### 3.3 关键实现要点

- 使用 CSS Flexbox 实现三栏布局，非 Grid（更好的侧边栏动画支持）
- 侧边栏折叠使用 `width` + `transition-all duration-300` 动画
- TopBar 使用 `sticky top-0` 固定在内容区顶部
- 内容区使用 `overflow-auto` 独立滚动，不影响侧边栏
- 使用 CSS 变量控制侧边栏宽度：`--sidebar-width: 240px` / `--sidebar-collapsed: 56px`

### 3.4 需要安装的 shadcn/ui 组件

```bash
npx shadcn@latest add tooltip
npx shadcn@latest add separator
npx shadcn@latest add scroll-area
npx shadcn@latest add dropdown-menu
npx shadcn@latest add avatar
```

---

## 4. 任务 1.2：路由系统

> **预估时间：** 2h
> **产出：** 完整的客户端路由，支持 5 个主页面

### 4.1 方案选择

**采用 Zustand 内部状态路由**（非 React Router），原因：

| 因素 | 状态路由 | React Router |
|------|---------|-------------|
| 桌面应用适配 | ✅ 无 URL 依赖 | ⚠️ 需要 HashRouter |
| 体积 | 0KB（已有 Zustand） | +15KB |
| 动画控制 | ✅ 完全可控 | 需额外库 |
| 嵌套路由 | 手动实现 | ✅ 内建 |
| 复杂度 | 简单（5 个顶层页面） | 过度设计 |

### 4.2 路由定义

```typescript
// src/stores/app-store.ts
type Route =
  | { page: "chat"; sessionId?: string }
  | { page: "skills" }
  | { page: "knowledge" }
  | { page: "dashboard" }
  | { page: "notifications" }
  | { page: "settings"; tab?: SettingsTab };

type SettingsTab = "general" | "models" | "mcp" | "appearance" | "about";
```

### 4.3 页面组件映射

```typescript
// src/pages/index.ts
export { ChatPage } from "./ChatPage";
export { SkillsPage } from "./SkillsPage";
export { KnowledgePage } from "./KnowledgePage";
export { DashboardPage } from "./DashboardPage";
export { NotificationsPage } from "./NotificationsPage";
export { SettingsPage } from "./SettingsPage";
```

### 4.4 内容区路由渲染

```tsx
// src/components/layout/ContentArea.tsx
function ContentArea() {
  const route = useAppStore((s) => s.route);

  switch (route.page) {
    case "chat":
      return <ChatPage />;
    case "skills":
      return <SkillsPage />;
    case "knowledge":
      return <KnowledgePage />;
    case "dashboard":
      return <DashboardPage />;
    case "notifications":
      return <NotificationsPage />;
    case "settings":
      return <SettingsPage />;
  }
}
```

---

## 5. 任务 1.3：侧边栏导航

> **预估时间：** 3h
> **产出：** 可折叠的侧边栏，含所有导航项

### 5.1 导航项定义

| 图标 | 标签 | Route / Action | 分组 |
|------|------|----------------|------|
| MessageSquare | Chat | `{ page: "chat" }` | 主导航 |
| Sparkles | Skills | `{ page: "skills" }` | 主导航 |
| BookOpen | Knowledge | `{ page: "knowledge" }` | 主导航 |
| LayoutDashboard | Dashboard | `{ page: "dashboard" }` | 主导航 |
| Bell | 通知 | `{ page: "notifications" }` | 底部固定（带未读 Badge） |
| Avatar | 用户头像 | 打开下拉菜单 | 底部固定（DropdownMenu） |
| PanelLeftClose | 折叠 | toggle sidebar | 底部固定 |

**用户头像下拉菜单项：**

| 图标 | 标签 | Action |
|------|------|--------|
| User | 个人中心 | 跳转个人中心（Phase 后续） |
| Settings | 设置 | `navigate({ page: "settings" })` |
| — | 分隔线 | — |
| LogOut | 退出登录 | 清除 session（Phase 后续） |

> 图标使用 `lucide-react`（已在 package.json 中）

### 5.2 侧边栏组件结构

```tsx
// src/components/layout/Sidebar.tsx
export function Sidebar() {
  const { sidebarCollapsed, toggleSidebar } = useAppStore();
  const { t } = useTranslation();

  return (
    <aside
      className={cn(
        "flex flex-col border-r border-border bg-sidebar transition-all duration-300",
        sidebarCollapsed ? "w-14" : "w-60"
      )}
    >
      {/* Logo 区域 */}
      <SidebarHeader />

      {/* 主导航 */}
      <nav className="flex-1 px-2 py-2 space-y-1">
        <NavItem icon={MessageSquare} label={t("nav.chat")} route={{ page: "chat" }} />
        <NavItem icon={Sparkles} label={t("nav.skills")} route={{ page: "skills" }} />
        <NavItem icon={BookOpen} label={t("nav.knowledge")} route={{ page: "knowledge" }} />
        <NavItem icon={LayoutDashboard} label={t("nav.dashboard")} route={{ page: "dashboard" }} />
      </nav>

      {/* 底部固定区域：通知 + 用户头像 + 折叠按钮 */}
      <SidebarFooter />
    </aside>
  );
}
```

### 5.4 SidebarFooter 组件设计

```tsx
// src/components/layout/SidebarFooter.tsx
export function SidebarFooter() {
  const { sidebarCollapsed, toggleSidebar, navigate } = useAppStore();
  const { t } = useTranslation();

  return (
    <div className="border-t border-border px-2 py-2 space-y-1">
      {/* 通知入口：带未读 Badge */}
      <NavItem
        icon={Bell}
        label={t("nav.notifications")}
        route={{ page: "notifications" }}
        badge={unreadCount}  // 未读通知数
      />

      {/* 用户头像 + 下拉菜单 */}
      <UserMenu collapsed={sidebarCollapsed} />

      {/* 折叠按钮 */}
      <CollapseButton collapsed={sidebarCollapsed} onToggle={toggleSidebar} />
    </div>
  );
}
```

### 5.5 UserMenu 组件设计

```tsx
// src/components/layout/UserMenu.tsx
export function UserMenu({ collapsed }: { collapsed: boolean }) {
  const { navigate } = useAppStore();
  const { t } = useTranslation();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button className="flex w-full items-center gap-3 rounded-md px-3 py-2 hover:bg-accent">
          <Avatar className="h-7 w-7">
            <AvatarFallback>U</AvatarFallback>
          </Avatar>
          {!collapsed && <span className="text-sm">{t("nav.user")}</span>}
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent side="right" align="end" className="w-48">
        <DropdownMenuItem onClick={() => { /* Phase 后续：个人中心 */ }}>
          <User className="mr-2 h-4 w-4" />
          {t("userMenu.profile")}
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => navigate({ page: "settings" })}>
          <Settings className="mr-2 h-4 w-4" />
          {t("userMenu.settings")}
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={() => { /* Phase 后续：退出登录 */ }}>
          <LogOut className="mr-2 h-4 w-4" />
          {t("userMenu.logout")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
```

**折叠态行为：**
- 用户头像：折叠时仅显示 Avatar 图标居中，点击仍弹出 DropdownMenu（`side="right"`）
- 通知：折叠时仅显示 Bell 图标 + Badge 圆点
- DropdownMenu 始终向右侧弹出（`side="right"`），避免被侧边栏遮挡

### 5.3 NavItem 组件设计

- 折叠时隐藏文字，仅显示图标居中
- 折叠时 hover 显示 Tooltip（使用 shadcn/ui Tooltip）
- Active 状态使用左侧 3px 竖线 + 浅色背景
- 使用 `aria-current="page"` 无障碍标记

---

## 6. 任务 1.4：设置页面

> **预估时间：** 6h
> **产出：** 完整的设置页面，含 5 个子 Tab

### 6.1 页面结构

```
src/pages/settings/
├── SettingsPage.tsx             # 设置页面主组件
├── GeneralSettings.tsx          # 通用设置
├── ModelSettings.tsx            # 模型/Provider 设置
├── McpSettings.tsx              # MCP 配置（Phase 3 实现内容，此处占位）
├── AppearanceSettings.tsx       # 外观设置
└── AboutSettings.tsx            # 关于页面
```

### 6.2 General 设置面板字段

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| Language | Select | "en" | 界面语言 |
| Log Level | Select | "info" | 日志级别 (debug/info/warn/error) |
| Auto Start Sidecar | Switch | false | 是否随应用启动 Python Sidecar |
| Sidecar Port | Input(number) | 9527 | Sidecar 监听端口 |

### 6.3 Appearance 设置面板字段

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| Theme | RadioGroup | "system" | 明/暗/跟随系统 |
| Accent Color | ColorPicker | "#6366f1" | 强调色 |
| Font Size | Slider | 14 | 基础字号 (12-18) |
| Sidebar Default | Switch | true | 默认展开侧边栏 |

### 6.4 About 页面内容

- 应用名称 + 版本号（读取 `tauri.conf.json` 中的 version）
- 技术栈信息（Tauri / React / Rust）
- 系统信息（OS / Architecture）
- 检查更新按钮（Phase 6 实现逻辑，此处占位）
- 开源许可链接
- GitHub 仓库链接

### 6.5 需要安装的 shadcn/ui 组件

```bash
npx shadcn@latest add tabs
npx shadcn@latest add input
npx shadcn@latest add label
npx shadcn@latest add select
npx shadcn@latest add switch
npx shadcn@latest add slider
npx shadcn@latest add radio-group
npx shadcn@latest add card
npx shadcn@latest add badge
npx shadcn@latest add toast
npx shadcn@latest add sonner
```

---

## 7. 任务 1.5：Rust 端设置 CRUD Commands

> **预估时间：** 3h
> **产出：** 完善的 Tauri Commands，支持设置项的完整 CRUD

### 7.1 现有实现分析

当前 `src-tauri/src/commands/settings.rs` 已有 `get_settings` 和 `update_setting` 两个基础 command，但功能简单（仅操作内存中的 AppConfig）。需要增强为支持 SQLite 持久化的完整设置系统。

### 7.2 增强方案

**双层设置存储架构：**

```
AppConfig (config.yaml)        → 应用级基础配置（主题、语言、端口等）
SQLite settings 表             → 动态设置（UI 偏好、窗口状态等细粒度设置）
SQLite router_configs 表       → Provider/API Key 配置（加密）
```

### 7.3 新增 Tauri Commands

```rust
// 应用配置相关
#[tauri::command]
fn get_app_config(state: State<AppState>) -> Result<AppConfig, String>;

#[tauri::command]
fn update_app_config(state: State<AppState>, config: AppConfig) -> Result<(), String>;

// SQLite settings 表相关（键值对）
#[tauri::command]
fn get_setting(state: State<AppState>, key: String) -> Result<Option<String>, String>;

#[tauri::command]
fn set_setting(state: State<AppState>, key: String, value: String) -> Result<(), String>;

#[tauri::command]
fn get_all_settings(state: State<AppState>) -> Result<HashMap<String, String>, String>;

// 系统信息
#[tauri::command]
fn get_system_info() -> Result<SystemInfo, String>;
```

### 7.4 SystemInfo 结构

```rust
#[derive(Serialize)]
pub struct SystemInfo {
    pub app_version: String,
    pub os: String,
    pub arch: String,
    pub data_dir: String,
    pub db_size_bytes: u64,
}
```

---

## 8. 任务 1.6：路由配置 UI + Rust 后端

> **预估时间：** 6h
> **产出：** 多 Provider API Key 管理，支持加密存储

### 8.1 支持的 Provider 列表

| Provider | 需要的字段 | 可用模型示例 |
|----------|-----------|------------|
| OpenAI | API Key, Base URL(可选) | gpt-4o, gpt-4o-mini, o1, o3 |
| Anthropic | API Key | claude-sonnet-4, claude-opus-4, claude-haiku |
| Google (Gemini) | API Key | gemini-2.5-pro, gemini-2.5-flash |
| DeepSeek | API Key, Base URL | deepseek-chat, deepseek-reasoner |
| Custom (OpenAI Compatible) | API Key, Base URL, Model Name | 自定义 |

### 8.2 UI 设计

**Provider 列表页：**
- 卡片式列表展示已配置的 Provider
- 每张卡片：Provider Logo + 名称 + 状态指示灯 + 已配置模型数
- 右上角 "+ 添加 Provider" 按钮

**Provider 编辑弹窗 (Dialog)：**
- Provider 类型选择（下拉）
- API Key 输入框（password 类型 + 显示/隐藏切换）
- Base URL（仅 OpenAI/Custom/DeepSeek 显示）
- 模型列表（多选或手动输入）
- 设为默认 Provider 开关
- 连接测试按钮

### 8.3 Rust 端 Commands

```rust
// Provider CRUD
#[tauri::command]
fn list_router_configs(state: State<AppState>) -> Result<Vec<RouterConfigView>, String>;

#[tauri::command]
fn create_router_config(state: State<AppState>, config: CreateRouterConfig) -> Result<String, String>;

#[tauri::command]
fn update_router_config(state: State<AppState>, id: String, config: UpdateRouterConfig) -> Result<(), String>;

#[tauri::command]
fn delete_router_config(state: State<AppState>, id: String) -> Result<(), String>;

#[tauri::command]
fn test_router_connection(state: State<AppState>, id: String) -> Result<ConnectionTestResult, String>;
```

### 8.4 API Key 加密方案

```rust
// 使用系统级密钥派生 + AES-256-GCM 加密
// 密钥来源：machine-id + 应用标识 + 用户目录哈希 → HKDF → 256-bit key
// 加密后以 Base64 形式存入 SQLite router_configs.api_key_encrypted 字段

use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
```

**需要新增 Cargo 依赖：**
```toml
aes-gcm = "0.10"
hkdf = "0.12"
sha2 = "0.10"
base64 = "0.22"
```

### 8.5 数据流

```
[前端] 用户输入 API Key
  → invoke("create_router_config", { provider, api_key, ... })
    → [Rust] 加密 API Key
    → [Rust] INSERT INTO router_configs
    → [Rust] 返回 config_id

[前端] 列表展示
  → invoke("list_router_configs")
    → [Rust] SELECT (不返回解密后的 key，仅返回 "sk-...***" 遮罩形式)
    → [前端] 渲染卡片列表

[前端] 测试连接
  → invoke("test_router_connection", { id })
    → [Rust] 解密 API Key → 调用 Provider 的 models/list API → 返回成功/失败
```

---

## 9. 任务 1.7：主题系统

> **预估时间：** 3h
> **产出：** 明/暗/跟随系统 + 强调色切换

### 9.1 实现方案

**采用 CSS 类切换方案（shadcn/ui 标准做法）：**

- `<html>` 标签上切换 `class="dark"` / `class="light"`
- 跟随系统：监听 `window.matchMedia('(prefers-color-scheme: dark')` 变化
- 主题存储在 Zustand store + 持久化到 config.yaml

### 9.2 强调色系统

预设 8 种强调色供用户选择：

| 名称 | 色值 | CSS 变量 |
|------|------|---------|
| Indigo | #6366F1 | `--accent-indigo` |
| Violet | #8B5CF6 | `--accent-violet` |
| Blue | #3B82F6 | `--accent-blue` |
| Cyan | #06B6D4 | `--accent-cyan` |
| Emerald | #10B981 | `--accent-emerald` |
| Amber | #F59E0B | `--accent-amber` |
| Rose | #F43F5E | `--accent-rose` |
| Zinc | #71717A | `--accent-zinc` |

### 9.3 主题 Store

```typescript
// src/stores/theme-store.ts
interface ThemeState {
  mode: "light" | "dark" | "system";
  accentColor: string;
  resolvedTheme: "light" | "dark"; // 实际应用的主题

  setMode: (mode: ThemeState["mode"]) => void;
  setAccentColor: (color: string) => void;
}
```

### 9.4 实现要点

1. 应用启动时从 config 加载主题设置
2. `system` 模式下使用 `matchMedia` 监听系统主题变化
3. 主题切换时同步写入 config.yaml（通过 Tauri IPC）
4. 强调色通过动态设置 CSS 变量实现：`document.documentElement.style.setProperty('--primary', hslValue)`
5. 使用 `useEffect` 监听 store 变化并同步 DOM 类名

---

## 10. 任务 1.8：i18n 国际化

> **预估时间：** 3h
> **产出：** 完整的中英文切换，所有 UI 文字已提取

### 10.1 技术方案

- 使用 `react-i18next` + `i18next`
- 语言文件存放在 `src/locales/`
- 命名空间：`common` + `settings` + `nav`（按功能拆分）
- 语言检测优先级：config.yaml 设置 > 系统语言 > 默认 en

### 10.2 目录结构

```
src/locales/
├── i18n.ts                  # i18next 初始化
├── en/
│   ├── common.json          # 通用文本
│   ├── nav.json             # 导航文本
│   └── settings.json        # 设置页文本
└── zh-CN/
    ├── common.json
    ├── nav.json
    └── settings.json
```

### 10.3 翻译键设计示例

```json
// en/nav.json
{
  "chat": "Chat",
  "skills": "Skills",
  "knowledge": "Knowledge",
  "dashboard": "Dashboard",
  "notifications": "Notifications",
  "settings": "Settings",
  "user": "User",
  "userMenu": {
    "profile": "Profile",
    "settings": "Settings",
    "logout": "Log Out"
  }
}

// zh-CN/nav.json
{
  "chat": "对话",
  "skills": "技能",
  "knowledge": "知识库",
  "dashboard": "仪表盘",
  "notifications": "通知",
  "settings": "设置",
  "user": "用户",
  "userMenu": {
    "profile": "个人中心",
    "settings": "设置",
    "logout": "退出登录"
  }
}
```

```json
// en/settings.json
{
  "title": "Settings",
  "general": {
    "title": "General",
    "language": "Language",
    "logLevel": "Log Level",
    "autoStartSidecar": "Auto-start Agent Sidecar",
    "sidecarPort": "Sidecar Port"
  },
  "models": {
    "title": "Models",
    "addProvider": "Add Provider",
    "apiKey": "API Key",
    "baseUrl": "Base URL",
    "testConnection": "Test Connection",
    "connectionSuccess": "Connection successful",
    "connectionFailed": "Connection failed",
    "setDefault": "Set as Default"
  },
  "appearance": {
    "title": "Appearance",
    "theme": "Theme",
    "themeLight": "Light",
    "themeDark": "Dark",
    "themeSystem": "System",
    "accentColor": "Accent Color",
    "fontSize": "Font Size"
  },
  "about": {
    "title": "About",
    "version": "Version",
    "checkUpdate": "Check for Updates",
    "systemInfo": "System Information",
    "os": "Operating System",
    "architecture": "Architecture",
    "dataDirectory": "Data Directory"
  }
}
```

### 10.4 需要安装的依赖

```bash
npm install react-i18next i18next
```

---

## 11. 任务 1.9：Zustand Stores 搭建

> **预估时间：** 2h
> **产出：** app / chat / settings 三个核心 store

### 11.1 Store 目录结构

```
src/stores/
├── index.ts                 # 导出所有 store
├── app-store.ts             # 应用全局状态（路由、侧边栏）
├── settings-store.ts        # 设置状态（与后端同步）
└── chat-store.ts            # 对话状态（Phase 2 细化，此处占位）
```

### 11.2 app-store

```typescript
// src/stores/app-store.ts
import { create } from "zustand";

interface AppState {
  // 路由
  route: Route;
  navigate: (route: Route) => void;

  // 侧边栏
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;

  // 全局 Loading
  globalLoading: boolean;
  setGlobalLoading: (loading: boolean) => void;
}
```

### 11.3 settings-store

```typescript
// src/stores/settings-store.ts
import { create } from "zustand";

interface SettingsState {
  // 从后端加载的配置
  config: AppConfig | null;
  providers: RouterConfig[];
  loading: boolean;

  // Actions
  loadConfig: () => Promise<void>;
  updateConfig: (partial: Partial<AppConfig>) => Promise<void>;
  loadProviders: () => Promise<void>;
  addProvider: (provider: CreateRouterConfig) => Promise<void>;
  updateProvider: (id: string, updates: UpdateRouterConfig) => Promise<void>;
  deleteProvider: (id: string) => Promise<void>;
}
```

### 11.4 chat-store（Phase 1 骨架）

```typescript
// src/stores/chat-store.ts
import { create } from "zustand";

interface ChatState {
  sessions: Session[];
  activeSessionId: string | null;

  // Phase 2 实现
  loadSessions: () => Promise<void>;
  createSession: () => Promise<string>;
  deleteSession: (id: string) => Promise<void>;
}
```

### 11.5 需要安装的依赖

```bash
npm install zustand
```

---

## 12. 任务 1.10：Tauri IPC 封装层

> **预估时间：** 3h
> **产出：** 类型安全的 invoke wrapper + 统一错误处理

### 12.1 封装目标

- **类型安全：** 每个 command 的参数和返回值都有 TypeScript 类型定义
- **错误处理：** 统一的 `IpcError` 类型，前端可友好展示
- **Loading 状态：** 封装 async 调用的 pending/success/error 状态
- **日志：** 开发模式下自动 console.log 所有 IPC 调用

### 12.2 目录结构

```
src/lib/
├── ipc/
│   ├── index.ts              # 导出所有 IPC 函数
│   ├── invoke.ts             # 核心 invoke wrapper
│   ├── types.ts              # 共享类型定义
│   ├── settings.ts           # 设置相关 IPC
│   └── router-configs.ts     # Provider 配置 IPC
└── utils.ts                  # 通用工具
```

### 12.3 核心 invoke wrapper

```typescript
// src/lib/ipc/invoke.ts
import { invoke as tauriInvoke } from "@tauri-apps/api/core";

export class IpcError extends Error {
  constructor(
    public readonly command: string,
    public readonly originalError: string
  ) {
    super(`IPC Error [${command}]: ${originalError}`);
    this.name = "IpcError";
  }
}

export async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (import.meta.env.DEV) {
    console.log(`[IPC] → ${command}`, args);
  }

  try {
    const result = await tauriInvoke<T>(command, args);
    if (import.meta.env.DEV) {
      console.log(`[IPC] ← ${command}`, result);
    }
    return result;
  } catch (error) {
    const message = typeof error === "string" ? error : String(error);
    throw new IpcError(command, message);
  }
}
```

### 12.4 类型安全的 command 函数

```typescript
// src/lib/ipc/settings.ts
import { invoke } from "./invoke";
import type { AppConfig, SystemInfo } from "./types";

export const settingsIpc = {
  getAppConfig: () => invoke<AppConfig>("get_app_config"),
  updateAppConfig: (config: AppConfig) => invoke<void>("update_app_config", { config }),
  getSetting: (key: string) => invoke<string | null>("get_setting", { key }),
  setSetting: (key: string, value: string) => invoke<void>("set_setting", { key, value }),
  getSystemInfo: () => invoke<SystemInfo>("get_system_info"),
};
```

### 12.5 React Hook 封装（可选，简化使用）

```typescript
// src/hooks/use-ipc.ts
import { useState, useCallback } from "react";
import { IpcError } from "@/lib/ipc/invoke";

interface UseIpcResult<T> {
  data: T | null;
  loading: boolean;
  error: IpcError | null;
  execute: (...args: unknown[]) => Promise<T>;
}

export function useIpc<T>(fn: (...args: unknown[]) => Promise<T>): UseIpcResult<T> {
  // 实现 loading/error/data 状态管理
}
```

---

## 13. Phase 1 完整验证清单

### 13.1 布局验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-1 | 应用启动 | 显示完整三栏布局，无闪烁 |
| V-2 | 侧边栏展开 | 宽度 240px，显示图标+文字 |
| V-3 | 侧边栏折叠 | 宽度 56px，仅显示图标，hover 显示 Tooltip |
| V-4 | 窗口缩至 900px | 侧边栏自动折叠 |
| V-5 | TopBar | 固定顶部，显示页面标题 |

### 13.2 导航验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-6 | 点击 Chat | 内容区切换到 Chat 页面，侧边栏 Chat 项高亮 |
| V-7 | 点击 Skills | 内容区切换到 Skills 页面 |
| V-8 | 点击 Knowledge | 内容区切换到 Knowledge 页面 |
| V-9 | 点击 Dashboard | 内容区切换到 Dashboard 页面 |
| V-10 | 点击通知图标 | 内容区切换到 Notifications 页面 |
| V-11 | 用户头像下拉菜单 | 点击头像弹出菜单，包含"个人中心""设置""退出登录" |
| V-12 | 菜单"设置"项 | 点击后跳转 Settings 页面 |
| V-13 | 折叠态用户头像 | 折叠后仅显示 Avatar，点击仍弹出右侧菜单 |

### 13.3 设置验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-14 | Settings Tab 切换 | General/Models/MCP/Appearance/About 五个 Tab 正常切换 |
| V-15 | 修改 Language | 下拉切换后界面文字立即更新 |
| V-16 | 修改 Theme | 主题立即切换（亮→暗、暗→亮、系统跟随） |
| V-17 | 修改 Accent Color | 选色后强调色立即变化 |
| V-18 | 添加 Provider | 弹窗填写信息后保存成功，列表中出现新卡片 |
| V-19 | 删除 Provider | 确认后删除，列表中消失 |
| V-20 | 测试连接 | 点击测试按钮，返回成功/失败提示 |
| V-21 | 重启验证 | 关闭并重新打开应用，之前的设置仍然保留 |

### 13.4 主题验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-22 | Light 模式 | 白色背景，深色文字 |
| V-23 | Dark 模式 | 深色背景，浅色文字 |
| V-24 | System 模式 | 跟随系统设置，系统切换后应用同步变化 |
| V-25 | 强调色更换 | 按钮、链接、选中态颜色同步变化 |

### 13.5 i18n 验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-26 | 英文 | 所有 UI 文字为英文（含通知、用户菜单文字） |
| V-27 | 中文 | 所有 UI 文字为中文（含通知、用户菜单文字） |
| V-28 | 切换语言 | 即时生效，无需重启 |

### 13.6 IPC 验证

| # | 检查项 | 预期结果 |
|---|--------|---------|
| V-29 | 正常调用 | 设置保存成功，控制台有 IPC 日志 |
| V-30 | 错误处理 | 模拟错误时前端展示友好错误信息（Toast） |

---

## 14. 详细 TODO 列表

### 1.9 Zustand Stores 搭建 [预估 2h]

- [x] **1.9.1** 安装 zustand：`npm install zustand`
- [x] **1.9.2** 创建 `src/stores/app-store.ts`（路由、侧边栏、全局 Loading）
- [x] **1.9.3** 创建 `src/stores/settings-store.ts`（配置、Provider 列表）
- [x] **1.9.4** 创建 `src/stores/chat-store.ts`（会话列表骨架）
- [x] **1.9.5** 创建 `src/stores/index.ts` 统一导出
- [x] **1.9.6** 验证 store 创建无 TypeScript 错误

### 1.10 Tauri IPC 封装层 [预估 3h]

- [x] **1.10.1** 创建 `src/lib/ipc/invoke.ts`（核心 invoke wrapper + IpcError）
- [x] **1.10.2** 创建 `src/lib/ipc/types.ts`（共享类型：AppConfig、RouterConfig、SystemInfo 等）
- [x] **1.10.3** 创建 `src/lib/ipc/settings.ts`（settingsIpc 对象）
- [x] **1.10.4** 创建 `src/lib/ipc/router-configs.ts`（routerConfigsIpc 对象）
- [x] **1.10.5** 创建 `src/lib/ipc/index.ts` 统一导出
- [x] **1.10.6** 创建 `src/hooks/use-ipc.ts`（可选的 React Hook 封装）
- [x] **1.10.7** 验证 TypeScript 类型完整且无错误

### 1.7 主题系统 [预估 3h]

- [x] **1.7.1** 创建 `src/lib/theme.ts`（主题应用逻辑：切换 class、监听 matchMedia）
- [x] **1.7.2** 在 `app-store` 或独立 `theme-store` 中实现主题状态管理
- [x] **1.7.3** 实现 `system` 模式的 matchMedia 监听
- [x] **1.7.4** 实现强调色 CSS 变量动态切换
- [x] **1.7.5** 在应用启动时从 config 加载主题并应用
- [x] **1.7.6** 主题切换时同步写回 config.yaml（通过 IPC）
- [x] **1.7.7** 验证亮/暗/系统三种模式切换正常

### 1.8 i18n 国际化 [预估 3h]

- [x] **1.8.1** 安装依赖：`npm install react-i18next i18next`
- [x] **1.8.2** 创建 `src/locales/i18n.ts`（i18next 初始化配置）
- [x] **1.8.3** 创建 `src/locales/en/common.json`
- [x] **1.8.4** 创建 `src/locales/en/nav.json`
- [x] **1.8.5** 创建 `src/locales/en/settings.json`
- [x] **1.8.6** 创建 `src/locales/zh-CN/common.json`
- [x] **1.8.7** 创建 `src/locales/zh-CN/nav.json`
- [x] **1.8.8** 创建 `src/locales/zh-CN/settings.json`
- [x] **1.8.9** 在 `main.tsx` 中引入 i18n 初始化
- [x] **1.8.10** 验证 `useTranslation()` hook 工作正常

### 1.1 AppShell 主布局 [预估 4h]

- [x] **1.1.1** 安装 shadcn/ui 组件：`npx shadcn@latest add tooltip separator scroll-area`
- [x] **1.1.2** 创建 `src/components/layout/AppShell.tsx`（主布局容器）
- [x] **1.1.3** 创建 `src/components/layout/Sidebar.tsx`（侧边栏骨架）
- [x] **1.1.4** 创建 `src/components/layout/TopBar.tsx`（顶栏）
- [x] **1.1.5** 创建 `src/components/layout/ContentArea.tsx`（路由内容区渲染）
- [x] **1.1.6** 修改 `src/App.tsx`，引入 AppShell 替换当前内容
- [x] **1.1.7** 实现侧边栏折叠/展开动画（CSS transition）
- [x] **1.1.8** 验证布局在 1280x800 和 900x600 下表现正常

### 1.2 路由系统 [预估 2h]

- [x] **1.2.1** 在 `app-store.ts` 中定义 Route 类型（含 notifications 页面）和 navigate action
- [x] **1.2.2** 创建 `src/pages/ChatPage.tsx`（占位）
- [x] **1.2.3** 创建 `src/pages/SkillsPage.tsx`（占位）
- [x] **1.2.4** 创建 `src/pages/KnowledgePage.tsx`（占位）
- [x] **1.2.5** 创建 `src/pages/DashboardPage.tsx`（占位）
- [x] **1.2.6** 创建 `src/pages/NotificationsPage.tsx`（占位，显示通知列表骨架）
- [x] **1.2.7** 创建 `src/pages/SettingsPage.tsx`（占位）
- [x] **1.2.8** 在 `ContentArea.tsx` 中实现 switch-case 路由渲染
- [x] **1.2.9** 验证点击导航时页面正确切换

### 1.3 侧边栏导航 [预估 3h]

- [x] **1.3.1** 创建 `src/components/layout/NavItem.tsx`（导航项组件，支持 badge 属性）
- [x] **1.3.2** 创建 `src/components/layout/SidebarHeader.tsx`（Logo 区域）
- [x] **1.3.3** 创建 `src/components/layout/SidebarFooter.tsx`（底部：通知入口 + 用户头像 + 折叠按钮）
- [x] **1.3.4** 创建 `src/components/layout/UserMenu.tsx`（用户头像 + DropdownMenu：个人中心/设置/退出）
- [x] **1.3.5** 实现 NavItem 的 active / hover / collapsed 三种视觉状态
- [x] **1.3.6** 实现通知入口的未读 Badge 显示（红色圆点/数字）
- [x] **1.3.7** 实现折叠态下的 Tooltip 提示
- [x] **1.3.8** 实现折叠态下用户头像 DropdownMenu 向右弹出
- [x] **1.3.9** 添加 `aria-current="page"` 无障碍标记
- [x] **1.3.10** 验证所有导航项点击、状态切换、下拉菜单正常

### 1.4 设置页面 [预估 6h]

- [x] **1.4.1** 安装 shadcn/ui 组件：`npx shadcn@latest add tabs input label select switch slider radio-group card badge sonner`
- [x] **1.4.2** 创建 `src/pages/settings/SettingsPage.tsx`（主容器 + Tab 导航）
- [x] **1.4.3** 创建 `src/pages/settings/GeneralSettings.tsx`（通用设置表单）
- [x] **1.4.4** 创建 `src/pages/settings/ModelSettings.tsx`（Provider 管理列表）
- [x] **1.4.5** 创建 `src/pages/settings/McpSettings.tsx`（占位，Phase 3 实现）
- [x] **1.4.6** 创建 `src/pages/settings/AppearanceSettings.tsx`（主题+强调色+字号）
- [x] **1.4.7** 创建 `src/pages/settings/AboutSettings.tsx`（版本信息+系统信息）
- [x] **1.4.8** 实现 Settings Tab 切换逻辑（与路由联动）
- [x] **1.4.9** 实现表单双向绑定（store ↔ 表单控件）
- [x] **1.4.10** 验证所有设置面板渲染正常

### 1.5 Rust 端设置 CRUD Commands [预估 3h]

- [x] **1.5.1** 重构 `src-tauri/src/commands/settings.rs`：增加 `get_app_config`、`update_app_config`
- [x] **1.5.2** 新增 `get_setting`、`set_setting`、`get_all_settings` 三个 SQLite 操作命令
- [x] **1.5.3** 新增 `get_system_info` 命令（版本、OS、架构、数据目录、DB 大小）
- [x] **1.5.4** 在 `lib.rs` 的 `invoke_handler` 中注册新命令
- [x] **1.5.5** 执行 `cargo check` 确认编译通过
- [x] **1.5.6** 验证前端 IPC 调用各命令正常

### 1.6 路由配置 UI + Rust 后端 [预估 6h]

- [x] **1.6.1** 新增 Cargo 依赖：`aes-gcm`、`hkdf`、`sha2`、`base64`
- [x] **1.6.2** 创建 `src-tauri/src/crypto.rs`（API Key 加密/解密工具）
- [x] **1.6.3** 创建 `src-tauri/src/commands/router_configs.rs`（CRUD + 测试连接命令）
- [x] **1.6.4** 在 `lib.rs` 注册 router_configs commands
- [x] **1.6.5** 执行 `cargo check` 确认编译通过
- [x] **1.6.6** 创建 `src/pages/settings/ProviderCard.tsx`（Provider 卡片组件）
- [x] **1.6.7** 创建 `src/pages/settings/ProviderDialog.tsx`（添加/编辑 Provider 弹窗）
- [x] **1.6.8** 在 `ModelSettings.tsx` 中集成 Provider 列表和操作
- [x] **1.6.9** 实现"测试连接"功能（调用后端 API → Toast 提示）
- [x] **1.6.10** 验证 Provider CRUD 全流程（添加 → 列表显示 → 编辑 → 删除 → 重启验证持久化）

### 收尾验证 [预估 1h]

- [ ] **F-1** 完整冒烟测试：启动应用 → 导航所有页面 → 修改设置 → 切换主题 → 切换语言
- [ ] **F-2** 持久化测试：关闭应用 → 重新打开 → 之前的设置仍然保留
- [ ] **F-3** 多次主题切换测试：亮→暗→系统→暗→亮，确认无闪烁/残留
- [ ] **F-4** Provider 安全测试：用 sqlite3 查看数据库确认 API Key 已加密存储（非明文）
- [ ] **F-5** 对照第 13 节验证清单，逐项确认全部通过

---

## 附录 A：新增依赖汇总

### 前端 (npm)

```bash
npm install zustand react-i18next i18next
```

### 后端 (Cargo.toml 新增)

```toml
aes-gcm = "0.10"
hkdf = "0.12"
sha2 = "0.10"
base64 = "0.22"
```

### shadcn/ui 组件

```bash
npx shadcn@latest add tooltip separator scroll-area dropdown-menu avatar tabs input label select switch slider radio-group card badge sonner
```

---

## 附录 B：文件创建清单

Phase 1 需要新增的文件（不含 shadcn/ui 自动生成的组件文件）：

```
src/
├── components/
│   └── layout/
│       ├── AppShell.tsx
│       ├── Sidebar.tsx
│       ├── SidebarHeader.tsx
│       ├── SidebarFooter.tsx
│       ├── NavItem.tsx
│       ├── UserMenu.tsx           # 用户头像 + 下拉菜单
│       ├── TopBar.tsx
│       └── ContentArea.tsx
├── hooks/
│   └── use-ipc.ts
├── lib/
│   ├── ipc/
│   │   ├── index.ts
│   │   ├── invoke.ts
│   │   ├── types.ts
│   │   ├── settings.ts
│   │   └── router-configs.ts
│   └── theme.ts
├── locales/
│   ├── i18n.ts
│   ├── en/
│   │   ├── common.json
│   │   ├── nav.json
│   │   └── settings.json
│   └── zh-CN/
│       ├── common.json
│       ├── nav.json
│       └── settings.json
├── pages/
│   ├── ChatPage.tsx
│   ├── SkillsPage.tsx
│   ├── KnowledgePage.tsx
│   ├── DashboardPage.tsx
│   ├── NotificationsPage.tsx      # 通知页面
│   └── settings/
│       ├── SettingsPage.tsx
│       ├── GeneralSettings.tsx
│       ├── ModelSettings.tsx
│       ├── McpSettings.tsx
│       ├── AppearanceSettings.tsx
│       ├── AboutSettings.tsx
│       ├── ProviderCard.tsx
│       └── ProviderDialog.tsx
├── stores/
│   ├── index.ts
│   ├── app-store.ts
│   ├── settings-store.ts
│   └── chat-store.ts
└── (App.tsx 修改)

src-tauri/src/
├── crypto.rs (新增)
├── commands/
│   ├── settings.rs (重构)
│   └── router_configs.rs (新增)
└── lib.rs (修改：注册新模块和命令)
```

---

> **文档结束**
>
> 本文档是 Phase 1 的详细执行指南，覆盖了从 UI 设计到前后端实现的完整方案。
> 所有 TODO 共 **70+ 项**，按顺序完成后即可交付完整的应用骨架 UI、设置系统和主题/国际化功能。
> 下一阶段为 Phase 2：对话系统与流式通信。
