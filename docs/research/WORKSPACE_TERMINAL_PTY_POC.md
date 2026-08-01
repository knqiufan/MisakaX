# Workspace Terminal PTY PoC

> **用途：** 固定 W3 的 PTY/xterm 依赖、跨平台进程模型、安全边界和当前平台验证证据。  
> **受众：** Rust/Tauri、React、测试、安全与发布维护者。  
> **最后审阅 / Last reviewed：** 2026-08-01  
> **实现基线：** `main@7a1f30e`（W6 Windows 当前机验证）。

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
- Windows 普通工作区把 canonical verbatim path 转为可读 cwd；超过 `CreateProcessW.lpCurrentDirectory` 限制时，PowerShell 从可信 root 启动并以已转义的 `Set-Location -LiteralPath` 进入 canonical 工作区。cmd 无法安全进入 extended-length cwd 时返回稳定 `shell_workspace_path`，不得静默落到错误目录。
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
  -> terminal:output(seq, base64)
  -> child exit: drop writer/master, drain reader
  -> terminal:exited(last_seq, exit_code/signal/reason)
  -> refresh WorkspaceContext
```

应用退出、window destroyed、会话删除和显式 kill 都终止整个进程树。Windows 关闭/终止 Job；Unix 对负 PGID 发送 SIGHUP/SIGKILL，guard drop 也会清理残留组。`terminal:exited.last_seq` 定义输出完成边界，W4 只能接受相同 terminal/generation 且严格递增的 output seq，并在 exited 后丢弃迟到事件。

## 平台矩阵

| 平台 | W3 证据 | 当前结论 |
|---|---|---|
| Windows 11 / ConPTY | Windows 11 Pro 10.0.26200；`terminal_manager_tests` 10/10 覆盖 PowerShell 5、cmd、pwsh 缺失回退、resize、超过 260 字符的 Unicode cwd、10 MiB 突发长行、名义 10 MiB/s 节流源、Job/崩溃回收与重开；Release UI 输出 `W6_RELEASE_OK` / `W6_SUSTAINED_OK` | 🟡 当前机 W6 证据通过；Windows 10、pwsh、原生 IME 和签名发布未验证 |
| macOS / PTY | `portable-pty 0.9.0` 官方支持；代码选择原生 PTY、验证 allowlisted `$SHELL` 并以 process group 清理 | 🟡 实现完成，真实 macOS 运行/打包证据留 W6 |
| Linux / PTY | `portable-pty 0.9.0` 官方支持；与 macOS 共用 Unix PTY/process-group 路径 | 🟡 实现完成，真实 Linux 运行/打包证据留 W6 |

这张矩阵不把 Windows 结果外推为 Windows 10、macOS/Linux 实机通过。W6 仍必须分别记录系统版本、shell、IME、TUI、resize、休眠恢复、崩溃清理、签名和 bundle 结果。

## W6 Windows 当前机证据

- 环境：Windows 11 Pro 10.0.26200（x64）、Windows PowerShell 5、cmd、Git 2.52.0.windows.1；`pwsh` 与 WSL 未安装。
- 长路径：PowerShell 5 在超过 260 字符且含中文的工作区完成 round trip；请求 cmd 进入同一路径会以稳定诊断失败，避免在错误 cwd 启动。请求 pwsh 时本机安全回退到 Windows PowerShell 并返回 fallback reason。
- 压力/恢复：10 MiB 单条输出在测试用 256 KiB/s 限制下触发 `OutputLimit`，有界输出不超过 512 KiB，进程被回收；另以 512 KiB/50 ms、共 24 次的名义 10 MiB/s 源验证持续输出在 20 秒/32 MiB 边界内正常退出或限流并归零。Windows ConPTY 实测先产生屏幕更新背压，因此该用例验证的是 10 MiB/s 源压力与有界行为，不伪称 manager 实收稳定 10 MiB/s。独立 helper 模拟应用异常终止后，Job Object 回收 shell grandchild，新 manager 可重新启动并执行 `W6_REOPEN_OK`。
- Release UI：本地未签名 `misaka-x.exe` 默认显示 Terminal，真实 PowerShell 工作区 prompt 可见，并连续输出 `W6_RELEASE_OK`、`W6_SUSTAINED_OK`；关闭窗口后对已核对的绝对路径进程树做归零确认。
- 静态离线边界：`dist/index.html` 只引用本地 favicon、JS 与 CSS，未发现远端 `src`/`href` 或缺失的 `/assets`。本轮没有断开主机网络，因此只把“无 CDN 静态依赖”记为通过，不把“物理断网启动”记为通过。

## 已知 Shell/TUI 差异与诊断

| 差异 | 行为 / 诊断 |
|---|---|
| Windows 长 cwd | PowerShell 使用 canonical `Set-Location -LiteralPath`；cmd 返回 `shell_workspace_path`，不回退到其他目录 |
| pwsh 不存在 | 按可信顺序回退到 `powershell.exe`，状态包含 fallback reason；本轮没有 pwsh 实机行为证据 |
| ConPTY DSR | 无头测试必须先回复 `ESC[1;1R`；真实 xterm 自动处理 |
| 输出过载 | manager 以有界 channel、批处理和字节率限制停止进程；面板只显示稳定退出/错误态，不记录原始输入输出 |
| Unix TUI | process-group 路径已实现，但 macOS/Linux 的 shell、IME、alternate screen、clipboard 与 bundle 仍需各自真机记录 |

## Windows PoC 的关键发现

1. ConPTY 启动会先发 `DSR ESC[6n` 查询光标位置；真实 xterm 会自动回 `ESC[1;1R`。无头测试必须模拟该响应，否则 Shell 会等待而不处理命令。
2. `portable-pty` 必须先 `spawn_command`，再释放 slave handles，之后才 clone reader/take writer；过早取 I/O 会破坏 ConPTY 启动时序。
3. child wait 返回后还需释放 writer/master，cloned reader 才能看到 EOF、排空最后输出并发出有正确 `last_seq` 的 exited 事件。
4. Job Object `KILL_ON_JOB_CLOSE` 的实机测试证明显式 shutdown 后 shell 与其 `Start-Process` grandchild 均不存在，active session map 归零。

## 已运行验证

```text
cargo test --all-features --test terminal_manager_tests  # 10 passed (6.72 s)
cargo test terminal --lib                                # 4 passed
cargo test --all-features -j 1                           # 全量通过
npm test -- --run                                        # 38 files / 271 tests
npm run build                                            # 通过；仅既有大 chunk 警告
npm run tauri build                                      # W6 Windows EXE/MSI/NSIS 通过
```

W6 Release 产物：`misaka-x.exe` 37,701,632 bytes / SHA-256 `94413D63F73E2DFFE25260D7581F5F8CC91FCA7569A504A0D86F19E974BE8E24`；MSI 16,744,448 bytes / `0CF296194A71A1A66D452DE33D1BCCF902D3A7114060FA2E77B4977A44755FDA`；NSIS 13,088,974 bytes / `17F6F3F094224F9F87743B75ED433F008103F04FF9783A44432D196F4A74DC02`。

`cargo nextest run --all-features --profile ci` 在并发编译阶段触发 Windows 页文件不足（OS 1455 / `0xc000012d`）；按仓库指南回退到串行 `cargo test --all-features -j 1` 后全量绿色，未执行 `cargo clean`。

## 尚未由 W3 解除的上线门

- W4：已由 `main@2a5b112` 交付 xterm UI、Fit/ResizeObserver、输入输出/exit、seq 丢弃、复制粘贴、工作区切换选择和可访问错误态。
- W5：已由 `main@b96d0e3` 删除主 WebView 通用 shell/fs/http 权限，配置生产 CSP、精确 command authorization 与 XSS/OSC/link 测试，并通过 Windows Release 真实 shell UI 复验。
- W6：Windows 11 当前机长路径/压力/崩溃恢复证据已补；Windows 10、pwsh、macOS/Linux 真机、三平台签名 bundle、原生 IME/睡眠恢复与持续吞吐矩阵仍是上线门。
- Sandbox：本 PoC 是明确标注的“本机权限”用户终端，不是 Agent Sandbox，也没有实现或改变 Sandbox provider。

## 一手资料

- [`portable-pty 0.9.0` crate 文档与许可证](https://docs.rs/crate/portable-pty/0.9.0)
- [`portable_pty` API](https://docs.rs/portable-pty/0.9.0/portable_pty/)
- [`CommandBuilder` cwd/env API](https://docs.rs/portable-pty/0.9.0/portable_pty/cmdbuilder/struct.CommandBuilder.html)
- [`@xterm/xterm 6.0.0`](https://www.npmjs.com/package/@xterm/xterm/v/6.0.0)
- [`@xterm/addon-fit 0.11.0`](https://www.npmjs.com/package/@xterm/addon-fit/v/0.11.0)
- [xterm.js security guide](https://xtermjs.org/docs/guides/security/)
