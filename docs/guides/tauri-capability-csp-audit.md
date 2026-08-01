# Tauri WebView Capability 与 CSP 审计

> **用途：** 记录 Workspace Terminal 主 WebView 的最小权限、生产 CSP 与 W5 Release 验证证据。
> **受众：** Tauri、Terminal、前端与安全维护者。
> **最后审阅 / Last reviewed：** 2026-08-01
> **基线：** `main@b96d0e3`（W5 implementation）。

## W5 结论

W5 已关闭旧的宽权限基线：主 WebView 不再拥有通用 Shell execute/spawn/stdin/kill、Filesystem、HTTP 或 Notification 插件权限。`default` capability 只绑定本地 `main` window，并精确包含：

- `core:event:allow-listen` / `allow-unlisten`；
- 仅用于安全外链的 `shell:allow-open`；
- `dialog:allow-open` / `allow-save`；
- `clipboard-manager:allow-read-text` / `allow-write-text`；
- 非终端自定义命令集合 `main-commands`；
- 只含 `terminal_spawn/write/resize/kill/get_state` 的 `terminal-runtime`。

`build.rs` 把 102 个已注册 custom commands 固定到 Tauri AppManifest；静态测试验证 invoke handler、manifest 和 permission 三者集合一致，并确保 Terminal commands 不混入 `main-commands`。不存在通配 custom-command permission，也没有为其他 window/webview 授予 Terminal runtime。

## 插件与调用面

| 能力 | W5 后 React / Rust 调用面 | 最终决定 |
|---|---|---|
| `shell:*` | React 只通过统一 helper 打开已校验的 HTTP(S) URL；`fs_reveal_in_explorer` 仍是 Rust command | 仅保留 `shell:allow-open`；删除 execute/spawn/stdin/kill |
| `fs:*` | Explorer 只通过 `fs_*` commands 使用 root containment、文本上限与二进制拒绝 | 删除 WebView FS 插件、Cargo 依赖、初始化和 capability |
| `http:*` | Skills catalog、模型与 Sidecar 请求由 Rust/Sidecar 发起 | 删除 WebView HTTP 插件、Cargo 依赖、初始化和 capability |
| Notification | 没有已审计的 WebView 使用点 | 删除插件、Cargo 依赖、初始化和 capability |
| Dialog | Skills 导入/导出、设置备份与会话导出 | 仅保留 open/save；业务 command 继续校验选择结果 |
| Clipboard | Workspace 复制与 Terminal 原生复制/粘贴 | 保留 read/write text；粘贴按 CR 规范化后直接写入窄 Terminal IPC |

安全外链 helper 只接受长度受限、无控制字符、无 credentials 的 `http:` / `https:` URL。Markdown、About 与 provider 字段都使用该 helper；相对路径、恶意 scheme 和无效 URL 渲染为不可导航文本，不用 `window.open`，也不允许当前 WebView 导航离开本地应用。

## Terminal 授权边界

Terminal 不调用通用 Shell plugin execute/spawn。WebView 只能提交 Chat Session、workspace generation、rows/cols 与 allowlisted shell profile；window owner 从 Tauri request 派生，cwd、任意 executable、argv 和 env 不能由前端指定。

Rust `TerminalManager` 继续负责 PTY、ownership、背压、限额和进程树回收。事件名固定为 `terminal:output` 与 `terminal:exited`；前端在 spawn 前安装 listener，并用有界 pre-spawn queue 消除 spawn/event 竞态。xterm 输出只写字节 buffer，不使用 `innerHTML`、WebLinksAddon、link provider 或自动外链。

Terminal 在 W5 后默认启用；`VITE_MISAKAX_WORKSPACE_TERMINAL=false|0` 和 `MISAKAX_WORKSPACE_TERMINAL=false|0` 只作为显式紧急 kill switch。UI 和 Rust 两侧必须同时允许，不能只绕过其中一层。

## 生产 CSP

当前生产 `csp` 是严格的本地资源策略：

```text
default-src 'self' customprotocol: asset:;
script-src 'self';
style-src 'self' 'unsafe-inline';
font-src 'self' data:;
img-src 'self' asset: http://asset.localhost blob: data:;
connect-src 'self' ipc: http://ipc.localhost;
worker-src 'self' blob:;
child-src 'self' blob:;
object-src 'none';
base-uri 'none';
frame-src 'none';
form-action 'none'
```

- 没有远端 script、`unsafe-eval`、任意 frame 或远端 connect origin。
- xterm、Monaco 和字体/样式均来自 Vite 本地 bundle；Release `index.html` 的资源引用仅为本地 `/assets/...`。
- Vite HMR 的 `http://localhost:1420` / `ws://localhost:1420` 只在 `devCsp` 生效，Release 运行时使用上面的生产 `csp`。
- 审计边界：Tauri 会把同一配置结构中的非活动 `devCsp` 字面量编译进 Windows EXE，因此二进制字符串扫描仍能看到 localhost（本次 EXE 计数：HTTP 8、WebSocket 1）。这不代表 Release 激活了开发策略；退出门是活动 production CSP、最终 capability、生成的前端资源和真实 Release 行为均通过。不得把“EXE 中没有 localhost 字面量”写成已满足事实。

## Release 证据

在 Windows 11 本机完成：

```text
npm test                                # 38 files / 271 tests
cargo check --all-features              # passed
cargo test --all-features -j1           # 73 lib tests + all integration/doc tests passed
npm run build                            # passed; only the existing large-chunk warning
npm run tauri build                      # passed; EXE + MSI + NSIS generated
```

产物与 SHA-256：

| 产物 | 大小 | SHA-256 |
|---|---:|---|
| `misaka-x.exe` | 37,730,816 bytes | `61C8827977488ECFEC8F5351E66E38F684957F61B351D619634B4C59CEC83F36` |
| `MisakaX_0.1.0_x64_en-US.msi` | 16,740,352 bytes | `8731A966E10AD275909B5F0501B9D08316E87956109868EB0C1208B7E32A21C8` |
| `MisakaX_0.1.0_x64-setup.exe` | 13,082,858 bytes | `89B2EBE69A35CF4768B0E08E29C18D41E465C3A05DFC06B71460668FFD0D450E` |

未设置 feature flag 的 Release EXE 默认显示 Terminal；真实 UI 路径成功启动 PowerShell，显示工作区 prompt，并通过原生 clipboard paste 输出 `RELEASE-W5-OK`、中文宽字符与 ANSI 颜色。关闭验证应用后确认其进程树归零。产物是本地未签名验证包，不替代 W6 的三平台签名/发布矩阵。

## 回归命令

```powershell
rg -n "@tauri-apps/plugin-(shell|fs|http|notification)" src
rg -n "tauri_plugin_(fs|http|notification)" src-tauri/src src-tauri/Cargo.toml
cargo test --all-features -j1
npm test
npm run build
npm run tauri build
```

## Tool Logs 决定

Tool Logs 继续作为聊天列内抽屉，入口位于消息 `ToolActionsGroup` 的明确“工具日志”动作，不与 Workspace Terminal 图标复用。Terminal toggle 使用统一 WorkspacePanel action；W5 已解除安全门，W6 只继续记录跨平台与压力发布证据。
