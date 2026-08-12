# 富内容与产物交付调研资料与本地证据

> **用途：** 保存本方案的可追溯依据，帮助后续实现者在依赖/API 变更时重新核验。
> **受众：** 架构、开发、安全和 AI Coding Agent。
> **最后审阅 / Last reviewed：** 2026-08-13
> **调研日期：** 2026-08-08；Sandbox 复核补充于 2026-08-12

---

## 1. 外部一手资料

| 主题 | 来源 | 方案采用的结论 |
|---|---|---|
| Tauri Rust FS 与前端 FS scope | [Tauri File System](https://v2.tauri.app/plugin/file-system/) | Rust 可使用 `std::fs`/`tokio::fs`；前端 FS plugin 的危险权限与 scope 默认受限。Artifact Store 应保留在 Rust，不开放通用 WebView FS。 |
| Tauri permission/capability | [Permissions](https://v2.tauri.app/security/permissions/)、[Capabilities](https://v2.tauri.app/security/capabilities/)、[Command scopes](https://v2.tauri.app/security/scope/) | capability 是 WebView 权限边界；deny 高于 allow；自定义 command 必须自行正确执行 scope。 |
| Tauri CSP | [Content Security Policy](https://v2.tauri.app/security/csp/) | CSP 应尽可能窄，避免远端脚本和不可信资源；不要为 renderer 添加宽泛 CDN/`unsafe-eval`。 |
| Tauri asset protocol | [Asset protocol scope](https://v2.tauri.app/security/asset-protocol/) | asset protocol 必须启用且有精确 scope；禁止 `$HOME/**/*`/`**/*` 式宽范围；动态目录不应成为默认方案。 |
| Tauri custom protocol/发布 | [Tauri 2.0 release](https://v2.tauri.app/blog/tauri-20/)、[Windows installer](https://v2.tauri.app/distribute/windows-installer/) | 可注册异步 custom URI protocol；Windows custom protocol 功能需在 WebView2/安装包测试中确认。 |
| ECharts 数据和无障碍 | [Dataset/encode](https://echarts.apache.org/handbook/en/concepts/dataset/)、[ARIA best practices](https://echarts.apache.org/handbook/en/best-practices/aria/) | 使用受限数据/编码 schema；显式引入 ARIA 组件，并提供表格替代。 |
| MapLibre 与 CSP | [MapLibre GL JS documentation](https://maplibre.org/maplibre-gl-js/docs) | MapLibre 使用 WebGL/workers，并有 CSP bundle 指引；远程 style/tile 将带来网络与 CSP 设计，不可由模型给 URL。 |
| PDF 本地查看 | [Mozilla PDF.js Getting Started](https://mozilla.github.io/pdf.js/getting_started/?lang=en) | PDF.js 可构建本地 display/viewer；worker/viewer 不能依赖 `file:`，需打包并施加资源限制。 |
| XLSX 本地读取 | [SheetJS Data Import](https://docs.sheetjs.com/docs/solutions/input/)、[Parsing options](https://docs.sheetjs.com/docs/api/parse-options/) | 浏览器应从受控字节/ArrayBuffer 读取，而非任意文件名；需要限制解析范围，且不执行宏/公式。 |
| iframe sandbox 边界 | [MDN iframe](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/iframe) | sandbox iframe 可降低用户生成内容风险，但不能作为唯一防线；内容在 frame 外打开等情况会破坏保护。首期不预览 HTML/SVG。 |

### 1.1 执行隔离复核的一手资料

详细论证、限制和访问日期见 [Sandbox 方案复核](../../research/SANDBOX_STRATEGY_REASSESSMENT_2026-08.md)。本目录只保留与富内容挂接直接相关的结论：

| 主题 | 来源 | 富内容计划采用的结论 |
|---|---|---|
| Windows AppContainer/LPAC | [Microsoft: Launch an AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer) | 无第二个可登录账户即可形成 capability/文件/凭据边界；作为 Windows 离线 parser/converter 的优先本地候选，仍需兼容与攻击 Spike。 |
| Windows restricted token / Job Object | [Restricted Tokens](https://learn.microsoft.com/en-us/windows/win32/secauthz/restricted-tokens)、[Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) | restricted token 是纵深防御；Job Object 用于资源与进程树回收，不能单独替代 AppContainer/网络边界。 |
| Windows experimental sandbox API | [Create Process In Sandbox APIs](https://learn.microsoft.com/en-us/windows/win32/secauthz/createprocessinsandbox) | `CreateProcessInSandbox` 可作为 Windows 11 Spike，但因 experimental/ABI/schema 限制只进入 Canary，不写成稳定安全承诺。 |
| macOS App Sandbox/XPC | [App Sandbox](https://developer.apple.com/documentation/security/app_sandbox)、[Process](https://developer.apple.com/documentation/foundation/process)、[Accessing files](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox) | 适合项目自带、签名、artifact-only I/O 的窄 parser helper；不适合透明承载用户任意 Git/Node/Python/Xcode。 |
| macOS Seatbelt 弃用风险 | [Apple Developer Forums: replacement for sandbox-exec](https://developer.apple.com/forums/thread/661939) | `sandbox-exec`/`sandbox.h` 不作为正式 strict provider 或 release gate；论坛答复仅作弃用佐证。 |
| macOS 本地 Linux VM | [Running Linux in a Virtual Machine](https://developer.apple.com/documentation/virtualization/running-linux-in-a-virtual-machine)、[Virtualization entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.virtualization) | 可作为后续隐私优先 strict 选项；需要签名 entitlement、镜像供应链、Intel/Apple silicon 实机和无 writable host share 验证。 |
| Remote microVM/session | [Firecracker](https://firecracker-microvm.github.io/)、[Azure Dynamic Sessions](https://learn.microsoft.com/en-us/azure/container-apps/sessions) | remote + snapshot 是跨平台 strict 的优先垂直切片；必须验证地区、egress、保留/销毁、取消、manifest 和凭据隔离。 |

外部依赖/API 会随时间演进。实际开工前，必须再核对项目 `package.json`、`src-tauri/Cargo.toml` 和相应官方文档中的锁定版本/许可，不应把这里的资料当作可直接复制的版本号。

## 2. 当前仓库实证

| 结论 | 实证位置 |
|---|---|
| 助手消息目前是 `content: string`，有 JSON `attachments`、thinking、tool calls，但没有 blocks/artifacts | `src/lib/ipc/types.ts`、`src-tauri/src/db/models.rs`、`db/repository/message_repo.rs` |
| React 的流式投影只有 token/thinking/complete/error/tool 事件 | `src/hooks/use-stream-listener.ts` |
| Markdown 使用 Streamdown，已含 code/math/mermaid/cjk plugin | `src/components/chat/markdown/MessageResponse.tsx` |
| MessageItem 只展示 Markdown 与已解析的图片附件，适合作为 legacy adapter 基线 | `src/components/chat/message/MessageItem.tsx` |
| 当前附件严格支持图片与小文本；PDF/Office 已明确是 planned | `src/components/chat/composer/attachmentUtils.ts` |
| Rig backend 已支持图片输入附件，但输出仍收集为文本/思考/usage | `src-tauri/src/services/llm/backend.rs`、`services/llm/traits.rs` |
| 主 WebView 仅有窄 capability，不带 FS/HTTP/Shell execute；生产 CSP 是本地资源策略 | `src-tauri/capabilities/default.json`、`src-tauri/tauri.conf.json`、`docs/guides/tauri-capability-csp-audit.md` |
| 现有自定义 workspace FS command 有 root containment 与文本大小限制，可借鉴但不能直接向 artifact 复用路径入参 | `src-tauri/src/commands/fs_explorer.rs` |
| Proposed Sandbox ADR/实施计划已改为 provider-neutral S0、snapshot/manifest 与非对称 Provider；S0 已实现，真实 Provider 待 Spike | `docs/architecture/SANDBOX_TECH_SELECTION.md`、`docs/planning/SANDBOX_IMPLEMENTATION_PLAN.md`、`docs/research/SANDBOX_STRATEGY_REASSESSMENT_2026-08.md`、`src-tauri/src/services/sandbox/` |
| Sidecar 真正主对话接管是 Phase 4 后续，而非已完成基础 | `docs/project/DEVELOPMENT_STATUS.md` |

## 3. 本次调研的推论边界

以下是基于上述资料和项目现状的工程推论，而非外部资料直接承诺：

1. 以 `ContentBlock` + renderer registry 承载聊天异构输出，比 Markdown directive 或 middleware 更适合持久化、顺序、状态和 fallback。
2. 默认采用自定义 artifact protocol 比宽 asset scope 更利于按 artifact/session/revision 授权；R1 Spike 后才能最终定案。
3. 静态图表、GeoJSON 和 raster 图片可以先不依赖完整执行隔离，但必须先有内容安全面。
4. 外部 converter、任意代码/HTML、复杂不可信原生 parser、网络瓦片需要与 Execution Isolation Broker 合并；strict 使用 snapshot/result manifest、默认断网和实际 capability evidence，禁止快捷宿主执行。
5. “在线查看”在产品文案中应定义为“应用内预览”，避免暗示第三方云服务或数据上传。
6. 富内容执行路径可共享上层保证但采用非对称 Provider：Windows 离线 AppContainer、Linux namespaces、macOS 窄 XPC helper、跨平台/联网 remote microVM；该工程建议已进入 Proposed ADR，但仍待各位置 Spike，不是外部资料直接保证。

这些推论必须在对应阶段的契约测试、URI Spike、恶意样本测试和三平台实机验证中确认；若结果不成立，先更新本目录决策与风险记录，再改变实现路线。

## 4. 后续研究待办

- [ ] 比较 `asset:` 与 custom URI protocol 的 range、缓存、MIME、Windows WebView2/macOS/Linux 行为，并记录 R1 Spike。
- [ ] 为 PDF/XLSX/DOCX 候选 parser 完成版本、许可证、体积、worker/WASM/native binary、CSP、恶意文件和资源上限调研。
- [ ] 明确图片模型/provider 输出格式、数据所有权、内容策略和下载原件保留规则。
- [ ] 确认地图服务商、许可、attribution、密钥保存、隐私、地区合规、缓存与离线策略；未确认前不接 R4b。
- [x] 修订 Sandbox ADR 与总实施计划；以 S0 contract/FakeProvider/snapshot/result manifest 取代富内容对旧 B0–B7 平台顺序的直接依赖。
- [ ] 完成 S1 remote snapshot 垂直切片：地区/上传摘要/保留与销毁、egress、凭据隔离、取消/kill tree、result manifest 与冲突写回。
- [ ] 完成 S2 Windows AppContainer Spike：Home/Pro/标准用户、实验 API 缺失、文件/凭据/网络/loopback/junction/进程树和工具兼容矩阵。
- [ ] 完成 S3 macOS 富内容窄 XPC helper Spike；若继续本地 VM，另测签名 entitlement、镜像 supply chain、Intel/Apple silicon 与无 writable host share。
- [ ] 完成 S4 默认启用/企业强化门：组织 provider、地区与保留、镜像 digest、密钥代理、审批/审计、修复/销毁事务和独立安全 review。
