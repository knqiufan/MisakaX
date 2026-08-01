# Tauri WebView Capability 与 CSP 审计

> **用途：** 记录 Workspace Terminal 上线前的主 WebView 权限基线、实际调用点和 W5 收窄目标。
> **受众：** Tauri、Terminal、前端与安全维护者。
> **最后审阅 / Last reviewed：** 2026-08-01
> **基线：** `main@fa24bd7`（W0 working tree）。

## 当前证据

`src-tauri/capabilities/default.json` 当前把以下能力直接授予 `main` WebView：

- Shell：`open`、`execute`、`spawn`、`stdin-write`、`kill`。
- Filesystem：默认能力以及 read/write/exists/mkdir/remove/rename。
- HTTP：默认能力与任意 fetch。
- Dialog、clipboard、notification 的当前业务权限。

`src-tauri/tauri.conf.json` 的生产 CSP 为 `null`。W0 特征测试
`security_config_baseline_tests.rs` 固定了这一待收口状态，W5 应把断言改为最小权限目标。

## 实际调用点

| 插件能力 | React 直接调用 | Rust/自定义 command | W5 决策 |
|---|---|---|---|
| `shell:*` | 未发现 `@tauri-apps/plugin-shell` import | `fs_reveal_in_explorer` 使用 Rust `std::process::Command`，不依赖 WebView Shell permission | 删除 execute/spawn/stdin/kill；若保留外链打开，只授予窄 `open` scope |
| `fs:*` | 未发现 `@tauri-apps/plugin-fs` import | Explorer 通过 `fs_*` commands 做 root containment、文本上限与二进制拒绝 | 删除主 WebView 通用 FS permissions，继续由窄 command 校验 |
| `http:*` | 未发现 `@tauri-apps/plugin-http` import | Skills catalog、模型与 Sidecar 请求由 Rust/Sidecar 发起 | 删除主 WebView HTTP fetch；如后续发现例外，按精确域名另行审计 |
| Dialog | Skills 导入/导出、设置备份、会话导出使用 | 路径仍须由 Rust command 重新校验 | 保留实际使用的 open/save 子权限 |
| Clipboard | Workspace 文件节点复制使用 | — | 保留 read/write 前再次核对最小调用面 |

审计命令：

```powershell
rg -n "@tauri-apps/plugin-(shell|fs|http|dialog|clipboard)" src
rg -n "tauri_plugin_(shell|fs|http)" src-tauri/src src-tauri/Cargo.toml
```

## W5 生产 CSP 目标

生产配置应从以下约束起步，并以 release bundle 实测调整，不得用 `*`、远端脚本或
`unsafe-eval` 解决兼容问题：

```text
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';
font-src 'self' data:;
img-src 'self' asset: http://asset.localhost blob: data:;
connect-src 'self' ipc: http://ipc.localhost;
worker-src 'self' blob:;
object-src 'none';
base-uri 'none';
frame-src 'none';
form-action 'none'
```

- Monaco/xterm worker 必须由 Vite 本地打包；仅在确有 blob worker 证据时保留 `blob:`。
- Sidecar、模型和目录网络请求继续走 Rust/Sidecar，不把远端域名加入 WebView `connect-src`。
- Vite HMR 的 localhost/WebSocket 例外只进入开发配置，不进入 release CSP。
- W5 必须同时运行 XSS/OSC/title/link 测试、capability 静态测试和 `tauri build` bundle 检查。

## Tool Logs 迁移决定

Tool Logs 保留其诊断价值并继续作为聊天列内抽屉；W2 将 WorkspaceBar 的 Terminal 图标
改为真实 Terminal toggle 后，Tool Logs 入口迁入消息工具组的显式“工具日志”动作。
在新入口测试通过前，不删除 `ToolLogsPanel` 或其 store 数据。
