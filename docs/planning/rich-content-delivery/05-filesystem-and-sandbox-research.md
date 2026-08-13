# 富内容功能的文件系统与 Sandbox 调研结论

> **用途：** 回答“实现图表、地图、文件预览/下载和图片显示是否需要文件系统、Sandbox，以及应如何接入”的决策问题。
> **受众：** 架构、安全、Rust、Sidecar 和发布维护者。
> **最后审阅 / Last reviewed：** 2026-08-13
> **状态：** 已按 [Sandbox 方案复核](../../research/SANDBOX_STRATEGY_REASSESSMENT_2026-08.md) 更新富内容挂接方案并同步 ADR；S0 合同已实现，真实 Provider 仍待 Spike Gate，不构成已接受的生产承诺。

---

## 1. 执行结论

### 1.1 文件系统：需要，但不需要给 WebView 通用文件系统权限

生成文件、图片、PDF/Office 预览缓存、哈希校验、保留清理和用户另存为都需要可靠的本地字节存储。因此必须增加 **Rust 持有的应用专属 Artifact Store**。这不是 `@tauri-apps/plugin-fs` 的使用场景：项目已有自定义 Rust `fs_*` command，并且现行 capability/CSP 审计明确主 WebView 不应拥有通用 FS 权限。

Tauri 官方文档也区分了 Rust 侧可直接使用 `std::fs`/`tokio::fs` 与前端 FS plugin，并说明 plugin 的危险命令和 scope 默认被阻断。故推荐继续由 Rust application service 实现，只为明确的 artifact read/export 暴露窄 command/URI，保持默认 capability 不增加 `fs:*`。来源见 [Tauri File System](https://v2.tauri.app/plugin/file-system/) 与 [Tauri Permissions](https://v2.tauri.app/security/permissions/)。

### 1.2 Sandbox：展示功能本身不依赖完整执行隔离，但内容安全不可省略

纯 Markdown、受限图表 spec、静态 GeoJSON、raster 图片显示和已存档文件下载并不运行不可信宿主程序，因此不应等待完整跨平台执行隔离才交付。然而它们仍必须有：MIME/magic 检查、配额、哈希、解析上限、URI 授权、严格 CSP、无脚本 schema 和安全降级。这是 **内容安全面**，不能拿“未来有 Sandbox”取代。

严格执行/网络隔离是以下场景的硬依赖或强烈建议：

- Agent/Skill/MCP 使用 shell、编译器、图像/Office 转换器或其他外部程序产生文件；
- 不可信文件交给原生二进制解析器/转换器处理；
- 需要网络访问、地理编码、远程瓦片或第三方生成服务；
- 解析器需要隔离 CPU、内存、文件系统、环境变量和子进程树。

这些路径必须进入 Rust `Execution Isolation Broker`（沿用 `SandboxBroker` 名称），不能因为“只是生成预览”退回直接继承宿主环境的 subprocess。Broker 是策略、审批、snapshot、provider 选择、lease、审计与结果回收的控制面，不是某个 OS runner。strict 的跨平台共同语义是：

- 输入以工作区/artifact snapshot 交付，默认排除 `.git`、`.misakax`、`.codex`、`.agents`、凭据和越界链接，不直接写真实工作区；
- 网络默认拒绝，联网必须是 remote provider 或已经证明不可绕过的 `provider_brokered`，不能把 `HTTP_PROXY` 当安全边界；
- provider 返回受限 result manifest、artifacts 与实际 capability evidence，Rust 重新验证后才进入 ArtifactService；
- 用户确认 diff 后才允许写回真实工作区；provider 不可用时返回 `SANDBOX_UNAVAILABLE`，绝不自动选择 `host_direct`。

最新复核已否定两个旧默认假设：Windows 专用账户不再是默认本地方案，优先 Spike 无账户 AppContainer；macOS 动态 Seatbelt/`sandbox-exec` 不再作为正式 strict 边界，项目自带窄 helper 使用 App Sandbox + XPC，通用命令优先 remote microVM、后续可选本地 Linux VM。该建议已于 2026-08-13 纳入仍为 Proposed 的 [Sandbox ADR](../../architecture/SANDBOX_TECH_SELECTION.md)；S0 合同已实现，但真实 Provider 仍须过对应 Gate。

## 2. 按功能的决策矩阵

| 功能 | 受控本地文件系统 | 内容安全面 | 执行/网络隔离 | 网络/CSP | 首期决策 |
|---|---:|---:|---:|---:|---|
| Markdown/Mermaid | 否 | 是（现有 Markdown 防护） | 否 | 否 | 保持现状 |
| 统计图 | 可选（数据导出时需要） | 是，受限 ChartSpec | 否 | 否 | 本地 renderer + 数据表 |
| 静态 GeoJSON 地图 | 可选（GeoJSON 导出/缓存） | 是，受限 MapSpec/要素数 | 否 | 否 | 先支持离线数据 |
| 远程瓦片地图 | 是（缓存） | 是 | **是（网络位置）** | 是，必须受控 | remote/verified broker 获取 + artifact/custom URI 后再启用 |
| 模型 raster 图片 | 是 | 是（MIME/像素/解码限制） | 否 | 否 | Artifact Store + 图片 viewer |
| 生成文本/CSV | 是 | 是（大小/编码） | 否 | 否 | 本地预览 + 下载 |
| PDF/XLSX/DOCX 预览 | 是 | 是（解析配额） | 浏览器 parser 通常否；原生 helper/converter 是 | 否 | worker 优先；Windows AppContainer/macOS XPC/remote 未就绪时仅下载 |
| HTML/SVG/可执行文件 | 是（下载） | 是 | 是（若要处理/执行） | 默认否 | 首期仅下载；未来需独立 renderer 边界 + execution gate |
| Agent/Skill/MCP 外部生成 | 是 | 是 | **是** | 可能 | snapshot execution + manifest/artifact 回收 |

## 3. 当前项目与建议边界

### 3.1 当前安全基线

当前 `src-tauri/capabilities/default.json` 只提供事件、外链、dialog、clipboard、自定义 command 和窄 terminal runtime；`src-tauri/tauri.conf.json` 的生产 CSP 仅允许本地/asset/blob/data 图像和本地 IPC。`docs/guides/tauri-capability-csp-audit.md` 进一步确认 WebView 不应获得通用 FS、HTTP、Shell execute/spawn。

此基线是本功能的优势，不应为“容易预览文件”而撤销。新增资源路径应是如下闭环：

```text
模型/工具字节
  -> Rust ingress (validate/hash/quota)
  -> app-owned artifact store
  -> ArtifactRecord + message block
  -> scoped URI / narrow read DTO
  -> React preview

React download click
  -> artifact_id command
  -> native Save dialog
  -> Rust verified copy
```

前端不得得到工作区、`~/.misakax`、文件缓存或用户选择文件夹的原始可拼接路径。

### 3.2 URI 方案调研

Tauri 的 `asset:` protocol 可将磁盘文件传给 WebView，但必须在 `app.security.assetProtocol` 启用并为精确文件树定义 scope；官方文档特别警告不要把 allow 扩为 `$HOME/**/*` 或 `**/*`。对动态用户选择目录还需 persisted-scope，而这正是本设计要避免暴露的能力。来源：[Asset protocol scope](https://v2.tauri.app/security/asset-protocol/)。

建议做 R1 Spike 比较两条路径：

| 方案 | 优点 | 风险/约束 | 建议用途 |
|---|---|---|---|
| 窄 `asset:` scope | 已有 CSP 支持，适合本地图片/PDF资源 | 静态 scope，不能细化到会话授权；必须谨慎处理 dot directory | artifact root 固定、公共只读预览 |
| `misakax-artifact:` 自定义协议 | opaque ID、可检查 session/revision/TTL、可加 range 与审计 | 需实现 HTTP response/MIME/缓存和跨平台测试 | 默认优先；尤其文件/PDF/权限敏感场景 |

Tauri 2 支持注册自定义 URI protocol，并有异步版本避免阻塞主线程；Windows custom scheme 与 WebView2 版本有发布约束，需加入 installer/三平台验收。来源：[Tauri 2.0 release](https://v2.tauri.app/blog/tauri-20/)、[Windows installer minimum WebView2](https://v2.tauri.app/distribute/windows-installer/)。

## 4. 图表、地图与 WebView 安全

### 4.1 图表

Apache ECharts 可使用 `dataset` 与 `series.encode` 分离数据和视觉编码，适合将受限 `ChartSpec` 编译为图表 option。其 ARIA 组件需显式加载，才能生成辅助技术描述；首期必须同时提供数据表而不能只依赖图形。来源：[ECharts Dataset](https://echarts.apache.org/handbook/en/concepts/dataset/) 与 [ECharts ARIA](https://echarts.apache.org/handbook/en/best-practices/aria/)。

安全要求：不接受 `formatter` function、任意 callback、HTML tooltip 或网络数据 URL。ECharts 是渲染器，不是模型输出协议。

### 4.2 地图

MapLibre GL JS 是浏览器 WebGL 地图渲染库，支持 style/source/layer，但官方文档说明它依赖 `worker-src blob:`、`child-src blob:`、`img-src data: blob:`，严格 CSP 环境可使用专用 CSP worker bundle。项目当前 CSP 已有 worker/child blob 基础，仍需在 R4 Spike 测试本地 bundle、懒加载、WebGL 失败和 CSP 违反。来源：[MapLibre GL JS documentation](https://maplibre.org/maplibre-gl-js/docs)。

真正的额外风险是瓦片/style 的网络请求：若在 WebView 直接允许宽泛 `https:`，模型或数据源可诱导网络请求并扩大 CSP。故首期只处理本地 GeoJSON；后续瓦片只能引用 `tile_source_id`，由 Rust broker/custom URI 映射到已审批的 provider、密钥与缓存，保持主 WebView `connect-src` 的最小化。

## 5. 文件预览的安全边界

### 5.1 本地解析优于在线查看器

将 PDF/DOCX/XLSX 上传到在线查看器会泄露聊天产物和用户数据、受外部 CSP/iframe 限制、并引入服务可用性与合规依赖。因此不作为默认选项。PDF.js 的 display/viewer 层可用于构建本地浏览器预览，但需要打包本地资源、worker 与 page/timeout 限制；PDF.js 文档也指出本地 `file://` 不适合作为 worker viewer 加载方式。来源：[Mozilla PDF.js Getting Started](https://mozilla.github.io/pdf.js/getting_started/?lang=en)。

XLSX 可从受控 ArrayBuffer/Uint8Array 读取；浏览器环境不应按文件名任意读取本地路径。首期解析后只渲染值，不执行公式、宏、外部链接。来源：[SheetJS Data Import](https://docs.sheetjs.com/docs/solutions/input/) 与 [SheetJS Parsing](https://docs.sheetjs.com/docs/api/parse-options/)。

### 5.2 不可信活跃内容

HTML、SVG、JS、可执行文件、Office macro 和嵌入对象都可能含主动内容。首期只允许下载，不在聊天主 WebView 解析或 inline。如果后续确有需求，必须新建独立“主动内容预览”设计：隔离 webview/capability、无 Tauri IPC、禁止宿主凭据和网络；若使用不带 `allow-same-origin` 的最小 sandboxed iframe，它也只能是纵深防御。MDN 明确指出 iframe sandbox 一旦能在框架外打开内容就可能失效，因此不能把 iframe 当作唯一防线；涉及原生 helper/转换器时还必须同时通过 Execution Isolation Broker。来源：[MDN iframe sandbox](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/iframe)。

当前生产 CSP 的 `frame-src 'none'` 是有意的安全基线。任何未来 iframe 需求都要单独 ADR、CSP 差异审计和攻击测试，不可为 DOCX/HTML 预览直接放宽。

## 6. 与最新 Sandbox 复核的合并方式

### 6.1 决策状态

[Sandbox 方案复核](../../research/SANDBOX_STRATEGY_REASSESSMENT_2026-08.md) 已纳入 2026-08-13 修订后的 Proposed ADR。因此：

1. 不再为富内容后续阶段新增对 Windows 专用账户、macOS Seatbelt 或旧 `B0–B7` 平台顺序的实现依赖。
2. 不绑定 Provider 的合同、FakeProvider、snapshot/result manifest 与写回预检已作为 S0 实现；事务 apply/审批/持久审计留在 S0.5。
3. 真实 Provider、联网和 host-direct UX 必须等待相应 Spike 与发布 Gate；未通过即保持 `unavailable`。

### 6.2 富内容侧的统一合同

```text
Agent / Skill / MCP / native previewer
  -> authenticated typed execution intent
  -> Rust Execution Isolation Broker
       policy + approval + immutable ExecutionPlan
       WorkspaceSnapshotService + provider selection + lease/audit
  -> AppContainer / XPC helper / local VM / remote microVM
  -> restricted result manifest + artifacts + IsolationEvidence
  -> Rust validation
  -> ArtifactService + ContentSafetyPolicy
  -> block ready / diff review / explicit apply
```

必须增加并复用 `ExecutionLocation`、`WorkspaceDelivery`、`NetworkMode`、`IsolationEvidence` 与 `ProviderAvailability`。`SandboxMode=read-only/workspace-write/full-access` 不能单独表达 remote、snapshot 或 host-direct 的真实边界。模型、React、Sidecar 与 MCP 不得传平台参数、provider 凭据、原始 host path 或 shell 字符串。

### 6.3 Provider 与场景映射

| 富内容执行场景 | Provider 方向 | 发布前证据 |
|---|---|---|
| Windows 离线 parser/converter | 无账户 AppContainer/LPAC + restricted token + Job Object；`CreateProcessInSandbox` 仅实验分支 | Home/Pro/标准用户、文件/凭据/IPv4/IPv6/DNS/loopback/进程树/实验 API 缺失攻击测试 |
| macOS 项目自带 parser helper | 签名 App Sandbox + XPC，artifact-only I/O，无 network entitlement | 签名/entitlement、bookmark、输入输出目录、环境/凭据、notarization 实机测试 |
| macOS 或跨平台通用命令 | remote Linux microVM；本地 Linux VM 为后续隐私选项 | 地区/保留期/egress/销毁/manifest；本地 VM 另测 Intel/Apple silicon、镜像和无 writable host share |
| 瓦片/地理编码/第三方生成 | remote provider 或已验证 brokered network | 域名/协议/端口/TTL、DNS redirect/private IP/metadata endpoint、密钥不注入、缓存与审计 |
| 宿主专有工具 | 用户每次明确批准的 `host_direct` | 明示无 OS Sandbox；理由/范围/TTL 审计；绝不由 provider 错误 fallback |

### 6.4 与 R 阶段的挂接

1. **R0/R1：** `ContentSafetyPolicy` 与 execution policy 独立；Artifact Store、窄 URI 和导出不等待 provider。
2. **R2：** 浏览器 parser 先完成资源/恶意 fixture；原生 helper/converter 先通过目标执行位置 Gate，否则仅下载。
3. **R4b：** 只有 remote 或 verified brokered network 可获取瓦片/地理编码，再经 ArtifactService/cache 交付；主 WebView 继续无任意网络。
4. **R5：** 非执行 normalizer/event 可独立推进；外部生成必须完成 S0 合同（FakeProvider、snapshot、manifest、host-direct 显式建模），并按位置等待 S1 remote、S2 Windows 或 S3 macOS Gate。
5. **R6：** 按 `IsolationEvidence` 对每条功能路径分别发布；默认启用/企业交付还要通过 S4 的组织 provider、地区、镜像 digest、密钥代理、审批、修复/销毁和独立安全评审。不能用另一平台、另一工具或“provider 已编译”代替当前路径证据。

## 7. 必须纳入计划的验证项

- [ ] asset/custom URI 不能读取 artifact root 以外的任何文件，不能用 `..`、编码路径、符号链接、junction 绕过。
- [ ] 任何无 artifact ID、无会话归属、过期 revision、错误 MIME 的请求均失败且不泄露真实路径。
- [ ] `dialog:allow-save` 下载取消、同名、Unicode、长路径、磁盘不足、只读目录在三平台行为可解释。
- [ ] PDF/XLSX/DOCX/图片畸形样本、解压炸弹、超尺寸、超时、取消不会卡死 UI/主进程。
- [ ] Chart/Map spec 不存在可执行 function、HTML、任意 URL 注入；图表/地图不放宽主 WebView CSP。
- [ ] 触发外部 converter、Agent shell、Skill/MCP helper 时只有 Execution Isolation Broker 路径可用；结构化请求不含平台配置、原始 host path 或 shell 拼接。
- [ ] strict snapshot 排除敏感目录/凭据/越界链接；result manifest 的路径、hash、大小、删除范围与 base generation 经 Rust 校验，未确认 diff 不写回。
- [ ] Windows AppContainer、macOS XPC/VM、remote provider 分别通过对应攻击 fixture；Seatbelt、专用账户或本地容器的存在不计作默认 strict 证据。
- [ ] 默认无网；允许网络时 DNS/redirect/private IP/metadata/IPv4/IPv6/loopback/child 绕过测试有证据，普通 proxy 环境变量不计入。
- [ ] provider 缺失、实验 API 变化、签名/镜像 hash 失效、地区不符、配额耗尽与取消失败均可诊断，且不会回退 `host_direct`。

## 8. 最终建议

继续把 “Artifact Store + 内容安全策略 + 窄 URI/导出” 作为 R0/R1 的独立前置项。对外部执行、受控网络和高风险原生解析器，不再等待一个抽象的“全平台同构 OS Sandbox”，而是先完成统一 Broker/snapshot/result contract，再按实际执行位置通过 remote、Windows AppContainer、macOS XPC/VM 的独立 evidence gate。这样既不阻塞安全的静态富内容，也不会用弃用机制、实验 API、普通代理或宿主 fallback 伪装成已完成隔离。
