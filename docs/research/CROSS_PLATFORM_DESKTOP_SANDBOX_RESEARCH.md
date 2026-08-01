# MisakaX 跨平台桌面 Agent 沙箱调研

> **用途：** 调研适用于 MisakaX 桌面 Agent 的 Windows、macOS、Linux 沙箱方案，为技术选型和实施提供事实依据。
> **受众：** 架构师、安全工程师、Rust/Python 维护者、发布工程师。
> **最后审阅 / Last reviewed：** 2026-08-01
> **调研基线：** `main@fa24bd7`，Tauri 2.x + Rust 2021 + React 19 + Python 3.11 Sidecar。
> **结论性质：** 本文是调研记录；最终决策见 [`SANDBOX_TECH_SELECTION.md`](../architecture/SANDBOX_TECH_SELECTION.md)。

---

## 1. 执行摘要

MisakaX 需要隔离的不是一个固定可执行文件，而是 Agent 驱动的开放式开发工作负载：Shell、Git、Python、Node、编译器、包管理器、Skill 脚本和它们派生的整个进程树。调研结论如下：

1. **没有一个原生 API 能以同一种实现覆盖三平台并兼顾开发工具兼容性。** 三个平台必须共享策略模型，但使用不同的 OS 适配器。
2. **文件系统隔离和网络隔离必须同时成立。** 只限制写路径仍允许读取密钥并外传；只断网仍允许破坏本机文件。
3. **容器不适合作为默认唯一后端。** Docker/Podman 能提供较强边界，但存在安装、虚拟化、资源、许可证和宿主工作区映射成本，适合作为可选强化模式。
4. **Wasm 不是任意开发命令的透明替代品。** Wasmtime/WASI 很适合未来的受控 Skill 插件 ABI，但不能直接运行现有任意本机工具链。
5. **推荐方向是“统一 Sandbox Broker + 平台适配器”。** Linux 使用 Bubblewrap/namespace/seccomp，macOS 使用 Seatbelt 子进程策略，Windows 使用受限令牌、专用沙箱账户、ACL 和防火墙；策略、审批、审计和生命周期由 Rust 统一管理。
6. **用户主动打开的本机终端与 Agent 自动执行必须分开标识。** 前者默认继承用户权限并明确显示“本机终端”，后者必须走沙箱；以后可增加“沙箱终端”配置，但不能静默改变用户终端语义。

## 2. 当前项目的真实安全基线

### 2.1 已有保护

- Python `WorkspaceBackend` 以工作区为逻辑根，并对挂载路径做规范化校验。
- MCP 已有审批和事件链，可复用于统一的高风险操作审批体验。
- Tauri 负责桌面能力和 Sidecar 生命周期，具备成为进程级策略所有者的条件。
- Skills 安装已有限制压缩包大小、文件数、压缩比、路径穿越、符号链接和原子替换的基础能力。

### 2.2 关键缺口

- `LocalShellBackend` 最终在宿主机直接启动命令，`inherit_env=true`，逻辑路径约束不等于 OS 安全边界。
- Python Sidecar 自身拥有当前用户权限；一旦依赖或工具实现被利用，可绕过前端和 Rust 约束。
- `src-tauri/capabilities/default.json` 当前向主 WebView 暴露了宽泛的 Shell、文件系统和 HTTP 权限。
- `tauri.conf.json` 的 CSP 为空；嵌入终端后，WebView 脚本上下文能够触达真实 Shell I/O，风险显著上升。
- 网络没有默认拒绝、域名代理、凭据隔离或子进程树级审计。
- 没有跨平台资源限制、进程树清理和沙箱能力自检。

因此，现有的“工作区路径守卫”只能视为应用逻辑校验，不能在产品文案中称为安全沙箱。

## 3. 威胁模型

### 3.1 需要防护的行为

- 恶意或被提示注入的 Skill 指示 Agent 读取 SSH Key、浏览器数据、云凭据或其他工作区。
- 下载后执行、反向 Shell、直接 Socket 外传、绕过代理的网络访问。
- 修改工作区外文件、启动项、注册表、Shell 配置、Agent 身份/记忆文件。
- 通过子进程、孙进程、后台守护进程逃逸生命周期管理。
- 利用符号链接、junction、Git worktree、路径大小写或竞态绕过可写根。
- 继承宿主环境变量、代理配置、凭据助手、SSH Agent 或系统句柄。
- 终端控制序列、链接处理或 WebView XSS 影响终端输入输出。
- 不受限制的 CPU、内存、进程数、文件数和磁盘占用造成拒绝服务。

### 3.2 非目标与边界

- 首期不承诺抵御内核 0-day 或已获得管理员/root 权限的恶意代码。
- 沙箱不能证明 Skill 的业务意图良性；它只限制影响范围，仍需安装前扫描和运行时审批。
- 本机终端是用户主动操作能力，默认不声称隔离；必须与 Agent 自动命令在 UI、事件和审计中区分。
- `danger-full-access` 类模式只作为用户显式、短时、可审计的逃生口，不属于默认安全保证。

## 4. 评价维度

| 维度 | 需要回答的问题 |
|---|---|
| 文件系统 | 能否实现只读根、工作区可写、`.git`/配置等子路径再次只读、拒绝其他敏感路径？ |
| 网络 | 能否默认断网，并通过可信代理实现域名级允许和审批？ |
| 进程树 | 限制是否覆盖所有后代进程，退出时能否可靠清理？ |
| 开发兼容性 | 能否运行 Git、Python、Node、编译器和包管理器等本机工具？ |
| 权限需求 | 首次设置和日常运行是否需要管理员/root？失败时如何降级？ |
| 可分发性 | Tauri 安装包能否合规捆绑；是否依赖用户安装大型运行时？ |
| 可观测性 | 能否输出命中策略、拒绝原因、网络目的地和资源使用？ |
| 三平台一致性 | 是否能将同一上层策略映射为可解释的近似等价保证？ |
| 维护风险 | API 稳定性、上游更新、许可证、测试矩阵和逃逸面是否可控？ |

## 5. 候选方案对比

| 方案 | Windows | macOS | Linux | 任意本机工具 | 网络强隔离 | 默认采用结论 |
|---|---:|---:|---:|---:|---:|---|
| OS 原生/低层组合适配器 | 是 | 是 | 是 | 是 | 可实现 | **推荐主方案** |
| Docker/Podman/OCI 容器 | 有条件 | 有条件 | 是 | 需镜像内工具 | 是 | 可选强化后端 |
| Windows Sandbox/轻量 VM | Pro 等版本 | 不适用 | 不适用 | 是 | 是 | 不作为产品默认 |
| AppContainer/LPAC | 是 | 不适用 | 不适用 | 兼容性受限 | 是 | 不适合开放式开发负载 |
| macOS App Sandbox（整应用） | 不适用 | 是 | 不适用 | 权限/签名复杂 | 是 | 可保护 App，但不能单独解决 Agent 命令策略 |
| Wasmtime/WASI | 是 | 是 | 是 | **否**，需编译为 Wasm | 能力式 | 未来插件后端，不替代 Shell 沙箱 |
| 仅路径校验/环境变量代理 | 是 | 是 | 是 | 是 | 否 | 仅纵深措施，不可称为沙箱 |
| 直接依赖 Codex/Claude CLI 沙箱 | 部分 | 是 | 是 | 是 | 取决于上游 | 不作为稳定产品依赖，可参考/复用经审计组件 |

## 6. 平台调研

### 6.1 Linux：Bubblewrap 为主，Landlock 为补充

Bubblewrap 可通过 user、mount、PID、network namespace 构造最小文件系统视图，配合 `PR_SET_NO_NEW_PRIVS`、seccomp 和新会话约束进程。它本身是低层构造器，**不是自带正确策略的完整沙箱**；安全性取决于调用方参数，尤其要处理绑定的 socket、D-Bus、TTY、符号链接和嵌套 namespace。

建议实现：

- 根文件系统默认只读；只把规范化后的工作区和每会话临时目录重新绑定为可写。
- `.git`、`.misakax`、`.codex`、`.agents` 和用户配置的保护路径在可写根内部重新只读或不可见。
- 默认 `--unshare-user --unshare-pid --unshare-net --new-session`，挂载新的 `/proc`，禁止提权。
- 通过外部可信网络 Broker 转发已批准域名；无代理模式下保持独立网络 namespace。
- 使用 seccomp 阻断不需要的 socket/namespace/ptrace 等能力，并设置资源限制。
- 启动前检测 user namespace、Bubblewrap 版本和内核能力；严格模式不可静默退化为宿主执行。
- Landlock 可作为额外文件访问层或兼容性回退，但内核支持和 ABI 能力需要运行时探测。

OpenAI Codex 当前 Linux 实现同样以 Bubblewrap 为默认文件系统沙箱，并将根只读、可写根重绑定、受保护子路径再次只读，网络受限时隔离 network namespace；这证明该路线适合 Agent 开发负载，但不能照搬未审计的命令行拼接。

### 6.2 macOS：Seatbelt 子进程策略，而非只依赖整应用 App Sandbox

Apple App Sandbox 是内核执行的应用级能力系统，适合通过 entitlement 预先声明资源；但 MisakaX 需要让 Agent 调用开放式本机工具链。仅把整个 Tauri App 放入 App Sandbox 会引入签名、用户选择目录、辅助程序 entitlement 和可执行文件隔离的复杂度，仍需对子进程生成更细粒度策略。

建议实现：

- 使用独立、签名的 runner 为每次命令生成 Seatbelt profile，工作区可写、其他用户数据拒绝或只读。
- 默认拒绝网络；需要联网时只允许连接本地 Broker，由 Broker 执行域名允许、DNS 固定/重绑定防护和审计。
- 限制 Mach 服务、进程控制、设备和敏感系统接口；runner 与主 App 分离，减少 entitlement 扩散。
- 将 `sandbox-exec`/Seatbelt 作为平台实现细节并建立每个 macOS 支持版本的实机回归；不要把未公开策略语法的稳定性当作永久保证。
- 如未来启用整应用 App Sandbox，使用 security-scoped bookmark 管理用户选择的工作区，并单独验证 helper/Sidecar 签名和继承关系。

Anthropic 的开源 Sandbox Runtime 采用 macOS Seatbelt、Linux Bubblewrap 和外部代理，但不覆盖 Windows，适合作为策略与网络代理设计参考，不足以单独满足 MisakaX 三平台要求。

### 6.3 Windows：开放式开发负载需要专用身份与强网络边界

Microsoft AppContainer/LPAC 是强能力边界，能隔离文件、注册表、网络、凭据和进程，但更适合预知能力的应用。开放式 Agent 需要调用用户已有的 Git、Python、Node、编译器和包管理器，兼容成本高。

Windows Sandbox 使用 Hyper-V 类虚拟化，隔离较强，但不支持 Home、需要虚拟化和额外资源，并且把真实工作区、工具链和凭据桥接到一次性桌面会损害体验，因此不适合作为默认后端。

OpenAI 公开的 Windows Codex 沙箱演进提供了更贴近本项目的证据：

- 仅使用环境变量“毒化代理”无法阻止直接 Socket，网络限制只是建议性。
- 写限制可以通过 synthetic SID、write-restricted token 和工作区 ACL 实现。
- 强网络隔离需要一次性提权设置：创建 online/offline 专用本地用户、用防火墙阻断 offline 用户，并由独立 command runner 创建受限令牌和子进程。
- Setup Helper 和 Command Runner 必须与普通权限主进程分离，减少日常 UAC 和平台代码扩散。

建议 MisakaX 采用相同的设计原则，但实现自己的窄接口与品牌隔离：

- `misakax-sandbox-setup.exe`：仅在用户明确启用强沙箱时提权，创建/校验专用账户、SID、ACL 和防火墙规则；可卸载、可修复、可审计。
- `misakax-sandbox-runner.exe`：作为专用沙箱用户运行，创建 write-restricted token 后启动真正命令。
- 使用 Job Object 限制并回收整个进程树；关闭句柄继承，构造最小环境变量集。
- offline 身份默认断网；online 身份仍必须经 Network Broker 或明确的策略批准，不把“可联网账户”当作无限制网络。
- 未完成强沙箱设置时，只能进入带显著提示的兼容模式；高风险 Skill 和无人值守执行必须 fail closed。

### 6.4 容器/VM：可选高隔离策略

Docker Desktop 覆盖三大桌面平台，但 Windows 通常依赖 WSL2/Hyper-V，Linux Desktop 本身运行 VM，至少需要数 GB 内存；较大企业使用还涉及付费许可。把它设为强制依赖会显著抬高首次使用门槛。

容器适合以下可选场景：

- 不信任程度高、工具链可由镜像固定的 Skill 试运行。
- CI 安全回归、恶意样本扫描、依赖安装和构建。
- 用户已安装 Docker/Podman，希望获得可丢弃环境。

容器后端必须避免挂载 Docker daemon socket；工作区挂载仍要只读/可写分区，网络默认关闭，并固定镜像 digest。

### 6.5 Wasmtime/WASI：适合未来受控插件，不适合当前通用终端

WebAssembly 通过显式 imports/exports 和 WASI capability 提供良好隔离，适合将未来的 Skill 执行逻辑限制在定义好的文件、网络和主机函数中。但现有 Skills 主要是 Markdown 指令和 Python/PowerShell/Bash/JavaScript 脚本，用户项目也依赖任意本机命令。强制转换为 Wasm 会破坏兼容性。

建议保留 `WasmSandboxProvider` 扩展点，但不把它列为首期 Agent Shell 后端。

## 7. 网络隔离必须是第一等能力

一个可接受的网络模型至少包括：

1. Agent 命令默认无网络；DNS 也不能绕过策略。
2. 允许联网时，子进程只连接本地 Broker/代理，不能直接出站。
3. 策略按域名、端口、协议、会话、Skill 和用途授权；IP 字面量、重定向、DNS rebinding、代理 CONNECT 需要校验。
4. API Key 不注入沙箱环境；由宿主侧模型/凭据网关代持，使用短期、范围受限的会话令牌。
5. 每次放行产生审计事件，并支持一次、会话、工作区三种有效期。
6. Git 远端操作后续应通过专门的 VCS Broker 附加凭据和校验目标，而不是把 SSH Agent 或凭据助手直接暴露给沙箱。

## 8. 与嵌入式终端的关系

嵌入终端会把真实 Shell I/O 暴露给 WebView 中的 xterm.js。xterm.js 官方明确提示：任何能操作同一脚本上下文的 JavaScript 都可能操纵终端输入输出，终端数据、标题和链接必须视为不可信。

MisakaX 应采用以下边界：

- 终端前端资源全部随应用静态打包，禁止 CDN 和运行时动态 JavaScript 更新。
- 启用严格 CSP，禁止 `eval`/任意远端脚本；不使用 `innerHTML` 渲染终端衍生数据。
- WebView 不能直接持有通用 Shell 插件权限，只能调用 `terminal_spawn/input/resize/kill` 等窄命令。
- Session ID 使用不可猜测随机值并绑定窗口、会话和工作区；输入、输出、resize 都校验所有权。
- 主进程不以管理员/root 权限驱动终端；退出、切换会话和应用关闭时清理 PTY 进程树。
- UI 明确区分“本机终端”和“Agent 沙箱执行”，避免用户误以为两者具有相同保护。

## 9. 调研结论

MisakaX 最佳路线不是寻找一个“跨平台沙箱库”，而是建立稳定的上层安全协议：`SandboxPolicy`、`SandboxBroker`、审批、网络代理、审计和能力探测；平台差异隔离在 runner adapter 内。该架构同时允许未来接入容器、远程工作区或 Wasm，而不会让 React、Skills 或 Agent 编排直接依赖平台细节。

## 10. 主要资料

以下均为截至 2026-08-01 查阅的官方或项目原始资料：

- [OpenAI：Building a safe, effective sandbox to enable Codex on Windows](https://openai.com/index/building-codex-windows-sandbox/)
- [OpenAI Codex Linux sandbox README](https://github.com/openai/codex/blob/main/codex-rs/linux-sandbox/README.md)
- [OpenAI Codex Rust README（各平台 sandbox 命令）](https://github.com/openai/codex/blob/main/codex-rs/README.md)
- [Anthropic：Making Claude Code more secure and autonomous with sandboxing](https://www.anthropic.com/engineering/claude-code-sandboxing)
- [Anthropic Sandbox Runtime](https://github.com/anthropic-experimental/sandbox-runtime)
- [Microsoft：AppContainer isolation](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation)
- [Microsoft：Launch an AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer)
- [Microsoft：Windows Sandbox requirements](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/)
- [Apple：App Sandbox](https://developer.apple.com/documentation/security/app-sandbox)
- [Apple：Accessing files from the macOS App Sandbox](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox)
- [Bubblewrap README](https://github.com/containers/bubblewrap/blob/main/README.md)
- [Linux Kernel：Landlock userspace API](https://www.kernel.org/doc/html/latest/userspace-api/landlock.html)
- [Docker Desktop on Windows](https://docs.docker.com/desktop/setup/install/windows-install/)
- [Docker Desktop on macOS](https://docs.docker.com/desktop/setup/install/mac-install/)
- [Docker Desktop on Linux](https://docs.docker.com/desktop/setup/install/linux/)
- [Wasmtime Security](https://docs.wasmtime.dev/security.html)
- [xterm.js Security](https://xtermjs.org/docs/guides/security/)
- [portable-pty 0.9.0](https://docs.rs/crate/portable-pty/0.9.0)
