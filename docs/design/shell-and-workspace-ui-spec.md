# MisakaX 壳层布局与工作区 UI 规范

| 属性 | 说明 |
|------|------|
| **用途** | 定义主窗口三栏结构（主导航 / 会话列表 / 主内容区）的布局语义、会话侧栏工具区，以及对话页顶栏（工作目录）的信息层级与样式约定。 |
| **受众** | 负责 `AppShell`、`Sidebar`、`ChatPage`、`SessionPanel`、`WorkspaceBar` 及相关布局的前端开发者。 |
| **最后审阅** | 2026-05-10 |

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

## 4. 小结

| 区域 | 要点 |
|------|------|
| 主导航收起 | 只控制第 1 列，不误伤会话列表。 |
| 会话工具栏 | 全宽对齐、可读字号、工具栏分区清晰。 |
| 工作目录顶栏 | 标题/说明/主次操作层级分明，禁用「过小过素」的一次性排版。 |
| 按钮与菜单 | 一律交叉参考 `button-menu-design-spec.md`。 |

编写或修改前端时，请将本规范与 [frontend-ui-guidelines.md](./frontend-ui-guidelines.md)、[button-menu-design-spec.md](./button-menu-design-spec.md) **一并查阅**，避免只做局部改动而破坏壳层语义或桌面端观感。
