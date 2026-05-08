# Codex Monitor — 完整 UI 设计报告

> 基于对全部 40+ CSS 文件、设计令牌系统、主题系统、布局系统、所有组件样式及交互状态的逐行分析生成。

---

## 目录

1. [设计哲学与风格定位](#1-设计哲学与风格定位)
2. [色彩体系](#2-色彩体系)
3. [设计令牌系统](#3-设计令牌系统)
4. [字体系统](#4-字体系统)
5. [布局架构](#5-布局架构)
6. [主题系统](#6-主题系统)
7. [核心组件 UI 设计](#7-核心组件-ui-设计)
8. [消息系统 UI 设计](#8-消息系统-ui-设计)
9. [编辑器 (Composer) UI 设计](#9-编辑器-composer-ui-设计)
10. [侧边栏 UI 设计](#10-侧边栏-ui-设计)
11. [Git/差异面板 UI 设计](#11-git差异面板-ui-设计)
12. [设置界面 UI 设计](#12-设置界面-ui-设计)
13. [主页 UI 设计](#13-主页-ui-设计)
14. [终端面板 UI 设计](#14-终端面板-ui-设计)
15. [调试面板 UI 设计](#15-调试面板-ui-设计)
16. [计划面板 UI 设计](#16-计划面板-ui-设计)
17. [设计系统原子组件](#17-设计系统原子组件)
18. [动画与过渡系统](#18-动画与过渡系统)
19. [响应式设计](#19-响应式设计)
20. [无障碍设计](#20-无障碍设计)
21. [平台适配](#21-平台适配)
22. [设计风格总结](#22-设计风格总结)

---

## 1. 设计哲学与风格定位

### 1.1 核心设计理念

**"透明层次，柔和过渡"** — Codex Monitor 的整体 UI 风格可被描述为 **Glassmorphism Dark（毛玻璃暗色风格）**，采用了以下核心设计原则：

- **半透明层次**：所有表面（surface）使用 RGBA 透明度构建视觉层级
- **模糊背景**：关键区域（顶栏、编辑器）使用 `backdrop-filter: blur()` 实现毛玻璃效果
- **微妙内阴影**：卡片和行项目使用 `inset box-shadow` 创建嵌入感
- **圆角优先**：几乎所有交互元素都使用圆角，从 6px 到 20px 不等
- **色彩克制**：以中性灰白透明为基础，仅用少量高饱和色作为状态指示

### 1.2 风格分类

| 维度 | 取值 |
|------|------|
| **主要风格** | Glassmorphism (毛玻璃) + Dark Mode |
| **次要风格** | Minimalism (极简主义) |
| **色系** | 暗色冷调，蓝/青/绿点缀 |
| **圆角风格** | 大圆角 (8px-20px)，药丸形按钮 |
| **阴影风格** | 柔和扩散阴影 + 内阴影 |
| **边框风格** | 极细半透明边框 |
| **字体风格** | 系统无衬线 + 等宽代码字体 |

---

## 2. 色彩体系

### 2.1 Dark 主题（默认）色谱

#### 背景 / 表面色 (Surfaces)

| 令牌名 | 颜色值 | 用途 |
|--------|--------|------|
| `--surface-sidebar` | `rgba(18, 18, 18, 0.5)` | 侧边栏背景 |
| `--surface-sidebar-opaque` | `rgb(18, 18, 18)` | 侧边栏不透明版本 |
| `--surface-topbar` | `rgba(10, 14, 20, 0.45)` | 顶栏背景 |
| `--surface-right-panel` | `rgba(9, 12, 20, 0.4)` | 右侧面板背景 |
| `--surface-composer` | `rgba(10, 14, 20, 0.45)` | 编辑器背景 |
| `--surface-messages` | `rgba(8, 10, 16, 0.45)` | 消息区域背景 |
| `--surface-card` | `rgba(255, 255, 255, 0.04)` | 卡片表面 |
| `--surface-card-strong` | `rgba(255, 255, 255, 0.12)` | 强调卡片 |
| `--surface-card-muted` | `rgba(255, 255, 255, 0.06)` | 弱化卡片 |
| `--surface-item` | `rgba(255, 255, 255, 0.02)` | 列表项表面 |
| `--surface-control` | `rgba(255, 255, 255, 0.08)` | 控件背景 |
| `--surface-control-hover` | `rgba(255, 255, 255, 0.14)` | 控件悬停 |
| `--surface-hover` | `rgba(255, 255, 255, 0.05)` | 通用悬停 |
| `--surface-active` | `rgba(100, 200, 255, 0.14)` | 激活状态（蓝调） |
| `--surface-approval` | `rgba(12, 16, 26, 0.6)` | 审批面板 |
| `--surface-debug` | `rgba(10, 14, 20, 0.55)` | 调试面板 |
| `--surface-command` | `rgba(10, 12, 18, 0.7)` | 命令/代码块背景 |
| `--surface-diff-card` | `rgba(10, 14, 20, 0.32)` | Diff 卡片 |
| `--surface-bubble` | `rgba(255, 255, 255, 0.12)` | 助手消息气泡 |
| `--surface-bubble-user` | `rgba(77, 153, 255, 0.45)` | 用户消息气泡 |
| `--surface-context-core` | `rgba(10, 14, 20, 0.9)` | 上下文核心面板 |
| `--surface-popover` | `rgba(10, 14, 20, 0.995)` | 弹出层 |
| `--surface-review` | `rgba(255, 144, 200, 0.1)` | 审查状态表面 |

#### 文本色 (Text)

所有文本色基于白色透明度构建，形成 12 级文本层次：

| 令牌名 | 等效色值 | 不透明度 | 用途 |
|--------|---------|----------|------|
| `--text-primary` | `#e6e7ea` | ~90% | 主要文本（body） |
| `--text-strong` | `#ffffff` | 100% | 强调文本 |
| `--text-emphasis` | | ~90% | 突出文本 |
| `--text-stronger` | | ~85% | 较强文本 |
| `--text-quiet` | | ~75% | 安静文本 |
| `--text-muted` | | ~70% | 弱化文本 |
| `--text-subtle` | | ~60% | 次要文本 |
| `--text-faint` | | ~50% | 淡文本 |
| `--text-fainter` | | ~45% | 更淡文本 |
| `--text-dim` | | ~35% | 极淡文本 |
| `--text-accent` | `rgba(164, 195, 255, 0.7)` | | 强调色文本（蓝紫调） |
| `--message-link-color` | `rgba(196, 154, 255, 0.96)` | | 消息内链接（紫色） |

#### 边框色 (Borders)

| 令牌名 | 等效色值 | 不透明度 | 用途 |
|--------|---------|----------|------|
| `--border-subtle` | | ~8% | 极淡边框 |
| `--border-muted` | | ~6% | 弱边框 |
| `--border-strong` | | ~14% | 标准边框 |
| `--border-stronger` | | ~18% | 较强边框 |
| `--border-quiet` | | ~20% | 安静边框 |
| `--border-accent` | `rgba(100, 200, 255, 0.6)` | | 强调色边框（蓝） |
| `--border-accent-soft` | `rgba(100, 200, 255, 0.3)` | | 柔和强调边框 |

#### 状态色 (Status Colors)

| 状态 | 颜色 | 色值 |
|------|------|------|
| **Success (成功)** | 绿色 | `rgba(120, 235, 190, 0.95)` `#78ebbe` |
| **Warning (警告)** | 橙色 | `rgba(255, 175, 85, 0.95)` `#ffaf55` |
| **Error (错误)** | 红色 | `rgba(255, 110, 110, 0.95)` `#ff6e6e` |
| **Unknown (未知)** | 灰色 | `rgba(255, 255, 255, 0.3)` |

#### 审查状态色 (Review Colors)

| 状态 | 颜色 |
|------|------|
| **审查表面** | `rgba(255, 144, 200, 0.1)` |
| **审查边框** | `rgba(255, 144, 200, 0.35)` |
| **审查激活表面** | `rgba(255, 144, 200, 0.18)` |
| **审查激活文本** | `rgba(255, 195, 226, 0.95)` |
| **审查完成表面** | `rgba(120, 255, 200, 0.18)` |
| **审查完成文本** | `rgba(170, 255, 220, 0.95)` |

#### 线程状态指示色

| 状态 | 颜色 | 色值 |
|------|------|------|
| **Processing** | 橙色 | `#ff9f43` / `#ffb870`(chip) |
| **Reviewing** | 青色 | `#2fd1c4` / `#7be9dd`(chip) |
| **Unread** | 蓝色 | `#4da3ff` / `#8ac0ff`(chip) |
| **Ready** | 绿色 | `#3fe47e` / `#7be9a4`(chip) |

#### Diff/Git 操作色

| 操作 | 颜色 | 色值 |
|------|------|------|
| **Added (添加)** | 绿色 | `#47d488` |
| **Modified (修改)** | 金色 | `#f5c363` |
| **Deleted (删除)** | 红色 | `#ff6b6b` |
| **Renamed (重命名)** | 灰色 | var(--text-faint) |

### 2.2 Dim 主题色谱

Dim 主题在 Dark 基础上将背景色调向蓝灰色偏移：

- 侧边栏: `rgba(41, 44, 51, 0.78)` → 更不透明，更偏蓝灰
- 所有表面色的透明度略微提升
- 白色系文本保持不变（共享 dark.css 的文本色定义）
- 整体色温比纯黑 Dark 主题更暖、更柔和

### 2.3 Light 主题色谱

Light 主题完全重新定义所有令牌值：

#### 背景色（浅色系）

| 令牌名 | 颜色值 |
|--------|--------|
| `--surface-sidebar` | `rgba(246, 247, 250, 0.82)` |
| `--surface-topbar` | `rgba(250, 251, 253, 0.9)` |
| `--surface-right-panel` | `rgba(245, 247, 250, 0.82)` |
| `--surface-composer` | `rgba(250, 251, 253, 0.9)` |
| `--surface-messages` | `rgba(238, 241, 246, 0.9)` |

#### 文本色（深色系）

所有文本色基于 `rgba(17, 20, 28, X)` 构建（深灰色），从 90% 到 35% 不透明度。

#### 控件色（深色半透明）

控件的透明基础从白色变为深色 `rgba(15, 23, 36, X)`。

#### 状态色

- Success: `rgba(30, 155, 110, 0.9)` — 更深绿
- Warning: `rgba(215, 120, 20, 0.9)` — 更深橙
- Error: `rgba(200, 45, 45, 0.9)` — 更深红

### 2.4 System 主题

通过 `prefers-color-scheme` 媒体查询自动跟随系统，色值与 Light 主题完全一致（当系统为浅色模式时）。

---

## 3. 设计令牌系统

### 3.1 两级令牌架构

```
根层 (themes/dark.css)
  └── 定义原始色值 (--surface-*, --text-*, --border-* 等)

DS 层 (ds-tokens.css)
  └── 映射到语义令牌 (--ds-surface-*, --ds-border-*, --ds-text-*)

CM 层 (base.css, .app 内)
  └── 应用级 chrome 语言 (--cm-surface-*, --cm-border-*)
```

### 3.2 CM Surface System（Chrome 语言）

`.app` 容器内使用 `color-mix()` 构建 12 级表面层次：

| 令牌 | 混合公式 | 不透明度 |
|------|----------|----------|
| `--cm-surface-row` | `color-mix(..., card 44%, transparent)` | ~44% |
| `--cm-surface-panel-soft` | 56% | 56% |
| `--cm-surface-panel` | 64% | 64% |
| `--cm-surface-panel-strong` | 72% | 72% |
| `--cm-surface-panel-elevated` | 78% | 78% |
| `--cm-surface-panel-quiet` | 80% | 80% |
| `--cm-surface-panel-loud` | 84% | 84% |
| `--cm-surface-panel-hover` | 90% | 90% |
| `--cm-surface-panel-solid` | 92% | 92% |
| `--cm-surface-panel-active` | `color-mix(..., surface-active 96%, transparent)` | 激活态 |

### 3.3 CM Border System

同样的分层方式应用在边框上：

- `--cm-border-soft` (72%) → `--cm-border-default` (78%) → `--cm-border-strong` (82%)
- `--cm-border-emphasis` (84%) → `--cm-border-elevated` (86%) → `--cm-border-heavy` (88%)
- `--cm-border-hover` (92%) → `--cm-border-contrast` (100%)
- `--cm-border-accent` (36%) / `--cm-border-accent-strong` (46%)

### 3.4 动效令牌

| 令牌 | 值 | 用途 |
|------|-----|------|
| `--ds-dur-fast` | 120ms | 快速过渡（悬停、切换） |
| `--ds-dur-normal` | 160ms | 常规过渡（输入聚焦、边框变化） |
| `--ds-dur-slow` | 220ms | 慢速过渡（侧边栏宽度、布局变化） |
| `--ds-dur-entrance` | 200ms | 入场动画（模态框、弹出层） |
| `--ds-ease-out` | `cubic-bezier(0.16, 1, 0.3, 1)` | 标准缓出 |
| `--ds-ease-out-soft` | `cubic-bezier(0.25, 0.46, 0.45, 0.94)` | 柔和缓出 |
| `--ds-ease-spring` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | 弹簧缓动（toggle knob） |
| `--ds-spinner-dur` | 0.7s | 旋转动画周期 |
| `--ds-active-scale` | 0.97 | 按钮按下缩放 |
| `--ds-active-scale-sm` | 0.92 | 图标按钮按下缩放 |
| `--ds-layer-modal` | 10000 | 模态框层级 |
| `--ds-layer-toast` | 11000 | 提示框层级 |

---

## 4. 字体系统

### 4.1 字体族

```css
/* UI 字体 */
--ui-font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI",
  Roboto, "Helvetica Neue", Arial, sans-serif;

/* 代码字体 */
--code-font-family: ui-monospace, "Cascadia Mono", "Segoe UI Mono", Menlo,
  Monaco, Consolas, "Liberation Mono", "Courier New", monospace;

/* Subagent 角色标签 专用等宽 */
font-family: "SF Mono", "JetBrains Mono", "Fira Code", ui-monospace, ...;
```

### 4.2 字号体系

| 用途 | 字号 | 字重 |
|------|------|------|
| 页面主标题 (home) | 44px | 600 |
| 子标题 (home) | 16px | normal |
| 侧边栏标题 | 20px | 600 |
| 工作区标题 | 16px | 600 |
| 设置区标题 | 15px | 600 |
| 工作区名称 | 14px | 600 |
| 消息正文 | 13-14px | normal |
| Markdown 正文 | 13px | normal |
| 设置字段标签 | 12px | 600 |
| 面板导航项 | 13px | 600 |
| 线程名称 | 12px | 500 |
| 侧边栏区域标题 | 12px | 600 |
| 辅助信息/标签 | 11px | 500-600 |
| 小号文本/元数据 | 10px | 500-600 |
| 代码字体 | 11px | 400 |
| 上下文用量值 | 6px | 600 |

### 4.3 代码字体特殊设置

```css
--code-font-size: 11px;
--code-font-weight: 400;
--code-line-height: 1.28;
```

---

## 5. 布局架构

### 5.1 全局布局网格

应用使用 CSS Grid 构建主布局：

```
.app (CSS Grid)
├── 列: [sidebar(280px)] [main(1fr)]
├── 行: 隐式
│
├── Sidebar (column 1)
│   ├── sidebar-header（标题 + 操作按钮）
│   ├── sidebar-search（sticky 搜索栏）
│   ├── sidebar-body（滚动容器，带 fade 遮罩）
│   │   ├── workspace-list（工作区列表）
│   │   ├── thread-list（线程列表）
│   │   └── worktree-section（工作树区域）
│   └── sidebar-bottom-rail（底部账户 + 用量）
│
├── Main (column 2, CSS Grid 自身)
│   ├── 列: [content(1fr)] [right-panel(230px)]
│   ├── 行: auto 1fr auto auto auto
│   │
│   ├── main-topbar (row 1, sticky, z-index: 3)
│   │   ├── titlebar-toggle（侧边栏/右面板折叠按钮）
│   │   ├── workspace-header（工作区标题 + 分支信息）
│   │   └── main-header-actions（操作按钮）
│   │
│   ├── content (row 1-3, 绝对定位层系统)
│   │   ├── content-layer-chat（聊天层）
│   │   └── content-layer-diff（Diff 层）
│   │
│   ├── composer (row 3, sticky bottom)
│   ├── terminal-panel (row 4, 可调整高度)
│   └── debug-panel (row 5, 可调整高度)
│
└── right-panel (column 2, 可调整宽度)
    ├── right-panel-top（Git Diff / 文件树）
    ├── right-panel-divider（可拖拽分隔线）
    └── right-panel-bottom（Plan 面板）
```

### 5.2 内容层系统

内容区域使用**绝对定位层叠加**架构，而非传统的显示/隐藏：

- 所有 `content-layer` 使用 `position: absolute; inset: 0`
- `.content-layer.is-active` → `opacity: 1; pointer-events: auto; z-index: 0`
- `.content-layer.is-hidden` → `opacity: 0; pointer-events: none; z-index: -1`
- Chat 层和 Diff 层可以**分屏显示**（通过 `--chat-diff-split-position-percent` CSS 变量控制）

### 5.3 可调整面板 / 分隔线 (Resizers)

所有面板分隔线遵循统一模式：

```css
.resizer {
  position: absolute;
  width: 8px;        /* 拖拽热区 */
  cursor: col-resize; /* 或 row-resize */
  z-index: 2;
}

.resizer::after {
  /* 视觉指示线 */
  width: 1px;
  background: var(--border-strong);
  opacity: 0;         /* 默认不可见 */
}

.resizer:hover::after {
  opacity: 1;         /* 悬停可见 */
}
```

| 分隔线 | 位置 | 方向 | 控制 |
|--------|------|------|------|
| `sidebar-resizer` | 侧边栏右侧 | 水平 | `--sidebar-width` |
| `right-panel-resizer` | 右面板左侧 | 水平 | `--right-panel-width` |
| `right-panel-divider` | 右面板中间 | 垂直 | Git/Plan 高度比 |
| `content-split-resizer` | Chat/Diff 之间 | 水平 | 分屏比例 |
| `terminal-panel-resizer` | 终端顶部 | 垂直 | 终端高度 |
| `debug-panel-resizer` | 调试面板顶部 | 垂直 | 调试面板高度 |

拖动时，`.app` 添加 `.is-resizing` 类禁用 `transition`，确保流畅跟随。

### 5.4 三端布局模式

```
Desktop (> 960px)
  ┌──────────┬───────────────────────────────┐
  │ Sidebar  │ Main (Grid)                   │
  │ (280px)  │ ┌─────────────┬─────────────┐ │
  │          │ │ Content     │ Right Panel │ │
  │          │ │             │ (230px)     │ │
  │          │ └─────────────┴─────────────┘ │
  └──────────┴───────────────────────────────┘

Tablet (720px - 960px)
  ┌────┬──────────┬──────────────────────────┐
  │Nav │ Sidebar  │ Main (紧凑)              │
  │72px│ (280px)  │                          │
  └────┴──────────┴──────────────────────────┘

Phone (< 720px)
  ┌──────────────────────────────────────────┐
  │ 全屏内容 (Chat / Sidebar / Git / Settings)│
  ├──────────────────────────────────────────┤
  │ TabBar (5 tabs)                          │
  └──────────────────────────────────────────┘
```

---

## 6. 主题系统

### 6.1 四套主题

| 主题 | CSS 选择器 | 色系 | 默认 |
|------|-----------|------|------|
| **Dark** | `:root` (默认) | 深色背景 + 白色文本 | ✓ |
| **Dim** | `:root[data-theme="dim"]` | 灰蓝深色 | |
| **Light** | `:root[data-theme="light"]` | 浅色背景 + 深灰文本 | |
| **System** | `:root:not([data-theme])` + `@media (prefers-color-scheme: light)` | 跟随系统 | |

### 6.2 主题切换机制

- 通过 `data-theme` 属性在 `:root` 上切换
- System 主题检查 `data-theme` 不存在时使用系统媒体查询
- 主题间过渡平滑（所有颜色属性使用 CSS 变量，由浏览器自然过渡）

### 6.3 Reduced Transparency 模式

当应用添加 `.reduced-transparency` 类时：
- 所有表面色的透明度大幅提升（接近不透明）
- `backdrop-filter` 被移除
- 卡片/面板的视觉分层效果减弱
- 使用更明确的边框来补偿透明度的缺失

### 6.4 Windows 平台特殊适配

Windows 平台（通过 `.app.is-windows` 检测）：
- 表面色透明度大幅降低（侧边栏从 0.5 降到 0.15）
- 消息/编辑器区域变为几乎不透明（1.0）
- 目的：配合 Windows 原生窗口透明效果

---

## 7. 核心组件 UI 设计

### 7.1 按钮系统

#### 按钮类型

| 类型 | CSS 类 | 背景 | 文字色 | 边框 |
|------|--------|------|--------|------|
| **Primary** | `.primary` | `linear-gradient(135deg, #62b7ff, #4fe3a3)` | `#0b0f1a` | 无 |
| **Secondary** | `.secondary` | `var(--surface-card-strong)` | `inherit` | 无 |
| **Ghost** | `.ghost` | `transparent` | `var(--text-muted)` | `1px solid var(--border-strong)` |
| **Icon Button** | `.icon-button` | `transparent` | `inherit` | 无 |

#### 按钮交互

```
默认态:    border-radius: 10px; padding: 8px 14px
Hover:     transform: translateY(-1px); box-shadow: 0 12px 18px rgba(0,0,0,0.2)
Active:    transform: scale(0.97); box-shadow: 0 4px 8px rgba(0,0,0,0.15)
Disabled:  opacity: 0.5; cursor: not-allowed
```

主按钮的 Active 额外有 `filter: brightness(0.92)`，Ghost 按钮 Active 额外有半透明白色背景。

### 7.2 主顶栏操作按钮 (Main Header Actions)

```css
.main-header-action {
  padding: 8px;
  min-width: 32px; min-height: 32px;
  border-radius: 8px;
  border: 1px solid var(--cm-border-emphasis);
  background: var(--cm-surface-panel-strong);
  color: var(--text-muted);
}

/* Hover */
border-color: var(--cm-border-contrast);
background: var(--cm-surface-panel-hover);
color: var(--text-strong);

/* Active */
background: var(--cm-surface-panel-active);
border-color: var(--cm-border-accent);
color: var(--text-stronger);
```

按钮图标支持 **Copy-Check 动画**：复制操作后图标从 copy 图标平滑过渡到 check 图标（通过 opacity + scale + blur 三重过渡实现）。

### 7.3 面板标签切换 (Panel Tabs)

```css
.panel-tabs {
  display: inline-flex; gap: 6px; padding: 4px;
  border-radius: 999px;  /* 完全圆角容器 */
  background: var(--surface-control);
  border: 1px solid var(--border-subtle);
}

.panel-tab {
  padding: 6px 8px;
  border-radius: 999px;  /* 药丸形标签 */
  /* Active: */ background: var(--surface-control-hover);
}
```

### 7.4 分段控件 (Segmented Control)

设置界面使用分段选择器（`.settings-segmented`）：

- 外层容器：圆角 999px，内边距 4px
- 使用 `::before` 伪元素作为滑动指示器
- 指示器通过 `transform: translateX()` 在选项间平滑滑动
- 过渡使用 `cubic-bezier(0.645, 0.045, 0.355, 1)`（easeInOutCubic 风格）
- 两个选项宽度均为 `calc(50% - 6px)`

### 7.5 Toggle 开关

设置界面的 Toggle 开关（`.settings-toggle`）：

```css
.settings-toggle {
  width: 44px; height: 24px;
  border-radius: 999px;
  border: 1px solid var(--border-strong);
  background: var(--surface-control);
  padding: 3px;
}

/* ON 状态 */
background: linear-gradient(135deg, rgba(100,200,255,0.6), rgba(120,235,190,0.6));
border-color: var(--border-accent);

/* 滑块 */
.settings-toggle-knob {
  width: 16px; height: 16px;
  border-radius: 999px;
  transition: transform var(--ds-dur-normal) var(--ds-ease-spring);
  /* ON: transform: translateX(20px) */
}
```

---

## 8. 消息系统 UI 设计

### 8.1 消息容器

```css
.messages {
  overflow-y: auto;
  flex: 1;
}

.messages-inner {
  width: min(100%, var(--conversation-column-width, 900px));
  margin: 0 auto;
  padding: clamp(8px, 1.6vw, 18px) 0 28px;
  display: flex; flex-direction: column; gap: 14px;
}
```

消息列宽度限制在 900px 以提供更好的阅读体验。

### 8.2 消息气泡 (Bubble)

#### 助手消息气泡

```css
.message.assistant .bubble {
  max-width: 100%; width: 100%;
  padding: 16px 18px;
  border-radius: 18px;
  border: 1px solid var(--cm-border-emphasis);
  background: var(--cm-surface-panel-strong);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
}
```

#### 用户消息气泡

```css
.message.user .bubble {
  max-width: min(72%, 560px);
  border-radius: 18px;
  background: var(--cm-surface-panel-active);  /* 蓝色调 */
  border-color: color-mix(in srgb, var(--border-accent) 28%, transparent);
}
```

#### 气泡悬停效果

```css
.message.assistant .bubble:hover {
  background: color-mix(in srgb, var(--cm-surface-panel-strong) 88%, white);
  border-color: var(--cm-border-default);
  box-shadow: 
    inset 0 1px 0 rgba(255, 255, 255, 0.06),
    0 10px 28px rgba(7, 12, 34, 0.12);  /* 扩散阴影 */
  backdrop-filter: saturate(1.08) brightness(1.05);  /* 微增饱和度和亮度 */
}
```

#### 消息气泡操作按钮

悬停消息时，气泡下方显示两个浮动按钮：

- **复制按钮**（右下）：圆形，`width: 24px; height: 24px`，初始 `opacity: 0; transform: translateY(4px)`
- **引用按钮**（复制按钮左侧）：同样的动画和行为
- 悬停气泡时按钮 `opacity: 1; transform: translateY(0)`

### 8.3 图片网格

图片以水平滚动缩略图形式展示：

```css
.message-image-thumb {
  width: 88px;
  aspect-ratio: 1 / 1;
  border-radius: 12px;
  cursor: zoom-in;
}
/* Hover: outline: 1px solid var(--border-accent) */
```

点击图片打开灯箱（Lightbox）：
- 固定定位，覆盖全屏
- 背景: `rgba(10, 12, 16, 0.7)` 半透明暗色
- 图片最大 90vw/90vh
- 图片圆角 16px，带大阴影
- 关闭按钮在右上角（圆形白色按钮）

### 8.4 Markdown 渲染

#### 正文样式

```css
.markdown {
  font-size: 13px; line-height: 1.5;
  color: var(--text-stronger);
}
.markdown > * + * { margin-top: 8px; }
```

#### 行内代码

```css
.markdown :not(pre) > code {
  background: color-mix(in srgb, var(--surface-command) 82%, transparent);
  border: 1px solid var(--border-subtle);
  padding: 1px 5px; border-radius: 6px;
}
```

#### 代码块

```css
.markdown-codeblock {
  border: 1px solid var(--border-stronger);
  border-radius: 10px;
  background: var(--surface-command);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.markdown-codeblock-header {
  /* 顶部栏: 语言标签 + 复制按钮 */
  padding: 4px 8px;
  background: var(--surface-control);
  border-bottom: 1px solid var(--border-strong);
  font-size: 10px; text-transform: uppercase;
  color: var(--text-faint);
}

.markdown-codeblock pre {
  padding: 10px 12px 12px;
  font-family: var(--code-font-family);
  font-size: 11px; line-height: 1.28;
}
```

#### 表格

```css
.markdown-table-wrap {
  border: 1px solid var(--border-strong);
  border-radius: 14px;
  background: color-mix(in srgb, var(--surface-card-strong) 92%, transparent);
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.12);
}

.markdown-table thead th {
  position: sticky; top: 0;
  font-size: 10px; text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-faint);
}

.markdown-table tbody tr:hover td {
  background: color-mix(in srgb, var(--surface-control-hover) 36%, transparent);
}
```

表格列宽：首列 22rem，第二列 10rem，第三/四列自适应，最后一列 8rem。

#### 引用块

```css
.markdown blockquote {
  padding: 6px 10px;
  border-left: 2px solid var(--border-accent-soft);
  background: color-mix(in srgb, var(--surface-card-strong) 65%, transparent);
  border-radius: 6px;
  color: var(--text-muted);
}
```

#### 链接

- 颜色: `var(--message-link-color)` = `rgba(196, 154, 255, 0.96)` (紫色)
- Hover: `var(--text-stronger)`

#### 文件链接

特别的文件链接样式：
```css
.message-file-link {
  padding: 2px 7px; border-radius: 8px;
  border: 1px solid var(--border-subtle);
  background: color-mix(in srgb, var(--surface-command) 82%, transparent);
  color: var(--message-link-color);
  font-family: var(--code-font-family);
}
```

#### 标题

```css
.markdown h1, h2, h3, h4 {
  font-size: 1em;
  font-weight: 650;
  letter-spacing: 0.01em;
}
```

### 8.5 工具调用内联展示 (Tool Inline)

工具调用结果以内联卡片形式嵌入消息流：

```css
.tool-inline {
  border: 1px solid var(--cm-border-soft);
  border-radius: 16px;
  padding: 12px 14px;
  background: color-mix(in srgb, var(--cm-surface-panel-soft) 88%, transparent);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.018),
    inset 0 0 0 1px rgba(255, 255, 255, 0.006);
}
```

工具状态指示器（左侧圆点）：
- **Completed**: `var(--status-success)` (绿色)
- **Processing**: `var(--status-warning)` (橙色)
- **Failed**: `var(--status-error)` (红色)
- **Unknown**: `var(--status-unknown)` (灰色)

命令文本使用特殊样式：
```css
.tool-inline-command {
  font-family: var(--code-font-family);
  font-size: 11px;
  background: var(--surface-command);
  padding: 2px 6px; border-radius: 6px;
  border: 1px solid var(--border-subtle);
}
```

#### 终端输出内联

```css
.tool-inline-terminal {
  background: var(--cm-surface-command-panel);
  border-radius: 14px;
  border: 1px solid var(--cm-border-heavy);
  padding: 10px 12px;
  /* 顶部 fade 遮罩 */
  mask-image: linear-gradient(180deg, transparent 0%, #000 18px);
}
```

### 8.6 工具组 (Tool Group)

多个相关工具调用可折叠成组：

```css
.tool-group-header { padding: 0 4px; }
.tool-group-toggle {
  font-size: 11px; color: var(--text-muted);
  letter-spacing: 0.02em;
}
.tool-group-body .tool-inline {
  /* 组内工具调用使用更紧凑的样式 */
  padding: 8px 12px;
  font-size: 10px-11px;
}
```

### 8.7 工作状态指示器 (Working State)

处理中状态以药丸形组件展示：

```css
.working {
  display: inline-flex; align-items: center; gap: 10px;
  padding: 8px 12px;
  border-radius: 999px;
  border: 1px solid var(--cm-border-strong);
  background: var(--cm-surface-panel-strong);
  font-size: 11px; color: var(--text-muted);
}
```

包含旋转动画的 spinner（14x14px 半圆环）和闪烁的 "working" 文本（shimmer 动画）。

### 8.8 消息卡片 (Item Card)

样式类似 assistant 气泡但用途不同的卡片：

```css
.item-card {
  border: 1px solid var(--cm-border-emphasis);
  border-radius: 18px;
  padding: 14px 16px;
  background: var(--cm-surface-panel);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
}
```

支持 `<details>` 元素的展开/折叠（with chevron 旋转动画）。

#### 审查卡片 (.review)

专门的审查状态样式：
- 边框色: `var(--border-review)` (粉色调)
- 背景: `var(--surface-review)` (粉色半透明)

审查徽章：
- `.review-badge.active`: 粉色表面 + 粉色文本
- `.review-badge.done`: 绿色表面 + 绿色文本

### 8.9 思考状态指示

```css
.thinking {
  font-size: 12px;
  color: var(--text-fainter);
}
```

---

## 9. 编辑器 (Composer) UI 设计

### 9.1 整体布局

```css
.composer {
  display: flex; flex-direction: column; gap: 10px;
  padding: 10px var(--main-panel-padding) 20px;
  border-top: 1px solid var(--cm-border-default);
  background: var(--cm-surface-composer);    /* 98% composer + 2% card */
  backdrop-filter: blur(18px) saturate(1.1);  /* 毛玻璃效果 */
}

/* 内容最大宽度限制 */
.composer > * {
  width: min(100%, var(--conversation-column-width, 900px));
  margin-inline: auto;
}
```

### 9.2 输入区域

```css
.composer-input-area {
  border: 1px solid var(--cm-border-heavy);
  background: var(--cm-surface-panel-elevated);
  border-radius: 20px;
  padding: 10px 12px 10px 10px;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
}
```

输入区域是一个大的圆角卡片（border-radius: 20px），内含：

```
┌────────────────────────────────────────────┐
│ [附件图标] [text area...............] [操作按钮] │
│            [语音波形] [上传图片预览区]          │
└────────────────────────────────────────────┘
```

### 9.3 文本输入

```css
.composer textarea {
  min-height: 72px; max-height: 120px;
  height: 72px;
  resize: none;
  border: none; background: transparent;
  font-size: 14px; line-height: 1.5;
  color: inherit;
}
/* Focus: outline: none (无聚焦环，由外层卡片处理) */
```

### 9.4 操作按钮

操作按钮（发送、麦克风、附件等）使用圆形药丸设计：

```css
.composer-action {
  width: 30px; height: 30px;
  border-radius: 999px;
  border: 1px solid var(--cm-border-emphasis);
  background: var(--cm-surface-panel-strong);
  color: var(--text-strong);
}
```

#### 麦克风按钮状态

- **Active (录音中)**: 绿色边框 + 绿色半透明背景
- **Processing (处理中)**: 蓝色边框 + 蓝色半透明背景
- **Stop (停止)**: 红色边框 + 红色半透明背景

#### 附件按钮

```css
.composer-attach {
  width: 28px; height: 28px;
  border-radius: 999px;
  border: 1px solid var(--cm-border-strong);
  background: var(--cm-surface-panel-solid);
  margin-top: 5px;  /* 与文本顶部对齐 */
}
```

### 9.5 附件预览

附件以药丸形标签展示在输入区上方：

```css
.composer-attachment {
  padding: 2px 8px; border-radius: 999px;
  background: var(--surface-card);
  border: 1px solid var(--border-muted);
  font-size: 11px;
}
```

悬停附件的图片预览：
- 定位在附件上方 8px，居中
- 240x180px，圆角 12px
- 从 `opacity: 0; transform: translateY(-2px) scale(0.98)` 入场
- 带 `0 12px 28px rgba(0,0,0,0.24)` 阴影

### 9.6 拖放状态

输入区域在拖放文件时添加虚线轮廓：

```css
.composer-input-area.is-drag-over {
  outline: 1px dashed color-mix(in srgb, var(--border-accent) 58%, transparent);
  outline-offset: 4px;
}
```

### 9.7 语音波形显示

```css
.composer-waveform {
  margin-top: 10px; padding: 6px 8px;
  border-radius: 10px;
  border: 1px solid var(--border-muted);
  background: var(--surface-card);
  height: 40px;
  display: flex; align-items: flex-end; gap: 3px;
}
```

- 多个竖条（`.composer-waveform-bar`）通过 `height` 过渡模拟波形
- Processing 状态：渐变背景（蓝→绿）
- 标签叠加在波形中间显示状态文本

### 9.8 底部工具栏 (Composer Bar)

```css
.composer-bar {
  display: flex; align-items: center;
  padding-top: 10px;
  border-top: 1px solid var(--cm-border-strong);
  gap: 12px;
}
```

#### 模型/协作选择器 (Select Wraps)

选择器包裹在药丸形容器中：

```css
.composer-select-wrap {
  padding: 4px 8px;
  border-radius: 999px;
  background: var(--cm-surface-panel-strong);
}
```

使用 CSS 自定义下拉箭头（border-right + border-bottom 旋转 45deg）。

原生的 `<select>` 被完全重样式：
- `appearance: none`
- 透明背景，药丸形容器提供背景色
- 选择框宽度受 CSS 变量控制（model: `--composer-model-select-width`）

#### 计划模式开关

```css
.composer-plan-toggle {
  display: inline-flex; align-items: center; gap: 6px;
}
```

- Checkbox + 图标 + 标签（"Plan"）
- 选中时图标和标签变为 `var(--text-strong)`
- Checkbox 使用紫色 `accent-color`

#### 上下文用量指示器 (Context Ring)

```css
.composer-context-ring {
  width: 20px; height: 20px;
  border-radius: 999px;
  /* 使用 conic-gradient 实现环形进度 */
  background:
    radial-gradient(circle, var(--surface-context-core) 54%, transparent 56%),
    conic-gradient(
      from 180deg,
      hsl(calc(120deg * var(--context-free) / 100), 80%, 55%) ...,  /* 绿→红渐变 */
      var(--border-strong) 0
    );
}
```

环形图内部显示百分比值（6px 字号），悬停时通过 `::after` 伪元素显示 tooltip。

### 9.9 自动补全建议 (Suggestions)

```css
.composer-suggestions {
  position: absolute;
  bottom: calc(100% + 6px);
  width: min(100%, 420px);
  max-height: 220px; overflow-y: auto;
  padding: 8px; border-radius: 16px;
  /* 继承 popover 背景和边框 */
}
```

每个建议项：
```css
.composer-suggestion {
  display: flex; flex-direction: column; gap: 2px;
  border-radius: 8px; padding: 6px 8px;
  /* Hover: background: var(--surface-item); border-color: var(--border-muted); */
}
```

建议项结构：
- 分区标题（大写标签）
- 图标（16x16）+ 标题（12px 粗体）+ 描述（10px，技能类限 2 行）

### 9.10 上下文操作按钮

```css
.composer-context-action {
  font-size: 11px;
  padding: 6px 11px;
  border-radius: 999px;
  border: 1px solid var(--cm-border-emphasis);
  background: var(--cm-surface-panel-strong);
  color: var(--text-muted);
}
```

### 9.11 待处理队列 (Queue) / 跟进提示

```css
.composer-followup-hint,
.composer-queue {
  padding: 10px 12px;
  border-radius: 16px;
  border: 1px solid var(--cm-border-emphasis);
  background: var(--cm-surface-panel);
}
```

---

## 10. 侧边栏 UI 设计

### 10.1 侧边栏容器

```css
.sidebar {
  padding: var(--sidebar-top-padding, 36px) var(--sidebar-padding, 20px) 20px;
  background: var(--surface-sidebar);
  border-right: 1px solid var(--border-subtle);
  display: flex; flex-direction: column; gap: 6px;
  overflow: hidden;
}
```

### 10.2 侧边栏头部

```
┌─────────────────────────────────┐
│ [标题 "codex monitor"]  [排序][搜索][刷新] │
└─────────────────────────────────┘
```

标题按钮：padding 4px 6px, border-radius 8px，hover 显示 `var(--surface-hover)`。

排序/搜索/刷新按钮：32x32px，完全透明背景，hover 显示 `var(--surface-hover)`。

### 10.3 搜索栏

```css
.sidebar-search {
  position: sticky; top: 0; z-index: 4;
  padding: 6px 4px 10px;
}

.sidebar-search-input {
  padding: 10px 32px 10px 14px;
  border-radius: 14px;
  border: 1px solid var(--border-quiet);
  background: var(--cm-surface-panel-strong);
  font-size: 13px;
}

.sidebar-search-input:focus {
  border-color: var(--border-accent);
  box-shadow: 0 0 0 2px var(--border-accent-soft);
}
```

搜索栏默认折叠（height: 0; overflow: hidden），展开时使用 sticky 定位保持在顶部。

### 10.4 工作区列表 (Workspace List)

#### 工作区卡片 (Workspace Row)

```css
.workspace-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 16px;  /* 注意：使用 ::before 伪元素实现 */
  background: transparent;  /* 实际背景在 ::before */
  position: relative; isolation: isolate;
}

.workspace-row::before {
  content: "";
  position: absolute; inset: 0;
  border-radius: 16px;
  background: var(--cm-surface-panel-strong);
  border: 1px solid var(--cm-border-default);
  box-shadow: 
    inset 0 1px 0 rgba(255, 255, 255, 0.02),
    inset 0 0 0 0 rgba(255, 255, 255, 0);
}
```

**为什么使用 `::before` 伪元素？** — 为了实现 hover 和 active 状态下的 smooth 过渡。伪元素的背景/阴影/边框变化不会影响布局。

#### 悬停状态

```css
.workspace-row:hover::before {
  background: color-mix(in srgb, var(--cm-surface-panel-hover) 78%, white);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.07),
    inset 0 0 0 1px color-mix(in srgb, var(--cm-border-default) 42%, transparent);
}
```

#### 激活状态

```css
.workspace-row.active::before {
  background: var(--cm-surface-panel-active);        /* 蓝色调 */
  border-color: var(--cm-border-accent-strong);       /* 蓝色边框 */
}
```

#### 添加/切换按钮

- 工作区折叠/展开 toggle（12px 图标，初始 opacity 0.45）
- "添加"按钮（22x22px 圆形，初始 opacity 0.46）
- 两者在行悬停时变为 opacity: 1

#### 分组折叠

工作区组使用 CSS Grid `grid-template-rows` 动画折叠：

```css
.workspace-group-list.collapsed {
  grid-template-rows: 0fr;
  opacity: 0;
  transform: translateY(-4px);
}
```

### 10.5 线程列表 (Thread List)

#### 线程行 (Thread Row)

```css
.thread-row {
  display: flex; align-items: center; gap: 10px;
  padding: 9px 12px 10px calc(10px + var(--thread-indent, 0px));
  border-radius: 14px;
  background: var(--cm-surface-row);
  border: 1px solid transparent;
  font-size: 12px;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.02);
}
```

线程缩进通过 `--thread-indent` 变量控制（子线程缩进）。

#### 线程状态指示器

```css
.thread-status {
  width: 8px; height: 8px;
  border-radius: 999px;
  flex-shrink: 0;
}
```

四种状态：
- **Processing** (橙色 `#ff9f43`): `box-shadow: 0 0 0 4px rgba(255,159,67,0.12)` + pulse 动画
- **Reviewing** (青色 `#2fd1c4`): 同样的光晕 + pulse 动画
- **Unread** (蓝色 `#4da3ff`): 静态光晕
- **Ready** (绿色 `#3fe47e`): 静态弱光晕

#### 线程悬停状态

```css
.thread-row:hover {
  background: color-mix(in srgb, var(--cm-surface-panel-hover) 76%, white);
  border-color: var(--cm-border-default);
  color: var(--text-strong);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.08),
    inset 0 0 0 1px color-mix(in srgb, var(--cm-border-default) 46%, transparent);
}
```

#### 线程激活状态

```css
.thread-row.active {
  background: color-mix(in srgb, var(--surface-active) 96%, transparent);
  border-color: color-mix(in srgb, var(--border-accent) 42%, transparent);
}
```

#### 线程内容布局

```
┌──────────────────────────────────────┐
│ ● 线程名称              [时间] [操作]   │
│   工作区标签 / Subagent pill          │
│   状态 chip  / 上下文标签             │
└──────────────────────────────────────┘
```

Subagent 标签使用 HSL 动态配色：
- 色相、饱和度变量由组件动态设置
- Light/Dark 主题各有独立的颜色映射
- 使用 `hsl()` 函数实现自适应明暗

#### 状态 Chip

```css
.thread-state-chip {
  padding: 2px 7px; border-radius: 999px;
  font-size: 10px; font-weight: 600;
}
```

每种状态有独立的文字色、背景色、边框色组合（processing/reviewing/unread/ready）。

#### 固定线程

```css
.thread-row.is-pinned {
  background: var(--cm-surface-panel-strong);
}
```

#### 骨架屏 (Skeleton)

```css
.thread-skeleton {
  height: 8px;
  border-radius: 999px;
  background: linear-gradient(
    110deg,
    rgba(255,255,255,0.04) 8%,
    rgba(255,255,255,0.18) 18%,
    rgba(255,255,255,0.04) 33%
  );
  background-size: 200% 100%;
  animation: shimmer 1.4s ease-in-out infinite;
}
```

### 10.6 工作树区域 (Worktree Section)

```css
.worktree-row {
  padding: 10px 12px;
  border-radius: 14px;
  background: var(--cm-surface-row);
  /* 样式逻辑与 workspace-row 类似 */
}
```

工作树删除状态：
- 整体 opacity 降至 0.6，pointer-events: none
- 显示删除 spinner（10x10px 半圆环旋转动画）

### 10.7 底部栏 (Sidebar Bottom Rail)

```
┌──────────────────────────────┐
│ ─── 用量面板 ───              │
│ [项目1] [████████░░]          │
│ [项目2] [██████░░░░]          │
│                              │
│ [账户按钮] [设置] [更新]       │
└──────────────────────────────┘
```

#### 用量条

```css
.sidebar-usage-bar {
  height: 4px; border-radius: 999px;
  background: color-mix(in srgb, var(--surface-card-muted) 86%, transparent);
}

.sidebar-usage-bar-fill {
  background: linear-gradient(90deg, 
    rgba(120, 235, 190, 0.92),    /* 绿 */
    rgba(100, 200, 255, 0.92));   /* 蓝 */
  box-shadow: 0 0 12px rgba(92, 168, 255, 0.18);
  transition: width 180ms ease;
}
```

#### 账户按钮

```css
.sidebar-labeled-button {
  height: 34px; padding: 4px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-quiet);
  background: color-mix(in srgb, var(--surface-hover) 82%, transparent);
}
```

#### 账户头像

```css
.sidebar-account-avatar {
  width: 20px; height: 20px;
  border-radius: 999px;
  background: linear-gradient(135deg, 
    rgba(120, 235, 190, 0.28),    /* 绿 */
    rgba(100, 200, 255, 0.26));   /* 蓝 */
}
```

#### 账户弹出层

- 向上展开（bottom: calc(100% + 8px)）
- 包含账户信息 + 操作按钮
- 使用 `.ds-popover` 样式

### 10.8 拖放覆盖层

当拖拽文件到侧边栏时：

```css
.workspace-drop-overlay {
  position: absolute; inset: 0; z-index: 8;
  background: rgba(10, 12, 16, 0.4);
  backdrop-filter: blur(10px);
  opacity: 0; transition: opacity 0.2s ease;
}
.workspace-drop-overlay.is-active { opacity: 1; }
```

覆盖层文字使用 shimmer 动画（与 skeleton 相同的渐变闪烁效果）。

### 10.9 滚动区域 Fade 遮罩

侧边栏 body 使用 CSS mask 实现顶部/底部淡出效果：

```css
.sidebar-body.fade-top.fade-bottom {
  mask-image: linear-gradient(
    to bottom,
    transparent 0, #000 16px,          /* 顶部 16px 淡入 */
    #000 calc(100% - 16px), transparent 100%  /* 底部 16px 淡出 */
  );
}
```

---

## 11. Git/差异面板 UI 设计

### 11.1 面板概览 (Overview)

```css
.git-panel-overview {
  display: flex; flex-direction: column; gap: 10px;
  padding: 0 0 12px;
  border-bottom: 1px solid var(--cm-border-soft);
}
```

包含：
- **分支选择器**（药丸形 select，带搜索图标前缀）
- **状态摘要**（文件变更统计，10px 大写标签）
- **分支信息行**（分支名 + 刷新按钮）

### 11.2 分支选择器

```css
.git-panel-select-input {
  padding: 9px 34px 9px 34px;
  border-radius: 999px;
  border: 1px solid var(--cm-border-default);
  background: var(--cm-surface-panel);
  font-size: 12px; font-weight: 600;
  appearance: none;
  /* 自定义下拉箭头 */
  background-image: 
    linear-gradient(45deg, transparent 50%, var(--text-dim) 50%),
    linear-gradient(135deg, var(--text-dim) 50%, transparent 50%);
  background-position: calc(100% - 16px) 50%, calc(100% - 11px) 50%;
  background-size: 5px 5px;
}
```

### 11.3 差异列表 (Diff List)

#### 列表项 (Diff Row)

```css
.diff-row {
  display: grid;
  grid-template-columns: 16px minmax(0, 1fr) auto;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 12px;
  transition: background 160ms ease, box-shadow 160ms ease;
}

.diff-row:hover {
  background: var(--cm-surface-panel-hover);
}

.diff-row.active {
  background: var(--cm-surface-panel-active);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--cm-border-accent) 65%, transparent);
}
```

#### 文件图标（状态小方块）

```css
.diff-icon {
  width: 16px; height: 16px;
  border-radius: 999px;
  font-size: 10px; font-weight: 700;
}
```

- **Added** (A): 绿色 (#47d488) + 绿色半透明背景
- **Modified** (M): 金色 (#f5c363) + 金色半透明背景
- **Deleted** (D): 红色 (#ff6b6b) + 红色半透明背景
- **Renamed** (R): 灰色

#### 行内操作按钮

悬停时从右侧滑入：
```css
.diff-row-actions {
  max-width: 0; overflow: hidden; opacity: 0;
  transform: translateX(6px);
  transition: max-width 180ms ease, opacity 140ms ease, transform 140ms ease;
}

.diff-row:hover .diff-row-actions {
  max-width: 88px; opacity: 1;
  transform: translateX(0);
}
```

操作按钮：
- **Stage** (绿色 hover)
- **Unstage** (金色 hover)
- **Discard** (红色 hover)
- **Review** (蓝色 hover)
- **Apply** (蓝色 hover)

### 11.4 按文件分组 (Per-File Group)

文件路径按目录分组，每行使用 3 列网格：
```css
.per-file-group-row {
  grid-template-columns: 14px minmax(0, 1fr) auto;
  gap: 8px; padding: 9px 10px; border-radius: 12px;
}
```

### 11.5 提交消息面板 (Commit Message)

```css
.commit-message-section {
  padding: 12px;
  border-radius: 18px;
  border: 1px solid var(--cm-border-default);
  background: var(--cm-surface-panel-soft);
}

.commit-message-input {
  font-family: var(--code-font-family);
  font-size: 11px; line-height: 1.5;
  border-radius: 14px;
  border: 1px solid var(--cm-border-default);
  background: var(--cm-surface-panel);
  min-height: 56px; max-height: 132px;
  padding: 11px 40px 11px 12px;
}

.commit-message-input:focus {
  background: var(--cm-surface-panel-hover);
  border-color: var(--cm-border-accent-strong);
}
```

生成按钮（AI 生成提交消息）位于输入框右上角，26x26px 圆形。

### 11.6 提交按钮

```css
.commit-button {
  width: 100%;
  padding: 10px 14px;
  border-radius: 14px;
  border: 1px solid var(--cm-border-default);
  background: var(--cm-surface-panel);
}

.commit-button:hover {
  background: var(--cm-surface-panel-hover);
  border-color: var(--cm-border-accent-strong);
  color: var(--text-strong);
}
```

### 11.7 Push/Sync 按钮

双按钮布局：
```css
.push-sync-buttons {
  display: flex; gap: 8px;
}
```

每个按钮占 flex: 1，样式同 commit 按钮。

### 11.8 Git Root（未初始化）面板

当工作区没有初始化 Git 时的引导面板：

```css
.git-root-panel {
  padding: 14px;
  border-radius: 18px;
  border: 1px solid var(--cm-border-default);
  background: var(--cm-surface-panel-soft);
}
```

包含：
- 标题（13px bold）
- Primary 操作按钮（蓝色调，`git-root-button.primary`）
- 次级操作按钮列表

### 11.9 Git Log / Issues / PR 列表

```css
.git-log-entry, .git-issue-entry, .git-pr-entry {
  padding: 9px 10px;
  border-radius: 12px;
  transition: background 160ms ease;
}
/* Hover: background: var(--cm-surface-panel-hover) */
/* Active: background + accent border inset shadow */
```

---

## 12. 设置界面 UI 设计

### 12.1 设置窗口

```css
.settings-window {
  width: min(980px, 94vw);
  height: min(680px, 88vh);
  border-radius: 18px;
  display: flex; flex-direction: column;
  overflow: hidden;
}
```

### 12.2 主从布局 (Master-Detail)

```css
.settings-body {
  display: grid;
  grid-template-columns: 200px minmax(0, 1fr);
}
```

```
┌─────────────────────────────────────────┐
│ Titlebar: 标题 "Settings"   [关闭按钮]    │
├──────────┬──────────────────────────────┤
│ Sidebar  │ Content                     │
│ (200px)  │ (scrollable)                │
│          │                              │
│ Nav Item │ 设置表单字段                  │
│ Nav Item │                              │
│ Nav Item │                              │
└──────────┴──────────────────────────────┘
```

移动端切换为单列 + 主从导航（master visible 或 detail visible）。

### 12.3 导航项

```css
.ds-panel-nav-item {
  width: 100%;
  padding: 8px 10px; border-radius: 10px;
  font-size: 13px; font-weight: 600;
  border: 1px solid transparent;
  color: var(--text-muted);
}

.ds-panel-nav-item:hover,
.ds-panel-nav-item.is-active {
  background: var(--surface-card);
  border-color: var(--border-strong);
  color: var(--text-strong);
}
```

### 12.4 表单字段

```css
.settings-field {
  display: flex; flex-direction: column; gap: 10px;
  margin-bottom: 18px;
}

.settings-field-label {
  font-size: 12px; font-weight: 600;
  color: var(--text-strong);
}
```

#### 文本输入

```css
.settings-input {
  padding: 10px 12px; border-radius: 10px;
  border: 1px solid var(--border-muted);
  background: var(--surface-control);
  color: var(--text-strong); font-size: 12px;
  transition: border-color 160ms ease, box-shadow 160ms ease;
}
```

#### 下拉选择

```css
.settings-select {
  padding: 10px 12px; border-radius: 10px;
  border: 1px solid var(--border-muted);
  background: var(--surface-control);
  color: var(--text-strong); font-size: 12px;
}
```

### 12.5 Toggle 行

```css
.settings-toggle-row {
  display: flex; align-items: center; justify-content: space-between;
  gap: 16px;
  padding: 14px 16px; border-radius: 12px;
  background: var(--surface-card);
  border: 1px solid var(--border-muted);
}
```

### 12.6 项目/组/覆盖行

通用的卡片式行布局：
```css
.settings-project-row {
  padding: 12px 14px; border-radius: 12px;
  background: var(--surface-card);
  border: 1px solid var(--border-muted);
}
```

### 12.7 下载进度条

```css
.settings-download-bar {
  height: 6px; border-radius: 999px;
  background: var(--surface-control);
}

.settings-download-fill {
  background: linear-gradient(90deg, 
    rgba(100, 200, 255, 0.7), 
    rgba(120, 235, 190, 0.8));
}
```

### 12.8 Agent 编辑器

- 大文本域（min-height: 150px，代码字体）
- Stepper 控件（圆形按钮 + 数值）
- Agent 卡片列表

---

## 13. 主页 UI 设计

### 13.1 整体布局

```css
.home {
  padding: calc(36px + var(--home-scroll-offset)) 32px 24px;
  overflow-y: auto;
  /* Desktop: 内容宽度限制在 720px */
}

.app.layout-desktop .home > * {
  width: min(100%, 720px);
  margin-inline: auto;
}
```

### 13.2 英雄区域 (Hero)

```css
.home-hero { text-align: center; gap: 6px; }
.home-title {
  font-size: 44px; font-weight: 600;
  letter-spacing: -0.02em;  /* 紧凑字间距 */
}
.home-subtitle {
  font-size: 16px; color: var(--text-muted);
}
```

### 13.3 最新工作区卡片

```css
.home-latest-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}

.home-latest-card {
  padding: 14px 14px 16px;
  border-radius: 16px;
  background:
    linear-gradient(135deg, rgba(100, 200, 255, 0.12), transparent 60%),
    var(--surface-card);
  border: 1px solid var(--border-subtle);
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.2);
}
```

卡片通过 `::after` 伪元素添加内边框。

卡片按钮 hover 时 `translateY(-1px)` + 更大阴影。

### 13.4 用量仪表盘

#### 用量卡片

```css
.home-usage-card {
  padding: 12px 14px; border-radius: 14px;
  background: 
    linear-gradient(135deg, rgba(98, 176, 255, 0.15), transparent 60%),
    var(--surface-card);
  border: 1px solid var(--border-subtle);
  box-shadow: 0 10px 22px rgba(0, 0, 0, 0.2);
  min-height: 96px;
}
```

趋势变体：
- `.is-up`: 蓝色调渐变
- `.is-down`: 橙色调渐变

数字显示：24px 粗体，带后缀（11px 大写）。

#### 柱状图

```css
.home-usage-chart {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  align-items: end;
  gap: 8px;
  height: 120px;
}

.home-usage-bar-fill {
  border-radius: 2px;
  background: linear-gradient(180deg, 
    rgba(99, 186, 255, 0.9), 
    rgba(78, 132, 255, 0.65));
  box-shadow: 0 6px 14px rgba(30, 100, 200, 0.25);
}
```

柱状图悬停显示 tooltip（通过 `::after` + `attr(data-value)`）。

### 13.5 快速操作按钮

```css
.home-actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.home-button {
  padding: 16px 20px; border-radius: 18px;
  font-size: 15px; min-height: 56px;
  background: var(--surface-card-strong);
  border: 1px solid var(--border-strong);
}
```

### 13.6 配置模型列表

模型以药丸形 chip 展示：
```css
.home-usage-model-chip {
  padding: 4px 10px; border-radius: 999px;
  font-size: 12px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border-subtle);
}
```

### 13.7 空状态

```css
.home-usage-empty {
  padding: 14px; border-radius: 14px;
  border: 1px dashed var(--border-subtle);
  background: rgba(255, 255, 255, 0.02);
}
```

---

## 14. 终端面板 UI 设计

### 14.1 面板容器

```css
.terminal-panel {
  border-top: 1px solid var(--border-subtle);
  background: var(--surface-debug);
  height: var(--terminal-panel-height, 220px);
}
```

### 14.2 终端头部

```
┌──────────────────────────────────────────┐
│ [Tab1] [Tab2] [Tab3] ... [+ 新建]        │
└──────────────────────────────────────────┘
```

```css
.terminal-tab {
  padding: 4px 12px; border-radius: 999px;
  font-size: 11px; text-transform: uppercase;
  letter-spacing: 0.04em;
  border: 1px solid transparent;
}

.terminal-tab.active {
  color: var(--text-stronger);
  border-color: var(--border-strong);
  background: rgba(255, 255, 255, 0.06);
}

.terminal-tab-add {
  border: 1px dashed var(--border-strong);
  padding: 4px 12px; border-radius: 999px;
}
```

### 14.3 xterm.js 集成

```css
.terminal-surface {
  background: var(--terminal-background);
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}
```

xterm 渲染时强制使用终端背景色覆盖。

### 14.4 状态覆盖层

```css
.terminal-overlay {
  position: absolute; inset: 0;
  display: flex; align-items: center; justify-content: center;
  pointer-events: none;
}

.terminal-status {
  background: var(--surface-card);
  border: 1px solid var(--border-subtle);
  border-radius: 10px;
  padding: 8px 12px;
}
```

---

## 15. 调试面板 UI 设计

```css
.debug-panel {
  border-top: 1px solid var(--border-subtle);
  background: var(--surface-debug);
}

.debug-panel.open {
  height: var(--debug-panel-height, 180px);
}
```

包含：
- **头部**: 标题 "DEBUG"（大写，letter-spacing 0.08em）+ 操作按钮
- **日志列表**: 按来源分组
  - 来源标签（药丸形，Error 源为红色，Stderr 源为橙色）
  - Payload 文本（等宽字体，pre-wrap）

---

## 16. 计划面板 UI 设计

```css
.plan-panel {
  margin: 0 12px;
  display: flex; flex-direction: column; gap: 8px;
}
```

### 16.1 头部

```css
.plan-header {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-subtle);
}
```

### 16.2 步骤列表

```css
.plan-step {
  display: flex; gap: 8px;
  font-size: 12px;
}

.plan-step-status {
  font-family: var(--code-font-family);
  font-size: 11px;
  color: var(--text-faint);
  /* inProgress: #86b7ff (蓝色) */
  /* completed: #47d488 (绿色) */
}
```

---

## 17. 设计系统原子组件

### 17.1 ModalShell (模态框)

```css
.ds-modal {
  position: fixed; inset: 0;
  z-index: var(--ds-layer-modal, 10000);
}

.ds-modal-backdrop {
  position: absolute; inset: 0;
  background: var(--ds-modal-backdrop);           /* rgba(6,8,12,0.55) */
  backdrop-filter: blur(8px);
  animation: ds-modal-backdrop-in 200ms ease-out;
}

.ds-modal-card {
  position: absolute;
  top: 50%; left: 50%;
  transform: translate(-50%, -50%);
  background: var(--ds-modal-card-bg);             /* surface-card-strong */
  border: 1px solid var(--ds-modal-card-border);   /* border-stronger */
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.35);
  border-radius: 由具体使用方设置;
}
```

入场动画：
```css
@keyframes ds-modal-card-in {
  from {
    opacity: 0;
    transform: translate(-50%, -48%) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translate(-50%, -50%) scale(1);
  }
}
```

模态框聚焦环：
```css
.ds-modal :where(input, textarea, select):focus-visible {
  outline: 2px solid var(--ds-modal-focus-ring);   /* 蓝色 focus ring */
  outline-offset: 1px;
}
```

### 17.2 ToastPrimitives (提示框)

```css
.ds-toast-viewport {
  display: grid; gap: 12px;
}

.ds-toast-card {
  background: var(--ds-toast-bg);              /* surface-context-core */
  border: 1px solid var(--ds-toast-border);    /* border-subtle */
  box-shadow: var(--ds-toast-shadow);          /* 0 16px 32px rgba(0,0,0,0.25) */
  border-radius: 12px;
  padding: 12px;
  animation: ds-toast-in 0.2s ease-out;
}
```

错误 toast 包含代码块样式的错误信息区域（等宽字体，pre-wrap）。

### 17.3 PopoverPrimitives (弹出层)

```css
.ds-popover {
  background: var(--ds-popover-bg);            /* surface-popover */
  border: 1px solid var(--ds-popover-border);  /* border-muted */
  box-shadow: var(--ds-popover-shadow);        /* 0 14px 34px rgba(0,0,0,0.3) */
  border-radius: 10px;
  animation: ds-popover-in 120ms ease-out both;
}

@keyframes ds-popover-in {
  from { opacity: 0; translate: 0 -4px; scale: 0.98; }
  to   { opacity: 1; translate: 0 0;    scale: 1; }
}
```

弹出项：
```css
.ds-popover-item {
  padding: 6px 8px; border-radius: 8px;
  font-size: 12px;
  color: var(--ds-popover-item-text);          /* text-muted */
  /* Hover/Active: */
  background: var(--surface-hover);
  color: var(--ds-popover-item-text-active);  /* text-stronger */
}
```

### 17.4 PanelPrimitives (面板)

```css
.ds-panel {
  display: flex; flex-direction: column;
  gap: var(--ds-panel-gap, 8px);
  padding: 12px 12px 0;
}

.ds-panel-header {
  display: flex; justify-content: space-between; align-items: center;
  min-height: var(--ds-panel-header-min-height, 26px);
  color: var(--ds-panel-header-text);          /* text-subtle */
}
```

在 reduced-transparency 模式下添加背景和圆角边框。

### 17.5 Tooltip

```css
.ds-tooltip-trigger[data-tooltip]::after {
  content: attr(data-tooltip);
  position: absolute;
  left: 50%; bottom: calc(100% + 8px);
  transform: translateX(-50%) translateY(4px);  /* 初始下移 */
  padding: 4px 8px; border-radius: 8px;
  background: var(--surface-command);
  color: var(--text-emphasis);
  font-size: 10px; white-space: nowrap;
  opacity: 0;
  transition: opacity 150ms ease, transform 150ms ease;
}

/* Hover: opacity: 1; transform: translateX(-50%) translateY(0); */
```

包含 CSS 箭头（使用 `::before` + border 旋转 45deg）。

### 17.6 TabBar (移动端)

```css
.tabbar {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 6px;
  padding: 8px 12px;
  border-top: 1px solid var(--border-subtle);
  background: var(--surface-topbar);
  backdrop-filter: blur(24px) saturate(1.2);
}
```

标签项：
```css
.tabbar-item {
  border-radius: 12px;
  font-size: 12px; font-weight: 600;
  display: flex; flex-direction: column;
  align-items: center; gap: 4px;
}

.tabbar-item.active {
  color: var(--text-strong);
  background: var(--surface-control-hover);
  border-color: var(--border-accent-soft);
  box-shadow: 0 0 0 1px rgba(100, 200, 255, 0.2);
}
```

编辑器聚焦时标签栏折叠隐藏（max-height: 0）。

---

## 18. 动画与过渡系统

### 18.1 关键帧动画

| 动画名 | 时长 | 缓动 | 用途 |
|--------|------|------|------|
| `pulse` | 1.2s | `cubic-bezier(0.4,0,0.6,1)` | 线程状态指示器脉冲 |
| `shimmer` | 1.4s | ease-in-out | 骨架屏闪烁 |
| `working-shimmer` | 2.2s | ease-in-out | 工作状态文本闪烁 |
| `working-spin` | 0.7s | linear | 加载旋转 |
| `sidebar-refresh-spin` | 0.7s | linear | 侧边栏刷新图标 |
| `composer-action-spin` | 0.7s | linear | 编辑器操作旋转 |
| `drop-text-shimmer` | 1.2s | ease-in-out | 拖放文本闪烁 |
| `ds-modal-backdrop-in` | 200ms | ease-out | 模态背景淡入 |
| `ds-modal-card-in` | 220ms | ease-out | 模态卡片入场 |
| `ds-popover-in` | 120ms | ease-out | 弹出层入场 |
| `ds-toast-in` | 200ms | ease-out | 提示框入场 |

### 18.2 Pulse 动画细节

```css
@keyframes pulse {
  0%   { transform: scale(0.92); opacity: 0.6; }
  50%  { transform: scale(1.1);  opacity: 1; }
  100% { transform: scale(0.92); opacity: 0.6; }
}
```

Processing 状态使用 1.2s 周期，Reviewing 使用 1.4s 周期（稍慢）。

### 18.3 Shimmer 动画细节

```css
@keyframes shimmer {
  0%   { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}
```

应用于：
- 线程/文件树骨架屏
- 工作树加载状态
- 主页最新活动骨架
- 拖放覆盖文本

### 18.4 面板折叠动画

侧边栏折叠：
```css
.app.sidebar-collapsed .sidebar {
  opacity: 0;
  transform: translateX(-12px);
  pointer-events: none;
  transition: opacity 200ms ease, transform 200ms ease;
}
```

右面板折叠：
```css
.app.right-panel-collapsed .right-panel {
  opacity: 0;
  transform: translateX(12px);
  pointer-events: none;
}
```

### 18.5 工作区/工作树折叠动画

使用 CSS Grid `grid-template-rows` 技巧实现平滑折叠：

```css
.workspace-group-list {
  display: grid;
  grid-template-rows: 1fr;       /* 展开 */
  opacity: 1;
  transition: grid-template-rows 0.2s ease, opacity 0.2s ease, transform 0.2s ease;
}

.workspace-group-list.collapsed {
  grid-template-rows: 0fr;       /* 折叠 */
  opacity: 0;
  transform: translateY(-4px);
}
```

### 18.6 Icon Copy-Check 过渡

复制操作的图标切换（应用于消息气泡、主顶栏按钮）：

```css
/* 初始状态 */
.icon-copy { opacity: 1; transform: scale(1); filter: blur(0); }
.icon-check { opacity: 0; transform: scale(0.82); filter: blur(2px); }

/* 复制完成状态 */
.is-copied .icon-copy { opacity: 0; transform: scale(0.82); filter: blur(2px); }
.is-copied .icon-check { opacity: 1; transform: scale(1); filter: blur(0); }
```

### 18.7 按钮交互过渡

```css
/* Hover */
button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 12px 18px rgba(0, 0, 0, 0.2);
  transition: transform 0.15s ease, box-shadow 0.15s ease, 
              background-color 0.15s ease, filter 0.15s ease;
}

/* Active (按下) */
button:active:not(:disabled) {
  transform: scale(var(--ds-active-scale, 0.97));
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
  transition-duration: 50ms;  /* 按下响应更快 */
}
```

### 18.8 侧边栏和主面板宽度过渡

```css
.app {
  transition: grid-template-columns 220ms ease;
}

.app.is-resizing,
.app.is-resizing .main {
  transition: none;  /* 拖拽时禁用过渡 */
}
```

### 18.9 元素悬停显隐

多项元素使用 "悬停时显示" 模式：
- 线程行上的时间/展开图标切换（opacity 交叉淡入）
- Diff 行上的操作按钮（max-width + opacity + transform 组合）
- 消息气泡上的复制/引用按钮（opacity + transform）
- 文件树行上的操作按钮（opacity + pointer-events）
- 工作区行上的添加/折叠按钮（opacity + pointer-events）

### 18.10 Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
  
  /* 但保留功能性 spinner 的动画 */
  .working-spinner, .git-panel-spinner, .commit-message-loader, ... {
    animation-duration: var(--ds-spinner-dur) !important;
    animation-iteration-count: infinite !important;
  }
}
```

---

## 19. 响应式设计

### 19.1 三个布局模式

| 模式 | CSS 类 | 断点 | 列布局 |
|------|--------|------|--------|
| **Desktop** | `.layout-desktop` (默认) | > 960px | Sidebar + Main + RightPanel |
| **Tablet** | `.layout-tablet` | 720px - 960px | Nav(72px) + Sidebar + Main |
| **Phone** | `.layout-phone` | < 720px | 全屏 + TabBar |

### 19.2 Phone 布局细节

- 侧边栏变为全屏可滚动面板
- 主顶栏添加 `safe-area-inset-top` 适配
- TabBar 显示 5 个导航标签
- 编辑器聚焦时 TabBar 折叠（为键盘腾空间）
- 编辑器 textarea 最小高度减小到 52px，字号增至 16px（防止 iOS 缩放）
- 附件/麦克风/展开按钮隐藏在移动菜单中
- 上下文用量环形图隐藏
- Composer bar 变为水平滚动

### 19.3 Tablet 布局细节

- 新增 72px 宽的侧导航栏（图标式标签）
- 中间保留 Sidebar（可调整宽度）
- 右侧主内容区无右面板

### 19.4 主页响应式

- Desktop: 3 列网格（最新活动 + 用量卡片）
- 900px: 2 列网格
- 640px: 单列 + 紧凑 padding

### 19.5 设置界面响应式

- Screen > 720px: 主从布局（200px 导航 + 内容）
- Screen ≤ 720px: 单列 + 主从导航切换（CSS `display: none`）

---

## 20. 无障碍设计

### 20.1 键盘导航

- 所有可交互元素支持键盘操作（button 元素，focus-visible 样式）
- 拖拽分隔线使用 `role="separator"` + `aria-orientation`
- `aria-label` 用于分隔线和控件说明

### 20.2 屏幕阅读器

- `.visually-hidden` / `.settings-visually-hidden` 类用于隐藏但可被读屏器访问的文本
- 内容层使用 `inert` 属性标记不可交互层
- 隐藏层使用 `aria-hidden="true"`

### 20.3 运动偏好

- `prefers-reduced-motion: reduce` 完全禁用非功能性动画
- 但保留 spinner 动画（功能性，传达加载状态）

### 20.4 颜色方案

- `prefers-color-scheme: light/dark` 通过 System 主题支持
- 所有颜色使用 CSS 变量，确保主题切换平滑

### 20.5 用户选择

- 侧边栏和主顶栏默认禁止文本选择（`user-select: none`）
- 输入框和可编辑内容允许文本选择

---

## 21. 平台适配

### 21.1 Windows 平台

- `.app.is-windows` 自动添加
- 大幅降低表面透明度（配合 Windows 原生窗口玻璃效果）
- 标题栏拖拽区域使用 `-webkit-app-region: no-drag` 覆盖 Tauri 默认行为

### 21.2 macOS 平台

- 使用默认的表面透明度
- 窗口控制按钮由系统提供（Tauri 处理）

### 21.3 Safe Area (移动端)

- 使用 `env(safe-area-inset-top)` 和 `env(safe-area-inset-bottom)` 适配刘海屏
- TabBar padding-bottom 包含 safe-area-inset-bottom
- Phone 布局的侧边栏/顶栏 padding-top 包含 safe-area-inset-top

### 21.4 Reduced Transparency

- 通过 `.app.reduced-transparency` 类启用
- 移除所有 backdrop-filter
- 提升背景不透明度
- 各面板添加明确的边框补偿

---

## 22. 设计风格总结

### 22.1 整体美学

Codex Monitor 的 UI 设计属于 **"Dark Glassmorphism Professional"** 风格：

- **基调**：深邃暗色 + 半透明层次 + 柔和模糊
- **氛围**：专业、现代、技术感、沉浸式
- **灵感来源**：macOS Big Sur 玻璃效果 + VS Code 暗色主题 + Linear 极简设计

### 22.2 核心设计决策

1. **无 CSS 框架依赖**：纯手工 CSS，无 Tailwind/Bootstrap，完全掌控细节
2. **以透明度构建层次**：而非阴影，通过 `color-mix()` 和 RGBA 构建 10+ 级表面层次
3. **圆角语言统一**：
   - 按钮: 8-10px
   - 卡片: 12-18px
   - 输入框: 10-20px
   - 药丸元素: 999px
4. **边框作为微妙分隔**：几乎所有元素都使用 1px 半透明边框
5. **内阴影增添深度**：广泛使用 `inset box-shadow` 创建嵌入感
6. **状态色克制使用**：仅在线程状态、Git 操作等关键位置使用高饱和色
7. **动画服务于功能**：所有动画都短（50ms-220ms），不影响操作效率

### 22.3 设计系统成熟度

- **40+ 独立 CSS 文件**按功能模块组织
- **3 级设计令牌体系**（原始值 → DS 令牌 → CM 应用令牌）
- **4 套主题**（Dark/Dim/Light/System）
- **3 种布局模式**（Desktop/Tablet/Phone）
- **完整的动效系统**（持续时间、缓动函数、缩放系数标准化）
- **统一的组件基元**（Modal/Popover/Toast/Panel/Tooltip）

### 22.4 颜色主题一览

| 主题 | 背景基调 | 文本基调 | 强调色 | 适用场景 |
|------|----------|----------|--------|----------|
| **Dark** | #0a0e14 (蓝黑) | 白色系 | 蓝+绿渐变 | 默认，夜间编程 |
| **Dim** | #292c33 (灰蓝) | 白色系 | 蓝偏亮 | 低对比度偏好 |
| **Light** | #f6f7fa (浅灰) | 深灰系 | 蓝+绿调 | 白天办公 |
| **System** | 跟随系统 | 跟随系统 | 蓝+绿调 | 自适应 |

---

*报告完成日期: 2026-05-04*
*分析方法: 逐行阅读全部 CSS 源码、设计令牌定义、主题文件、布局系统和组件样式*
