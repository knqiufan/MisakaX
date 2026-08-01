# Skills 仓库、设置集成与安全检查实施计划

> **用途：** 将 Skills 迁入 Settings，重做详情文件浏览，补齐全来源启停闭环，并建立安装前安全检查。
> **受众：** React、Rust、Python Sidecar、SQLite、测试与安全维护者。
> **最后审阅 / Last reviewed：** 2026-08-01
> **规划基线：** `main@fa24bd7`。
> **关联：** [总体架构](../architecture/WORKSPACE_SKILLS_SECURITY_ARCHITECTURE.md) · [扫描调研](../research/SKILL_SECURITY_SCANNING_RESEARCH.md) · [UI 设计](../design/SKILLS_WORKSPACE_TERMINAL_UI_DESIGN.md)

---

## 1. 范围

### 1.1 本计划交付

- 受管和外部已安装 Skills 都可启用/禁用，状态持久化。
- 禁用后从启动元数据、自动选择、手动选择、当前 composer chip、消息发送和 Sidecar 挂载中消失。
- Skills 从独立页面迁入 Settings，位于 MCP 下方。
- 详情默认只展示概要、文件树和安全状态；点击文件才加载有上限的内容。
- 本地上传、在线下载和外部发现统一经过安全检查。
- 扫描报告、finding、策略裁决、人工复核、重新扫描和文件变更失效。
- 存量 Skills 扫描迁移和兼容 API 下线。

### 1.2 不在本计划

- Skill 在线发布、评论、评分、购买或账号体系。
- 自动执行 Skill 脚本的独立运行器；运行时隔离由 Sandbox 计划交付。
- 把扫描结果描述为绝对安全证明。
- 在线目录协议的全面重写；只增加安全 envelope 和缓存边界。

## 2. 验收标准

1. 任何来源的 Skill 都有稳定 `skill_id` 和持久启用状态。
2. 后端是启用有效性的唯一裁决者；篡改前端 payload 也不能使用无效 Skill。
3. `enabled=false` 后，同一事件周期内选择器和 chips 清理；下一次 Agent 请求的 activation view 不包含该 Skill。
4. 详情首屏网络/IPC payload 不包含 `SKILL.md` 正文；不点击文件时正文读取次数为 0。
5. 单文件读取不能越过 Skill 根、跟随逃逸链接或一次返回无界内容。
6. 本地、远端和外部路径都经过 `SkillSecurityGate`；未扫描/过期/阻止时不能启用。
7. 文件被修改后旧 scan hash 失效并自动禁用 effective activation。
8. Settings > Skills 在键盘、窄窗口、深链和独立滚动场景通过 UI 回归。
9. 所有旧消息和已安装受管 Skill 在迁移后可读；升级失败可回滚数据库。

## 3. 实施策略

采用 strangler/branch-by-abstraction：保留现有 `SkillInstaller` 和 commands 作为外观，逐步把身份、扫描、激活拆到新服务。每个阶段先加新 API，再切 UI/Sidecar，最后删除旧字段，避免同一提交同时改数据库、路由、UI 和 Agent 协议。

Feature flags 建议：

- `skills_settings_tab_v2`
- `skills_lazy_file_preview`
- `skills_security_gate`
- `skills_activation_view`
- `skills_deep_scanner`

开发版可单独启用；发布版只启用通过对应 gate 的组合。

## 4. Phase S0：特征测试与契约冻结

### 4.1 工作内容

- 为当前 list/detail/install/enable/select/send/mount 行为建立特征测试，记录预期和已知缺陷。
- 定义统一 `AppError` code、SkillId、scan DTO、event envelope 和 generation。
- 记录现有 `skills`、`message_skill_selections` 数据迁移样本。
- 为 `SkillDetail.skill_markdown` 标记 deprecated；新 UI 不再依赖。

### 4.2 TODO

- [x] Rust：补受管 Skill enable/disable、安装覆盖、损坏文件、同名来源冲突的特征测试。
- [x] Rust：补外部 Codex/Claude/Cursor 发现和优先级测试。
- [x] Rust：补 `installed_selection` 对 disabled/unhealthy 的服务端拒绝测试。
- [x] Python：补全目录 `/skills` 挂载会暴露 disabled Skill 的回归用例，先以 expected-failure 标记根因。
- [x] React：补 `SkillSelectorPopover`、slash menu、chips 在 inventory 变化时的测试。
- [x] React：补独立 Skills 路由和 Settings 导航快照/交互基线。
- [x] 定义 `SkillId`、`ScanState`、`ScanDecision`、`FindingSeverity` 和错误码文档/TS 镜像。
- [x] 增加 feature flag 读取和测试 helper，默认保持旧行为。
- [x] 记录迁移前数据库 fixture，并验证 rollback fixture。

### 4.3 退出门

- 旧行为测试稳定；新契约经 Rust/TS/Python 三方 review。
- 能明确看到“UI disabled 但 Sidecar 目录仍可见”的失败测试，后续用 activation view 关闭。

## 5. Phase S1：稳定身份、全来源开关与激活视图

### 5.1 数据库迁移

建议新增 `skill_sources`，而不是继续让外部 Skill 每次列表时临时构造：

```sql
CREATE TABLE skill_sources (
  id TEXT PRIMARY KEY,
  slug TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  source_locator TEXT NOT NULL,
  managed INTEGER NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 0,
  health TEXT NOT NULL,
  artifact_hash TEXT,
  current_scan_id TEXT,
  disabled_reason TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(source_kind, source_locator)
);
```

实际 migration 需沿用项目 SQLite timestamp/foreign key 约定。现有 `skills` 数据映射到 stable ID；外部 source 首次发现写入 Registry，默认禁用/待扫描，不修改其原始目录。

### 5.2 启用服务

引入 `SkillActivationService`：

- `set_user_enabled(skill_id, bool)` 只记录用户意图。
- `evaluate_effective(skill_id)` 结合健康、扫描 hash、策略和撤销状态。
- `create_activation_view(session_id)` 产生只读 manifest/映射，只暴露有效 Skill。
- `validate_selection(session_id, skill_ids, generation)` 发送前再次验证。
- 状态变化发送 `skills.activation.changed`，前端清理 chips，Sidecar 更新 generation。

### 5.3 TODO

- [x] 新增 `skill_sources` migration、repo 和 rollback 测试。
- [x] 为已有受管 Skills 生成稳定 ID；保留 slug 查询兼容 facade。
- [x] 外部扫描结果写 Registry，不再在 list 时构造 `enabled=true` 临时记录。
- [x] 实现来源冲突排序和 UI 可解释的 `effective_rank/conflict` 字段。
- [x] 扩展 `message_skill_selections` 支持 `skill_id`、slug/hash snapshot，先 dual-write。
- [x] 实现 `SkillActivationService` 和 `effective_active` 规格测试。
- [x] 实现会话 activation manifest/generation；目录或虚拟映射必须只读。
- [x] Python `build_agent` 改为读取 activation view，不再挂载整个 `settings.skills_dir`。
- [x] Python selection path 改用 stable ID + generation，保留一次旧 slug 协议兼容。
- [x] 后端在 enable、selection、mount 三个入口调用同一 `SkillSecurityGate`。
- [x] inventory/activation 事件使 composer 自动移除刚禁用的 chip，并显示一次非阻塞说明。
- [x] 增加并发测试：扫描/禁用与发送同时发生时，旧 generation 必须被拒绝。
- [x] 增加外部文件删除、重命名、同名来源新增和应用重启后的持久化测试。

### 5.4 退出门

- disabled Skill 在 UI、消息 payload 和 Python `/skills` 视图中均不可见。
- 不修改外部目录也能可靠禁用；重启后状态保持。

## 6. Phase S2：Settings 迁移与详情按需文件浏览

### 6.1 路由与组件

- `SettingsTab` 增加 `skills`，导航位于 MCP 后。
- `SkillsPage` 的领域内容抽为 `SkillsSettingsFeature`；旧 route 只做重定向。
- Settings 内容 shell 增加 tab 级 `contentWidth: normal | wide`，Skills 使用 wide。
- 保留现有 toolbar、inventory hooks 和列表分页，避免重复实现在线发现。

### 6.2 API 拆分

```text
skills_get_summary(skill_id)
skills_list_files(skill_id, parent?, cursor?, limit?)
skills_read_file(skill_id, path, offset=0, limit<=200KiB)
skills_get_scan_summary(skill_id)
skills_list_findings(scan_id, filter, cursor)
```

`skills_get_remote_detail` 改为远端 manifest/tree 摘要或缓存查询，不为普通详情自动下载并解压整个制品。完整 artifact 只在安装/显式扫描流程下载到隔离区。

### 6.3 文件读取防护

- 后端从 Registry 解析根，不接受前端绝对路径。
- 清理 `.`/`..`，拒绝绝对路径、alternate stream、Windows device path、链接/junction 逃逸。
- canonical root containment 校验在打开文件前和打开后尽可能做抗竞态验证。
- 只读取普通文件；文件大小、单次读取、总预览和并发请求有限制。
- 文本编码明确；二进制只返回 metadata，不以 Base64 塞入详情 DTO。
- Markdown/HTML 预览禁用脚本、iframe、远端图片和协议链接自动执行。

### 6.4 TODO

- [x] 更新 app-store route union，加入 Settings `skills` tab 和旧 route redirect。
- [x] 更新 Settings nav 顺序、i18n、图标、选中态和 deep-link 测试。
- [x] 抽取 `SkillsSettingsFeature`，移除对顶层 `SkillsPage`/route 的硬依赖。
- [x] 实现 Settings tab 宽度 descriptor，不改变其他 tab 的阅读宽度。
- [x] 新增 summary/file tree/read file IPC DTO 和薄 commands。
- [x] 为本地、外部和缓存远端 source 实现统一 `SkillFileProvider` adapter。
- [x] 加入路径逃逸、junction/symlink、ADS、Unicode、大小、编码和读取竞态测试。
- [x] `SkillDetailPanel` 改为固定 header + `文件/安全/概览` tabs，默认 Files。
- [x] Files tab 实现文件树和独立预览 pane；没有选择时不请求任何正文。
- [x] 点击文件加载，切换时取消/丢弃旧 generation；大文件分段继续加载。
- [x] 二进制和不支持编码显示 metadata 空状态，不直接渲染。
- [x] 安全 tab 先接 scan summary 占位契约，随后接 findings。
- [x] 删除 UI 对 `skill_markdown` 的读取，统计新详情首屏 payload/IPC 次数。
- [x] 补键盘 tree、tabs、switch、窄窗口、独立滚动、长文件和 500 文件性能测试。
- [x] 更新三份现有 UI 规范的相关章节和 Last reviewed。

### 6.5 性能预算

- 详情概要 P95 < 150 ms（本地 SSD、100 Skills 基线）。
- 文件树首批 P95 < 200 ms；超大树分页/按目录加载。
- 未点击文件时正文传输 0 byte。
- 单次预览默认不超过 200 KiB；UI 主线程长任务 < 50 ms。

## 7. Phase S3：安全扫描基础设施与强制安装门

### 7.1 存储和状态

- 下载/上传先到 `quarantine/<scan_id>/artifact`，hash 后再解包。
- `skill_artifacts` 去重制品；`skill_scan_runs` 保存引擎/策略版本；`skill_findings` 保存结构化证据。
- 状态机支持 queued/scanning/passed/warnings/review_required/blocked/error/stale。
- 所有安装入口只接收 `ApprovedArtifactId`，不能直接传任意临时路径绕过。

### 7.2 内置引擎

在现有 `archive.rs` 上扩展，但不要让一个文件承担全部职责：

- `archive_validator`
- `manifest_analyzer`
- `content_inventory`
- `secret_analyzer`
- `command_rule_analyzer`
- `permission_consistency_analyzer`
- `policy_engine`

扫描器只读输入，绝不执行脚本。

### 7.3 策略

默认 balanced：

- Critical/High 阻止。
- Medium 或高不确定性进入人工复核，批准后默认仍以 disabled 安装，再由用户启用。
- Low/Info 允许并展示 warnings。
- engine error/timeout 对新安装 fail closed。
- 已安装存量在迁移宽限期显示“待扫描”并禁用自动注入；用户可批量扫描。

### 7.4 TODO

- [x] 新增 artifact/scan/finding/approval migrations、repo、索引、清理策略和 rollback 测试。
- [x] 实现 quarantine 配额、崩溃恢复、过期清理和原子发布。
- [x] 把本地 install、remote install 和 external discover 都路由到 `SkillSecurityGate`。
- [x] 拆分现有 `archive.rs`，保留已验证的 ZIP 安全规则并加嵌套归档/MIME/Unicode 检查。
- [x] 实现统一 `Finding` schema、fingerprint、severity 和 remediation。
- [x] 实现 Secret、危险命令、下载执行、持久化、混淆、凭据访问和权限不一致规则。
- [x] 建立 versioned policy config 和 `allow/review/block` 单元测试表。
- [x] 实现扫描队列、取消、超时、并发上限、progress 事件和应用重启恢复。
- [x] 实现“扫描通过 != 永久有效”：hash/规则/策略/source revocation 触发 stale。
- [x] 文件 watcher debounce 自动触发状态更新；选择/启用时仍同步校验 hash 防漏报。
- [x] 实现人工批准/拒绝/撤销，记录 actor、reason、scope、expires 和 correlation ID。
- [x] UI 接 findings 分页、过滤、证据、修复建议、重新扫描和隐私提示。
- [x] VirusTotal 只查 hash；文件上传与云 LLM 扫描默认关闭并单独确认。
- [x] 构建恶意/良性样本 corpus，覆盖三平台脚本、Prompt Injection、外传、持久化、混淆、路径和压缩攻击。
- [x] 增加 SARIF/JSON 导出，确保不泄露 Secret 原文。
- [x] 增加性能/DoS 测试：大树、长行、复杂正则、嵌套编码、超时与取消。

### 7.5 退出门

- 三种来源 100% 经过 gate；通过直接调用旧 command 无法旁路。
- 高危样本按策略阻止，良性基线误报率达到团队约定门槛。
- 扫描器崩溃不影响主库完整性，制品不会提前进入激活目录。

退出证据记录于 [`WORKSPACE_SECURITY_SKILLS_PROGRESS.md`](../project/WORKSPACE_SECURITY_SKILLS_PROGRESS.md) 的 S3 条目；第三方 deep scanner 的 Sandbox helper 隔离不属于本阶段，也未被内置扫描器绕过。

## 8. Phase S4：深度扫描器与供应链信号

### 8.1 第三方 PoC

优先验证固定版本 Cisco AI Skill Scanner，以独立 helper/JSON adapter 运行；禁止网络、输入只读、输出大小受限。业务层只认识内部 `Finding`，不依赖第三方 Python 类型。

PoC 评价：三平台打包、Nuitka 兼容、启动/扫描耗时、内存、规则覆盖、误报、许可证/SBOM、更新策略和离线行为。若直接并入 Agent Sidecar 过重，改为独立可选 scanner helper。

### 8.2 TODO

- [ ] 记录第三方许可证、传递依赖、SBOM 和精确版本锁。
- [ ] 实现 `DeepScanEngine` JSON 协议、schema version、超时和输出上限。
- [ ] 在 Sandbox read-only/offline profile 运行 helper，验证其不能执行目标 Skill。
- [ ] 将第三方 rule/severity 映射到内部 taxonomy，保留原始 engine/rule ID。
- [ ] 对 corpus 比较内置、Cisco 和人工结果，记录 precision/recall 近似指标。
- [ ] 评估 OSV adapter；离线无缓存时显示 unknown，不误报 passed。
- [ ] 评估 VirusTotal hash adapter；默认不上传未知文件。
- [ ] 评估 LLM analyzer 的隐私、成本、脱敏和 consensus；保持可选。
- [ ] 制定 scanner/rule 更新签名、回滚和紧急撤销流程。
- [ ] 完成 Windows/macOS/Linux 安装包和冷启动回归后决定内置或可选分发。

## 9. Phase S5：存量迁移与旧路径清理

### 9.1 TODO

- [ ] 首次升级扫描所有已知受管/外部 source，展示批量进度和失败恢复。
- [ ] 宽限期内存量设为 `unscanned`，不自动注入；保留用户手动查看/删除能力。
- [ ] 验证所有活跃会话已使用 activation generation。
- [ ] 停止 dual-write slug-only selection，迁移历史消息并做完整性检查。
- [ ] 删除 UI 和新 API 的 `skill_markdown` 字段；旧 command 经一个版本弃用后移除。
- [ ] 删除独立 Skills route/title/nav 分支和不再使用的页面 wrapper。
- [ ] 删除 Python 整目录挂载和生产 `LocalShellBackend` Skills fallback。
- [ ] 清理旧 `risk_json` 的兼容读取或迁移到 scan summary。
- [ ] 更新 README、PROJECT_STRUCTURE、DEVELOPMENT_STATUS 中已过时的 Skills 进度。
- [ ] 完成升级/降级、数据库备份恢复和外部目录不可写测试。

## 10. 测试矩阵

| 层 | 必测内容 |
|---|---|
| Rust unit | 状态机、policy、路径、hash、来源冲突、finding 映射 |
| Rust integration | migrations、quarantine、安装原子性、commands 无旁路、events |
| Python | activation view、generation、disabled/missing/changed、broker error |
| React unit | Settings route、Switch、tabs、lazy read、旧请求丢弃、chips 清理 |
| E2E | 本地/远端/外部安装，扫描、启用、禁用、重启、文件变化、阻止/批准 |
| Security | ZIP bomb、traversal、junction/symlink、Prompt Injection、Secret、下载执行、DoS |
| Performance | 100 Skills、500 文件/Skill、200 KiB 分段、并发扫描和事件风暴 |

## 11. 发布与回滚

- P1 先在开发渠道 audit-only 收集误报，不上传任何内容。
- 新安装 gate 可以先强制，存量使用短期迁移窗口；窗口结束前不允许自动注入未扫描 Skill。
- 数据迁移前自动备份数据库；artifact 目录使用 content-addressed storage，回滚不删除用户原始外部目录。
- feature flag 回滚只能恢复 UI/查询路径，不能绕过已启用的高风险阻止策略。
- 遇到规则误报用签名 policy hotfix/例外，不直接关闭整个扫描系统。

## 12. 完成定义

- 所有 TODO 已关闭或在进度文档中有明确延期理由和新负责人。
- 验收标准、三平台 E2E、安全 corpus、升级/回滚全部通过。
- 旧全文详情和整目录 Skills 挂载从生产路径删除。
- UI 规范、架构、项目结构和实施进度文档同步。
