# Workspace Terminal PTY PoC

> **用途：** 固定 W3 的 PTY/xterm 依赖、跨平台进程模型、安全边界和当前平台验证证据。  
> **受众：** Rust/Tauri、React、测试、安全与发布维护者。  
> **最后审阅 / Last reviewed：** 2026-08-01  
> **实现基线：** `main@9249915`。

## 结论

W3 采用以下精确版本：

| 组件 | 锁定版本 | 许可证 | 用途 |
|---|---:|---|---|
| `portable-pty` | `0.9.0` | MIT | Windows ConPTY、macOS/Linux PTY 抽象 |
| `@xterm/xterm` | `6.0.0` | MIT | W4 本地静态打包的终端模拟器 |
| `@xterm/addon-fit` | `0.11.0` | MIT | W4 cols/rows 自适应 |
| `windows-sys` | `0.61.2` | MIT/Apache-2.0 | Windows Job Object 进程树回收 |

`portable-pty` 提供 `native_pty_system/openpty`、结构化 `CommandBuilder`、reader/writer、resize、child wait/kill 和 Unix process-group leader。它没有替 MisakaX 建立 Windows Job Object，因此 W3 额外实现 `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`，绑定失败即停止 child 并拒绝启动，不能静默降级为只杀父进程。

## 窄边界

- WebView 的 spawn 请求只有 Chat Session、workspace generation、rows/cols 和 allowlisted shell profile；没有 cwd、任意 executable、argv 或 env。
- cwd 由 Rust 从 SQLite 会话重新读取并 canonicalize；spawn 与工作区更新/会话删除共享 mutation lock，防止检查后换目录。
- terminal ID 使用 UUID v4，并绑定 window label、Chat Session 与 workspace generation。write/resize/kill/get-state 每次重新校验三元 ownership。
- Windows 只解析可信绝对系统路径：`pwsh -> powershell.exe -> cmd.exe`；PowerShell 使用 `-NoProfile`，cmd 使用 `/D`，避免 Job 绑定前运行 profile/AutoRun 派生进程。
- Unix 只允许绝对、存在、可执行且 basename 在 `zsh/bash/sh/fish` allowlist 的 `$SHELL`；否则按 zsh/bash/sh 回退。
- 继承本机开发环境，但移除 `MISAKAX_*`、Sandbox、Tauri signing/channel 和内部 bridge token；不记录 env value、terminal input 或 terminal output。
- 输出 reader 使用专用阻塞线程；64 × 8 KiB 有界通道形成 512 KiB 背压上限，按最大 32 KiB 或 16 ms 批处理。慢事件消费者会反压 PTY pipe，不会无限扩张内存。
- 默认限制为全局 8 session、每窗口 4、每 Chat Session 1；另有 spawn/command/write/resize 频率、64 KiB 单次输入、输入/输出字节率和 2–512 rows/cols 限制。

## 进程与事件生命周期

```text
backend session lookup + generation check
  -> resolve allowlisted shell
  -> open PTY / spawn child
  -> attach Job Object (Windows) or retain PTY process group (Unix)
  -> bounded reader + batch event thread + blocking child wait thread
  -> terminal.output(seq, base64)
  -> child exit: drop writer/master, drain reader
  -> terminal.exited(last_seq, exit_code/signal/reason)
  -> refresh WorkspaceContext
```

应用退出、window destroyed、会话删除和显式 kill 都终止整个进程树。Windows 关闭/终止 Job；Unix 对负 PGID 发送 SIGHUP/SIGKILL，guard drop 也会清理残留组。`terminal.exited.last_seq` 定义输出完成边界，W4 只能接受相同 terminal/generation 且严格递增的 output seq，并在 exited 后丢弃迟到事件。

## 平台矩阵

| 平台 | W3 证据 | 当前结论 |
|---|---|---|
| Windows 11 / ConPTY | 当前实机运行 `terminal_manager_tests`：DSR 握手、CR 输入、resize、Unicode、SGR 颜色、alternate screen/TUI 序列、exit code、owner 篡改、输出洪水、shell + grandchild Job 回收 | ✅ W3 实机 PoC 通过 |
| macOS / PTY | `portable-pty 0.9.0` 官方支持；代码选择原生 PTY、验证 allowlisted `$SHELL` 并以 process group 清理 | 🟡 实现完成，真实 macOS 运行/打包证据留 W6 |
| Linux / PTY | `portable-pty 0.9.0` 官方支持；与 macOS 共用 Unix PTY/process-group 路径 | 🟡 实现完成，真实 Linux 运行/打包证据留 W6 |

这张矩阵不把 Windows 结果外推为 macOS/Linux 实机通过。W6 仍必须分别记录系统版本、shell、IME、TUI、resize、休眠恢复、崩溃清理和 bundle 结果。

## Windows PoC 的关键发现

1. ConPTY 启动会先发 `DSR ESC[6n` 查询光标位置；真实 xterm 会自动回 `ESC[1;1R`。无头测试必须模拟该响应，否则 Shell 会等待而不处理命令。
2. `portable-pty` 必须先 `spawn_command`，再释放 slave handles，之后才 clone reader/take writer；过早取 I/O 会破坏 ConPTY 启动时序。
3. child wait 返回后还需释放 writer/master，cloned reader 才能看到 EOF、排空最后输出并发出有正确 `last_seq` 的 exited 事件。
4. Job Object `KILL_ON_JOB_CLOSE` 的实机测试证明显式 shutdown 后 shell 与其 `Start-Process` grandchild 均不存在，active session map 归零。

## 已运行验证

```text
cargo test --test terminal_manager_tests                 # 5 passed
cargo test terminal --lib                                # 4 passed
cargo test --all-features -j 1                           # 全量通过
npm test                                                 # 35 files / 260 tests
npm run build                                            # 通过；仅既有大 chunk 警告
```

`cargo nextest run --all-features --profile ci` 在并发编译阶段触发 Windows 页文件不足（OS 1455 / `0xc000012d`）；按仓库指南回退到串行 `cargo test --all-features -j 1` 后全量绿色，未执行 `cargo clean`。

## 尚未由 W3 解除的上线门

- W4：真正的 xterm UI、Fit/ResizeObserver、输入输出/exit、seq 丢弃、复制粘贴、工作区切换选择和可访问性。
- W5：删除主 WebView 通用 shell/fs/http 权限、配置生产 CSP 和 command authorization/XSS/OSC/link 测试。
- W6：macOS/Linux 真机、三平台 bundle、IME/TUI/睡眠恢复与性能矩阵。
- Sandbox：本 PoC 是明确标注的“本机权限”用户终端，不是 Agent Sandbox，也没有实现或改变 Sandbox provider。

## 一手资料

- [`portable-pty 0.9.0` crate 文档与许可证](https://docs.rs/crate/portable-pty/0.9.0)
- [`portable_pty` API](https://docs.rs/portable-pty/0.9.0/portable_pty/)
- [`CommandBuilder` cwd/env API](https://docs.rs/portable-pty/0.9.0/portable_pty/cmdbuilder/struct.CommandBuilder.html)
- [`@xterm/xterm 6.0.0`](https://www.npmjs.com/package/@xterm/xterm/v/6.0.0)
- [`@xterm/addon-fit 0.11.0`](https://www.npmjs.com/package/@xterm/addon-fit/v/0.11.0)
- [xterm.js security guide](https://xtermjs.org/docs/guides/security/)
