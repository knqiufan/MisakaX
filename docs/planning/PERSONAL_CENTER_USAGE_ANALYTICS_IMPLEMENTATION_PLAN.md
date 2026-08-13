# MisakaX 个人中心与 Token 用量统计实施计划

> **用途：** 将个人中心、活动日历、按模型 Token 趋势与可靠用量计量拆成可验证、可独立审查的实施阶段和 Todo。
> **受众：** React、Rust、Python Sidecar、测试、设计与发布维护者。
> **最后审阅 / Last reviewed：** 2026-08-13
> **状态：** 实施中；P0–P1 已完成，P2 待开始。
> **规划基线：** `main@5569c45`，Schema v14。
> **实施分支：** `codex/personal-center-usage-analytics`
> **关联架构：** [个人中心与 Token 用量统计功能架构](../architecture/PERSONAL_CENTER_USAGE_ANALYTICS_ARCHITECTURE.md)

---

## 0. 实施进度与工作记录

### 0.1 阶段状态

| 阶段 | 状态 | 阶段提交 | 验证摘要 |
|---|---|---|---|
| P0 契约冻结与特征测试 | ✅ 完成 | `1876e4b` | Rust 定向测试 47 项通过（usage 10、MCP 10、Sidecar SSE 11、streaming 16） |
| P1 Schema v15 与数据访问层 | ✅ 完成 | 本阶段提交 `feat(usage): deliver P1 ledger repositories` | `cargo check --all-features`；数据库/迁移/消息/会话/Profile/Usage 回归 85 项通过 |
| P2 Token 采集闭环与原子终结 | ⏳ 待开始 | — | — |
| P3 聚合查询、IPC 与历史回填 | ⏳ 待开始 | — | — |
| P4 个人中心壳层、档案头与总览卡 | ⏳ 待开始 | — | — |
| P5 活动日历与绿色主题 | ⏳ 待开始 | — | — |
| P6 最近 30 天按模型趋势 | ⏳ 待开始 | — | — |
| P7 档案编辑、数据生命周期与性能 | ⏳ 待开始 | — | — |
| P8 QA、规范同步与交付 | ⏳ 待开始 | — | — |

### 0.2 工作日志

| 时间（Asia/Shanghai） | 阶段 | 记录 |
|---|---|---|
| 2026-08-13 14:20 | 启动 | 在干净保护现有四份任务文档后，从最新 `main@5569c45` 创建 `codex/personal-center-usage-analytics` 并恢复文档。 |
| 2026-08-13 14:34 | P0 | 新增 canonical usage/DTO/SSE v1 契约、streak 纯函数与现有 Rig/MCP/Sidecar 特征测试；冻结首版仅展示“每日”活动视图，未实现的每周/累计不进入 UI。 |
| 2026-08-13 14:44 | P1 | 新增 Schema v15、升级前 `.pre-v15.sqlite3` 备份、默认 local profile、追加式 usage ledger、弱引用与参数绑定仓储；两轮定向/兼容回归共 85 项通过。 |

### 0.3 首版 LLM operation 计入口径

| 调用点 | 当前代码入口 | totals | activity | trend | 首版处理 |
|---|---|---:|---:|---:|---|
| 普通聊天（Rig） | `commands/chat.rs` → `services/llm` / `services/mcp/tool_loop.rs` | 是 | 是 | 是 | P2 统一进入 assistant operation |
| 深度研究（Sidecar） | `commands/chat.rs` → `services/sidecar_sse.rs` → Python `/agent/stream` | 是 | 是 | 是 | P2 由 Sidecar 按 run/model 上报 |
| 自动会话标题 | `commands/chat.rs` → `RigBackend::prompt_once` | 是 | 否 | 是 | P2 记录 `session_title` |
| 模型探测 | `services/model_probe.rs` | 否 | 否 | 否 | 仅 diagnostics，不污染总览 |
| MCP 工具多轮 | `services/mcp/tool_loop.rs` | 合并到 chat | 合并到 chat | 合并到 chat | 多轮合计后只终结一次 |

---

## 1. 交付目标与冻结口径

### 1.1 用户可见目标

- 用户菜单的“个人中心”可进入独立页面。
- 页面顶部居中显示本地头像和名称，并可安全编辑。
- 页面下部显示总 Token、总使用天数、当前连续使用天数。
- 近一年活动日历使用绿色强度，鼠标和键盘都可查看某天 Token 详情。
- 最近 30 天按实际调用模型绘制多折线趋势；真正无调用的日期为 0、仅未知用量的日期留 gap，模型过多时合并“其他”。
- 精确值、估算值、历史迁移值、未知值、空状态、错误、浅/深主题和中英文都能被正确解释。

### 1.2 实施前不可随意改变的定义

- 统计唯一事实源为新增 `llm_usage_events`，不是 session totals 或前端消息列表。
- 总使用天数按 `counts_toward_activity` 的 `local_date` 去重。
- 当前 streak 的最近活动日必须是今天或昨天，否则为 0。
- 总 Token 不重复叠加 cache/reasoning 明细。
- 重生成会增加真实总消耗；删除消息/会话不默认抹去用量历史。
- Sidecar 内部调用只能由 Sidecar 采集/估算；Rust 不根据可见回复猜内部上下文。
- Profile 与 Dashboard 是不同 route；统计 feature 组件允许未来复用。

### 1.3 建议交付节奏

| 阶段 | 主题 | 建议工期 | 依赖 |
|---|---|---:|---|
| P0 | 契约冻结与特征测试 | 0.5–1 天 | 无 |
| P1 | Schema v15、Profile/Usage repository | 1–2 天 | P0 |
| P2 | Rig/Sidecar Token 采集闭环与幂等终结 | 2–3 天 | P1 |
| P3 | 聚合查询、IPC、历史回填 | 1.5–2 天 | P1–P2 |
| P4 | Profile 路由、档案头、统计总览 | 1.5–2 天 | P3 |
| P5 | 活动日历与绿色主题 | 1.5–2 天 | P3–P4 |
| P6 | 30 天模型趋势与 ECharts 降级 | 1–1.5 天 | P3–P4 |
| P7 | 编辑/清理、导入导出、隐私与性能 | 1.5–2 天 | P4–P6 |
| P8 | 全量 QA、规范同步与交付记录 | 1–2 天 | 全部 |

总量约 **12–18 个工程日**。Token 采集链路和迁移风险高于 UI 本身，不建议压缩为一个“大页面 PR”。

---

## 2. 依赖图与 PR 切片

```text
P0 contracts/tests
  ├─> P1 schema/repos
  │     ├─> P2 capture/finalize
  │     └─> P3 query/IPC/backfill
  │                 ├─> P4 profile shell/overview
  │                 ├─> P5 activity calendar
  │                 └─> P6 trend chart
  └────────────────────────────> P7 privacy/export/perf
                                      └─> P8 QA/release
```

建议 PR：

1. `usage-contract-schema`：领域类型、v15 migration、repos、migration tests。
2. `usage-capture-pipeline`：Rig/Sidecar usage、estimator、FinalizeTurn transaction。
3. `usage-query-ipc`：overview/activity/trend 聚合、前端 IPC 契约、legacy backfill。
4. `profile-shell`：local profile、route、UserMenu、TopBar、档案头和总览卡。
5. `profile-analytics-ui`：活动日历、趋势图、a11y、空态/错误态。
6. `usage-data-lifecycle`：清理、导入导出、性能基准、文档与全量验证。

每个 PR 都应可独立回滚，不能让“新 Schema 已写但旧代码不能启动”或“UI 已开放但默认 Sidecar 永远无数据”进入主分支。

---

## 3. Phase P0：契约冻结与特征测试

### 目标

先锁定当前行为和新统计口径，避免在改造双后端流式链路时丢消息、重复完成或破坏旧 TokenBadge。

### Todo

- [x] 在 Rust 定义 `UsageMeasurement`、`MeasurementSource`、`UsageOperationKind`、`UsageOutcome` 草案，并用 serde snapshot/unit tests 固定字段名。
- [x] 定义 `UsageDashboardV1`、`UsageOverviewV1`、`DailyUsageV1`、`ModelUsageSeriesV1`，显式区分 exact/estimated/legacy/unknown；Token 字段用十进制字符串跨 IPC，所有顶层响应带 `schema_version`。
- [x] 定义 Sidecar SSE `usage` v1 fixture：单模型、工具多轮、多个 run_id、重复 end、缺字段、异常负数。
- [x] 为当前 Rig 流程补特征测试：single round usage、MCP multi-round merge、abort 有/无 usage。
- [x] 为当前 Sidecar 流程补一个失败特征测试，明确现状 `usage = None`，随后在 P2 翻转为成功断言。
- [x] 固定 streak 测试表：今天、昨天、跨月、跨年、闰日、断档、时区边界。
- [x] 冻结首版时区策略：只跟随系统时区，事件用 `chrono::Local` 固化 local date/offset；IANA 名由 `Intl` 尽力提供。除非产品改为自定义时区，否则不新增 `chrono-tz` 类依赖。
- [x] 固定“总 Token 不重复加 cache/reasoning”的 invariant tests。
- [x] 记录所有 LLM operation call sites：chat、research、session title、model probe；明确首版计入 flags。
- [x] 在设计评审中确认“每日/每周/累计”是否首发全做；若不全做，未实现选项不得出现在 UI。

### 退出门

- DTO、SSE 与统计口径都有自动化测试或测试表。
- 团队不再使用“消息总和”和“真实消耗”这两个不同概念指同一指标。

---

## 4. Phase P1：Schema v15 与数据访问层

### 目标

建立本地 profile 和追加式用量账本，不改变现有用户可见页面。

### 代码落点

- `src-tauri/src/db/migrations.rs`
- `src-tauri/src/db/models.rs`
- `src-tauri/src/db/repository/{profile_repo,usage_repo}.rs`
- `src-tauri/src/db/repository/mod.rs`
- `src-tauri/src/db/mod.rs`
- `src-tauri/tests/{db_migrations_tests,usage_repo_tests,profile_repo_tests}.rs`

### Todo：migration

- [x] 新增 `migrate_v15`，创建 `user_profiles` 与 `llm_usage_events`、约束和索引。
- [x] `run_migrations` 增加 v15；`test_migration_idempotent` 期望更新为 15。
- [x] `init_database` 的 `backup_before_migration` 目标更新为 15，验证 `.pre-v15.sqlite3`。
- [x] 为 v14 fixture → v15 添加恢复/幂等测试；不得删除或重写现有 message/session 字段。
- [x] 首次启动创建默认 local profile，并写 `settings.profile.current_id`；重复启动不得产生多个默认 profile。
- [x] 默认 profile 写 `timezone_mode=system`；`timezone_id` 可空，`utc_offset_minutes` 在每条 usage event 上必填。
- [x] `local_date`、measurement source（含 `legacy_migrated`）、operation kind、outcome、三类计入 flags 和非负 Token 使用 CHECK。
- [x] session/message 弱引用采用 `ON DELETE SET NULL` 或经测试的逻辑约束；避免删除正文级联删除真实用量。

### Todo：repository

- [x] `ProfileRepo::get_current/create_default/update_display_name/update_avatar/clear_avatar`。
- [x] `UsageRepo::insert_batch_idempotent` 按 `measurement_key` 返回实际插入集合，调用方只按该集合更新 session projection。
- [x] `UsageRepo::find_by_operation_key` 供重放/诊断使用。
- [x] `UsageRepo::clear_profile_history` 必须在显式事务中执行。
- [x] 所有查询使用参数绑定；禁止把日期、profile 或排序片段直接拼接 SQL。
- [x] 测试 64 位大数、NULL usage、重复 measurement key、同 operation 多模型、配置删除后快照仍可读。

### 退出门

- v14 数据库可升级、可恢复、重复 migration 无副作用。
- 用量事件重复插入不会重复计数，删除会话后事件仍保留非正文统计。

---

## 5. Phase P2：Token 采集闭环与原子终结

### 目标

让默认 Sidecar 和 Rig 都能产出统一 usage，并通过一个事务完成消息、账本与 session projection。

### 代码落点

- `src-tauri/src/services/usage/{types,collector,estimator,finalize}.rs`
- `src-tauri/src/services/llm/{traits,streaming,backend}.rs`
- `src-tauri/src/services/{chat,sidecar_sse,sidecar_client}.rs`
- `src-tauri/src/services/mcp/tool_loop.rs`
- `src-tauri/src/commands/chat.rs`
- `agent/app/{usage,stream_content}.py`
- `agent/app/routers/agent.py`

### Todo：canonical 类型

- [ ] 合并/适配现有 `db::models::TokenUsage`、`StreamUsage`、`TokenUsageInfo`、`AgentTokenUsage`，避免字段继续漂移。
- [ ] 保留 input/output/total/cache read/cache creation/reasoning；未知用 `Option`，不是 0。
- [ ] 写 invariant：有 provider total 时以其为准；缺 total 但 input/output 都有时安全求和并检查溢出。
- [ ] measurement source 和 estimator 版本进入 message JSON 与 ledger，但 TokenBadge 仍兼容旧 JSON。

### Todo：Rig

- [ ] 将每轮 `GetTokenUsage` 映射为 canonical measurement。
- [ ] MCP 多轮按 operation 聚合；多轮全部相加，最终只写一个 message operation event。
- [ ] `prompt_once` 返回包含 usage 的 outcome；为 session title 记录 `counts_toward_activity=false` 事件。
- [ ] 中止时若 provider 返回部分 usage 正常记录；无值再尝试 estimator。
- [ ] 若 provider adapter 可提供 cache/reasoning，补充字段映射和 fixture。

### Todo：Sidecar

- [ ] 新建 `agent/app/usage.py`，从 chunk `usage_metadata`、response metadata、model end output 规范化 usage。
- [ ] 使用 run_id 去重同一次模型调用的 stream/end 信息，选择字段最完整版本。
- [ ] DeepAgents 多 run 聚合时保留实际 model；不同模型不得错误合并为主模型。
- [ ] `_stream_agent` 在 `done` 前发送 `event: usage` v1；异常/中止尽可能先发送已收集部分。
- [ ] Rust `MappedSidecarEvent` 新增 Usage，解析时校验 schema/非负/上限，写入 accumulator。
- [ ] Python 与 Rust 分别添加重复 run_id、工具调用、研究模式、缺失 metadata 测试。

### Todo：fallback estimator

- [ ] 定义 `TokenEstimator` port 与 model-family registry；模型匹配逻辑集中一处。
- [ ] 评估并锁定 tokenizer 依赖版本；实施前用该主版本官方文档/API 和本项目 manifest 复核。
- [ ] 对无精确 tokenizer 的 provider 实现版本化 heuristic，标记 `heuristic_estimated`。
- [ ] 为英文、中文、混合代码、长上下文、工具消息、图片附件创建 golden fixture；不得把图片字节换成精确 Token。
- [ ] 估算器异常时降为 `unavailable`，不能阻止 assistant 正文保存。

### Todo：FinalizeTurn transaction

- [ ] 新增 `FinalizeTurnService`：message finalize、usage insert、session projection update 在一个 SQLite transaction。
- [ ] `send_message` 与 `regenerate_message` 都调用同一 facade，移除分散的 `update_assistant_message` + `update_session_stats` 两步。
- [ ] `operation_key = assistant:{assistant_message_id}` 负责分组；`measurement_key = {operation_key}:model:{series_hash}:source:{source}` 负责幂等，不同模型或质量拆行、同模型同质量多轮聚合。
- [ ] 只有 measurement 实际插入时才把其 Token 累加到 session totals；message JSON 保存该 operation 下所有 measurement 的规范化合计。
- [ ] 事务 commit 后 emit `usage:recorded`；emit 失败只记 warn，不回滚已提交数据。
- [ ] 错误/中止路径把 placeholder 终结为稳定状态，不能永久停在 `streaming`。
- [ ] 验证重放两次 complete、命令重试和 listener 重订阅都不重复计数。

### 退出门

- Sidecar 与 Rig 都有端到端测试落出 usage event。
- 同一 assistant operation 重放不会重复累计。
- Token 采集失败不影响消息正文持久化，UI 能明确显示 unknown/estimated。

---

## 6. Phase P3：聚合查询、IPC 与历史回填

### 目标

提供一次快照读取的页面 DTO，并尽量诚实迁移 v14 历史数据。

### 代码落点

- `src-tauri/src/services/usage/query.rs`
- `src-tauri/src/commands/{usage,profile}.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/lib.rs`
- `src/lib/ipc/{usage,profile,types,index}.ts`
- `src/__tests__/{usage-ipc,profile-ipc}.test.ts`

### Todo：聚合查询

- [ ] 实现 overview：known/exact/estimated/legacy totals、按 distinct operation 计算的 unknown count、total days、current/longest streak。
- [ ] 实现 activity：连续 365 个本地日期补零，返回 exact/estimated/legacy/unknown operation counts。
- [ ] 实现 trend：只读取 `counts_toward_trend=true` 且模型已知的 measurement，连续 30 天补点、按 provider snapshot + effective model 分组；Top 5 按 known Token→operation count→stable key 排序，others 保留 known/unknown 质量分布。
- [ ] 日期序列和 streak 用 Rust 纯函数，覆盖闰年、DST、跨月/年、Monday/Sunday week start。
- [ ] 查询在一个只读快照/同一 DB lock 中完成，避免三个卡片口径不一致。
- [ ] 对请求 days/series 上限做后端校验；非法参数返回稳定错误，不 panic。

### Todo：IPC

- [ ] 注册 `profile_get_current`、`profile_update`、`usage_get_dashboard`、`usage_clear_history`。
- [ ] TypeScript DTO 与 Rust serde 字段建立 contract test；验证超过 `Number.MAX_SAFE_INTEGER` 的十进制 Token 字符串不失真，禁止页面直接使用 DB record。
- [ ] `usageIpc.getDashboard` 统一 camelCase→Tauri 参数映射，增加 mock 测试。
- [ ] `usage:recorded` 事件 payload 只包含 profile ID、operation key、occurred_at，前端只 debounce refresh。
- [ ] 若 Tauri capability/permission 需要显式项，生成最小 allow permission；不扩 shell/fs/http 权限。

### Todo：legacy backfill

- [ ] 解析可识别的 `messages.token_usage`，生成确定性 `legacy:message:{id}` 事件。
- [ ] 旧 JSON 缺 cache/source 字段时保留 NULL 并标为 `legacy_migrated`，不做假精度补全。
- [ ] session residual 的模型保持 NULL，且 `counts_toward_activity/trend=false`；不得进入活动或按模型逐日趋势。
- [ ] 收集坏 JSON、无法匹配日期/模型的诊断计数；migration 不因单行坏数据整体失败。
- [ ] 测试 legacy backfill 幂等、重复 import、不完整消息、session totals 小于 message sum 的异常情况。

### 退出门

- 一次 IPC 返回页面完整快照，365/30 天数组长度和日期连续性固定。
- 新数据、legacy 数据、未知数据的总览/日历/趋势口径可解释且有测试。

---

## 7. Phase P4：个人中心壳层、档案头与总览卡

### 目标

先开放真实可用的页面入口，完成页面骨架、档案和三项总览，不提前塞入假图表。

### 必读规范

- `docs/design/frontend-ui-guidelines.md`
- `docs/design/shell-and-workspace-ui-spec.md`
- `docs/design/button-menu-design-spec.md`
- `.cursor/rules/misaka-frontend-ui-specs.mdc`

### 代码落点

- `src/stores/app-store.ts`
- `src/components/layout/{ContentArea,UnifiedTopBar,UserMenu}.tsx`
- `src/pages/index.ts`
- `src/features/profile/*`
- `src/features/usage-analytics/{UsageOverviewCards,useUsageDashboard}.tsx`
- `src/locales/{zh-CN,en}/{nav,profile}.json`

### Todo：路由与入口

- [ ] `Route` 增加 `{ page: "profile" }`；补 app-store navigation test。
- [ ] `ContentArea` 和 pages index 接入 `ProfilePage`。
- [ ] `UnifiedTopBar.PAGE_TITLE_KEYS` 增加 profile；保持非 chat 的返回按钮。
- [ ] `UserMenu` 个人中心取消 disabled，导航到 profile；触发器名称/头像读取 profile。
- [ ] Profile 无左栏；`AppShell.showLeftColumn` 仍只包含 chat/settings。
- [ ] 保持 Dashboard route 与占位页不变，避免需求范围漂移。

### Todo：ProfileHeader

- [ ] 使用共享 Avatar primitive，目标尺寸 80px，加载失败回退到本地化首字/`U`。
- [ ] 名称居中、1–40 字符、长文本截断；无邮箱/会员/在线状态等虚假信息。
- [ ] 编辑入口使用共享 Button/Dialog、visible focus、i18n、pending disabled 和就地错误。
- [ ] 页面布局 `mx-auto max-w-6xl p-6 lg:p-10`，独立纵向滚动；顶部不机械占 50vh。

### Todo：总览卡

- [ ] 三张等权卡：总 Token、总使用天数、连续使用天数；宽屏三列，窄宽纵排。
- [ ] 总 Token 以 `bigint` 安全格式化 K/M/B，并保留 Tooltip/可访问全值；不得先转成 JS number 再累计。
- [ ] current streak 主显示，longest streak 放说明；含估算/未知次数以次级文字展示。
- [ ] Skeleton、empty、partial、error 高度稳定；查询失败显示就地重试。
- [ ] 不使用渐变、彩色发光、悬停上浮、active scale 或网页式营销卡。

### 测试

- [ ] UserMenu profile navigation、TopBar title/back、ContentArea route。
- [ ] 名称长文本、无头像、头像错误、loading/error/empty/partial overview。
- [ ] 浅/深主题快照或 DOM class 合约；375/768/1024/1440 宽度人工检查。

### 退出门

- 用户可稳定打开个人中心，看到本地档案和真实三指标。
- 默认 Sidecar 有数据时总览会更新；无数据时不显示假值。

---

## 8. Phase P5：活动日历与绿色主题

### 目标

实现参考图的年度活动感知，同时满足桌面可访问性和 MisakaX 主题规范。

### 代码落点

- `src/features/usage-analytics/{ActivityCalendar,ActivityCell,usage-calendar}.ts(x)`
- `src/styles/{theme-light,theme-dark}.css`
- `src/index.css`
- `src/locales/{zh-CN,en}/profile.json`

### Todo：日历算法

- [ ] 纯函数生成 7 行×52/53 周网格，正确处理范围起止、月标签、未来日期、locale week start。
- [ ] 计算 P95 + `log1p` 的 4 档强度；固定 fixture 保证单个 outlier 不压平其它天。
- [ ] 有已知 measurement 的活动日至少映射到 1 档绿色；已知值为 0、全区间为 0 时也不得与无活动混淆。
- [ ] 有活动但 Token unknown 生成独立 `unknown` visual state，不等同无活动。
- [ ] 单元测试 leap day、year boundary、range 365、Sunday/Monday、全零、单 outlier、全同值。

### Todo：视觉与交互

- [ ] 增加 `--usage-heat-0..4` 浅/深主题 token；绿色仅用于数据编码，不替换 primary/交互焦点色。
- [ ] Cell 维持最小可辨尺寸、暖灰空态、2–4px gap；卡片窄宽使用内部横向滚动。
- [ ] Tooltip 复用 `src/components/ui/tooltip.tsx`，显示日期、总 Token、input/output、调用数、主要模型、数据质量。
- [ ] Hover 不位移不缩放，只改 border/outline；focus ring 不被 overflow 裁剪。
- [ ] 图例显示“少 → 多”、未知样式和数值说明；颜色不是唯一信息来源。

### Todo：键盘与读屏

- [ ] Grid/row/cell 语义或等价可访问结构；每格有本地化 `aria-label`。
- [ ] roving tabindex：Tab 只进入当前 cell，方向键移动，Home/End 可选，Escape 关闭显式 popover（若使用）。
- [ ] Tooltip 在 focus 时可见；触屏/点击路径若未来支持，不能只依赖 hover。
- [ ] 提供按日期排序的可访问数据表/摘要入口，避免 Canvas/颜色成为唯一通道。

### Todo：可选聚合 Tabs

- [ ] 若产品确认首发“每日/每周/累计”，复用 Radix Tabs：每日热力图、每周聚合、累计线都接真实数据。
- [ ] 若任一视图未实现，删除该 Trigger；禁止 disabled/Coming soon 占位。
- [ ] 切换只改本地展示或使用同一快照，不反复请求相同数据。

### 退出门

- 365 天网格日期正确，鼠标/键盘都能读单日用量。
- 浅/深主题为绿色 4 档且对比可辨；未知/无活动不会混淆。

---

## 9. Phase P6：最近 30 天按模型趋势

### 目标

复用现有 ECharts 依赖，以稳定、可降级的方式呈现多模型 Token 变化。

### 代码落点

- `src/features/usage-analytics/{UsageTrendChart,UsageDataTable,usage-chart-options}.ts(x)`
- 可选提取 `src/components/charts/EChartCanvas.tsx`
- 调整 `src/features/chat-content/renderers/ChartBlockRenderer.tsx` 复用宿主，但不得引入 profile 领域耦合

### Todo

- [ ] 从现有 `ChartCanvas` 提取 ECharts 动态 import、init、ResizeObserver、dispose、error fallback 的无领域宿主。
- [ ] `usage-chart-options.ts` 纯函数生成 30 天 category xAxis、多 series、axis tooltip、scroll legend。
- [ ] 区分 no-call=0、unknown-only=NULL/gap、known+unknown=已知值加质量标记；Tooltip/表格显示 unknown operation count。
- [ ] Y 轴从 0 开始，axis label 用 K/M/B，Tooltip 保留原始整数与 estimated/unknown 说明。
- [ ] chart adapter 对安全整数直接转 number；超出时用 BigInt 比例缩放绘制并保留原值 Tooltip/表格，禁止静默截断。
- [ ] series key 对颜色做稳定映射；同名不同 provider 展示可区分 label。
- [ ] 最多 5 个模型 + others；legend 可隐藏系列，但不改变顶部总量。
- [ ] `animation=false` 或 150ms 并检测 reduced motion；禁止 smooth 过度导致小数据误读，可默认直线或弱 smooth。
- [ ] ECharts `aria.enabled` + decal；同时提供可切换/自动降级的数据表。
- [ ] 图表空态、只有一个点、全零、百万/十亿级、5+ 模型、Resize、主题切换均测试。
- [ ] 不新增 chart dependency，不在页面 bundle 启动时同步加载 ECharts。

### 退出门

- 最近 30 个自然日连续、各模型数据准确、日期缺口为 0。
- ECharts 加载失败时用户仍能读取相同数据表。

---

## 10. Phase P7：档案编辑、数据生命周期与性能

### 目标

补齐“能长期用”的编辑、清理、迁移、导入导出和性能边界。

### Todo：头像与名称

- [ ] 头像仅接受 PNG/JPEG/WebP、≤5 MiB；后端解码验证、限制像素、重新编码/缩放并存入 app-data。
- [ ] DB 只保存 storage key/hash；绝不保存或删除用户选择的原文件。
- [ ] 头像替换使用临时文件 + 原子 rename；失败保留旧头像；清理孤儿文件。
- [ ] `profile_update` 校验 Unicode 长度/空白；成功后 UserMenu 与 ProfileHeader 同步刷新。

### Todo：清空与删除语义

- [ ] “清空用量历史”使用标准 Dialog，说明不删除聊天正文且会清零统计投影。
- [ ] 清空事件与 session projection 在一个事务完成；失败回滚。
- [ ] 删除消息/会话的现有确认文案补充统计保留语义，或提供产品明确选择；不可静默改变。
- [ ] 测试清空后新事件可继续记录、旧 TokenBadge 仍可显示消息内 JSON 的预期行为。

### Todo：导入导出

- [ ] `ExportData.version` 升级并定义 usage event 可选字段，保持旧文件可导入。
- [ ] 使用 `source_installation_id + source_event_id` 去重；导入不重复累计。
- [ ] profile 仅导出名称、时区模式/标识等安全 metadata；头像 bundle 另立版本，不塞 base64 到普通 JSON。
- [ ] 导入 legacy session 的统计来源/质量进入 metadata。

### Todo：性能与可观测性

- [ ] 建立 1k/10k/100k usage events benchmark fixture；组合查询 P95 目标 <100ms。
- [ ] 检查 `EXPLAIN QUERY PLAN` 命中 profile/date/model 索引。
- [ ] 页面进入只取一次 dashboard；`usage:recorded` 事件 debounce，避免事件风暴。
- [ ] 记录 query duration、insert conflict、measurement source 分布；日志不含内容/路径/key。
- [ ] 若 100k 未达标，再设计可重建 daily rollup；不得提前双写第二事实源。

### 退出门

- 头像处理不越界、不破坏原文件；清空/导入/导出可恢复且无重复。
- 100k events 查询达到性能门槛或有经证据支持的 rollup 方案。

---

## 11. Phase P8：QA、规范同步与交付

### 自动化验证

- [ ] `npm run build`
- [ ] `npm test`
- [ ] `cargo fmt --check`
- [ ] `cargo check --all-features`
- [ ] `cargo test --all-features --test db_migrations_tests`
- [ ] 新增的 `usage_*` / `profile_*` 定向 Rust tests
- [ ] `cd agent && python -m pytest tests/test_stream_sse.py tests/test_agent.py tests/test_models.py`
- [ ] 提交前优先 `cargo nextest run --all-features --profile ci`；未安装或资源受限时按仓库指南串行回退，**不执行例行 `cargo clean`**。
- [ ] `git diff --check`

### 人工矩阵

| 维度 | 必测值 |
|---|---|
| 路径 | Rig、Sidecar chat、Sidecar research、MCP 多轮、abort、regenerate |
| 数据 | 无数据、仅精确、仅估算、混合、unknown、legacy、5+ 模型、超大数 |
| 日期 | 今天/昨天 streak、断档、跨月/年、闰日、系统时区变化 |
| 主题 | Light、Dark、System 切换、Reduced motion、Reduced transparency |
| 布局 | 375、768、1024、1440 宽；热力图内部滚动；窗口动态 resize |
| 输入 | 鼠标、键盘 Tab/方向键、屏幕阅读器名称、Tooltip focus |
| 生命周期 | 重启、重复 complete、删除会话、清空统计、导入重复文件、头像替换失败 |

### 文档同步

- [ ] 更新 `docs/design/shell-and-workspace-ui-spec.md` 的 profile 壳层、卡片、图表与活动色规范，并 bump Last reviewed。
- [ ] 若新增全局 usage tokens，更新 `docs/design/frontend-ui-guidelines.md`，不要把专页细节重复两份。
- [ ] 更新 `docs/project/PROJECT_STRUCTURE.md` 的 profile/usage 模块表。
- [ ] 实施完成后更新 `docs/project/DEVELOPMENT_STATUS.md`，写清 exact/estimated/legacy 覆盖与残余限制。
- [ ] 记录 migration 版本、测试数字、性能基准和手工矩阵证据。

### 退出门 / Definition of Done

- [ ] 用户菜单入口、Profile 页面、三指标、绿色活动日历、30 天模型折线全部真实工作。
- [ ] 默认 Sidecar 与 Rig 都能可靠写 usage；估算/未知不被伪装为精确值/0。
- [ ] operation 幂等、重生成、删除、清空、导入与时区行为通过测试。
- [ ] 无新增高风险 capability，无任意路径删除，无敏感内容进入 usage/profile 表。
- [ ] 浅/深主题、键盘、读屏、降级表格、响应式布局通过。
- [ ] 相关设计/架构/项目状态文档同步，代码与文档口径一致。

---

## 12. 测试用例清单（实施时逐项落地）

### 12.1 采集正确性

- [ ] provider total 与 input+output 不一致时按 provider total 保存并记录 metadata。
- [ ] cache read/create 是明细，不重复加入 total。
- [ ] Rig 两个 MCP round 的用量准确相加。
- [ ] Sidecar 同 run_id 的 stream/end usage 只计算一次。
- [ ] Sidecar 两个不同 run_id 计算两次；不同模型拆分 series。
- [ ] abort 有 partial usage、abort 无 usage fallback、failed before call 三种结果不同。
- [ ] title operation 计总量不计活动；model probe 默认两者都不计。

### 12.2 幂等与生命周期

- [ ] 同 operation 连续 finalize 两次，所有 measurement 行数不变，session delta 只应用一次。
- [ ] regenerate 保留旧 usage 并新增一条 usage。
- [ ] delete message/session 后 account total 不变化，引用被置空/弱化。
- [ ] clear history 后 ledger=0、session projections=0，消息正文仍在。
- [ ] migration/backfill/import 重放不产生重复事件。

### 12.3 日期与聚合

- [ ] 365 天包含今天且日期连续。
- [ ] 30 天每个 series 点数恒为 30。
- [ ] 今天连续 3 天 → current=3；最后活动昨天连续 3 天 → current=3；最后活动前天 → current=0。
- [ ] 2 月 29 日、12 月 31 日、DST 切换日不重复/丢失。
- [ ] unknown operation 计活动但不把 total 加 0；UI 明示 unknown。
- [ ] 同模型不同 provider 不合并；Top 5 之外精确进入 others。

### 12.4 UI 与可访问性

- [ ] 热力格 hover/focus 内容一致，方向键顺序符合 week grid。
- [ ] 颜色关闭/无法辨认时仍可从 label、图例、表格获取信息。
- [ ] 图表 legend 隐藏系列不改变源数据或总览。
- [ ] ECharts import reject 时显示数据表。
- [ ] 头像 alt/fallback、编辑按钮 aria-label、Dialog focus return 正确。
- [ ] loading/error/empty 不发生大幅布局跳动。

---

## 13. 风险登记与缓解

| 风险 | 级别 | 缓解 |
|---|---|---|
| LangChain provider usage metadata 形态不一致 | 高 | adapter + fixture；run_id 去重；unknown/estimated fallback |
| 旧 Sidecar 数据无法精确回填 | 高 | 只迁移可信值，明确 legacy/unknown，不伪造历史 |
| canonical usage 改造影响现有消息流 | 高 | P0 特征测试；Facade 渐进替换；保持旧 JSON 兼容 |
| 重生成/重放重复计数 | 高 | unique measurement key + 事务中仅 inserted 才更新投影 |
| 时区/streak 边界错误 | 中 | local_date snapshot + 纯函数 fixture + DST/闰日测试 |
| 365-cell 键盘体验差 | 中 | roving tabindex + 方向键 + 表格替代 |
| 模型系列过多导致图表不可读 | 中 | Top 5 + others、scroll legend、表格 |
| 头像文件引入路径/解码风险 | 中 | app-data 副本、MIME/大小/像素限制、原子替换、无任意删除 |
| 旧设计资料与现行规范冲突 | 中 | 以三份 `docs/design` 规范为准；绿色只作数据色，不恢复蓝紫品牌或浮动卡 |
| 统计查询未来变慢 | 低/中 | 索引 + 100k benchmark；证据不足前不引入 rollup |

---

## 14. 后续扩展 Backlog（不阻塞首版）

- [ ] 成本分析：版本化价格目录、币种、折扣、缓存价、价格生效时间和历史回算规则。
- [ ] 自定义范围：7/30/90/365 天与日期选择，但保持查询上限。
- [ ] Provider/operation filter：chat/research/title/diagnostics。
- [ ] 输入/输出/cache/reasoning 堆叠趋势。
- [ ] 全局 Dashboard 复用 UsageOverview/Trend，并增加会话/工具/Skills 指标。
- [ ] 多 profile / 账号同步：冲突合并、installation ID、事件 tombstone 与端到端加密。
- [ ] 预算与用量告警：本地阈值、通知去重、按 provider/model 预算。
- [ ] 可重建 daily rollup 与更长历史；账本继续作为唯一事实源。
