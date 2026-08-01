# MisakaX 按钮与菜单 UI 设计规范

**最后审阅 / Last reviewed:** 2026-08-01（v15）

> 主色已切换为 **charcoal**（非冷蓝强调色）。文中若仍出现历史「蓝调」示例，以实现侧 CSS 变量与 [frontend-ui-guidelines.md](./frontend-ui-guidelines.md) 为准。
>
> **现行 shadcn `Button`（权威）**：基类 `rounded-full`；focus `ring-ring/30 ring-[3px]`；主色/destructive 带 `[&_svg]:!text-current`；允许 `active:not-aria-[haspopup]:translate-y-px`（1px 微按压，禁止 scale）。Size：`default` / `xs` / `sm` / `lg` / `icon` / `icon-xs` / `icon-sm`(32px) / `icon-lg`。下文若出现「圆角 10px」等旧桌面稿数值，以本段与 `src/components/ui/button.tsx` 为准。

## 目录

1. [设计令牌速查](#1-设计令牌速查)
2. [基础按钮系统](#2-基础按钮系统)
3. [按钮变体](#3-按钮变体)
4. [图标按钮](#4-图标按钮)
5. [Composer 编辑器按钮](#5-composer-编辑器按钮)
6. [侧边栏按钮](#6-侧边栏按钮)
7. [Diff/文件操作按钮](#7-diff文件操作按钮)
8. [Git 面板按钮](#8-git-面板按钮)
9. [设置面板按钮与控件](#9-设置面板按钮与控件)
10. [弹出菜单系统 (Popover)](#10-弹出菜单系统-popover)
11. [下拉选择控件](#11-下拉选择控件)
11.1 [右键菜单 (ContextMenu)](#111-右键菜单-contextmenu)
12. [开关与分段控件](#12-开关与分段控件)
13. [模态框按钮](#13-模态框按钮)
14. [Toast 提示按钮](#14-toast-提示按钮)
15. [Tooltip 提示系统](#15-tooltip-提示系统)
16. [窗口控制按钮](#16-窗口控制按钮)
17. [动画时间线汇总](#17-动画时间线汇总)

---

## 1. 设计令牌速查

### 1.1 动效令牌

| 令牌 | 值 | 用途 |
|------|-----|------|
| `--ds-dur-fast` | `120ms` | 微交互（hover 进出、颜色切换） |
| `--ds-dur-normal` | `160ms` | 标准过渡（边框、阴影变化） |
| `--ds-dur-slow` | `220ms` | 大型过渡（面板展开、模态框） |
| `--ds-dur-entrance` | `200ms` | 入场动画 |
| `--ds-ease-out` | `cubic-bezier(0.16, 1, 0.3, 1)` | 标准缓出，有轻微超调感 |
| `--ds-ease-out-soft` | `cubic-bezier(0.25, 0.46, 0.45, 0.94)` | 柔和缓出，更平缓 |
| `--ds-ease-spring` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | 弹簧效果，有回弹 |
| `--ds-spinner-dur` | `0.7s` | 旋转动画周期 |
| `--ds-active-scale` | `0.97` | 按下缩小比例 |
| `--ds-active-scale-sm` | `0.92` | 图标按钮按下缩小比例 |

### 1.2 层级令牌

| 令牌 | 值 | 用途 |
|------|-----|------|
| `--ds-layer-modal` | `10000` | 模态框层级 |
| `--ds-layer-toast` | `11000` | Toast 层级 |

### 1.3 常用表面色令牌（Dark 主题）

| 令牌 | 值 | 视觉效果 |
|------|-----|----------|
| `--surface-card` | `rgba(255,255,255,0.04)` | 卡片底色 |
| `--surface-card-strong` | `rgba(255,255,255,0.12)` | 强化卡片 |
| `--surface-control` | `rgba(255,255,255,0.08)` | 控件底色 |
| `--surface-control-hover` | `rgba(255,255,255,0.14)` | 控件悬停 |
| `--surface-hover` | `rgba(255,255,255,0.05)` | 通用悬停 |
| `--surface-active` | `oklch(primary 低透明)` / dark `white/10%` | 选中态（中性 charcoal，非冷蓝） |
| `--surface-popover` | 抬升表面 | 弹出层背景 |

### 1.4 CM 应用层表面令牌

这些令牌使用 `color-mix(in srgb, var(--surface-card) N%, transparent)` 生成半透明层级：

| 令牌 | 透明度 | 用途 |
|------|--------|------|
| `--surface-row` | 44% | 行默认背景 |
| `--surface-panel-soft` | 56% | 柔和面板 |
| `--surface-panel` | 64% | 标准面板 |
| `--surface-panel-strong` | 72% | 强化面板（按钮默认背景） |
| `--surface-panel-elevated` | 78% | 抬高面板（输入框） |
| `--surface-panel-quiet` | 80% | 安静面板 |
| `--surface-panel-loud` | 84% | 响亮面板 |
| `--surface-panel-hover` | 90% | 悬停面板 |
| `--surface-panel-solid` | 92% | 实色面板（hover 加强） |
| `--surface-panel-active` | 96% (基于 surface-active) | 激活面板 |

### 1.5 CM 边框令牌

| 令牌 | 透明度 | 用途 |
|------|--------|------|
| `--border-soft` | 72% | 柔和边框 |
| `--border-default` | 78% | 默认边框 |
| `--border-strong` | 82% | 强化边框 |
| `--border-emphasis` | 84% | 强调边框 |
| `--border-elevated` | 86% | 抬高边框 |
| `--border-heavy` | 88% | 厚重边框 |
| `--border-hover` | 92% | 悬停边框 |
| `--border-contrast` | 100% | 最高对比边框 |
| `--border-accent` | 36% (基于 border-accent) | 强调色边框 |
| `--border-accent-strong` | 46% (基于 border-accent) | 强化强调色边框 |

---

## 2. 基础按钮系统

### 2.1 通用 `<button>` 基础样式

```css
button {
  border: none;
  border-radius: 10px;
  padding: 8px 14px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.15s ease,
              box-shadow 0.15s ease,
              background-color 0.15s ease,
              filter 0.15s ease;
  -webkit-app-region: no-drag;
}
```

**尺寸规格：**
- 内边距：上下 8px，左右 14px
- 圆角：10px
- 字号：13px
- 字重：600

### 2.2 按钮交互状态规范

#### Hover（悬停）状态

```css
button:hover:not(:disabled) {
  background-color: ...; /* keep existing color change */
  box-shadow: none;
  transform: none;
}
```

**行为：**
- 不浮动、不扩展阴影，仅颜色/背景过渡
- 过渡时间：150ms，ease 缓动

#### Active（按下）状态

```css
button:active:not(:disabled) {
  transform: none;
  box-shadow: none;
  transition-duration: 50ms;
}
```

**行为：**
- 不缩放、不收缩阴影，仅颜色/背景过渡
- 过渡时间极短：50ms（按下是一瞬间的操作）
- 松开后恢复，150ms ease

#### Disabled（禁用）状态

```css
button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

**行为：**
- 整体不透明度降为 0.5
- 鼠标变为禁止图标 `not-allowed`
- 不响应 hover/active

---

## 3. 按钮变体

### 3.1 Primary（主按钮）

```css
.primary {
  background: linear-gradient(135deg, #62b7ff, #4fe3a3);
  color: #0b0f1a;
  box-shadow: 0 12px 22px var(--shadow-accent);
}
```

**Normal 状态：**
| 属性 | 值 |
|------|-----|
| 背景 | `linear-gradient(135deg, #62b7ff, #4fe3a3)` — 蓝→绿 135° 渐变 |
| 文字色 | `#0b0f1a` — 极深蓝黑色 |
| 阴影 | `0 12px 22px` 使用中性 / primary 低透明阴影（**禁止**蓝调 `rgba(92,168,255,*)`） |
| 圆角 | 10px |
| 内边距 | 8px 14px |
| 字号 | 13px |
| 字重 | 600 |

**Hover 状态：**
- 继承基础 hover：`transform: none; box-shadow: none`，仅颜色/背景过渡
- 基础 hover/active 无位移、无缩放，仅颜色变化

**Active 状态：**
```css
.primary:active:not(:disabled) {
  filter: brightness(0.92);  /* 变暗 8% */
}
```
- 继承基础 active：`transform: none; box-shadow: none` + 50ms
- 额外降低亮度至 92%（使渐变颜色稍微变暗）

**完整交互序列：**
```
Normal:   scale(1),   shadow=0 12px 22px accent,  brightness(1)
 ↓ hover
Hover:    transform=none, shadow=none, brightness(1)
 ↓ press
Active:   transform=none, shadow=none, brightness(0.92)  [50ms]
 ↓ release
Hover:    transform=none, shadow=none, brightness(1)  [150ms]
```

### 3.2 Secondary（次要按钮）

```css
.secondary {
  background: var(--surface-card-strong);  /* rgba(255,255,255,0.12) */
  color: inherit;
}
```

**Normal 状态：**
| 属性 | 值 |
|------|-----|
| 背景 | `var(--surface-card-strong)` — 半透明白色 12% |
| 文字色 | `inherit` — 继承父级（通常为 `--text-primary`） |
| 阴影 | 无 |
| 圆角 | 10px |

**Hover 状态：**
- 继承基础 hover：`transform: none; box-shadow: none`

**Active 状态：**
- 继承基础 active：`transform: none; box-shadow: none` + 50ms

### 3.3 Ghost（幽灵按钮）

```css
.ghost {
  background: transparent;
  color: var(--text-muted);              /* rgba(255,255,255,0.7) */
  border: 1px solid var(--border-strong); /* rgba(255,255,255,0.14) */
}
```

**Normal 状态：**
| 属性 | 值 |
|------|-----|
| 背景 | `transparent` |
| 文字色 | `--text-muted` — 70% 白色 |
| 边框 | `1px solid --border-strong` — 14% 白色 |
| 圆角 | 10px |

**Hover 状态：**
- 继承基础 hover：`transform: none; box-shadow: none`

**Active 状态：**
```css
.ghost:active:not(:disabled) {
  background: rgba(255, 255, 255, 0.06);
}
```
- 继承基础 active：`transform: none; box-shadow: none` + 50ms
- 额外添加微弱白色背景 `rgba(255,255,255,0.06)`

---

## 4. 图标按钮

### 4.1 通用图标按钮 (`.icon-button`)

```css
.icon-button {
  padding: 6px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.icon-button svg {
  width: 16px;
  height: 16px;
}
```

**尺寸规格：**
| 属性 | 值 |
|------|-----|
| 内边距 | 6px（四周相等） |
| 图标尺寸 | 16×16px |
| 布局 | `inline-flex` + 居中 |
| 圆角 | 10px（继承基础 button） |

**Active 状态（与普通按钮不同）：**
```css
.icon-button:active:not(:disabled) {
  transform: scale(var(--ds-active-scale-sm));  /* 0.92 — 缩小更多 */
}
```
- 缩小至 92%（比普通按钮的 97% 更多）
- 不改变阴影
- 过渡 50ms

### 4.2 标题栏图标按钮 (`.main-header-action`)

```css
.titlebar-toggle .main-header-action {
  width: 28px;           /* var(--titlebar-toggle-size) */
  height: 28px;
  padding: 0;
  border-radius: 8px;
  transition: background-color 120ms ease, color 120ms ease, opacity 120ms ease;
}
```

**尺寸：** 28×28px 正方形
**圆角：** 8px
**内边距：** 0

**Hover 状态：**
- `background: var(--surface-hover)` — 微弱白色背景
- `color: var(--text-strong)` — 文字/图标变亮
- 无 transform（`transform: none`）
- 无 shadow（`box-shadow: none`）
- 过渡 120ms ease

### 4.3 侧边栏标题按钮 (`.sidebar-title-button`)

```css
.sidebar-title-button {
  min-width: 0;
  margin-left: -6px;
  padding: 4px 6px;
  border-radius: 8px;
  transition: background-color 120ms ease, color 120ms ease;
}
```

**尺寸：**
- 内边距：4px 上下，6px 左右
- 圆角：8px
- 背景：transparent

**Hover 状态：**
```css
.sidebar-title-button:hover,
.sidebar-title-button:focus-visible {
  background: var(--surface-hover);  /* rgba(255,255,255,0.05) */
  transform: none;                    /* 不浮动 */
  box-shadow: none;                   /* 无阴影 */
}
```

**关键区别：** 此按钮明确禁止基础 button 的 hover 浮动和阴影效果。

### 4.4 添加按钮 (`.sidebar-title-add`)

```css
.sidebar-title-add {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--text-muted);           /* rgba(255,255,255,0.7) */
  line-height: 1;
  border-radius: 8px;
  box-shadow: none;
  transform: none;
  transition: background-color 120ms ease, color 120ms ease;
  flex-shrink: 0;
}

.sidebar-title-add svg {
  width: 16px;
  height: 16px;
}
```

**尺寸：** 30×30px 正方形
**圆角：** 8px
**图标：** 16×16px

**Hover 状态：**
```css
.sidebar-title-add:hover,
.sidebar-title-add:focus-visible {
  color: var(--text-strong);          /* 纯白 */
  background: var(--surface-hover);   /* 微弱白色背景 */
  box-shadow: none;
  transform: none;
  border-radius: 8px;                 /* 保持圆角不变 */
}
```

**注意：** hover 时明确设置 border-radius: 8px，覆盖基础 button 的 10px。

### 4.5 搜索/排序/刷新图标按钮 (`.sidebar-search-toggle`, `.sidebar-sort-toggle`, `.sidebar-refresh-toggle`)

```css
.sidebar-search-toggle,
.sidebar-sort-toggle,
.sidebar-refresh-toggle {
  width: 32px;
  height: 32px;
  padding: 0;
  border-radius: 8px;
  border: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

/* 排序/刷新图标 */
.sidebar-sort-toggle svg,
.sidebar-refresh-toggle svg,
.sidebar-search-toggle svg {
  width: 14px;
  height: 14px;
}
```

**尺寸：** 32×32px 正方形
**圆角：** 8px
**图标尺寸：** 14×14px
**颜色：** `--text-muted` (70% 白色)

**Hover 状态：**
```css
.sidebar-search-toggle:hover,
.sidebar-sort-toggle:hover,
.sidebar-refresh-toggle:hover {
  color: var(--text-strong);          /* 纯白 */
  background: var(--surface-hover);   /* 微弱白色背景 */
  transform: none;
  box-shadow: none;
}
```

**激活状态（搜索按钮特有）：**
```css
.sidebar-search-toggle.is-active {
  color: var(--text-strong);
  background: var(--surface-hover);
  box-shadow: inset 0 0 0 1px var(--border-quiet);  /* 内嵌边框 */
}
```

**刷新旋转动画：**
```css
.sidebar-refresh-icon.spinning {
  animation: sidebar-refresh-spin 0.7s linear infinite;
}

@keyframes sidebar-refresh-spin {
  to { transform: rotate(360deg); }
}
```

**禁用状态（刷新按钮）：**
```css
.sidebar-refresh-toggle:disabled {
  opacity: 0.55;
}
```

### 4.6 工作区添加按钮 (`.workspace-add`)

```css
.workspace-add {
  width: 22px;
  height: 22px;
  border-radius: 999px;                    /* 正圆形 */
  border: 1px solid var(--border-stronger); /* rgba(255,255,255,0.18) */
  background: var(--surface-panel-loud); /* surface-card 84% */
  color: var(--text-muted);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  line-height: 1;
  -webkit-app-region: no-drag;
  flex-shrink: 0;
  opacity: 0.46;                          /* 默认半透明 ! */
  transition: opacity 0.15s ease,
              background-color 0.15s ease,
              color 0.15s ease;
}
```

**尺寸：** 22×22px 正圆
**关键设计：** 默认 `opacity: 0.46`，在行 hover 时才完全显示

**行 hover 时显现：**
```css
.workspace-row:hover .workspace-add {
  opacity: 1;
}
```

**自身 hover 时：**
```css
.workspace-add:hover {
  color: var(--text-strong);
  background: var(--surface-card-strong);  /* 背景加强 */
}
```

### 4.7 All Threads 添加按钮 (`.all-threads-add`)

```css
.all-threads-add {
  width: 24px;
  height: 24px;
  border-radius: 999px;                    /* 正圆形 */
  border: 1px solid var(--border-stronger);
  background: var(--surface-panel-loud);
  color: var(--text-muted);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  opacity: 0.62;                          /* 默认 62% 不透明度 */
  transition: opacity 0.15s ease,
              background-color 0.15s ease,
              color 0.15s ease;
}

.all-threads-add svg {
  width: 14px;
  height: 14px;
}
```

**尺寸：** 24×24px 正圆
**图标：** 14×14px
**默认不透明度：** 0.62

**容器 hover 时：**
```css
.workspace-group-header-all-threads:hover .all-threads-add {
  opacity: 1;
}
```

**自身 hover 时：**
```css
.all-threads-add:hover {
  color: var(--text-strong);
  background: var(--surface-card-strong);
}
```

### 4.8 WorkspaceBar Panel 与工具日志按钮

- Explorer/Terminal 使用共享 `BarIconButton`：`ghost size-7`，仅当右栏已打开且 mode 匹配时切为 `secondary`；同 mode 再按一次关闭，另一 mode 切换。禁止用 disabled 表示已打开。
- 两个按钮都必须有准确 `aria-label`、`aria-pressed`、`aria-keyshortcuts` 和 Tooltip；Explorer 显示 `Ctrl/Cmd+Shift+E`，Terminal 显示 `Ctrl/Cmd+反引号`。图标选中态可提高 stroke，不增加缩放、位移或彩色发光。
- Terminal feature 未完成或 rollout flag 关闭时不渲染该按钮；不得显示可点击的“即将推出”空动作。
- Tool Logs 不再占 WorkspaceBar Terminal 图标。消息工具组的日志入口使用 `ScrollText` 语义图标、`ghost icon-xs/size-7`、明确“打开工具日志”名称；它是独立 button，不能嵌套进 Collapsible trigger，点击不得顺带改变消息工具组展开态。

---

## 5. Composer 编辑器按钮

### 5.0 工作区上下文 badge（非按钮）

- `WorkspaceContextBadge` 使用紧凑状态表面（`h-6 px-2 rounded-full text-[11px]`），但语义上是只读文本，禁止套用 `Button`、`aria-pressed`、chevron 或 hover/active 位移。
- Tooltip 只补充完整 branch/detached 文案或脱敏诊断；不得显示绝对 cwd、Git stderr、PATH 或可点击命令。错误 ID 只显示短 correlation ID。
- 未来 `WorkspaceContextAction` 若真的有 provider，再按本规范新增独立按钮/menu；没有 action 时不得渲染空 trigger。窄宽度隐藏 badge 文本时必须保留图标、`aria-label` 与 tooltip。

### 5.1 操作按钮 (`.composer-action`)

```css
.composer-action {
  border: 1px solid var(--border-emphasis);   /* border-subtle 84% */
  background: var(--surface-panel-strong);     /* surface-card 72% */
  color: var(--text-strong);                      /* 纯白 */
  padding: 0;
  border-radius: 999px;                           /* 正圆形 */
  font-size: 12px;
  cursor: pointer;
  width: 30px;
  height: 30px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.composer-action svg {
  width: 12px;
  height: 12px;
}
```

**尺寸规格：**

| 属性 | 规格 |
|------|------|
| 宽高 | **32×32px**（`size-8` / Button `icon-sm`） |
| 形状 | 正圆 `rounded-full` |
| 图标 | 约 14–16px（`size-3.5`–`size-4`） |
| 附件入口 | 外置左侧圆形 `+`，与发送同尺寸、同行 `items-center` |
| 发送 | 可发时 `variant="default"` / `bg-primary text-primary-foreground`；hover 须保持浅色图标（`hover:text-primary-foreground`）；**禁止**可发态用 `ghost`（会 hover 成 `accent-foreground` 近黑） |
| 停止 | 既有停止色边框/底，图标 `Square` |

> 历史稿中的 30×30 已废弃；Composer 附件与发送必须同为 32px，避免视觉阶梯错位。

**Light 主题变体：**
```css
:root[data-theme="light"] .composer-action {
  border-color: var(--border-strong);
  color: var(--text-strong);
}
```

**Hover 状态：**
```css
.composer-action:hover {
  background: var(--surface-panel-solid);   /* surface-card 92% — 更实 */
  color: var(--text-strong);                    /* 保持白色 */
}
```

**禁用状态：**
```css
.composer-action:disabled,
.composer-action.is-disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background: var(--surface-control-disabled);  /* white 5% */
  color: var(--text-fainter);                   /* white 45% */
  border-color: var(--border-subtle);           /* white 8% */
}
```

### 5.2 麦克风按钮 (`.composer-action--mic`)

**激活状态（正在录音）：**
```css
.composer-action--mic.is-active {
  border-color: rgba(120, 235, 190, 0.6);    /* 绿色边框 */
  background: rgba(120, 235, 190, 0.12);      /* 绿色背景 */
}

.composer-action--mic.is-active:hover {
  background: rgba(120, 235, 190, 0.18);      /* hover 加深绿色 */
}
```

**处理中状态（识别中）：**
```css
.composer-action--mic.is-processing {
  border-color: rgba(160, 200, 255, 0.6);     /* 蓝色边框 */
  background: rgba(160, 200, 255, 0.12);       /* 蓝色背景 */
}
```

### 5.3 停止按钮 (`.composer-action.is-stop`)

```css
.composer-action.is-stop {
  border-color: rgba(255, 107, 107, 0.6);      /* 红色边框 */
  background: rgba(255, 107, 107, 0.12);        /* 红色背景 */
}

.composer-action.is-stop:hover {
  background: rgba(255, 107, 107, 0.2);         /* hover 加深红色 */
}
```

**停止按钮内的方形图标：**
```css
.composer-action-stop-square {
  width: 6px;
  height: 6px;
  border-radius: 2px;
  background: currentColor;
}
```

**停止按钮禁用态：**
```css
.composer-action.is-stop:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  background: rgba(255, 107, 107, 0.08);
  color: rgba(255, 196, 196, 0.4);
}
```

### 5.4 附件按钮 (`.composer-attach`)

```css
.composer-attach {
  border: 1px solid var(--border-strong);    /* border-subtle 82% */
  background: var(--surface-panel-solid);    /* surface-card 92% */
  color: var(--text-muted);
  padding: 0;
  border-radius: 999px;                         /* 正圆形 */
  cursor: pointer;
  width: 32px;
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
```

**尺寸：** 对话 Composer 附件入口为 **32×32px** 正圆（与发送同阶，见 §5.1），图标为 `Plus`（约 `size-3.5`）；按钮放在输入框容器外部左侧，不放在 textarea 内部。历史稿 36×36 已废弃。

**Hover 状态：**
```css
.composer-attach:hover {
  background: var(--surface-panel-solid);   /* 保持 */
  color: var(--text-strong);                    /* 文字变亮 */
}
```

**禁用状态：**
```css
.composer-attach:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background: var(--surface-control-disabled);  /* white 5% */
  color: var(--text-fainter);                   /* white 45% */
  border-color: var(--border-subtle);           /* white 8% */
}
```

### 5.5 上下文环形图 (`.composer-context-ring`)

### 5.5 Composer 底部触发器（Model / MCP / Skill）

- Composer 底部可并列展示模型选择、MCP 工具视图与 Skill 选择扩展口，三者均使用紧凑胶囊 trigger。
- Trigger 背景使用 `--surface-panel-strong`，hover 使用 `--surface-panel-solid`，仅做颜色/背景变化，不使用位移或缩放。
- MCP trigger 只用于查看当前连接的 server 与 tool 列表，不直接执行工具调用；Skill trigger 在模块未完成时显示占位，但保留多选状态接口。
- Trigger 文案需支持截断，最大宽度约 120px，数量角标可内联显示为 `MCP (3)` 形式。

### 5.6 上下文环形图 (`.composer-context-ring`)

虽然不算按钮，但是一个重要的交互元素：

```css
.composer-context-ring {
  --context-free: 0;
  width: 20px;
  height: 20px;
  border-radius: 999px;
  display: grid;
  place-items: center;
  position: relative;
  background:
    radial-gradient(circle, var(--surface-context-core) 54%, transparent 56%),
    conic-gradient(
      from 180deg,
      hsl(calc(120deg * var(--context-free) / 100), 80%, 55%)
        calc(var(--context-free) * 1%),
      var(--border-strong) 0
    );
  transition: background 0.2s ease;
}
```

**结构：** 20×20px 正圆
**实现：** `radial-gradient` 挖空中心（54%半径处是核心色，56%处变透明），`conic-gradient` 绘制环形进度条
**颜色：** HSL 动态色相，从绿色 (120°) 到红色 (0°)
**过渡：** background 0.2s ease

**Hover 弹出 tooltip：**
```css
.composer-context-ring::after {
  content: attr(data-tooltip);
  position: absolute;
  right: 0;
  bottom: calc(100% + 6px);
  transform: translateY(4px);
  padding: 4px 8px;
  border-radius: 999px;
  background: var(--surface-command);
  color: var(--text-emphasis);
  font-size: 10px;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.15s ease, transform 0.15s ease;
  border: 1px solid var(--border-subtle);
}

.composer-context-ring:hover::after {
  opacity: 1;
  transform: translateY(0);
}
```

---

## 6. 侧边栏按钮

### 6.1 标签按钮 (`.sidebar-labeled-button`)

这是侧边栏底部的多功能按钮（账户、设置等）：

```css
.sidebar-labeled-button {
  width: 100%;
  height: 34px;
  padding: 4px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-quiet);            /* white 20% */
  background: color-mix(in srgb, var(--surface-hover) 82%, transparent);
  color: var(--text-muted);
  display: inline-flex;
  align-items: center;
  gap: 8px;
  white-space: nowrap;
  transition: border-color 120ms ease,
              background-color 120ms ease,
              color 120ms ease;
}

.sidebar-labeled-button span:last-child {
  font-size: 11px;
  font-weight: 600;
  line-height: 1;
}
```

**尺寸规格：**

| 属性 | 值 |
|------|-----|
| 高度 | 34px |
| 宽度 | 100%（填满容器） |
| 内边距 | 4px 上下，10px 左右 |
| 圆角 | 10px |
| 边框 | `1px solid --border-quiet` |
| 文字 | 11px，字重 600 |
| 图标间距 | 8px |

**图标容器：**
```css
.sidebar-labeled-button-icon {
  width: 20px;
  height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
```

**Hover / 打开状态：**
```css
.sidebar-labeled-button:hover,
.sidebar-labeled-button:focus-visible,
.sidebar-labeled-button.is-open {
  color: var(--text-stronger);           /* white 85% */
  border-color: var(--border-subtle);    /* 边框变淡 */
  background: var(--surface-hover);      /* 背景变化 */
  box-shadow: none;                       /* 无阴影 */
  transform: none;                        /* 无浮动 */
}
```

**关键设计：** 此按钮明确抑制了基础 button 的 hover 浮动和阴影效果。

### 6.2 账户头像 (`.sidebar-account-avatar`)

```css
.sidebar-account-avatar {
  width: 20px;
  height: 20px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--text-stronger);
  font-size: 10px;
  font-weight: 700;
  background: linear-gradient(135deg,
    rgba(120, 235, 190, 0.28),
    rgba(100, 200, 255, 0.26)
  );
}
```

**尺寸：** 20×20px 正圆
**背景：** 绿→蓝 135° 渐变

### 6.3 账户弹出面板 (`.sidebar-account-popover`)

```css
.sidebar-account-popover {
  position: absolute;
  left: 0;
  bottom: calc(100% + 8px);             /* 按钮上方 8px */
  min-width: 220px;
  padding: 10px 12px;
  display: grid;
  gap: 8px;
  z-index: 10;
}
```

**账户操作按钮：**
```css
.sidebar-account-action {
  width: 100%;
  justify-content: center;
  font-size: 11px;
}
```

---

## 7. Diff/文件操作按钮

### 7.1 Diff 行操作按钮 (`.diff-row-action`)

```css
.diff-row-action {
  width: 22px;
  height: 22px;
  border-radius: 999px;                      /* 正圆 */
  padding: 0;
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-faint);                  /* white 50% */
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 160ms ease,
              border-color 160ms ease,
              color 160ms ease;
  position: relative;
}
```

**尺寸：** 22×22px 正圆
**初始状态：** 完全透明，边框透明，颜色很淡

**行 hover 时显现：**
```css
.diff-row:hover .diff-row-action,
.diff-row:focus-within .diff-row-action {
  color: var(--text-muted);                  /* 稍微可见 */
}
```

**自身 hover 时：**
```css
.diff-row-action:hover {
  background: var(--surface-control-hover);  /* white 14% */
  border-color: var(--border-subtle);        /* white 8% */
  color: var(--text-emphasis);               /* white 90% */
  transform: none;                            /* 不浮动 */
  box-shadow: none;                           /* 无阴影 */
}
```

**Focus 可见：**
```css
.diff-row-action:focus-visible {
  outline: 2px solid var(--border-accent-soft);
  outline-offset: 2px;
}
```

### 7.2 颜色编码的操作按钮

```css
/* Stage（暂存）— 绿色 */
.diff-row-action--stage:hover {
  background: rgba(71, 212, 136, 0.14);
  border-color: rgba(71, 212, 136, 0.35);
  color: #47d488;
}

/* Unstage（取消暂存）— 金色 */
.diff-row-action--unstage:hover {
  background: rgba(245, 195, 99, 0.14);
  border-color: rgba(245, 195, 99, 0.35);
  color: #f5c363;
}

/* Discard（丢弃）— 红色 */
.diff-row-action--discard:hover {
  background: rgba(255, 107, 107, 0.14);
  border-color: rgba(255, 107, 107, 0.35);
  color: #ff6b6b;
}

/* Review（审查）/ Apply（应用）— 蓝色 */
.diff-row-action--review:hover,
.diff-row-action--apply:hover {
  background: rgba(90, 169, 255, 0.14);
  border-color: rgba(90, 169, 255, 0.35);
  color: #5aa9ff;
}
```

**颜色方案总结：**

| 操作 | 背景 (hover) | 边框 (hover) | 文字色 (hover) |
|------|-------------|-------------|---------------|
| Stage | `rgba(71,212,136,0.14)` | `rgba(71,212,136,0.35)` | `#47d488` |
| Unstage | `rgba(245,195,99,0.14)` | `rgba(245,195,99,0.35)` | `#f5c363` |
| Discard | `rgba(255,107,107,0.14)` | `rgba(255,107,107,0.35)` | `#ff6b6b` |
| Review/Apply | `rgba(90,169,255,0.14)` | `rgba(90,169,255,0.35)` | `#5aa9ff` |

### 7.3 操作按钮的滑入动画

整个操作按钮组在行 hover 时从右侧滑入：

```css
.diff-row-actions {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  max-width: 0;                    /* 初始宽度为 0 */
  overflow: hidden;
  margin-left: 0;
  opacity: 0;
  pointer-events: none;
  transform: translateX(6px);      /* 初始右移 6px */
  transition: max-width 180ms ease,
              opacity 140ms ease,
              transform 140ms ease,
              margin-left 180ms ease;
}

.diff-row:hover .diff-row-actions,
.diff-row:focus-within .diff-row-actions {
  max-width: 88px;                 /* 展开至 88px */
  margin-left: 4px;
  opacity: 1;
  pointer-events: auto;
  overflow: visible;
  transform: translateX(0);        /* 归位 */
}
```

---

## 8. Git 面板按钮

### 8.1 Git Root 按钮 (`.git-root-button`)

```css
.git-root-button {
  padding: 7px 11px;
  font-size: 12px;
  border-radius: 999px;                       /* 胶囊形 */
  border: 1px solid var(--border-default);
  background: var(--surface-panel);        /* surface-card 64% */
  color: var(--text-emphasis);                /* white 90% */
  box-shadow: none;
  transform: none;
  transition: background 160ms ease,
              border-color 160ms ease,
              color 160ms ease;
}
```

**Hover 状态：**
```css
.git-root-button:hover:not(:disabled),
.git-root-button:focus-visible {
  background: var(--surface-panel-hover);   /* surface-card 90% */
  border-color: var(--border-hover);        /* border-subtle 92% */
  box-shadow: none;
  transform: none;
  outline: none;
}
```

**Primary 变体：**
```css
.git-root-button.primary {
  background: color-mix(in srgb,
    var(--surface-panel-hover) 72%,
    var(--surface-active) 28%
  );                                           /* 混合 hover 面板 + 蓝调 */
  border-color: var(--border-accent-strong);
  color: var(--text-strong);
}

.git-root-button.primary:hover:not(:disabled) {
  background: color-mix(in srgb,
    var(--surface-panel-hover) 58%,
    var(--surface-active) 42%
  );                                           /* 蓝调比例增加 */
  border-color: var(--border-accent-strong);
}
```

**带图标的变体：**
```css
.git-root-button--icon {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.git-root-button-icon {
  width: 12px;
  height: 12px;
}
```

### 8.2 Commit 按钮 (`.commit-button`)

```css
.commit-button {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 14px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-emphasis);
  background: var(--surface-panel);
  border: 1px solid var(--border-default);
  border-radius: 14px;
  cursor: pointer;
  box-shadow: none;
  transform: none;
  transition: background 160ms ease,
              border-color 160ms ease,
              color 160ms ease;
}
```

**Hover 状态：**
```css
.commit-button:hover:not(:disabled) {
  background: var(--surface-panel-hover);
  border-color: var(--border-accent-strong);  /* 边框变蓝 */
  color: var(--text-strong);
  box-shadow: none;
  transform: none;
}
```

**禁用状态：**
```css
.commit-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

### 8.3 Commit Message 生成按钮 (`.commit-message-generate-button`)

```css
.commit-message-generate-button {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 999px;
  border: 1px solid var(--border-soft);
  background: var(--surface-panel-soft);
  color: var(--text-muted);
  cursor: pointer;
  transition: background 160ms ease,
              border-color 160ms ease,
              color 160ms ease;
}

.commit-message-generate-button svg {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}
```

**尺寸：** 26×26px 正圆
**图标：** 14×14px
**位置：** 绝对定位在输入框右上角（top: 8px, right: 8px）

**Hover 状态：**
```css
.commit-message-generate-button:hover:not(:disabled) {
  background: var(--surface-panel-hover);
  border-color: var(--border-hover);
  color: var(--text-emphasis);
}
```

### 8.4 Push 按钮 (`.push-button`)

```css
.push-button {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-emphasis);
  background: var(--surface-row);           /* surface-card 44% */
  border: 1px solid var(--border-default);
  border-radius: 14px;
  cursor: pointer;
  box-shadow: none;
  transform: none;
  transition: background 160ms ease,
              border-color 160ms ease,
              color 160ms ease;
}
```

**Hover 状态：**
```css
.push-button:hover:not(:disabled) {
  background: var(--surface-panel-hover);
  border-color: var(--border-accent-strong);
  color: var(--text-strong);
  box-shadow: none;
  transform: none;
}
```

### 8.5 分支刷新按钮 (`.diff-branch-refresh`)

```css
.diff-branch-refresh {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 999px;
  border: 1px solid var(--border-default);
  background: var(--surface-panel);
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
  box-shadow: none;
  transform: none;
  transition: background 160ms ease,
              border-color 160ms ease,
              color 160ms ease;
}
```

**尺寸：** 28×28px 正圆

**Hover 状态：**
```css
.diff-branch-refresh:hover:not(:disabled) {
  background: var(--surface-panel-hover);
  border-color: var(--border-hover);
  color: var(--text-strong);
  box-shadow: none;
  transform: none;
}
```

**禁用状态：**
```css
.diff-branch-refresh:disabled {
  opacity: 0.6;
  cursor: default;
}
```

---

## 9. 设置面板按钮与控件

### 9.1 设置导航项 (`.settings-nav-item` / `.ds-panel-nav-item`)

设置导航顺序固定为 `General → Models → MCP → Skills → Appearance → About`。Skills 与其他项使用相同 16px Lucide 图标、`h-9` 命中区和焦点环；当前项必须暴露 `aria-current="page"`，不能只靠背景色表达选中。

```css
.ds-panel-nav-item {
  width: 100%;
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-muted);
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  text-align: left;
  cursor: pointer;
}
```

**图标：** 16×16px（在 `.ds-panel-nav-item-icon` 中）
**箭头图标：** 16×16px（在 `.ds-panel-nav-item-disclosure` 中）

**Hover / Active 状态：**
```css
.ds-panel-nav-item:hover:not(:disabled),
.ds-panel-nav-item:focus-visible,
.ds-panel-nav-item.is-active {
  background: var(--surface-card);
  border-color: var(--border-strong);
  color: var(--text-strong);
}
```

### 9.2 设置图标按钮 (`.settings-icon-button`)

```css
.settings-icon-button {
  width: 28px;
  height: 28px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
}

.settings-icon-button svg {
  width: 14px;
  height: 14px;
}
```

**尺寸：** 28×28px
**圆角：** 8px
**图标：** 14×14px

### 9.3 步进器按钮 (`.settings-agents-stepper-button`)

```css
.settings-agents-stepper-button {
  width: 28px;
  height: 28px;
  padding: 0;
  line-height: 1;
  font-size: 14px;
  font-weight: 700;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
```

**尺寸：** 28×28px

### 9.4 紧凑按钮 (`.settings-button-compact`)

```css
.settings-button-compact {
  padding: 6px 10px;
  font-size: 12px;
}
```

### 9.5 输入框控件

```css
.settings-input {
  flex: 1;
  min-width: 0;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--border-muted);
  background: var(--surface-control);
  color: var(--text-strong);
  font-size: 12px;
  transition: border-color 160ms cubic-bezier(0.25, 0.46, 0.45, 0.94),
              box-shadow 160ms cubic-bezier(0.25, 0.46, 0.45, 0.94);
}
```

**Focus 状态：** 边框和阴影变化，无默认 outline

### 9.6 选择控件 (`.settings-select`)

```css
.settings-select {
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--border-muted);
  background: var(--surface-control);
  color: var(--text-strong);
  font-size: 12px;
  transition: border-color 160ms cubic-bezier(0.25, 0.46, 0.45, 0.94),
              box-shadow 160ms cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.settings-select option {
  background-color: var(--surface-popover);
  color: var(--text-strong);
}
```

---

## 10. 弹出菜单系统 (Popover)

### 10.0 实施约定（权威）

- 动效依赖：`tw-animate-css`（见 [frontend-ui-guidelines.md §2.3](./frontend-ui-guidelines.md)）。
- Radix `DropdownMenuContent` / `SelectContent` / `TooltipContent` 统一使用 `OVERLAY_MOTION` + `OVERLAY_SIDE_SLIDE`（`duration-150` + `ease-out` + fade 入/出 + `slide-in-from-*-1`，位移 ≤4px）。
- `ContextMenuContent` 仅用 `OVERLAY_MOTION`（fade，不加侧向滑入），避免右键菜单位移感过强。
- `DialogOverlay` / `DialogContent` 使用 `DIALOG_MOTION`（`--ds-dur-entrance`）。
- **禁止** `zoom-in*` / `zoom-out*`；**禁止** `slide-in-from-*-2` 及以上。
- 自定义非 Portal 面板关闭时必须走 `usePresence`，保证 `data-state=closed` 的 `animate-out` 播完再卸载。

### 10.1 Popover 容器 (`.ds-popover`)

```css
.ds-popover {
  background: var(--surface-popover);      /* rgba(10,14,20,0.995) — 几乎不透明 */
  border: 1px solid var(--border-muted);   /* rgba(255,255,255,0.06) */
  box-shadow: 0 14px 34px rgba(0, 0, 0, 0.3);  /* 深阴影 */
  border-radius: 10px;
  /* 现行实现：tw-animate fade + ≤4px slide；禁止 scale/zoom */
}
```

**入场 / 退场（与 `OVERLAY_MOTION` 对齐）：**
- 打开：`animate-in fade-in-0` + 对应方向 `slide-in-from-*-1`，`duration-150` `ease-out`
- 关闭：`animate-out fade-out-0`（自定义面板可加对称 `slide-out-to-*-1`）

### 10.2 Popover 菜单项 (`.ds-popover-item`)

```css
.ds-popover-item {
  width: 100%;
  border: none;
  background: transparent;
  color: var(--text-muted);              /* white 70% */
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 8px;
  font-size: 12px;
  text-align: left;
  cursor: pointer;
  transition: background-color 120ms cubic-bezier(0.25, 0.46, 0.45, 0.94),
              color 120ms cubic-bezier(0.25, 0.46, 0.45, 0.94);
}
```

**菜单项图标：**
```css
.ds-popover-item-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.ds-popover-item-icon svg,
.ds-popover-item-icon img {
  width: 14px;
  height: 14px;
  display: block;
}
```

**Hover / Active 状态：**
```css
.ds-popover-item:hover:not(:disabled),
.ds-popover-item:focus-visible,
.ds-popover-item.is-active {
  background: var(--surface-hover);          /* white 5% */
  color: var(--text-stronger);               /* white 85% */
  transform: none;                            /* 禁止浮动 */
  box-shadow: none;                           /* 禁止阴影 */
}
```

**禁用状态：**
```css
.ds-popover-item:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
```

### 10.3 侧边栏排序下拉 (`.sidebar-sort-dropdown`)

```css
.sidebar-sort-dropdown {
  position: absolute;
  top: calc(100% + 8px);                   /* 触发按钮下方 8px */
  right: 0;
  width: 196px;
  max-width: calc(100vw - 24px);
  border-radius: 10px;
  padding: 6px;
  max-height: min(72vh, 420px);
  overflow-y: auto;
  z-index: 14;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
```

**继承：** `.ds-popover` 的背景、边框、阴影（通过组件应用）

**分组标题：**
```css
.sidebar-sort-section-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-faint);                /* white 50% */
  padding: 4px 8px 2px;
}
```

**分隔线：**
```css
.sidebar-sort-divider {
  height: 1px;
  margin: 4px 6px;
  background: var(--border-subtle);        /* white 8% */
}
```

**排序选项项：**
```css
.sidebar-sort-option {
  width: 100%;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text-muted);
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  gap: 8px;
  padding: 6px 8px;
  font-size: 12px;
  text-align: left;
}

.sidebar-sort-option svg {
  width: 13px;
  height: 13px;
  flex-shrink: 0;
}

.sidebar-sort-option:hover,
.sidebar-sort-option:focus-visible,
.sidebar-sort-option.is-active {
  color: var(--text-strong);
  background: var(--surface-hover);
  transform: none;
  box-shadow: none;
}
```

### 10.4 工作区添加菜单 (`.workspace-add-menu`)

```css
.workspace-add-menu {
  position: fixed;
  isolation: isolate;
  border-radius: 10px;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 160px;
  z-index: 9999;                           /* 极高层级 */
}

.workspace-add-option {
  border: none;
  background: transparent;
  color: var(--text-strong);
  font-size: 12px;
  text-align: left;
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
}

.workspace-add-option:hover {
  background: var(--surface-hover);
}
```

### 10.5 Composer 队列项弹出菜单

```css
.composer-queue-item-popover {
  position: absolute;
  right: 0;
  bottom: calc(100% + 4px);
  min-width: 110px;
  padding: 4px;
  z-index: 40;
}
```

### 10.6 Composer 移动端操作菜单

```css
.composer-mobile-actions-popover {
  position: absolute;
  left: 0;
  bottom: calc(100% + 8px);
  min-width: 170px;
  padding: 6px;
  display: grid;
  gap: 4px;
  z-index: 30;
}
```

---

## 11. 下拉选择控件

### 11.1 Composer 选择框包装器 (`.composer-select-wrap`)

```css
.composer-select-wrap {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 999px;                      /* 胶囊形 */
  background: var(--surface-panel-strong); /* surface-card 72% */
  width: max-content;
}
```

**自定义下拉箭头（CSS 三角形）：**
```css
.composer-select-wrap:has(.composer-select)::after {
  content: "";
  position: absolute;
  right: 10px;
  top: 50%;
  width: 7px;
  height: 7px;
  border-right: 1.5px solid var(--select-caret);
  border-bottom: 1.5px solid var(--select-caret);
  transform: translateY(-62%) rotate(45deg);  /* 旋转 45° 形成箭头 */
  pointer-events: none;
  opacity: 0.9;
}
```

**Focus 时箭头变亮：**
```css
.composer-select-wrap:has(.composer-select:focus)::after {
  border-right-color: var(--text-strong);
  border-bottom-color: var(--text-strong);
}
```

### 11.2 Composer 原生 Select (`.composer-select`)

```css
.composer-select {
  appearance: none;                        /* 隐藏原生箭头 */
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 11px;
  padding: 2px 18px 2px 0;               /* 右侧留空给自定义箭头 */
  cursor: pointer;
  width: auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  background-image: none;
}
```

**Focus 状态：**
```css
.composer-select:focus {
  outline: none;
  color: var(--text-strong);
}

.composer-select option {
  background-color: var(--surface-popover);
  color: var(--text-strong);
}
```

**各 Select 宽度：**

| 选择器 | 宽度 |
|--------|------|
| `--model` | `var(--composer-model-select-width, auto)` |
| `--collab` | 78px |
| `--effort` | 80px |
| `--approval` | 90px |

### 11.2.1 工具栏来源选择（Radix Select）

- 设置、仓库等需要切换**内容来源**的工具栏控件必须使用项目的 `Select` 组件；禁止新增浏览器原生 `<select>`，以避免系统原生菜单的视觉割裂、不可控样式和缺失的开闭动画。
- Trigger 使用紧凑胶囊（`size="sm"`、`h-8` 或 `h-9`、`rounded-full`），宽度按最长本地化文案预留；图标箭头由组件统一提供，不另绘 CSS 三角形。
- `SelectContent` 使用 `position="popper"`，右侧工具栏对齐 `align="end"`；必须复用 `OVERLAY_MOTION` 与 `OVERLAY_SIDE_SLIDE`，即 150ms fade + 不超过 4px 的微位移，禁止缩放或原生菜单。
- 每项保持单一来源名称和可见焦点，当前项由组件的勾选指示表达；来源切换后内容区应立即进入加载态，不保留已过期来源的结果。

### 11.3 Git 面板分支选择器 (`.git-panel-select-input`)

```css
.git-panel-select-input {
  border: 1px solid var(--border-default);
  background: var(--surface-panel);
  color: var(--text-emphasis);
  font-size: 12px;
  font-weight: 600;
  padding: 9px 34px 9px 34px;              /* 左右各 34px：左侧图标 + 右侧箭头 */
  border-radius: 999px;                     /* 胶囊形 */
  cursor: pointer;
  min-height: 34px;
  appearance: none;
  /* 双三角形下拉箭头 */
  background-image:
    linear-gradient(45deg, transparent 50%, var(--text-dim) 50%),
    linear-gradient(135deg, var(--text-dim) 50%, transparent 50%);
  background-position:
    calc(100% - 16px) 50%,
    calc(100% - 11px) 50%;
  background-size: 5px 5px, 5px 5px;
  background-repeat: no-repeat;
  transition: background 160ms ease,
              border-color 160ms ease,
              color 160ms ease;
}
```

**Hover 状态：**
```css
.git-panel-select-input:hover,
.git-panel-select-input:focus-visible {
  background-color: var(--surface-panel-hover);
  border-color: var(--border-hover);
  outline: none;
}
```

---

## 12. 开关与分段控件

### 12.1 Toggle 开关 (`.settings-toggle`)

```css
.settings-toggle {
  width: 44px;
  height: 24px;
  border-radius: 999px;                     /* 完全圆角 = 胶囊/跑道形 */
  border: 1px solid var(--border-strong);
  background: var(--surface-control);       /* white 8% */
  padding: 3px;                             /* 给 knob 留空间 */
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  transition: background 160ms cubic-bezier(0.25, 0.46, 0.45, 0.94),
              border-color 160ms cubic-bezier(0.25, 0.46, 0.45, 0.94);
}
```

**尺寸：** 44×24px（宽高比 1.83:1）

**ON 状态：**
```css
.settings-toggle.on {
  background: linear-gradient(135deg,
    rgba(100, 200, 255, 0.6),
    rgba(120, 235, 190, 0.6)
  );                                         /* 蓝→绿 渐变 */
  border-color: var(--border-accent);
}
```

**Reduced Transparency 时 OFF 状态加强：**
```css
.app.reduced-transparency .settings-toggle:not(.on) {
  background: var(--surface-control-hover);
  border-color: var(--border-stronger);
}
```

### 12.2 Toggle 旋钮 (`.settings-toggle-knob`)

```css
.settings-toggle-knob {
  width: 16px;
  height: 16px;
  border-radius: 999px;                     /* 正圆形 */
  background: var(--surface-card-strong);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
  transition: transform 160ms cubic-bezier(0.34, 1.56, 0.64, 1);  /* 弹簧缓动 */
}
```

**尺寸：** 16×16px 正圆
**阴影：** `0 2px 6px rgba(0,0,0,0.2)`

**ON 状态位移（弹簧动画）：**
```css
.settings-toggle.on .settings-toggle-knob {
  transform: translateX(20px);              /* 向右移动 20px，弹簧效果 */
}
```

**Toggle 开关完整规格：**
- 轨道：44×24px，圆角 999px
- 旋钮：16×16px，圆角 999px
- OFF 背景：`--surface-control`（white 8%）
- ON 背景：蓝→绿 渐变
- OFF→ON 移动距离：20px
- 动画：弹簧缓动 `cubic-bezier(0.34, 1.56, 0.64, 1)` 160ms

> **现行实现校正：** React 页面统一使用 `src/components/ui/switch.tsx` 的 Radix Switch 与主题 token；禁止复制上面的蓝绿渐变或弹簧 easing。状态变化只用现有轨道/旋钮位移与 `--ds-dur-fast`，遵循全局“无弹跳、无缩放”规则。

Skills inventory 与详情头部的 Switch 还必须满足：

- accessible name 为“启用 {skillName}”，并暴露 `checked`、`disabled` 与异步时的 `aria-busy`；关键开关始终可见，不藏在 hover action 中。
- pending 时禁用重复操作；成功后以 inventory/event 的后端状态刷新，失败回滚并显示本地化错误。
- unhealthy 或后续扫描状态不允许启用时，Switch disabled，原因在相邻状态文字/安全 tab 解释；禁止把完整原因只放进 disabled tooltip。
- Switch 不得嵌套在 Skill 选择 `<button>` 内；行内选择区与 Switch 是并列控件，分别拥有明确焦点顺序。

### 12.2.1 Skills 详情 Tabs 与文件树动作

- 详情 tabs 固定为 `文件 / 安全 / 概览`，复用共享 Radix `Tabs` 的 line variant、可见 focus ring 与键盘行为；默认 Files，不用手写只响应鼠标的伪 tab。
- 文件树行是紧凑、整行可点击的 `treeitem`，目录 disclosure 与文件图标使用 Lucide；hover/selected 只改颜色和背景，不缩放或位移。
- “继续加载文件/预览”是显式小型 outline/text action，必须有不少于 32px 的行高、pending disabled 和可见焦点；不得以无限自动读取替代用户确认。

### 12.2.2 Skills 安全筛选与审批动作

- 严重度筛选复用 `size="xs"` Button 组；当前值用 secondary、其他值用 ghost，并同步 `aria-pressed`。不要为 severity 自造彩色胶囊或只靠颜色区分。
- 批准/拒绝是 reason 输入后的并列动作：拒绝使用 outline，批准使用默认主按钮；输入不足、请求 pending 或 scan 已失效时两者 disabled，禁止用连续 Toast 代替就地状态。
- “撤销批准”是可逆审计动作，使用 outline，不做 destructive 红色；删除 Skill/拒绝制品等不可逆动作仍遵循 Dialog 确认规则。
- JSON/SARIF 导出使用带 Download 图标的 outline 小按钮；必须先打开系统保存对话框，取消保存不显示成功 Toast。
- 存量扫描失败后的“重试失败项”使用 `size="xs"` outline Button，固定出现在迁移状态条右侧；只重试持久化 failed items，不提供“跳过扫描并启用”动作。

### 12.3 分段控件 (`.settings-segmented`)

```css
.settings-segmented {
  position: relative;
  display: inline-flex;
  align-items: center;
  align-self: flex-start;
  gap: 4px;
  padding: 4px;
  border-radius: 999px;                     /* 外框胶囊形 */
  border: 1px solid var(--border-muted);
  background: var(--surface-control);
}
```

**滑动指示器 (`.settings-segmented::before`)：**
```css
.settings-segmented::before {
  content: "";
  position: absolute;
  top: 4px;
  left: 4px;
  width: calc(50% - 6px);                   /* 一半宽度减去间距 */
  height: calc(100% - 8px);                 /* 高度减去上下间距 */
  border-radius: 999px;
  background: var(--surface-card);
  box-shadow: inset 0 0 0 1px var(--border-strong);
  transition: transform 200ms cubic-bezier(0.645, 0.045, 0.355, 1);
  z-index: 0;
}

/* 第二个选项激活时，滑动指示器移动到右侧 */
.settings-segmented.is-second-active::before {
  transform: translateX(calc(100% + 4px));
}
```

**分段选项：**
```css
.settings-segmented-option {
  position: relative;
  z-index: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: transparent;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 600;
  padding: 0;
  min-width: 72px;
  overflow: hidden;
  transition: color 220ms cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.settings-segmented-option-label {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  padding: 6px 12px;
}
```

**Hover（非禁用、非激活）：**
```css
.settings-segmented-option:hover:not(.is-disabled) {
  color: var(--text-strong);
  background: color-mix(in srgb, var(--surface-card) 55%, transparent);
}
```

**Active 状态：**
```css
.settings-segmented-option.is-active {
  color: var(--text-strong);
}
```

**Disabled 状态：**
```css
.settings-segmented-option.is-disabled {
  cursor: not-allowed;
  color: var(--text-faint);
}
```

**Focus 可见：**
```css
.settings-segmented-input:focus-visible + .settings-segmented-option-label {
  outline: 2px solid var(--focus-ring);
  outline-offset: -2px;
}
```

---

## 13. 模态框按钮

### 13.1 模态框容器

```css
.ds-modal {
  position: fixed;
  inset: 0;
  z-index: 10000;                           /* --ds-layer-modal */
}

.ds-modal-backdrop {
  position: absolute;
  inset: 0;
  background: rgba(6, 8, 12, 0.55);        /* --ds-modal-backdrop */
  backdrop-filter: blur(8px);
  animation: ds-modal-backdrop-in 200ms cubic-bezier(0.16, 1, 0.3, 1) both;
}

@keyframes ds-modal-backdrop-in {
  from { opacity: 0; }
  to   { opacity: 1; }
}
```

**模态框卡片：**
```css
.ds-modal-card {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: var(--surface-card-strong);
  border: 1px solid var(--border-stronger);
  color: var(--text-strong);
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.35);
  animation: ds-modal-card-in 220ms cubic-bezier(0.16, 1, 0.3, 1) both;
}

@keyframes ds-modal-card-in {
  from {
    opacity: 0;
    transform: translate(-50%, -48%) scale(0.98);  /* 从上方+缩小 */
  }
  to {
    opacity: 1;
    transform: translate(-50%, -50%) scale(1);
  }
}
```

### 13.2 模态框按钮 (`.ds-modal-button`)

```css
.ds-modal-button {
  padding: 6px 12px;
  border-radius: 10px;
}
```

### 13.3 Reduced Transparency 适配

```css
.app.reduced-transparency .ds-modal-backdrop {
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}
```

### 13.4 设置页专用模态框

**添加远程仓库卡片：**
```css
.settings-add-remote-card {
  width: min(420px, calc(100vw - 40px));
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  background: #141b27;                       /* 固定深色背景 */
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 16px;
  box-shadow: 0 24px 60px rgba(0, 0, 0, 0.55);
  color: #eef3ff;
}

.settings-add-remote-card .ghost {
  color: #dce6f7;
  border-color: rgba(255, 255, 255, 0.24);
  background: rgba(255, 255, 255, 0.06);
}

.settings-add-remote-card .ghost:hover {
  background: rgba(255, 255, 255, 0.11);
}
```

**删除确认卡片：**
```css
.settings-delete-remote-card {
  width: min(380px, calc(100vw - 40px));
  padding: 16px;
  /* 其他与 add-remote-card 相同 */
}
```

### 13.5 扩展尺寸弹窗

用于 Provider 配置、模型批量选择等信息密度高但仍属于单任务的设置弹窗。

- 宽度优先使用 `max-w-[96rem]`（约为原 `max-w-3xl` / 48rem 的两倍）；需要适配窄屏时叠加 `w-[min(96rem,calc(100vw-2rem))]`，避免内容贴边或横向溢出。
- **必须**同时写 `sm:max-w-[96rem]`：`DialogContent` 基类含 `sm:max-w-lg`，与无断点 `max-w-*` 不会被 `tailwind-merge` 互斥剔除，窄屏上限仍在 `sm+` 生效；显式覆盖才可避免桌面宽度被钉在约 32rem。
- 高度上限使用 `max-h-[85vh]`；内容区应在弹窗内部滚动，页脚操作按钮固定在可见区域下方，不依赖整页滚动。
- Provider 等三段式弹窗使用 `grid-rows-[auto_minmax(0,1fr)_auto]`：外层必须是不可滚动的 `overflow-clip`，仅中间内容区使用 `min-h-0 overflow-y-auto`，页脚保持 `auto` 行。禁止用 `overflow-hidden` 代替外层裁剪；它仍可被焦点滚动并连同页脚一起产生位移。
- 弹窗只允许承载一个主流程。若流程需要二次筛选（如获取模型后的多选列表），使用内嵌二级 Dialog，并将二级 Dialog 控制在 `max-w-2xl`、`max-h-[80vh]`。
- 长列表区域须有独立高度约束（如 `h-[360px]` 或 `max-h-[360px]`）和内部滚动，不让列表把弹窗撑出视口。
- 进入和退出动画仍遵守全局克制动效：以透明度和极小位移为主，不使用夸张缩放、弹跳或悬停位移。

---

## 14. Toast 提示按钮

### 14.1 Toast 容器

```css
.ds-toast-card {
  background: var(--surface-context-core);   /* rgba(10,14,20,0.9) */
  border: 1px solid var(--border-subtle);
  box-shadow: 0 16px 32px rgba(0, 0, 0, 0.25);
  border-radius: 12px;
  padding: 12px;
  pointer-events: auto;
  max-width: 100%;
  animation: ds-toast-in 0.2s ease-out;
}

@keyframes ds-toast-in {
  from {
    opacity: 0;
    transform: translateY(-6px);              /* 从上方 6px 滑入 */
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
```

**层级：** z-index: 11000（`--ds-layer-toast`），高于模态框

---

## 15. Tooltip 提示系统

### 15.1 Tooltip 触发器

```css
.ds-tooltip-trigger {
  position: relative;
}
```

### 15.2 Tooltip 气泡 (::after)

```css
.ds-tooltip-trigger[data-tooltip]::after {
  content: attr(data-tooltip);               /* 从 data 属性读取文本 */
  position: absolute;
  left: 50%;
  bottom: calc(100% + 8px);                  /* 触发元素上方 8px */
  transform: translateX(-50%) translateY(4px);
  padding: 4px 8px;
  border-radius: 8px;
  background: var(--surface-command);
  color: var(--text-emphasis);
  font-size: 10px;
  line-height: 1.2;
  white-space: nowrap;
  border: 1px solid var(--border-subtle);
  box-shadow: 0 14px 24px rgba(0, 0, 0, 0.22);
}
```

### 15.3 Tooltip 箭头 (::before)

```css
.ds-tooltip-trigger[data-tooltip]::before {
  content: "";
  position: absolute;
  left: 50%;
  bottom: calc(100% + 4px);                  /* 气泡下方 4px */
  transform: translateX(-50%) translateY(4px) rotate(45deg);
  width: 8px;
  height: 8px;
  background: var(--surface-command);
  border-left: 1px solid var(--border-subtle);
  border-top: 1px solid var(--border-subtle);
}
```

**箭头实现：** 8×8px 正方形，旋转 45°，只显示左边和上边边框

### 15.4 显示/隐藏

```css
/* 默认隐藏 */
.ds-tooltip-trigger[data-tooltip]::before,
.ds-tooltip-trigger[data-tooltip]::after {
  opacity: 0;
  pointer-events: none;
  transition: opacity 150ms ease, transform 150ms ease;
  z-index: 20;
}

/* Hover/Focus 时显示 */
.ds-tooltip-trigger[data-tooltip]:hover::after,
.ds-tooltip-trigger[data-tooltip]:focus-visible::after {
  opacity: 1;
  transform: translateX(-50%) translateY(0);   /* 向上滑动 4px */
}

.ds-tooltip-trigger[data-tooltip]:hover::before,
.ds-tooltip-trigger[data-tooltip]:focus-visible::before {
  opacity: 1;
  transform: translateX(-50%) translateY(0) rotate(45deg);
}
```

### 15.5 对齐变体

**左对齐 (data-tooltip-align="start")：**
```css
.ds-tooltip-trigger[data-tooltip][data-tooltip-align="start"]::after {
  left: 0;
  transform: translateY(4px);                /* 不居中 */
}

.ds-tooltip-trigger[data-tooltip][data-tooltip-align="start"]::before {
  left: 8px;                                 /* 箭头在左侧 */
  transform: translateY(4px) rotate(45deg);
}
```

**右对齐 (data-tooltip-align="end")：**
```css
.ds-tooltip-trigger[data-tooltip][data-tooltip-align="end"]::after {
  left: auto;
  right: 0;
  transform: translateY(4px);
}

.ds-tooltip-trigger[data-tooltip][data-tooltip-align="end"]::before {
  left: auto;
  right: 8px;
  transform: translateY(4px) rotate(45deg);
}
```

**底部放置 (data-tooltip-placement="bottom")：**
```css
.ds-tooltip-trigger[data-tooltip][data-tooltip-placement="bottom"]::after {
  top: calc(100% + 8px);                     /* 改为下方 */
  bottom: auto;
  transform: translateX(-50%) translateY(-4px);
}

.ds-tooltip-trigger[data-tooltip][data-tooltip-placement="bottom"]::before {
  display: none;                             /* 底部无箭头 */
}
```

### 15.6 Radix Tooltip 实施约束（必读，新增）

`src/components/ui/tooltip.tsx` 是项目内统一的 Radix 封装，所有 UI 改动必须基于该封装而**非**重新自实现：

- **尺寸**：使用 `px-2 py-1` + `text-[12px] font-medium leading-[18px] tracking-tight`；**禁止**使用 `px-3 py-2` 这种与按钮文字气泡同尺寸的 padding，会让 Tooltip 看起来像一个对话框。
- **颜色**：背景 `bg-[color:var(--surface-popover,var(--popover))]/95 backdrop-blur-md`；文字 `text-popover-foreground/95 text-balance`；边框 `border-[color:var(--border-strong)]/70`（70% 透明度，让边缘融入而非生硬）。
- **圆角**：必须用 `rounded-[var(--radius-ui-md)]`，与 Dropdown/ContextMenu 共享圆角语言；不得退回 `rounded-md` 这种字面值。
- **阴影**：使用双层柔阴影 `shadow-[0_10px_24px_-12px_rgba(0,0,0,0.45),0_2px_6px_-1px_rgba(0,0,0,0.25)]`，避免单层硬阴影。
- **箭头（Arrow）**：默认**不渲染** `TooltipPrimitive.Arrow`。旧实现里带边框的小三角在多种 side 下会与气泡边框错位、形成「断角」视觉缺陷，现已删除。
- **入场 / 退场动画**：`animate-in fade-in-0 ease-out duration-150` + 微位移 `data-[side=*]:slide-in-from-*-1`；关闭 `data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:duration-150`。不得使用 `slide-in-from-*-2` 等大于 4px 的位移；不得省略 fade-out（避免气泡「闪断」）。
- **Composer 附件 / 发送**：局部 `Tooltip` 须设 `delayDuration={2000}`（约 2s 后再显示），与壳层默认 300ms 解耦。
- **z-index**：使用 `z-[var(--ds-layer-modal)]`；**禁止**使用 `z-[calc(var(--ds-layer-modal)+2)]` 这种 magic 偏移，Modal 之上再有 toast 等更高层级时由 toast 自己处理。
- **指针**：`pointer-events-none`，避免遮挡其它控件。

---

## 16. 窗口控制按钮

### 16.1 窗口标题栏按钮

```css
.window-caption-control {
  width: 46px;
  height: 44px;                              /* --main-topbar-height */
  border: none;
  border-radius: 0;                          /* 无圆角 */
  padding: 0;
  background: transparent;
  color: var(--text-muted);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-shadow: none;
  transform: none;
}

.window-caption-control svg {
  width: 14px;
  height: 14px;
}
```

**Hover 状态：**
```css
.window-caption-control:hover,
.window-caption-control:focus-visible {
  background: var(--surface-hover);
  color: var(--text-strong);
  box-shadow: none;
  transform: none;                           /* 禁止浮动 */
}
```

**关闭按钮 Hover（红色）：**
```css
.window-caption-control-close:hover,
.window-caption-control-close:focus-visible {
  background: #e81123;                       /* Windows 红色 */
  color: #fff;
}
```

### 16.2 Sidebar 折叠按钮 (`.titlebar-toggle`)

```css
.titlebar-toggle .main-header-action {
  width: 28px;          /* --titlebar-toggle-size */
  height: 28px;
  padding: 0;
  border-radius: 8px;
  transition: background-color 120ms ease,
              color 120ms ease,
              opacity 120ms ease;
}
```

---

## 17. 动画时间线汇总

### 17.1 按钮交互时间线

```
HOVER IN:   150ms ease  (transform, box-shadow, background-color, filter)
PRESS DOWN:  50ms ease  (transform, box-shadow)
RELEASE:    150ms ease  (transform, box-shadow, background-color, filter)
```

### 17.2 各组件过渡时间

| 组件 | 属性 | 时长 | 缓动 |
|------|------|------|------|
| 基础 button | transform, shadow, bg, filter | 150ms | ease |
| 基础 button active | transform, shadow | 50ms | ease |
| 侧边栏图标按钮 | background-color, color | 120ms | ease |
| Composer action | background | (继承基础) | — |
| Diff row action | background, border-color, color | 160ms | ease |
| Diff row actions 容器 | max-width, opacity, transform | 140-180ms | ease |
| Git 面板按钮 | background, border-color, color | 160ms | ease |
| Popover / Dropdown / Select | opacity + ≤4px slide | 150ms | ease-out（`OVERLAY_MOTION`） |
| ContextMenu | opacity | 150ms | ease-out（仅 fade） |
| Dialog overlay / card | opacity | `--ds-dur-entrance` | `--ds-ease-out`（`DIALOG_MOTION`） |
| Segmented control 滑块 | transform | 200ms | `cubic-bezier(0.645, 0.045, 0.355, 1)` |
| Toggle 旋钮 | transform | 160ms | ease-spring |
| Modal 卡片入场 | opacity, transform | 220ms | ease-out |
| Modal 背景入场 | opacity | 200ms | ease-out |
| Toast 入场 | opacity, transform | 200ms | ease-out |
| Tooltip 显示/隐藏 | opacity, transform | 150ms | ease |
| Sidebar 宽度 | grid-template-columns | 220ms | ease |

### 17.3 缓动函数一览

| 名称 | 值 | 视觉特征 |
|------|-----|----------|
| `ease` | 浏览器默认 | 标准缓入缓出 |
| `--ds-ease-out` | `cubic-bezier(0.16, 1, 0.3, 1)` | 快速开始，缓慢结束，微超调 |
| `--ds-ease-out-soft` | `cubic-bezier(0.25, 0.46, 0.45, 0.94)` | 更平缓的缓出 |
| `--ds-ease-spring` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | 弹簧回弹效果 |
| Segmented slider | `cubic-bezier(0.645, 0.045, 0.355, 1)` | 先慢后快再慢（ease-in-out 变体） |

### 17.4 Reduced Motion 策略

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }

  /* 保留功能性旋转器（加载状态指示） */
  .working-spinner,
  .git-panel-spinner,
  .commit-message-loader,
  .commit-button-spinner,
  .sidebar-refresh-icon.spinning,
  .worktree-deleting-spinner,
  .sidebar-account-spinner,
  .settings-agents-generate-loader,
  .composer-action-spinner {
    animation-duration: 0.7s !important;
    animation-iteration-count: infinite !important;
  }
}
```

**设计原则：** 所有装饰性动画（hover 浮动、淡入淡出、缩放）均被禁用，但功能性旋转器（Loading、刷新指示器）保持运行，因为它们传递关键的 "正在处理" 状态信息。

---

## 附录 A：尺寸速查表

### 按钮尺寸一览

| 按钮类型 | 宽度 | 高度 | 内边距 | 圆角 | 字号 |
|----------|------|------|--------|------|------|
| 基础 button | auto | auto | 8px 14px | 10px | 13px |
| 标题栏图标 | 28px | 28px | 0 | 8px | — |
| 侧边栏标题 | auto | auto | 4px 6px | 8px | — |
| 侧边栏添加 | 30px | 30px | 0 | 8px | — |
| 搜索/排序/刷新 | 32px | 32px | 0 | 8px | — |
| 工作区添加 | 22px | 22px | 0 | 999px | 13px |
| All Threads 添加 | 24px | 24px | 0 | 999px | — |
| Composer action | 32px | 32px | 0 | 999px | 14px |
| Composer attach | 32px | 32px | 0 | 999px | 14px |
| Diff row action | 22px | 22px | 0 | 999px | — |
| Git root button | auto | auto | 7px 11px | 999px | 12px |
| Commit button | 100% | auto | 10px 14px | 14px | 12px |
| Commit generate | 26px | 26px | 0 | 999px | — |
| Push button | flex:1 | auto | 10px 12px | 14px | 12px |
| Branch refresh | 28px | 28px | 0 | 999px | — |
| Settings nav item | 100% | auto | 8px 10px | 10px | 13px |
| Settings icon button | 28px | 28px | 0 | 8px | — |
| Labeled button | 100% | 34px | 4px 10px | 10px | 11px |
| Modal button | auto | auto | 6px 12px | 10px | — |

### 图标尺寸一览

| 容器 | 图标尺寸 |
|------|----------|
| `.icon-button svg` | 16×16px |
| `.sidebar-title-add svg` | 16×16px |
| `.sidebar-search-toggle svg` | 14×14px |
| `.sidebar-sort-toggle svg` | 14×14px |
| `.sidebar-refresh-toggle svg` | 14×14px |
| `.all-threads-add svg` | 14×14px |
| `.composer-action svg` | 12×12px |
| `.composer-attach` (无svg wrapper) | — |
| `.commit-message-generate-button svg` | 14×14px |
| `.ds-popover-item-icon svg` | 14×14px |
| `.settings-icon-button svg` | 14×14px |
| `.settings-nav-item-icon svg` | 16×16px |
| `.git-root-button-icon` | 12×12px |
| `.window-caption-control svg` | 14×14px |

### 选中态颜色一览

| 元素 | 背景色 | 边框色 |
|------|--------|--------|
| Thread row `.active` | `color-mix(in srgb, var(--surface-active) 96%, transparent)` | `color-mix(in srgb, var(--border-accent) 42%, transparent)` |
| Workspace row `.active` | `var(--surface-panel-active)` | `var(--border-accent-strong)` |
| Diff row `.active` | `var(--surface-panel-active)` | `inset 0 0 0 1px color-mix(in srgb, var(--border-accent) 65%, transparent)` |
| Git log entry `.active` | `var(--surface-panel-active)` | `inset 0 0 0 1px color-mix(in srgb, var(--border-accent) 65%, transparent)` |
| Per-file edit row `.active` | `var(--surface-panel-active)` | `inset 0 0 0 1px color-mix(in srgb, var(--border-accent) 55%, transparent)` |
| Settings nav item `.is-active` | `var(--surface-card)` | `var(--border-strong)` |
| Popover item `.is-active` | `var(--surface-hover)` | none |

### 操作颜色编码

| 操作 | 颜色 | Hex |
|------|------|-----|
| 成功 / Stage / Add | 绿色 | `#47d488` |
| 警告 / Unstage / Modified | 金色 | `#f5c363` |
| 危险 / Discard / Delete | 红色 | `#ff6b6b` |
| 信息 / Review / Apply | 蓝色 | `#5aa9ff` |
| Primary 渐变起 | 蓝色 | `#62b7ff` |
| Primary 渐变止 | 绿色 | `#4fe3a3` |
| 关闭按钮 hover | 红色 | `#e81123` |

---

## 11.1 右键菜单 (ContextMenu)

**实现位置**：`src/components/ui/context-menu.tsx`（基于 `radix-ui` 的 `ContextMenu`）。

### 11.1.1 适用场景

- 文件树节点、消息条目等需要二级操作但「显式按钮会喧宾夺主」的密集列表。
- **不得**用 ContextMenu 承载主要操作；主操作必须保留在显式按钮、Dropdown 或 Toolbar 中。

### 11.1.2 视觉规范

- 复用 `DropdownMenuContent` 的视觉语言：圆角 `var(--radius-ui-lg)`、边框 `var(--border-strong)`、`bg-popover/95` + `backdrop-blur-xl` + `shadow-lg`。
- 入场/退场使用共享 `OVERLAY_MOTION`（`animate-in/out` + `fade-in-0` / `fade-out-0`，`duration-150`）；不得使用 `zoom-in*` 或大幅 `slide-*`，与全局动效规范一致。
- Item 高度与 Dropdown 对齐（`py-1.5`、`text-sm`），图标统一 `size-4`，左侧 8px gap。
- 提供 `ContextMenuSeparator` 分组；分组数量 ≤ 3，超过应抽象为子菜单或 Popover。

### 11.1.3 行为约束

- 必须正确阻止默认浏览器右键菜单（Radix Trigger 已内置）。
- 危险操作使用 `variant="destructive"`，并以分隔线与其它操作分组到菜单底部。
- 文案必须 i18n：禁止在 `ContextMenuItem` 内硬编码语言字符串。
- 触发后若调用了 Tauri IPC，失败必须通过 `toast.error` 反馈，并使用语义化的 i18n key（如 `workspace.explorer.openInExplorerFailed`）。

### 11.1.4 文件树节点菜单（项目专用约定）

文件树节点的 ContextMenu 内容、顺序与 i18n key 已在
[shell-and-workspace-ui-spec.md §5.6](./shell-and-workspace-ui-spec.md) 中规范化。
新增/调整文件树右键菜单时必须**同时**满足该处约定与本节通用 ContextMenu 行为约束。
