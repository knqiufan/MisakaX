# 跨平台 Sandbox 实施计划

> **用途：** 基于已选架构，把 Agent、Skill 和 MCP 的自动执行逐步迁移到 Windows/macOS/Linux 的真实 OS 沙箱。
> **受众：** Rust、Python Sidecar、安全、构建发布和测试维护者。
> **最后审阅 / Last reviewed：** 2026-08-01
> **状态：** 规划；平台 Spike Gate 未通过前不得宣称 Sandbox 已完成。
> **关联：** [调研](../research/CROSS_PLATFORM_DESKTOP_SANDBOX_RESEARCH.md) · [ADR](../architecture/SANDBOX_TECH_SELECTION.md) · [总体架构](../architecture/WORKSPACE_SKILLS_SECURITY_ARCHITECTURE.md)

---

## 1. 目标与保证等级

### 1.1 交付目标

- Agent Shell、Skill 脚本、扫描 helper 和本地 MCP 子进程通过统一 `SandboxBroker` 启动。
- 文件系统策略支持只读、工作区可写、保护子路径和显式 full access。
- 网络默认拒绝；通过可信 Broker 实现域名级审批和审计。
- Windows/macOS/Linux 的 provider 具有近似等价的上层策略语义和明确能力差异。
- Python Sidecar 不再直接用 `LocalShellBackend` 在宿主继承环境执行生产命令。
- 沙箱不可用、配置损坏或策略构造失败时严格模式 fail closed。
- UI 能显示当前保证、设置/修复步骤、审批和审计，而不是只有“执行失败”。

### 1.2 产品保证等级

| 等级 | 含义 | 使用场景 |
|---|---|---|
| `strict` | 文件 + 网络 + 进程树 + 环境/资源控制通过能力自检 | 默认 Agent/未知 Skill/无人值守 |
| `restricted-compat` | 文件边界成立，但部分网络/资源保证不足；持续警告 | 用户显式选择的受限兼容 |
| `full-access` | 当前用户权限执行，无沙箱保证；逐操作批准并审计 | 明确逃生操作 |
| `unavailable` | provider/setup 不满足最低要求 | 禁止自动执行 |

不能把 `restricted-compat` 显示成“已启用沙箱”，必须列出缺失保证。

## 2. 安全不变量

实现和重构期间必须始终成立：

1. 沙箱 policy 由 Rust 生成，前端和模型不能直接提供 platform args/profile。
2. cwd 和 writable roots 来自当前会话的 canonical workspace，不信任模型字符串。
3. 默认无网络；环境变量 proxy 只能纵深防御，不能作为强网络边界。
4. API Key、SSH Agent、云凭据和 bridge master token 不进入子进程环境。
5. 限制覆盖完整进程树；主进程退出或取消后不能留后台任务。
6. 所有允许/拒绝、能力降级和 full-access 都有结构化审计。
7. provider 异常绝不能 fallback 到普通 `subprocess`/Tauri Shell。
8. 用户本机终端不经过本 Broker，必须用类型和 UI 明确区分。

## 3. 实施顺序

```text
B0 威胁模型/测试工具/三平台 Spike
  -> B1 统一领域契约与 Broker（先使用 fake provider）
    -> B2 Linux provider ┐
    -> B3 macOS provider ├─ 可并行，分别过平台 gate
    -> B4 Windows provider┘
      -> B5 Python/MCP/Skill 执行链迁移
        -> B6 Network/Approval/Audit 强化
          -> B7 默认启用、发布和运维
```

平台 provider 可以并行，但 B5 合并前必须统一 runner protocol 和 contract tests。

## 4. Phase B0：威胁模型、基线和平台 Spike

### 4.1 Spike 产物

每个平台建立独立测试 runner，接受同一 JSON policy fixture 并返回能力和结果。Spike 不接入生产 UI，先验证安全原语和打包。

代表性攻击 fixture：

- 写工作区内/外、写 `.git`、符号链接/junction 逃逸、rename 交换竞态。
- 读 SSH/云/浏览器目录、读取环境变量和打开父进程句柄。
- HTTP/DNS/direct socket/IPv6/localhost/Unix socket/命名管道。
- child/grandchild/background daemon、双 fork、Windows detached process。
- CPU/memory/process/file/descriptor 炸弹。
- 路径空格、Unicode、长路径、Git worktree/submodule。

### 4.2 TODO

- [ ] 将调研威胁模型转为可执行的跨平台 JSON fixture 和 expected result。
- [ ] 定义能力枚举：filesystem/network/process_tree/resources/env/audit/setup_required。
- [ ] 定义 runner protocol v1：framed JSON/pipe、结构化 argv、cwd、policy hash、nonce、result。
- [ ] 为 runner 消息做 schema validation、大小上限、超时和 replay/ownership 设计。
- [ ] Linux Spike：Bubblewrap 版本选择、system/bundled 解析、userns、seccomp、network namespace、Landlock 探测。
- [ ] macOS Spike：Seatbelt profile、签名 helper、network Broker、支持版本和 App Sandbox/notarization 组合。
- [ ] Windows Spike：write-restricted token、synthetic SID、ACL、Job Object、专用账户、防火墙、DPAPI、UAC setup。
- [ ] 容器 Spike（可选）：Docker/Podman 检测、只读镜像、无 daemon socket、workspace mount 和启动开销。
- [ ] 测试 helper/runner 的许可证、供应链、签名和打包路径。
- [ ] 记录每平台不支持项；确定 strict 的最低能力，不以最弱平台无条件拉低全部标准。
- [ ] 安全 review Spike 代码，禁止未经 review 的命令拼接进入后续实现。

### 4.3 退出门

- 三平台都能执行同一组核心 filesystem/network/process fixtures。
- Windows 强网络方案已证明可行；若只有 proxy 环境变量，Windows gate 不通过。
- 选定 runner protocol v1 和 helper 分发策略。

## 5. Phase B1：Sandbox Domain、Broker 与 Fake Provider

### 5.1 模块

- `SandboxPolicy`：文件、网络、环境、资源、审批和模式。
- `SandboxPolicyCompiler`：从工作区/设置/操作来源生成不可变 policy。
- `SandboxProvider`：prepare/spawn/terminate/capabilities。
- `SandboxBroker`：provider 选择、lifecycle、审批和审计协调。
- `ExecutionService`：Agent/Skill/MCP 面向的 application API。
- `FakeSandboxProvider`：测试，不允许编入 release 默认路径。

### 5.2 TODO

- [ ] 新增 domain types，确保不依赖 Tauri、Python 或 platform crates。
- [ ] 实现 policy normalization、稳定 hash 和等价性测试。
- [ ] 实现 canonical workspace/writable/readonly/denied roots 解析，处理嵌套优先级。
- [ ] 实现 environment allowlist/denylist，默认 `inherit_env=false`。
- [ ] 实现 resource policy（timeout、process、CPU、memory、files/handles、output）。
- [ ] 实现 provider registry/strategy 和 capability negotiation。
- [ ] 实现 `SandboxBroker` state machine：preparing/running/approval_wait/terminating/exited/failed。
- [ ] 实现 execution ownership、cancellation、idempotent terminate 和 app shutdown cleanup。
- [ ] 定义结构化 audit events、Secret redaction 和 correlation ID。
- [ ] 实现 fake provider/contract test suite，所有 platform adapter 必须复用。
- [ ] 新增 feature flags/settings schema，但 UI 先显示“实验性/不可用”。
- [ ] 确保 release build 在没有真实 provider 时返回 `SANDBOX_UNAVAILABLE`，不调用宿主命令。

## 6. Phase B2：Linux Provider

### 6.1 实现要点

- 根 `ro-bind / /`，按策略 bind writable roots；保护子路径按路径特异性重新 ro-bind/deny。
- `--unshare-user --unshare-pid --unshare-net --new-session`，新 `/proc`，`no_new_privs`。
- seccomp 限制网络/namespace/ptrace/危险 ioctl；不绑定 D-Bus、SSH agent、Docker socket。
- managed proxy 模式由外部 Broker 通过最小 Unix socket 桥接；无网络模式彻底断开。
- 启动时检查 system bwrap 可信路径和版本；如捆绑 binary，校验签名/hash。
- Landlock 可增加文件约束或作为明确的兼容 provider，不自动宣称与 bwrap 等强。

### 6.2 TODO

- [ ] 实现 Linux capability probe 和诊断码。
- [ ] 实现 bwrap argv builder，使用类型化 mount entries，不接受原始用户参数。
- [ ] 正确处理 nested writable/readonly/denied roots 和不存在/符号链接路径。
- [ ] 应用 no-new-privs、PID/user/network namespaces 和新 session。
- [ ] 设计并加载最小 seccomp profile，做架构差异测试。
- [ ] 实现进程组/cgroup 可用时资源限制和完整 kill tree。
- [ ] 实现 network broker socket 映射，禁止任意 Unix socket。
- [ ] 测试 system bwrap PATH 劫持、旧版本和 userns 禁用。
- [ ] 在 Ubuntu/Debian/Fedora/RHEL 支持线运行 contract/attack/packaging tests。
- [ ] 验证 AppImage/deb/rpm 中 helper 权限、hash 和升级替换。
- [ ] 编写用户可操作诊断：如何启用 userns/为什么 strict 不可用。

## 7. Phase B3：macOS Provider

### 7.1 实现要点

- 独立签名 runner 为每个 policy 生成 Seatbelt profile；主 App 不扩大到同等权限。
- workspace user-selected access/bookmark 与 runner 可见路径协同，避免用临时全盘 entitlement。
- 默认禁止网络和不必要 Mach services；联网只连本地 Broker。
- 使用 process group 和资源限制回收子进程；验证 helper/Sidecar 的 Hardened Runtime/notarization。
- 对 Seatbelt/profile 行为建立每个支持 macOS 大版本实机测试。

### 7.2 TODO

- [ ] 实现 macOS capability probe、runner 签名校验和系统版本矩阵。
- [ ] 实现类型化 Seatbelt profile compiler 和转义/fuzz tests。
- [ ] 映射 readonly/writable/denied roots，覆盖 symlink、bookmark、APFS/case sensitivity。
- [ ] 实现网络默认拒绝和只允许本地 Broker 的 profile。
- [ ] 限制 Mach/IPC/device/process-control 能力并记录必要例外。
- [ ] 实现 process group、timeout、output/resource control 和 kill tree。
- [ ] 验证主 App 启用/不启用 App Sandbox 两种分发路线的影响，固定正式路线。
- [ ] 验证 helper、Python Sidecar、PTY helper 的签名、entitlement、notarization 和升级。
- [ ] 在当前及前两个受支持 macOS 大版本运行 attack/compat/packaging tests。
- [ ] 对 profile compile/launch failure 严格拒绝并提供诊断，不回退宿主执行。

## 8. Phase B4：Windows Provider

### 8.1 组件

- `misakax-sandbox-setup.exe`：需要时以 UAC 提权，幂等创建/校验/修复/卸载账户、SID、ACL、防火墙。
- `misakax-sandbox-runner.exe`：在专用用户上下文创建受限 token，启动 child。
- 主 Tauri 进程：普通用户权限，调用 setup/runner 的窄协议。

建议身份：

- `MisakaXSandboxOffline`：默认 Agent 命令，防火墙阻断出站。
- `MisakaXSandboxOnline`：只在 Network Broker 场景使用；仍不把无限网络直接交给命令。

密码/凭据使用 DPAPI 加密，存放位置对 sandbox user 不可读；日志不得记录。

### 8.2 TODO

- [ ] 完成账户/SID/ACL/firewall 命名、升级兼容和卸载 threat review。
- [ ] 实现 setup helper 的 create/validate/repair/remove，所有操作幂等并输出结构化结果。
- [ ] UAC 只用于 setup/repair/remove；日常执行不提权。
- [ ] 实现 DPAPI secret storage，校验 sandbox 用户无法读取密文/相关主密钥上下文。
- [ ] 工作区和额外 writable roots 应用精准 ACL；保护 `.git/.misakax/.codex/.agents`。
- [ ] 评估 ACL 应用性能和继承语义，避免不可逆污染用户项目；记录/恢复修改。
- [ ] 实现 runner 的 `LogonUser/CreateRestrictedToken/CreateProcessAsUser` 等等价流程。
- [ ] 使用 Job Object 限制/回收完整进程树，处理 breakaway、detached 和 nested job。
- [ ] 创建/校验 offline outbound firewall rules；测试 direct socket、IPv4/IPv6、DNS、QUIC。
- [ ] online 模式只通过 Network Broker 或受控规则，避免任意出站。
- [ ] 构造最小 environment/block handle inheritance；保护 user profile、credential manager 和 named pipes。
- [ ] 支持路径空格、Unicode、长路径、junction、reparse point、Git worktree。
- [ ] 验证 Windows Home/Pro 的支持差异；不依赖 Windows Sandbox/Hyper-V。
- [ ] 在 Windows 10/11、标准用户、企业策略/防火墙异常下运行 attack/compat/installer tests。
- [ ] 实现 setup health UI：未设置、设置中、可用、需修复、卸载，并提供 correlation ID。
- [ ] 卸载验证无专用账户、危险 ACL、凭据和防火墙残留；失败时提供修复工具。

## 9. Phase B5：Python、Skills 和 MCP 执行链迁移

### 9.1 执行桥

推荐由 Rust 持有沙箱权威，通过绑定 `127.0.0.1` 或本地 pipe 的 authenticated bridge 接受 Python 请求：

- 主进程为每个 Sidecar/Chat Session 生成短期随机 token。
- token 绑定 session、workspace generation、允许工具和到期时间。
- 请求是结构化 execute/read/write/list，不传任意 host path。
- Rust 再次执行 path、policy、approval 和 ownership 校验。

### 9.2 TODO

- [ ] 设计 bridge transport（loopback/UDS/named pipe）并完成本地攻击面评审。
- [ ] token 使用足够熵、短期、scope、rotation 和 constant-time 验证；不放环境/日志可被 child 读取的位置。
- [ ] Python 实现 `BrokeredWorkspaceBackend`，映射 DeepAgents 所需 filesystem/shell API。
- [ ] production `build_agent` 停止使用直接 `LocalShellBackend`；测试可显式 fake。
- [ ] 文件工具也经 broker/canonical root，防止 Python 插件绕过 Shell 沙箱直接读宿主。
- [ ] `inherit_env=true` 从生产执行路径删除。
- [ ] Skills activation view 只读挂载；Skill script 请求标注 skill_id/artifact_hash。
- [ ] 本地 MCP server spawn 走 ExecutionService；远端 MCP 网络走 NetworkPolicy。
- [ ] 扫描 helper 用 read-only/offline policy；证明扫描不会执行目标文件。
- [ ] 实现 bridge disconnect/cancel/retry 幂等；Sidecar 重启使旧 token 失效。
- [ ] 对现有工具调用和流式事件做兼容测试，审批等待不能阻塞整个 Sidecar event loop。
- [ ] 记录仍留在 Python 宿主权限控制面的可信代码清单和后续 worker isolation 计划。

## 10. Phase B6：Network Broker、审批和审计

### 10.1 Network Broker

- deny-by-default；策略包含 domain、port、protocol、request source、scope 和 TTL。
- 解析 DNS 后校验目标；跟随 redirect 时重新检查；防止 IP literal、rebinding 和内网/metadata endpoint 绕过。
- 只让 sandbox child 连接 Broker，不把模型 API Key 或 Git 凭据放进 sandbox。
- 对 HTTP(S)、Git 和包管理器定义兼容测试；不承诺任意自定义协议都能透明代理。

### 10.2 Approval Broker

复用 MCP 现有审批体验，但抽为通用领域服务：filesystem escalation、network domain、full access、Skill review 和 Git credentialed operation 都使用同一批准记录模型。

### 10.3 TODO

- [ ] 定义 network policy、domain normalization、port/protocol、scope/TTL 和 deny precedence。
- [ ] 实现 DNS/redirect/rebinding/private range/metadata endpoint 检查。
- [ ] 建立每平台 child -> Broker 的唯一允许通道，验证 direct socket 失败。
- [ ] 实现宿主侧 Model Gateway/凭据代持的最小 PoC，避免 Key 注入 child。
- [ ] 抽取通用 `ApprovalBroker`，迁移 MCP 审批而不改变现有行为。
- [ ] 审批 UI 显示操作来源、工作区、Skill、目标域/路径、持续时间和风险。
- [ ] 支持 allow once/session/workspace，默认最短；永久允许需要设置级操作。
- [ ] 所有 approval 可撤销、过期并写 audit；拒绝结果返回稳定错误码。
- [ ] audit 做 Secret/path redaction、retention、导出和用户清理策略。
- [ ] 安全团队建立 proxy/approval fuzz、并发和 confused-deputy 测试。

## 11. Phase B7：默认启用、发布和运维

### 11.1 TODO

- [ ] dev 渠道先 shadow/audit：计算 policy 和能力但不宣称隔离，比较预期影响。
- [ ] 按平台逐步启用 strict；未过 gate 的平台保持 unavailable，不用兼容模式冒充完成。
- [ ] Settings 增加沙箱状态、provider、保证列表、诊断、设置/修复和审计入口。
- [ ] Agent 执行区持续显示当前 mode；full access 每次清晰提示。
- [ ] 定义 policy/provider/rule 的版本升级、签名、回滚和紧急禁用机制。
- [ ] 安装包生成 SBOM，签名 setup/runner/Sidecar，验证更新原子性。
- [ ] CI 建立 Windows/macOS/Linux attack suite；夜间跑重型资源/逃逸测试。
- [ ] 发布前安排独立安全 review/渗透测试，特别关注 runner protocol、Windows ACL/firewall 和 Network Broker。
- [ ] 制定事故响应：撤销 Skill、禁用 provider、收集非敏感诊断、清理残留进程/账户/规则。
- [ ] 更新用户文档：保证、限制、兼容模式、终端区别和卸载清理。

## 12. 平台发布 Gate

每个平台分别签字，不允许以“代码可编译”代替：

- [ ] 工作区外写、受保护子路径写和敏感读取失败。
- [ ] HTTP/DNS/direct socket/IPv6/常见工具联网在 offline 模式失败。
- [ ] allowlisted network 正常，redirect/rebinding/内网绕过失败。
- [ ] child/grandchild/background/breakaway 被回收。
- [ ] Python/Node/Git/Rust/包管理器兼容样本达到约定通过率。
- [ ] resource/output limits 有效，拒绝服务不会拖垮主 App。
- [ ] provider/setup 损坏时 fail closed 且诊断可操作。
- [ ] 签名、安装、升级、修复、卸载通过，无危险残留。
- [ ] CSP/capabilities/bridge token/Secret redaction 通过安全 review。
- [ ] 用户看到的保证与实际 capability probe 完全一致。

## 13. 回滚原则

- provider 发布故障时可通过签名配置把 strict 标为 unavailable；**不能自动回退 full access**。
- Windows setup/ACL/firewall 变更必须记录 transaction/marker，repair/remove 可重放。
- policy migration 保留上一个已签名版本；回滚需记录 audit。
- Python backend 保留 fake/legacy adapter 只用于测试和显式开发开关，release 不包含自动 fallback。
- 审计和用户文件不随 provider 回滚删除。

## 14. 完成定义

- 三平台 Spike 和发布 Gate 都通过，ADR 状态转 Accepted。
- Agent/Skill/MCP 生产执行路径不存在直接宿主 Shell fallback。
- strict 模式同时具备文件、网络、进程树和环境/资源保证。
- 安装/升级/卸载、诊断、审批、审计和事故流程完备。
- UI、架构、项目结构和进度文档与实际代码一致。
