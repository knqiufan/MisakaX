# R0–R4 富内容交接

> **用途：** 交接 R0–R4 的本地实现、验证结果、开关与后续阶段边界。
> **受众：** 接手该功能的开发者、评审者和发布负责人。
> **最后审阅 / Last reviewed：** 2026-08-13
> **交接状态：** R0–R4 已完成代码审查、本地验证、非强制推送与 required CI 门禁；R5/R6 的执行隔离前置 S0 合同已实现，producer integration、真实 Provider 与发布 Gate 尚未开始。

---

## 已交付范围

| 阶段 | 实现结果 | 关键边界 |
|---|---|---|
| R0 | `ContentBlock`、`ArtifactRecord`、安全策略、v14 数据迁移、双读导入导出、feature flag | 新块读/写默认关闭；保留 `messages.content` 作为旧 Markdown 路径 |
| R1 | App-owned SHA-256 artifact store、元数据/预览/导出/过期 IPC、图片块 | WebView 不接收路径；保存只能经原生保存对话框和 Rust copy |
| R2 | 文本/Markdown/JSON/CSV、PDF、XLSX、DOCX、图片的本地只读预览 | 懒加载；失败或不支持时保留下载；不渲染 HTML/SVG/iframe/可执行内容 |
| R3 | `ChartSpecV1` validator、ECharts adapter、数据表、ARIA、CSV 导出 | 只编译允许的图表规格，绝不接受 ECharts option、函数、HTML 或 URL |
| R4a | `MapSpecV1` validator、MapLibre 本地 GeoJSON 空白底图、要素列表/坐标复制/GeoJSON 导出 | 不加载远程瓦片，不修改 CSP，不允许 tile URL；R4b 未开始 |

## 阶段门禁证据

| 阶段 | 最终实现提交 | required CI（8/8） |
|---|---|---|
| R0 | `b1ed884` | [#31271639940](https://github.com/knqiufan/MisakaX/actions/runs/31271639940) 成功 |
| R1 | `53a079d` | [#31273318019](https://github.com/knqiufan/MisakaX/actions/runs/31273318019) 成功 |
| R2 | `2e979e1` | [#31285793427](https://github.com/knqiufan/MisakaX/actions/runs/31285793427) 成功 |
| R3 | `ee6eea7` | [#31287278455](https://github.com/knqiufan/MisakaX/actions/runs/31287278455) 成功 |
| R4a | `0a64123` | [#31288007260](https://github.com/knqiufan/MisakaX/actions/runs/31288007260) 成功 |

所有提交均已以非强制方式推送至 `origin/codex/rich-content-r0-r4`。每次 CI 均覆盖 Rust、前端、终端 smoke test，以及 Windows/macOS/Ubuntu 的 Tauri 构建。

## 运行与开关

默认行为保持兼容：

- `MISAKAX_RICH_CONTENT_WRITE=1` 才允许 `artifact_register` 与 `append_content_block` 写入。
- `MISAKAX_RICH_CONTENT_RENDER=1` 才由 `get_messages`/`get_message_blocks` 返回 blocks；关闭时前端只渲染既有 `messages.content`。
- 两个开关都使用 `1` 或 `true` 作为启用值；其他值及缺失都为关闭。

## 主要落点

| 范围 | 文件/目录 |
|---|---|
| Content DTO、验证、normalizer seam | `src-tauri/src/services/content/` |
| Artifact store、preview registry、安全策略 | `src-tauri/src/services/artifacts/` |
| 数据库迁移与 repositories | `src-tauri/src/db/migrations.rs`、`src-tauri/src/db/repository/message_block_repo.rs`、`artifact_repo.rs` |
| IPC 与 use cases | `src-tauri/src/commands/artifacts.rs`、`src/lib/ipc/artifacts.ts`、`src/lib/ipc/types.ts` |
| 前端 block registry/renderers/预览 | `src/features/chat-content/` |
| 消息接线与翻译 | `src/components/chat/message/MessageItem.tsx`、`src/locales/{en,zh-CN}/chat.json` |

## 重要安全与兼容性决定

- Artifact 是 content-addressed（`objects/sha256`）；数据库不把字节存进消息文本。读、预览、导出都校验 owner session、retention、storage key 路径边界与 SHA-256。
- 文件声明 MIME 必须与魔数/安全文本类型匹配。PNG/JPEG/WebP/GIF 在进入预览前检查尺寸；SVG、HTML、未知二进制及含 `vbaProject.bin` 的 Office ZIP 被拒绝。
- PDF 最多渲染 200 页；文本、CSV、XLSX 的显示分别限制预览字符/行与 200 × 50 的可见表格区域，XLSX 最多暴露前 12 个 sheet。DOCX 仅以 DOMParser 提取文本，永不插入其 HTML。
- 图表和地图只接受 Rust validator 的 JSON schema。地图是 MapLibre 本地 `FeatureCollection` + marker；基础 style 无 source/tile URL。任何渲染器错误降级为表格/要素/notice，不能击穿整条消息。
- 导出的图表 CSV 会给 `= + - @` 开头的文本加单引号，避免电子表格公式注入。导出的 CSV/GeoJSON 先注册为临时 artifact，再经 Rust 保存，随后标为 expired。
- R0–R4 没有实现或宣称任何 OS execution provider；现有浏览器内预览限制属于内容安全面。后续原生 helper/converter、受控网络和 Agent 外部生成必须按 2026-08 Sandbox 复核接入 Broker/snapshot/result manifest，不能把本期 renderer/worker 当作执行隔离证据。

## 已验证

| 命令 | 结果 |
|---|---|
| `cargo fmt --check` | 通过 |
| `cargo clippy --all-targets --all-features -- -D warnings` | 通过 |
| `cargo test --all-features --lib content` | 5 passed |
| `cargo test --all-features --lib artifact` | 8 passed |
| `cargo test --all-features --lib map` | 3 passed |
| `cargo test --all-features --test security_config_baseline_tests` | 5 passed |
| `npm test -- --run` | 39 files / 273 passed（jsdom 对 Canvas 的已知提示不影响结果） |
| `npm run build` | 通过；大懒加载 chunk 有 Vite 性能 warning |

新增/覆盖的定向验证包括：安全图表与外部 tile 拒绝、跨会话/路径越权拒绝、伪造 SVG、超像素 PNG、message block 往返且旧 content 不变、CSV 公式注入转义。阶段远程门禁见上表。

## 后续接手顺序（不属于 R0–R4 完成门禁）

1. **R5 producer integration。** `services/content/normalizer.rs` 已预留 seam，但 Sidecar、provider、MCP 尚未产生 blocks/artifacts 或 lifecycle events；接入时必须先注册 artifact 再写 block，并沿用同一 validator。
2. **R1 URI Spike。** 当前预览通过受限 base64 IPC，而不是 `asset:`/custom protocol。若需性能、range request 或大媒体，请比较并验证窄 `asset:` scope 与 `misakax-artifact:` 协议，再替换读字节接口。
3. **R2 加固/跨平台 QA。** 在 Windows/macOS/Linux 手工验证 PDF、XLSX、DOCX、图片、WebGL 失败与保存取消；加入加密/损坏/压缩炸弹 fixture。纯浏览器 parser 先验证 worker/资源预算；若必须使用原生 helper/converter，则 Windows 等待 AppContainer Gate、macOS 项目自带 helper 等待 App Sandbox + XPC Gate，其他情况转 remote microVM。未通过时仅下载。不要把本期输出限制误认为压缩包解压预算或执行隔离的完整替代。
4. **R4b 保持禁止。** 尚未确定瓦片供应商、许可、attribution、隐私、缓存、密钥和安全网络位置；必须由 remote provider 或已通过 egress 攻击测试的 brokered network 获取，再经 artifact/custom URI 交付。不得加入宽 `https:` CSP、让模型提供 URL，或用 `HTTP_PROXY` 代替网络隔离。
5. **R5 执行路径继续过新决策门。** Broker S0 contract、snapshot/result manifest、显式 `host_direct`、写回预检和 FakeProvider 测试已完成；非执行 normalizer/event 仍等待 Phase 4。任何外部生成先完成 S0.5，再按实际位置等待 S1 remote、S2 Windows/Linux 或 S3 macOS Gate；当前不得把任何 Provider 标为可发布。
6. **R6 默认启用另过 S4。** 即使单个 S1/S2/S3 provider 能运行，也必须补齐组织/地区、镜像 digest、密钥代理、审批审计、修复/销毁和独立安全评审，才能默认开启或作为企业能力发布。

R5、R6 或 R4b 需作为新的独立阶段执行；不得把它们并入本次已验证的 R0–R4 交付。

## 已知非阻塞项

- `npm install` 的 audit 输出为 9 个问题（1 low、3 moderate、5 high）。未执行 `npm audit fix`，以免未经审查地改动依赖树；接手者应在独立依赖审计中定位来源。
- Vite 构建通过，但输出提示部分按需预览包超过 500 kB。功能依赖均为动态导入；如需优化，应先用 bundle analysis 再调整 chunk strategy，不能为了消警告而取消本地惰性加载。
