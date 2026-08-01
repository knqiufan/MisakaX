# 工作区 Git 标识与嵌入式终端实施计划

> **用途：** 在 composer 下方增加 Git/本地项目标识，并将 WorkspaceBar 的日志图标替换为右侧嵌入式终端入口。
> **受众：** React、Rust/Tauri、测试和安全维护者。
> **最后审阅 / Last reviewed：** 2026-08-01
> **规划基线：** `main@fa24bd7`。
> **关联：** [总体架构](../architecture/WORKSPACE_SKILLS_SECURITY_ARCHITECTURE.md) · [UI 设计](../design/SKILLS_WORKSPACE_TERMINAL_UI_DESIGN.md) · [沙箱选型](../architecture/SANDBOX_TECH_SELECTION.md)

---

## 1. 范围与非目标

### 1.1 交付

- composer footer 显示当前工作区 Git 分支；非 Git 显示“本地项目”。
- 支持 detached HEAD、Git worktree、submodule、无 Git CLI 和超时退化。
- 为未来 Git UI 定义 `VcsProvider`/panel extension seam，但不实现写操作。
- 右侧面板支持 `explorer | terminal` 两种模式，复用现有 resize shell。
- WorkspaceBar 终端图标打开真实 PTY，并自动在当前会话工作区启动。
- Windows/macOS/Linux 支持系统默认 Shell、输入、输出、resize、退出和清理。
- 删除主 WebView 的通用 Shell 能力，启用终端所需的 CSP 与窄 IPC。

### 1.2 明确不做

- Git status/diff/stage/commit/push/pull/branch checkout。
- 多 terminal tabs、split terminal、SSH、端口转发、终端协作/回放。
- 将用户终端静默放入 Agent 沙箱；首期显示“本机权限”。
- 重写 Workspace Explorer 或 Monaco。

## 2. 验收标准

1. Git 分支/本地标识在工作区切换后正确更新，不阻塞 composer 输入。
2. detached HEAD 显示短 hash，不显示错误分支名。
3. Git 查询使用结构化 argv、超时和输出上限，不接受前端命令字符串。
4. 点击 Terminal 后右栏打开，PTY 的 cwd 与后端解析的当前会话工作区完全一致。
5. Explorer/Terminal 切换不丢失 Explorer tabs；终端 session 生命周期符合设计。
6. WebView 无法通过 Tauri Shell 插件启动任意命令，只能操作其拥有的 PTY session。
7. 应用/会话关闭后没有遗留 Shell 进程；session ownership 篡改被拒绝。
8. 三平台至少通过 PowerShell/pwsh、zsh/bash 和常见 TUI/Unicode/resize 用例。

## 3. Phase W0：基线与安全收口前置

### 3.1 TODO

- [x] 为现有 `WorkspaceBar -> ToolLogsPanel`、Explorer open/close/resize 建特征测试。
- [x] 记录 Tool Logs 当前唯一入口和用户价值，决定迁移到消息工具组还是 overflow menu。
- [x] 审计 `default.json` 中 shell/fs/http permissions 与实际调用点。
- [x] 为生产 `tauri.conf.json` 设计 CSP，区分 Vite dev 例外。
- [x] 定义 `WorkspaceContext`、`WorkspacePanelMode`、`TerminalSessionId`、错误码和事件 DTO。
- [x] 加 feature flags：`workspace_context_badge`、`workspace_terminal`、`narrow_webview_capabilities`。
- [x] 补工作区路径规范化、会话切换和右栏持久状态测试 fixture。

### 3.2 退出门

- 清楚知道收窄每个 capability 会影响哪些现有功能。
- Tool Logs 有明确新入口，替换图标不会造成诊断功能丢失。

## 4. Phase W1：只读 WorkspaceContext 与 Git Provider

### 4.1 结构

```rust
pub trait VcsProvider {
    async fn detect(&self, workspace: &CanonicalWorkspace) -> Result<Option<VcsContext>>;
}

pub enum WorkspaceKind { Git, Local }
```

`WorkspaceContextService` 依次询问已注册 provider；首期只有 `GitCliProvider`。未检测到 Git 或 Git 不可用都返回 Local，只有诊断日志区分原因。

建议 Git 命令：

- `git -C <cwd> rev-parse --is-inside-work-tree --show-toplevel`
- `git -C <cwd> symbolic-ref -q --short HEAD`
- detached fallback：`git -C <cwd> rev-parse --short HEAD`
- resolved git dir：`git -C <cwd> rev-parse --git-dir --git-common-dir`

命令分别或使用可靠的 `--path-format=absolute` 能力检测执行；不要解析 `.git` 文件代替 Git，因为 worktree/submodule/环境变量语义复杂。

### 4.2 缓存和刷新

- cache key：canonical workspace path + workspace generation。
- 首次进入异步查询，不阻塞发送。
- 会话/工作区切换立即 invalidation。
- 窗口 focus、终端进程退出后 debounce refresh。
- watch resolved `HEAD` 和 common dir 的相关 ref；watcher 是体验优化，读取时仍可按 TTL 复核。
- 同一工作区并发请求 single-flight；Git 命令超时建议 1–2 秒，输出限 KB 级。

### 4.3 TODO

- [x] 新增 `workspace` domain types、`VcsProvider` 和 `WorkspaceContextService`。
- [x] 实现 `GitCliProvider` 的结构化命令、超时、取消、输出上限和日志脱敏。
- [x] 覆盖普通分支、detached、worktree、submodule、bare/non-worktree、路径空格/Unicode。
- [x] Git CLI 缺失/被 PATH 劫持时给出安全诊断；只使用可信 PATH 解析策略。
- [x] 新增 `workspace_get_context` command 和 `workspace.context.changed` 事件。
- [x] 实现 single-flight cache、generation、focus/terminal-exit/debounced watcher 刷新。
- [x] React 新增 `useWorkspaceContext`，处理 loading/stale/error 和旧 generation。
- [x] 在 composer footer 加 `WorkspaceContextBadge`，实现长分支截断和 detached 文案。
- [x] 非 Git 和查询失败显示“本地项目”；tooltip 不暴露敏感绝对路径给屏幕共享场景，路径按现有设置决定。
- [x] 给未来 provider/panel action 定义 interface，但首期不渲染 chevron/menu。
- [x] 补 React/IPC/E2E 测试，测工作区快速切换和 watcher 事件风暴。

### 4.4 退出门

- 所有 Git 读取是只读的；代码库中不存在 stage/commit/checkout 命令。
- composer 在 Git 超时/崩溃时仍可正常输入和发送。

## 5. Phase W2：WorkspacePanel 容器

### 5.1 状态设计

在现有 Explorer store 上渐进扩展或建立 facade：

```ts
type WorkspacePanelState = {
  open: boolean;
  mode: "explorer" | "terminal";
  size: number;
  sessionId: string;
  workspaceGeneration: number;
};
```

Explorer 自己的 tabs/activePath 继续由原 store 管理。Panel store 不接管文件业务，避免形成巨型全局 store。

### 5.2 TODO

- [ ] 用 `WorkspacePanel` 包裹当前右侧 `WorkspaceExplorer`，不改变 Explorer API。
- [ ] 将 open/size/mode 与 explorer tabs 分离，增加 store migration 测试。
- [ ] WorkspaceBar Explorer/Terminal 按钮统一通过 panel actions 切换。
- [ ] 终端图标替换现有 Tool Logs action；把 Tool Logs 移到已决定的新入口。
- [ ] 实现 active/hover/focus/tooltip/快捷键，遵循 button-menu spec。
- [ ] mode 切换保持 Explorer tabs，隐藏 Terminal 时保持或暂停渲染而不丢 session。
- [ ] 小窗口/最小宽度下定义自动关闭/overlay 策略，避免挤坏对话区。
- [ ] 补 resize、持久化、会话切换和快速 toggle 的 UI 测试。

## 6. Phase W3：Rust PTY 与窄 IPC

### 6.1 技术路线

- 前端：`@xterm/xterm` 与 `@xterm/addon-fit`，资源静态打包。
- 后端：优先 `portable-pty 0.9.x`，在当前 Cargo 锁定线完成 PoC 后再写入 manifest。
- `TerminalManager` 持有 session map、owner window/session/workspace、PTY writer/reader、child 和 cancellation token。
- Windows 使用 ConPTY，macOS/Linux 使用 PTY；Shell profile 由后端解析。

### 6.2 IPC

```text
terminal_spawn(chat_session_id, workspace_generation, rows, cols, shell_profile?)
terminal_write(terminal_id, bytes)
terminal_resize(terminal_id, rows, cols)
terminal_kill(terminal_id, reason)
terminal_get_state(terminal_id)

events:
terminal.output { terminal_id, seq, bytes/base64 }
terminal.exited { terminal_id, exit_code?, reason }
```

不要让前端传绝对 cwd、任意可执行路径或完整环境变量。用户选择的 shell profile 必须来自后端允许列表。

### 6.3 Shell 解析

- Windows：用户配置的可信 `pwsh` -> `powershell.exe` -> `cmd.exe` fallback，记录选择结果。
- macOS/Linux：验证 `$SHELL` 是绝对、存在且允许的 executable；否则 zsh/bash/sh fallback。
- 环境变量使用明确继承策略，至少移除 MisakaX 内部 bridge token；本机终端仍继承用户开发环境，但不继承沙箱专用凭据。
- cwd 只从当前 Chat Session workspace 获取并 canonicalize；不存在时不启动。

### 6.4 TODO

- [ ] 做 `portable-pty` 三平台 PoC，验证 ConPTY、resize、Unicode、颜色、TUI、进程退出和许可证。
- [ ] 在锁定 manifests 中加入确定版本的 xterm/fit/PTY 依赖。
- [ ] 实现 `TerminalManager`、随机 ID、ownership、session limits、output seq 和 cleanup。
- [ ] command 只接受 rows/cols/profile/session generation；后端解析 cwd。
- [ ] output 通道增加背压/批处理和最大缓冲，慢前端不能耗尽内存。
- [ ] 输入/resize/kill 校验 window + chat session + workspace generation ownership。
- [ ] 应用退出、窗口关闭、会话删除和明确关闭面板时按策略终止进程树。
- [ ] Windows 使用 Job Object 或等价手段回收子进程；Unix 使用 process group/session。
- [ ] 实现 Shell profile 探测和可诊断 fallback。
- [ ] 实现结构化 `terminal.output/exited` 事件和迟到 seq 丢弃。
- [ ] 增加 session 数、输出速率、输入大小、尺寸范围和命令频率限制。
- [ ] 补 Rust unit/integration，覆盖 owner 篡改、cwd 竞态、崩溃、输出洪水和应用关闭。

### 6.5 退出门

- WebView 不能指定工作区外 cwd 或任意 host executable。
- 关闭应用后测试确认没有遗留 child/grandchild。

## 7. Phase W4：xterm UI 与生命周期

### 7.1 TODO

- [ ] 实现 `TerminalPanel`，按主题 token 创建/销毁 xterm instance。
- [ ] 使用 Fit addon，ResizeObserver debounce 后发送 cols/rows；避免像素尺寸直接传后端。
- [ ] 连接 input/output/exit；二进制/UTF-8 分片在协议层正确处理。
- [ ] 不用 `innerHTML` 处理标题、链接、selection 或终端衍生数据。
- [ ] 默认禁用或严格验证 terminal link provider；外部链接需走现有安全打开流程。
- [ ] 标题固定显示“本机权限”，Shell/cwd 使用非敏感摘要。
- [ ] 工作区切换时提供“在新工作区重启/保留旧终端”，记录明确选择。
- [ ] Shell exit 显示 code/reason 和重启按钮；不无限自动重启。
- [ ] 实现清屏、复制、粘贴、focus restore 和快捷键冲突测试。
- [ ] 为 IME、中文、Emoji、宽字符、ANSI 色、滚动、TUI alternate screen 做实机测试。
- [ ] 终端输出不进入普通 screen-reader live stream；退出/错误使用独立 live region。
- [ ] panel 隐藏/显示和 React StrictMode 下不重复 spawn。

## 8. Phase W5：Tauri 能力与 CSP 收窄

这是终端上线门，而不是可选清理。

### 8.1 TODO

- [ ] 删除主 WebView 的 `shell:allow-execute`、`shell:allow-spawn`、`shell:allow-stdin-write`、`shell:allow-kill`。
- [ ] 检查现有功能是否仍需 `tauri-plugin-shell`；需要的外链打开使用单独窄 permission/scope。
- [ ] 收窄 fs read/write/remove/rename 到确切业务 commands 或 scoped paths。
- [ ] 收窄 HTTP capability 到模型/目录所需域名，敏感请求优先由 Rust/Sidecar 发起。
- [ ] 生产 CSP 禁止远端 script、动态 eval 和任意 frame；xterm/Monaco 所需 worker 采用本地静态配置。
- [ ] 开发 CSP 例外只在 dev config，不能进入 release bundle。
- [ ] 对所有 custom commands 建 capability/authorization 测试，验证其他 window/webview 无权调用 terminal。
- [ ] 做前端 XSS 回归：终端 title、OSC、链接和输出不能注入 DOM/IPC。
- [ ] 运行 `tauri build` 后检查最终 capabilities、CSP 和 bundle 资源，不只检查源码。

## 9. Phase W6：三平台发布验证

### 9.1 TODO

- [ ] Windows 10/11：PowerShell 5、pwsh、cmd、ConPTY resize、长路径、中文路径、Job cleanup。
- [ ] macOS 当前和前两个支持版本：zsh/bash、签名/notarization、PTY 权限、IME。
- [ ] Linux 支持发行版：bash/zsh/fish 可选、Wayland/X11 clipboard、PTY、AppImage/deb/rpm 打包。
- [ ] 测试 Git CLI 缺失、旧版本、worktree、submodule、detached 和 PATH 异常。
- [ ] 测试 10 MB/s 输出、超长行、持续进程、应用崩溃恢复和重新打开。
- [ ] 确认安装包不依赖 CDN，离线启动终端可用。
- [ ] 记录已知 Shell/TUI 兼容差异和诊断入口。
- [ ] 更新 UI 三规范、PROJECT_STRUCTURE 和进度文档。

## 10. 关键风险

| 风险 | 控制 |
|---|---|
| xterm 把真实 Shell 暴露给 WebView | CSP、本地 bundle、窄 IPC、ownership、删除通用 shell permission |
| PTY 输出压垮 event loop | chunk/batch/backpressure、buffer cap、sequence |
| cwd 与 UI 展示不一致 | 后端会话 generation 解析，不接受前端 cwd |
| Windows 子进程遗留 | Job Object + app lifecycle cleanup + crash tests |
| Tool Logs 入口丢失 | 替换图标前先迁移入口和测试 |
| Git 查询频繁/卡顿 | async cache、single-flight、watch debounce、timeout |
| 用户误以为终端已沙箱 | 持续“本机权限”标签；Agent sandbox 采用不同状态和文案 |

## 11. 测试与性能预算

- workspace context 查询 P95 < 300 ms；超时不超过 2 s，期间 composer 可用。
- terminal 打开到首个 prompt P95 < 800 ms（排除首次 Shell profile 自身启动异常）。
- resize 发送频率 <= 10/s；输出事件合并目标 16–32 ms 一批。
- TerminalManager 默认每窗口/会话 session 上限明确；缓冲和 scrollback 有硬上限。
- E2E 必须包含切换会话时的旧事件、关闭面板、关闭窗口、应用退出和 owner 篡改。

## 12. 完成定义

- 所有验收标准和三平台矩阵通过。
- 终端不依赖通用 Tauri Shell 权限，release CSP 已验证。
- Git 仅只读上下文，无未授权扩展功能。
- Tool Logs 有可发现的新入口。
- UI 规范和进度文档同步，未来 Git/terminal 扩展点有接口但无空壳 UI。
