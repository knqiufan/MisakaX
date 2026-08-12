# 富内容与产物交付 UI/UX 设计

> **用途：** 规定聊天内图表、地图、文件和图片组件的视觉语言、交互、可访问性与响应式行为。
> **受众：** React UI、设计、测试和本地化维护者。
> **最后审阅 / Last reviewed：** 2026-08-12
> **状态：** Proposed；实施时必须同步现有 `docs/design/` 规范。

---

## 1. 继承的界面基线

本方案遵循项目的桌面 Agent 视觉语言：助手消息无大型气泡背景、内容横向铺满可用消息列；用户消息继续使用 `bg-muted` 的紧凑圆角气泡。富内容是消息正文中的“内容卡”，不能被设计成网页 dashboard、营销卡片或独立应用窗口。

实现时优先复用项目 token 和基础组件，特别是 `--surface-*`、`--border-*`、`--radius-ui-*`、`--ds-layer-*`、`OVERLAY_MOTION`、Dialog、Tooltip、DropdownMenu、Button 和 Sonner。不得硬编码另一套色板、夸张渐变、hover 缩放、弹跳动画或 emoji 图标。

相关权威规范：

- [frontend-ui-guidelines.md](../../design/frontend-ui-guidelines.md)
- [shell-and-workspace-ui-spec.md](../../design/shell-and-workspace-ui-spec.md)
- [button-menu-design-spec.md](../../design/button-menu-design-spec.md)
- [现有 Chat 规范](../../ui/02-chat.md) 与 [Markdown 规范](../../ui/06-markdown-message-tools.md)

## 2. 共同的 RichContentCard

### 2.1 结构

```text
RichContentCard
  header: icon + title + optional compact status + overflow actions
  body: block-specific visualization / preview / image
  footer: accessible summary, attribution or metadata + primary actions
```

- 位于 assistant 正文流内，默认 `my-4`；宽度为消息列，`min-w-0`，不突破聊天内容最大宽度。
- 外层为低对比表面（`bg-muted/20` 或项目定义的 card surface）、`border-border/40`、`rounded-xl`、`overflow-hidden`。不得使用高饱和色作整卡背景。
- Header 高度紧凑，左侧使用统一 Lucide 14–16px 图标；右侧操作使用现有 28px icon button/菜单。常用下载动作可明确显示，次要动作用溢出菜单。
- Footer 仅在需要 attribution、生成元数据、错误或操作时展示；正文信息密度高时不重复标题。

### 2.2 状态

| 状态 | 视觉 | 行为 |
|---|---|---|
| 加载中 | 固定最小高度 skeleton + 简短文案 | 保留空间，禁止重复点击 |
| 完成 | 常规 surface/边框 | 操作可用、焦点可达 |
| 部分可用 | 中性提示 + 可下载/可查看数据 | 不用危险色误导为失败 |
| 失败 | `destructive` 文本/图标的小型 notice | 说明下一步，不清空同消息正文 |
| 不支持 | muted notice + 原件下载 | 不显示原始 JSON/堆栈 |
| 已过期 | muted/disabled metadata | 不发起重复加载 |

加载、展开、预览只使用 opacity 和不超过 4–6px 的位移；遵循 `prefers-reduced-motion`。不能为流式块使用逐字/逐帧动效。

## 3. 图表 UI

### 3.1 默认布局

```text
┌ [Chart] 销售趋势                         [更多] ┐
│ 2026 Q2 环比 +18%，增长主要来自华东。             │
│                                                    │
│              可缩放/悬停的图表画布                │
│                                                    │
├ 数据表  导出数据  导出图像                 12 条数据 ┤
└────────────────────────────────────────────────────┘
```

- 图面默认高度约 260–320px，窄窗口降低到 220px；固定/受控高度避免虚拟列表测量抖动。
- 标题与结论在图面前，避免只让颜色和 tooltip 传递含义。
- 图例支持键盘焦点与清晰 selected 状态；系列颜色用语义色板、pattern/marker 区分，深浅主题对比度足够。
- “数据表”是可切换的正文区域而不是仅 hover tooltip；表格沿用现有 Markdown table 的横向滚动和操作语言。
- 高密度数据必须把标签/tooltip 降噪，图例超过合理数量时优先显示可搜索的列表或数据表。

### 3.2 操作和键盘

- Tab 顺序：卡片操作 → 图例/可交互控件 → 数据表/导出。
- `Enter`/`Space` 激活；`Escape` 关闭数据表、tooltip 锁定或 Dialog；不劫持消息列表的常规方向滚动。
- 图表画布若无法提供完整键盘交互，必须通过 summary + 数据表提供等价可访问路径，并标记为 `aria-describedby`。

## 4. 地图 UI

### 4.1 默认布局

地图卡沿用 RichContentCard，图面默认 300px，高宽变化时动态 `resize`，不可 overflow 到侧栏。Header 必须显式展示数据来源/attribution 入口；Footer 提供“要素列表”“复制坐标”“导出 GeoJSON”与可用时的“重置视图”。

选择要素时，在地图上使用克制的边框/halo，并在下面的可访问要素列表同步选中；不得只用 marker 颜色表示选择。地图或 WebGL 不可用时，直接显示列表、边界、坐标文本和原因提示，不显示永远旋转的 loader。

### 4.2 隐私与在线态

- 地图瓦片加载/离线状态在 footer 以小型状态文字表达，不弹 toast 打断对话。
- 受控网络不可用时显示“离线底图/仅要素数据”，不提示用户放宽 WebView 网络权限。若由云端安全位置获取瓦片，详情入口应说明服务/地区/缓存与 attribution，不暴露密钥或 provider 内部参数。
- 用户位置按钮若后续加入，属于显式主动作，有清晰权限解释；默认不出现追踪含义的图标/文案。
- attribution 永远可见或一键可达；不使用模型生成的品牌/服务商 logo 作为依据。

## 5. 文件预览 UI

### 5.1 产物卡

```text
┌ [FileTypeIcon] 月度分析.xlsx                         [更多] ┐
│ XLSX · 248 KB · 由 Agent 生成 · 刚刚                  │
│ [预览]  [下载]                                         │
└────────────────────────────────────────────────────────┘
```

- 文件名单行截断但可 Tooltip/辅助文字查看全名；显示类型和大小，来源仅作辅助信息。
- `预览` 是主操作（支持时），`下载` 保持可见；不支持预览时把“仅支持下载”的原因写清楚。
- 预览在消息中显示轻量摘要，复杂 PDF/表格/DOCX 在二级 Dialog 中打开。Dialog 遵循现有最大宽度/高度、内部滚动和固定 footer 规则；不能让长文档把主页面拖到失去上下文。
- PDF 页码、Sheet tabs、表格列冻结/横向滚动均由内容区负责，Dialog header/footer 不随内容滚动。

### 5.2 失败与可恢复性

解析失败、文件过大、已过期、权限不足分别给出不同 i18n 文案；下载仍可用时不得禁用。用户取消系统保存对话框不是错误，不显示 destructive toast。

### 5.3 需要执行隔离的预览

仅在原生 converter、OCR、Office helper 或其他外部进程确实参与时展示执行位置；纯浏览器 PDF/XLSX/DOCX worker 不增加“沙箱”徽章。文案面向保证而不是工程 provider 名：

| 状态 | 卡片/对话框应显示 | 禁止行为 |
|---|---|---|
| `本机安全（离线）` | 无网络、仅副本输入、不会直接修改真实项目、已验证工具兼容性 | 把实验 API 或 AppContainer profile 暴露成用户设置 |
| `云端安全` | 服务/组织、地区、上传摘要、保留期、允许域、费用/配额、销毁状态 | 静默上传工作区或只写“云端处理” |
| `本机安全 VM（实验性）` | 镜像大小、磁盘/内存预算、Linux 工具限制和网络状态 | 暗示支持 Xcode/macOS 原生工具 |
| `本机直接执行` | destructive 风险说明、当前用户权限、可能访问的文件/网络、原因和批准到期 | 使用安全色或“受保护”文案；provider 失败后自动切入 |

严格 provider 不可用时，卡片使用“此环境无法安全预览/生成”的部分可用状态，保留下载和诊断入口；不要用主 CTA 引导用户立即改成 `本机直接执行`。snapshot 产生工作区变更时，写回必须进入独立 diff/冲突确认界面，富内容卡只能展示摘要与入口，不能在后台自动 apply。

## 6. 图片 UI

- 消息中图片以 `max-width: 100%`、受控 `max-height`、`object-contain` 显示，预留尺寸防布局跳动；不裁掉信息性图像。
- 图像点击进入预览 Dialog；header 显示文件名/尺寸，footer 提供缩放、适配和下载。关闭按钮有明确 label，`Escape` 可关闭并将焦点还给触发缩略图。
- 重要图片 alt text 来自模型的结构化说明或用户给定文件名；没有可靠说明时用中性描述而不杜撰视觉内容。
- GIF 等动图尊重减少动态偏好；超大图片在加载前显示轻量占位或缩略图，避免主线程卡顿。

## 7. 响应式、虚拟列表与性能

本产品是桌面优先，但必须适配最小窗口。卡片在窄宽度下改为纵向 action layout，保持按钮 44px 最小触控目标与清晰焦点。图表/地图不得产生聊天列横向滚动；表格和代码等需要横向查看的内容使用内部 `overflow-x-auto`。

消息虚拟列表中，富内容块要声明稳定的 loading 高度。图表/地图进入视口后再懒加载 renderer；离开视口可暂停昂贵渲染但不得丢失选择/缩放状态。PDF/XLSX 预览只在用户打开 Dialog 后加载。

## 8. 本地化与无障碍验收

- 所有按钮、Tooltip、状态、错误、单位、文件大小和时间均使用 i18n key；禁止在新增 JSX 写死中英文。
- icon-only action 必须有 `aria-label`；每个图/地图有 title/summary；图片有 alt；失败消息用合适的 `role`/live region。
- 正文最低对比度达到 4.5:1；选中、错误、离线不只依赖颜色。
- 完成键盘流、屏幕阅读器摘要、深色/浅色、`prefers-reduced-motion`、最小窗口、100%/200% 缩放测试后才能标记 UI 阶段完成。

## 9. 设计文档同步规则

实现 UI 时，将可复用规则合并到最小范围的现有 `docs/design/` 文档：聊天布局进入 `shell-and-workspace-ui-spec.md` 或 `frontend-ui-guidelines.md`，按钮/菜单/Dialog 规则进入 `button-menu-design-spec.md`，Markdown/消息渲染细节同步 `docs/ui/06-markdown-message-tools.md`。更新对应“Last reviewed”日期，并在 [06-implementation-log.md](./06-implementation-log.md) 记录具体变更。
