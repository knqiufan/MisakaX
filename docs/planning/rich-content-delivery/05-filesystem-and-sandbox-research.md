# 富内容功能的文件系统与 Sandbox 调研结论

> **用途：** 回答“实现图表、地图、文件预览/下载和图片显示是否需要文件系统、Sandbox，以及应如何接入”的决策问题。
> **受众：** 架构、安全、Rust、Sidecar 和发布维护者。
> **最后审阅 / Last reviewed：** 2026-08-08
> **状态：** 调研结论已纳入实施计划；需在实现 Spike 中验证 Tauri URI 与三平台行为。

---

## 1. 执行结论

### 1.1 文件系统：需要，但不需要给 WebView 通用文件系统权限

生成文件、图片、PDF/Office 预览缓存、哈希校验、保留清理和用户另存为都需要可靠的本地字节存储。因此必须增加 **Rust 持有的应用专属 Artifact Store**。这不是 `@tauri-apps/plugin-fs` 的使用场景：项目已有自定义 Rust `fs_*` command，并且现行 capability/CSP 审计明确主 WebView 不应拥有通用 FS 权限。

Tauri 官方文档也区分了 Rust 侧可直接使用 `std::fs`/`tokio::fs` 与前端 FS plugin，并说明 plugin 的危险命令和 scope 默认被阻断。故推荐继续由 Rust application service 实现，只为明确的 artifact read/export 暴露窄 command/URI，保持默认 capability 不增加 `fs:*`。来源见 [Tauri File System](https://v2.tauri.app/plugin/file-system/) 与 [Tauri Permissions](https://v2.tauri.app/security/permissions/)。

### 1.2 Sandbox：展示功能本身不依赖完整 OS Sandbox，但内容安全不可省略

纯 Markdown、受限图表 spec、静态 GeoJSON、raster 图片显示和已存档文件下载并不运行不可信宿主程序，因此不应等待完整跨平台 Sandbox 才交付。然而它们仍必须有：MIME/magic 检查、配额、哈希、解析上限、URI 授权、严格 CSP、无脚本 schema 和安全降级。这是 **内容安全面**，不能拿“未来有 Sandbox”取代。

完整 OS Sandbox 是以下场景的硬依赖或强烈建议：

- Agent/Skill/MCP 使用 shell、编译器、图像/Office 转换器或其他外部程序产生文件；
- 不可信文件交给原生二进制解析器/转换器处理；
- 需要网络访问、地理编码、远程瓦片或第三方生成服务；
- 解析器需要隔离 CPU、内存、文件系统、环境变量和子进程树。

这些路径必须复用已有 [Sandbox ADR](../../architecture/SANDBOX_TECH_SELECTION.md) 的 `SandboxBroker`、平台 provider、网络 policy、审批与审计；不能因为“只是生成预览”退回直接继承宿主环境的 subprocess。

## 2. 按功能的决策矩阵

| 功能 | 受控本地文件系统 | 内容安全面 | OS Sandbox | 网络/CSP | 首期决策 |
|---|---:|---:|---:|---:|---|
| Markdown/Mermaid | 否 | 是（现有 Markdown 防护） | 否 | 否 | 保持现状 |
| 统计图 | 可选（数据导出时需要） | 是，受限 ChartSpec | 否 | 否 | 本地 renderer + 数据表 |
| 静态 GeoJSON 地图 | 可选（GeoJSON 导出/缓存） | 是，受限 MapSpec/要素数 | 否 | 否 | 先支持离线数据 |
| 远程瓦片地图 | 是（缓存） | 是 | 不一定 | 是，必须受控 | registry + Rust broker 后再启用 |
| 模型 raster 图片 | 是 | 是（MIME/像素/解码限制） | 否 | 否 | Artifact Store + 图片 viewer |
| 生成文本/CSV | 是 | 是（大小/编码） | 否 | 否 | 本地预览 + 下载 |
| PDF/XLSX/DOCX 预览 | 是 | 是（解析配额） | 视解析器而定 | 否 | 本地 parser；高风险/原生转换后接 Sandbox |
| HTML/SVG/可执行文件 | 是（下载） | 是 | 是（若要处理） | 否 | 首期仅下载，禁止内联预览 |
| Agent/Skill/MCP 外部生成 | 是 | 是 | **是** | 可能 | 进入已有 Sandbox 阶段 |

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

HTML、SVG、JS、可执行文件、Office macro 和嵌入对象都可能含主动内容。首期只允许下载，不在聊天主 WebView 解析或 inline。如果后续确有需求，必须新建独立“内容预览 sandbox”设计：隔离 webview/capability、无 Tauri IPC、禁止宿主凭据和网络，或使用不带 `allow-same-origin` 的最小 sandboxed iframe；但 MDN 明确指出 iframe sandbox 一旦能在框架外打开内容就失效，因此不能把 iframe 当作唯一防线。来源：[MDN iframe sandbox](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/iframe)。

当前生产 CSP 的 `frame-src 'none'` 是有意的安全基线。任何未来 iframe 需求都要单独 ADR、CSP 差异审计和攻击测试，不可为 DOCX/HTML 预览直接放宽。

## 6. 与既有 Sandbox 计划的合并方式

不复制或重新发明现有 Sandbox B0–B7；新增以下挂接点：

1. **R0 / Sandbox B1：** `ContentSafetyPolicy` 与 `SandboxPolicy` 各自独立，均由 Rust 领域层构造并审计。前者保护字节/renderer，后者保护执行/进程。
2. **R2 / Sandbox B0：** 对原生解析器或 converter 建立攻击 fixture：path traversal、symlink/junction、压缩炸弹、超页/超像素、子进程、网络尝试、资源耗尽。
3. **R5 / Sandbox B5：** Agent/Skill/MCP 的 `create_artifact` 若依赖外部命令，使用 authenticated execution bridge → `ExecutionService` → `SandboxBroker`，不能直连 Sidecar 本地 shell。
4. **R4b / Sandbox B6：** 地图瓦片与地理编码归类为网络 policy，记录域名、批准、缓存与审计；即使不运行 shell，也不能让 WebView 任意出网。
5. **R6 / Sandbox B7：** 严格 provider 缺失时，禁用需要外部执行的预览/生成器并给出诊断，绝不降级为 full-access。

## 7. 必须纳入计划的验证项

- [ ] asset/custom URI 不能读取 artifact root 以外的任何文件，不能用 `..`、编码路径、符号链接、junction 绕过。
- [ ] 任何无 artifact ID、无会话归属、过期 revision、错误 MIME 的请求均失败且不泄露真实路径。
- [ ] `dialog:allow-save` 下载取消、同名、Unicode、长路径、磁盘不足、只读目录在三平台行为可解释。
- [ ] PDF/XLSX/DOCX/图片畸形样本、解压炸弹、超尺寸、超时、取消不会卡死 UI/主进程。
- [ ] Chart/Map spec 不存在可执行 function、HTML、任意 URL 注入；图表/地图不放宽主 WebView CSP。
- [ ] 触发外部 converter、Agent shell、Skill/MCP helper 时只有 Sandbox Broker 路径可用。

## 8. 最终建议

将 “Artifact Store + 内容安全策略 + 窄 URI/导出” 作为 R0/R1 的明确前置项；将 “Sandbox 对外部执行、网络和高风险解析器的接入” 作为 R2/R5/R6 的依赖项。这一拆分既能尽快交付安全的图片/文件/图表/静态地图，又不淡化已有跨平台 Sandbox 方案必须完成的真实隔离工作。
