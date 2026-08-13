# Execution Isolation / Sandbox 实施计划

> **用途：** 按最新 ADR 将 Agent、Skill、MCP、外部转换器和高风险 helper 逐步迁移到可验证、可审计且 fail-closed 的执行隔离链路。
> **受众：** Rust、Python Sidecar、安全、构建发布和测试维护者。
> **最后审阅 / Last reviewed：** 2026-08-13
> **状态：** S0 provider-neutral 合同已实现并通过定向测试；S1–S4 未开始，当前没有可发布 strict Provider。
> **关联：** [ADR](../architecture/SANDBOX_TECH_SELECTION.md) · [2026-08 复核](../research/SANDBOX_STRATEGY_REASSESSMENT_2026-08.md) · [富内容计划](./rich-content-delivery/README.md)

---

## 1. 目标与当前边界

### 1.1 交付目标

- 所有自动外部执行只通过 Rust `SandboxBroker` / `ExecutionService`，不由 React、Python Sidecar、Skill 或 MCP 直接启动宿主进程。
- strict 使用 snapshot，真实工作区写回经过 result manifest、hash、base generation、diff 与用户批准。
- 网络默认拒绝；联网只走具备攻击测试证据的 Provider broker。
- Provider 精确选择、按执行位置给出事实证据；不可用时稳定拒绝，不 fallback 到 host-direct。
- 取消、崩溃和应用退出回收完整进程树/remote lease/临时数据；审计不记录 secret 或用户文件内容。
- UI 只显示真实保证、缺口、执行位置、地区/网络和修复动作。

### 1.2 当前完成边界（2026-08-13）

已实现 `src-tauri/src/services/sandbox/`：

- `ExecutionIntent`、`ExecutionPolicy`、`ExecutionPlan`、`ExecutionLocation`、`WorkspaceDelivery`、`NetworkMode`、`ProviderAvailability`、`IsolationEvidence`。
- 仅允许 registered tool / typed converter + structured argv；拒绝 host path、平台参数与敏感环境名。
- strict snapshot、敏感目录/凭据/链接排除、SHA-256 和大小/文件数限制。
- result change/artifact manifest 校验、result root containment、hash/配额、写回 approval/base generation/conflict 预检。
- Provider registry、create/collect/destroy 生命周期、exact-location 选择、evidence gate、稳定错误码和脱敏 audit event。
- `FakeProvider` 仅在 `#[cfg(test)]` 中；生产代码没有真实 Provider、host process spawn、Tauri command 或 capability 扩张。

尚未实现：事务化 writeback apply、持久审批/审计、执行桥、真实 remote/AppContainer/Linux/XPC/VM Provider、UI、三平台 Spike、默认启用和发布运维。因此 R5/R6 总阶段仍不能标记完成。

## 2. 安全不变量

1. 策略由 Rust 从可信 session/workspace/settings/approval 构造；模型与前端不传 Provider credential、host path、ACL/profile、VM/mount 或网络规则。
2. strict 固定 snapshot；`.git`、`.misakax`、`.codex`、`.agents`、凭据、越界链接不进入 Provider。
3. 结构化 argv 不经 platform shell 二次解释；确需 Shell 的未来合同必须单独类型化、批准和审计。
4. 默认无网；proxy 环境变量不是边界。模型 API key、Git/SSH/cloud credential、浏览器 session 不进入 guest/child。
5. result manifest 是不可信输入；路径、hash、大小、删除范围、generation、evidence 和 artifact MIME 必须由 Rust 复验。
6. Provider 异常、实验 API 缺失、签名/镜像/地区不符、destroy 失败均不能触发宿主 fallback。
7. 用户 PTY 终端是 `LocalTerminalSession`，不经过 Broker，也不得在 UI/类型中冒充 `SandboxExecution`。
8. 内容安全与执行隔离叠加；strict 输出仍经过 `ArtifactService` / `ContentSafetyPolicy`。

## 3. 实施顺序

```text
S0 合同 + snapshot + FakeProvider（当前完成）
  -> S0.5 transaction/audit/approval application service
    -> S1 remote snapshot vertical slice
      -> S2 Windows/Linux local offline providers ┐
      -> S3 macOS XPC helper / local VM           ├─ 按位置独立 Gate
        -> R5 Sidecar/Skill/MCP execution bridge  ┘
          -> S4 default-enable / enterprise / release
```

S1–S3 可在 S0.5 稳定后并行，但每条产品功能只按它实际使用的 Provider/evidence 发布。remote 可以提供三平台 strict 通用命令路径；本机 Provider 不因另一平台通过而自动通过。

## 4. Phase S0：Provider-neutral 合同（已实现）

### 4.1 代码落点

| 模块 | 职责 |
|---|---|
| `services/sandbox/types.rs` | 版本化领域类型、policy/intent validation、errors/digests |
| `services/sandbox/snapshot.rs` | snapshot、result/artifact validation、writeback preflight |
| `services/sandbox/provider.rs` | create/collect/cancel/destroy Provider port；result root 仅 Rust 可见 |
| `services/sandbox/broker.rs` | exact registry、lifecycle、evidence、cleanup、audit events |
| `services/sandbox/tests.rs` | test-only FakeProvider 与 S0/R6 攻击回归 |

### 4.2 已完成检查

- [x] 领域类型不依赖 Tauri、Python、云 SDK 或平台 crate。
- [x] strict 禁止 `host_direct`、`direct_mount` 和 `explicit_host_direct` network。
- [x] full access 只能是 host-direct + direct mount + explicit network + 非空 approval。
- [x] credential/proxy 环境名不进入 allowlist。
- [x] snapshot 排除控制目录、常见凭据、symlink/junction/reparse point。
- [x] snapshot/result root 与真实工作区互不包含；manifest 不序列化绝对路径。
- [x] result 拒绝 traversal、反斜杠歧义、受保护路径、重复/越权 change、hash/size 不符。
- [x] writeback preflight 校验 manifest approval、generation 与用户并发修改。
- [x] strict Provider capability 和最终 result evidence 双重检查。
- [x] collect 后总是尝试 destroy；invalid lease identity 也尝试清理。
- [x] FakeProvider 仅测试编译；安全基线测试禁止生产 S0 出现 host process shortcut。

### 4.3 S0 后续缺口（S0.5）

- [ ] 为 `ValidatedWriteback` 实现同卷 staging、原子替换、删除备份、失败回滚与 crash recovery journal。
- [ ] 将 approval 绑定用户、window/session、manifest digest、TTL 和一次性消费；拒绝 replay/confused deputy。
- [ ] 建立 SQLite audit/recovery schema，记录 plan/evidence/lease/cleanup，不存 argv/环境值/输出/路径明文。
- [ ] 增加并发 execution registry、取消、idempotent terminate、应用 shutdown cleanup 和遗留 lease recovery。
- [ ] 对 snapshot/result copy 加强 TOCTOU：平台 file identity、no-follow/open-handle 校验、rename race fixture。
- [ ] 增加 signed/attested result manifest port；签名算法与 key lifecycle 由具体 Provider threat model 决定。
- [ ] 添加 feature flag/settings schema；无 Provider 时仅显示 unavailable diagnostic。

### 4.4 S0.5 退出门

- writeback 的每个失败点均不会留下未审计的半应用状态；crash 后可确定恢复或回滚。
- approval replay、旧 generation、跨 session/window、manifest 篡改和重复提交均被拒绝。
- app shutdown、取消和 provider disconnect 后 lease/临时目录可重试清理。
- 没有真实 Provider 时生产 API 稳定返回 `SANDBOX_UNAVAILABLE`。

## 5. Phase S1：Remote + snapshot 垂直切片

### 5.1 范围

选一个开发期 remote Provider（组织已有 Azure、自托管 Firecracker 或经审查的试点供应商），只用无 secret 测试工作区实现统一端口：

```text
create_execution(plan, encrypted snapshot reference)
  -> stream structured output/diagnostics
  -> cancel
  -> collect signed result manifest + artifacts + evidence
  -> destroy execution and temporary storage
```

供应商 SDK/HTTP client 位于 Rust adapter；Sidecar 不直连。credential 存 OS keychain/企业 token broker，sandbox 只得到短期 execution identity。

### 5.2 TODO

- [ ] threat model 与 DPA/地区/保留期/费用/配额评审；固定开发期 Provider 与版本。
- [ ] snapshot upload 只含 manifest 已允许文件；UI/日志可显示摘要但不显示绝对路径和内容。
- [ ] create/stream/cancel/collect/destroy、超时、重试、幂等和 lease expiry。
- [ ] result manifest 签名/nonce/execution ownership/replay validation。
- [ ] 默认 deny egress；allowlist 验证 DNS、redirect、private IP、metadata endpoint、IPv4/IPv6/loopback。
- [ ] 不注入模型/Git/SSH/cloud/browser credential；环境与进程树证据可验证。
- [ ] 中断、区域不符、镜像 digest 变化、配额耗尽和 destroy 失败的可操作诊断。
- [ ] artifact 经过 `ArtifactService`；workspace diff 经过 S0.5 approval/writeback。
- [ ] Windows/macOS/Linux 客户端兼容与网络代理/企业 TLS 场景测试。

### 5.3 退出门

- 同一无 secret fixture 在三平台客户端得到一致 plan/result 语义。
- 未批准文件/凭据/网络均不可达；取消和 crash 后 remote process/lease/storage 可证明销毁。
- Provider 不可用时只返回 unavailable，不命中 host-direct 或本地 subprocess。
- UI 准确显示服务、地区、上传范围、保留期、网络和实际 evidence。

## 6. Phase S2：Windows / Linux 本机离线 Provider

### 6.1 Windows AppContainer

- 优先 Spike stable AppContainer/LPAC + restricted token + Job Object；`Experimental_CreateProcessInSandbox` 通过 runtime probe 和 opaque FFI 隔离，仅 Canary。
- 不创建可登录专用账户，不授予 internet capability；snapshot 是唯一可写根，只读工具路径来自签名 registry。
- 测试 home/浏览器/SSH/cloud credential、junction/reparse、rename race、named pipe、IPv4/IPv6/DNS/localhost/QUIC、child/grandchild/breakaway 和标准用户/企业策略。

### 6.2 Linux namespaces

- 候选为受审版本的 Bubblewrap/namespaces + seccomp + cgroup/process group；默认 unshare network，不绑定 D-Bus、SSH agent、Docker socket。
- 检查可信 binary 路径、版本/hash、userns、发行版安全策略和打包；缺能力时 unavailable，不把 path guard/Landlock-only 冒充同等 strict。
- 覆盖 symlink/mount、Unix socket、PID/daemon、resource/output bomb 和 Ubuntu/Debian/Fedora/RHEL 支持矩阵。

### 6.3 共同退出门

- 工作区外写、敏感读取、直接网络和后台进程逃逸失败。
- Python/Node/Git/Rust 等“已登记工具”逐项记录兼容矩阵，不泛称支持任意本机工具链。
- Unicode、长路径、worktree/submodule、受管设备和能力缺失都有稳定诊断。
- provider probe/runner/实验 API 失败不调用宿主进程。

## 7. Phase S3：macOS XPC 窄 helper 与本地 VM

### 7.1 App Sandbox + XPC helper

- 只用于项目签名、自带的 parser/converter；输入/输出均为 app container 中的 artifact bytes。
- 最小 entitlements，无 network client、Automation、Accessibility；不继承父进程凭据和任意 host path。
- 验证 code signature、notarization、bookmark 选择/撤销/失效、Intel/Apple silicon 和支持的 macOS 大版本。

### 7.2 本地 Linux VM（可选隐私模式）

- 签名应用使用 Virtualization.framework，下载并校验架构匹配的 Linux image。
- 默认无网络、无 writable VirtioFS host share；通过 snapshot disk/受认证 channel 返回 manifest/artifacts。
- 明示仅 Linux 工具链；Xcode/macOS SDK/iOS 签名走 remote macOS 服务或逐操作 host-direct。

### 7.3 退出门

- XPC/VM 的文件、环境、凭据、网络、资源、进程回收、签名和升级证据完成。
- VM image 供应链、磁盘加密/清理、休眠/崩溃、Intel/Apple silicon 通过。
- `sandbox-exec`/动态 Seatbelt 不存在于正式 runner，也不计入 strict 证据。

## 8. R5：Sidecar、Skills、MCP 与富内容接入

R5 的非执行 `RichOutputAdapter`/块事件依赖 Phase 4 对话迁移，可独立于 Provider 开发；任何外部生成必须等待 S0.5 和目标 Provider Gate。

### 8.1 执行桥 TODO

- [ ] 选 UDS/named pipe/loopback transport 并完成本地攻击面评审。
- [ ] token 高熵、短期、scope、rotation、constant-time 验证；绑定 session/workspace generation/tool/TTL，Sidecar 重启即失效。
- [ ] Python `BrokeredWorkspaceBackend` 映射文件/执行 API；production 停止使用直接 `LocalShellBackend`。
- [ ] 文件工具同样经 snapshot/canonical policy，避免 Python 插件绕过 shell 隔离。
- [ ] Skill activation view 只读，执行请求带 skill ID/hash；本地 MCP spawn 走 `ExecutionService`。
- [ ] 审批等待不阻塞 Sidecar event loop；disconnect/cancel/retry 幂等。
- [ ] 外部结果先回 Broker/Artifact ingress，再写 canonical content block；取消/重生成不留孤立 artifact。

### 8.2 R5 sandbox 退出门

- 生产 Agent/Skill/MCP 外部执行不存在 host subprocess fallback。
- 每个执行可追踪 session/message/source、snapshot、location、network、lease、evidence 和 result manifest。
- provider unavailable、Sidecar restart、MCP reject、cancel/stream failure 均安全收口。

## 9. R6 / Phase S4：加固、默认启用与发布

### 9.1 安全与性能

- [ ] snapshot/result 大仓库基准、内存/磁盘/文件数/输出/取消预算与产品阈值。
- [ ] traversal、symlink/junction/reparse、rename race、Windows 保留名、Unicode、长路径、zip/resource bomb。
- [ ] DNS/redirect/private IP/metadata/IPv4/IPv6/loopback/named pipe/Unix socket/child 绕过。
- [ ] provider/result replay、错误身份、旧 generation、approval replay/confused deputy、签名/镜像失效。
- [ ] 应用 crash/restart、kill tree、remote destroy、临时目录和 writeback journal recovery。
- [ ] CSP/capability diff 保持主 WebView 无 FS/HTTP/Shell execute；安全基线测试持续覆盖。

### 9.2 产品与运维

- [ ] Settings 显示执行位置、实际保证/缺口、地区、网络、上传摘要、配额、setup/repair 和审计入口。
- [ ] host-direct 明确 full access 风险、理由、范围和 TTL；每次批准，不提供隐式永久 fallback。
- [ ] Provider/policy/runner/image 版本、签名、SBOM、原子更新、紧急禁用和回滚。
- [ ] 企业 Provider 的区域、镜像 digest、密钥代理、组织审批、审计/保留和私有端点。
- [ ] 独立安全 review/渗透测试；事故响应涵盖 Provider 禁用、lease/进程/临时数据清理和非敏感诊断。

### 9.3 发布 Gate

每个执行位置分别签字：

- [ ] 文件、凭据、网络、进程树、环境、资源、审计 evidence 与 UI 声明一致。
- [ ] capability/provider/setup 损坏时 fail closed，诊断可操作。
- [ ] 签名、安装、升级、修复、卸载/销毁无危险残留。
- [ ] 兼容工具、平台/版本/架构、性能阈值和已知限制有证据。
- [ ] 未通过的位置保持 unavailable，不以另一位置通过或“代码可编译”替代。

## 10. 测试与验证命令

```powershell
cd src-tauri

# S0/R6 定向合同与攻击回归
cargo test --all-features --lib sandbox
cargo test --all-features --test security_config_baseline_tests

# Rust 质量门
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --profile ci  # 提交/推送前；未安装则 cargo test --all-features
```

真实 Provider 另需平台 runner/attack/packaging suite；普通单元测试或当前 Windows 开发机结果不能替代三平台实机证据。

## 11. 回滚原则

- Provider 发布故障时标为 unavailable；不能自动回退 full access。
- policy/provider/image 版本与 evidence 保留可审计迁移；回滚不删除用户文件和审计。
- writeback apply 必须有 journal/backup/recovery；未确认 diff 永不写真实工作区。
- Python legacy/fake backend 只允许测试或显式开发开关，release 不包含自动 fallback。

## 12. 完成定义

- 目标平台各有至少一个真实 strict 路径通过对应 Gate，ADR 按事实转 Accepted。
- Agent/Skill/MCP/外部 converter 生产链路不存在直接宿主执行绕过。
- snapshot、result、approval、writeback、network、cancel、cleanup、audit 和发布运维完整。
- UI/文档只声明 capability evidence 已证明的保证；R5/R6 阶段记录包含本地与远程 CI 证据。
