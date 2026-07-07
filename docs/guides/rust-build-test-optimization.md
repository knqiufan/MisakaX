# Rust / Tauri 构建与测试效率优化指南

> **用途：** 说明 Misaka-Tauri 后端（`src-tauri/`）编译与测试变慢的原因，并按 **P0 → P3** 给出可执行的优化步骤与验收标准。  
> **受众：** 维护者、协作者、AI 辅助开发。  
> **最后审阅 / Last reviewed:** 2026-07-07

**相关文档：**

- 环境诊断：[`RUST_TAURI_ENV_DIAGNOSTIC_REPORT.md`](./RUST_TAURI_ENV_DIAGNOSTIC_REPORT.md)
- 开发状态与测试入口：[`../project/DEVELOPMENT_STATUS.md`](../project/DEVELOPMENT_STATUS.md)

---

## 1. 问题现象与根因

### 1.1 常见现象

| 场景 | 典型耗时（冷编译） | 典型耗时（增量，理想情况） |
|------|-------------------|---------------------------|
| `cargo check` | 3–8 分钟 | 10–60 秒 |
| `cargo test`（全量 27 个集成测试） | 5–12 分钟 | 30 秒–3 分钟 |
| `npm run tauri dev` | 5–15 分钟 | 1–5 分钟（前端 HMR + Rust 增量） |
| 只改一行 Rust 却等很久 | — | 往往说明 **增量编译未生效** |

### 1.2 本项目的结构性负担

当前 `src-tauri/` 是 **单体 Tauri crate**，依赖链较重：

| 因素 | 位置 | 影响 |
|------|------|------|
| Tauri 2 + 7 个 plugin | `Cargo.toml` | 编译量大；`build.rs` 每次执行 `tauri_build::build()` |
| `rusqlite` + `bundled` | `Cargo.toml` | 冷编译需编译 SQLite C 源码 |
| `tokio = { features = ["full"] }` | `Cargo.toml` | 拉入大量未直接使用的能力 |
| `rig-core`、`rmcp` | `Cargo.toml` | LLM/MCP 依赖链长 |
| `crate-type = ["staticlib", "cdylib", "rlib"]` | `Cargo.toml` | 同一库以多种形态产出，增加链接成本 |
| **27 个** `tests/*.rs` 集成测试 | `src-tauri/tests/` | 每个文件独立 test binary，首次全量测试需编 27 个可执行文件 |
| 文档要求每次前 `cargo clean` | `AGENTS.md`、`README.md` 等 | **抹掉增量缓存，强制全量重编**（最大人为因素） |

### 1.3 关键认知

- **`cargo test` 慢，多数时候是「编译慢」，不是「测试逻辑慢」。** 即便 `tests/minimal.rs` 只有 `2+2=4`，Cargo 仍要先编完依赖树和 `misaka_x_lib`。
- **Rust 增量编译默认开启（debug 模式）。** 只要不 `clean`、不切换 toolchain/大版本依赖，改少量代码后应明显变快。
- **`.cursor/hooks/post-edit-test.sh`** 在 Rust 文件变更时跑 `cargo check` + 按 §4.2 映射的精准 `--test`（无映射时仅 check）

---

## 2. 优化优先级总览

| 优先级 | 主题 | 预期收益 | 实施成本 | 是否改代码 |
|--------|------|----------|----------|------------|
| **P0** | 停止日常 `cargo clean`；修正文档与习惯 | **极大（5–10×）** | 极低 | 仅文档/流程 |
| **P1** | 精准测试、`cargo check` 优先、Windows 环境调优 | **大（2–5×）** | 低 | 可选改 hook |
| **P2** | sccache、nextest、链接器、`.cargo/config.toml` | **中–大** | 中 | 配置为主 |
| **P3** | Workspace 拆分、feature 收紧、测试分层 | **长期最大** | 高 | 架构重构 |

**建议执行顺序：** 先完成 P0 并验证增量编译生效 → P1 改日常习惯 → P2 装工具链 → P3 排期分阶段做。

---

## 3. P0 — 恢复增量编译（最高优先级）

### 3.1 目标

日常开发/单测 **不再** 执行 `cargo clean`；仅在异常情况下清理 `target/`。

### 3.2 操作步骤

#### 步骤 1：建立新的默认习惯

```powershell
cd d:\code\Misaka-Tauri\src-tauri

# 日常 — 编译检查（最快反馈）
cargo check

# 日常 — 跑相关测试（见 §4.2 映射表）
cargo test --test crypto_tests

# 提交前 / CI — 全量
cargo test

# 不要在这里加 cargo clean
```

#### 步骤 2：明确「何时才需要 clean」

仅在以下情况执行 `cargo clean`（或删除 `src-tauri/target/`）：

- 链接错误、`metadata mismatch`、莫名其妙的 trait 冲突
- 切换 Git 分支后出现无法解释的编译失败
- 升级 Rust toolchain（`rustup update`）或 **大版本** 依赖变更后
- `target/` 体积异常膨胀（见 §3.4）且其他手段无效
- 发布前希望「完全干净」的可复现构建（可选，非日常）

**频率建议：** 平时 **零次**；出问题时 **按需一次**；CI 缓存策略允许时可 **从不 clean**。

#### 步骤 3：修订仓库文档（实施 P0 时的必改项）

将「每次构建前 clean」改为「按需 clean」：

| 文件 | 当前问题 | 建议改法 |
|------|----------|----------|
| `AGENTS.md` | §Dev Commands 多处 `cargo clean` | 删除日常命令中的 `cargo clean`；保留「按需清理」说明 |
| `CLAUDE.md` | 同上 | 与 `AGENTS.md` 同步 |
| `README.md` | `tauri dev` / `cargo test` 前要求 clean | 改为可选 troubleshooting 步骤 |
| `docs/project/DEVELOPMENT_STATUS.md` | 测试命令含 `cargo clean &&` | 改为 `cargo test`，脚注说明按需 clean |

#### 步骤 4：验证增量编译已生效

```powershell
cd d:\code\Misaka-Tauri\src-tauri

# 第一次：建立缓存（会较慢）
cargo check

# 改一行无关注释，例如 src/crypto.rs 末尾加空行
cargo check
# 期望：Finished 出现在数秒～一分钟内，而非数分钟
```

若第二次仍像冷编译一样慢，检查：

1. 是否仍在脚本/hook 里调用了 `cargo clean`
2. 杀毒软件是否扫描 `target/`（见 P1）
3. `CARGO_INCREMENTAL` 是否被设为 `0`（应为默认或未设置）

### 3.3 关于「Cargo hygiene / 防止 stale artifacts」

原规则担心 `target/` 积累脏产物。**正确做法不是每次 clean，而是：**

- 日常信任增量编译
- CI 使用 **缓存** `target/`（GitHub Actions `Swatinem/rust-cache` 等）
- 偶发问题时 `cargo clean` 一次
- 定期（如每月）或 `target/` 超过 N GB 时再清理

### 3.4 验收标准（P0）

- [x] 文档中不再要求「每次 `cargo check/test/build` 前 clean」
- [ ] 连续两次 `cargo check`（仅改一行代码）第二次明显快于第一次
- [x] 团队成员/Agent 默认工作流不含 `cargo clean`

---

## 4. P1 — 日常开发工作流与精准测试

### 4.1 命令分层：按场景选最轻的命令

| 你在做什么 | 推荐命令 | 避免 |
|------------|----------|------|
| 改 Rust 逻辑，快速看能否编译 | `cargo check` | `cargo build`、`tauri dev` |
| 验证某个模块 | `cargo test --test <file>` | `cargo test` 全量 |
| 验证单个用例 | `cargo test --test db_migrations_tests test_name` | — |
| 需要 Tauri IPC / UI 联调 | `npm run tauri:dev` | 每次小改都跑 |
| 提交前 | `cargo test` | — |
| CI | `cargo nextest run`（P2）或 `cargo test` | 无缓存 + 每次 clean |

### 4.2 改动路径 → 推荐测试文件映射

Agent / 开发者改代码后，优先跑 **1–2 个** 相关集成测试，而不是 27 个全跑：

| 改动目录 / 模块 | 推荐 `--test` |
|-----------------|---------------|
| `src/crypto.rs` | `crypto_tests` |
| `src/config.rs` | `config_tests` |
| `src/db/`、`migrations` | `database_tests`, `db_migrations_tests` |
| `src/db/repository/session_repo.rs` | `session_repo_tests` |
| `src/db/repository/message_repo.rs` | `message_repo_tests` |
| `src/db/repository/workspace_repo.rs` | `workspace_repo_tests` |
| `src/db/repository/tool_permission_repo.rs` | `tool_permission_repo_tests` |
| `src/db/repository/mcp_server_repo.rs` | `mcp_repo_tests` |
| `src/commands/settings.rs` | `settings_command_tests` |
| `src/commands/session.rs` | `session_commands_tests`（需 `--features test-private`） |
| `src/commands/workspace.rs` | `workspace_commands_tests`（需 `--features test-private`） |
| `src/commands/chat.rs` | `chat_commands_tests`（需 `--features test-private`） |
| `src/commands/models.rs` | `models_command_tests` |
| `src/commands/fs_explorer.rs` | `fs_explorer_tests` |
| `src/commands/router_configs.rs` | `router_configs_tests` |
| `src/services/llm/` | `llm_backend_tests`, `llm_factory_tests`, `llm_registry_tests`, `llm_catalog_tests`, `streaming_tests` |
| `src/services/mcp/` | `mcp_config_tests`, `mcp_manager_tests`, `mcp_types_tests` |
| `src/sidecar.rs` | `sidecar_tests`（需 `--features test-private`） |
| `src/services/sidecar_client.rs` | `sidecar_tests` |
| 附件序列化相关 | `attachment_serde_tests` |
| 仅验证工具链 / 冒烟 | `minimal` |

**带 `test-private` feature 的示例：**

```powershell
cargo test --features test-private --test chat_commands_tests
cargo test --features test-private --test session_commands_tests
cargo test --features test-private --test workspace_commands_tests
cargo test --features test-private --test sidecar_tests
```

### 4.3 Cursor post-edit hook（已实施）

`.cursor/hooks/post-edit-test.sh` 在 Rust 变更时：

1. 执行 `cargo check`
2. 根据 `git diff` 路径查 §4.2 映射表，聚合为 `cargo test --test a --test b`（命中 `test-private` 测试时自动加 `--features test-private`）
3. 若无映射命中（如 `lib.rs`、`build.rs`、`commands/mod.rs`），仅 `cargo check`，不全量 test
4. 全量 `cargo test` 保留给提交前 / CI

### 4.4 Windows 环境调优

#### 4.4.1 排除杀毒实时扫描

将以下路径加入 Windows Defender「排除项」（或其他杀毒软件等价设置）：

```
D:\code\Misaka-Tauri\src-tauri\target\
D:\code\Misaka-Tauri\node_modules\
```

链接阶段生成大量 `.exe`/`.dll` 时，实时扫描可拖慢 **30%–200%**。

#### 4.4.2 磁盘与路径

- 项目与 `target/` 放在 **SSD**
- 避免同步盘（OneDrive 等）直接同步 `target/`

#### 4.4.3 工具链一致性

参考 [`RUST_TAURI_ENV_DIAGNOSTIC_REPORT.md`](./RUST_TAURI_ENV_DIAGNOSTIC_REPORT.md)：MSYS2 / Anaconda / Git Bash 多套 GCC 并存会导致异常 rebuild。确保 Rust target 与链接器配置一致（`x86_64-pc-windows-msvc` 或 gnu，不要混用）。

### 4.5 验收标准（P1）

- [x] 日常改 Rust 默认用 `cargo check` + 精准 `--test`（文档与 AGENTS.md 已说明）
- [x] Hook 不再对每次编辑跑 27 个集成测试（`.cursor/hooks/post-edit-test.sh`）
- [ ] Defender 已排除 `target/`（本机 Windows 设置，见 §4.4.1）
- [x] 团队成员知晓 §4.2 映射表（见优化指南与 AGENTS.md）

---

## 5. P2 — 工具链与 Cargo 配置加速

### 5.1 cargo-nextest（并行测试调度）

**作用：** 比默认 `cargo test` 更快调度、更好并行、输出更清晰。不改变编译量，但缩短 **测试执行阶段** 总墙钟时间。

```powershell
cargo install cargo-nextest --locked

cd d:\code\Misaka-Tauri\src-tauri
cargo nextest run                          # 全量
cargo nextest run --test crypto_tests        # 单个文件
cargo nextest run --features test-private --test chat_commands_tests
```

**CI 示例：**

```yaml
- run: cargo nextest run --manifest-path src-tauri/Cargo.toml
```

可选：在仓库根目录添加 `.config/nextest.toml` 配置超时、重试策略（按需）。

### 5.2 sccache（编译结果缓存）

**作用：** 缓存 rustc 输出；同一依赖在不同分支/机器（共享缓存时）可命中，**显著缩短冷启动**。

```powershell
cargo install sccache

# 用户级环境变量（PowerShell Profile 或系统环境变量）
$env:RUSTC_WRAPPER = "sccache"
$env:SCCACHE_DIR = "D:\cache\sccache"   # 可选，指定缓存目录
```

在 `src-tauri/.cargo/config.toml` 中写入（见 §5.4 模板）：

```toml
[build]
rustc-wrapper = "sccache"
```

**验证：**

```powershell
sccache --show-stats
# 编译几次后 cache hits 应上升
```

### 5.3 更快的链接器（Windows）

MSVC 默认 `link.exe` 在大型 Rust 项目上偏慢。可选方案（按本机 toolchain **二选一** 验证）：

**方案 A — LLVM lld（MSVC 工具链常用）：**

```toml
# src-tauri/.cargo/config.toml
[target.x86_64-pc-windows-msvc]
linker = "rust-lld.exe"
# 或 link-arg = ["/LINK", "/lld"]
```

**方案 B — mold（若已安装并在 PATH 中）：**

查阅 mold 官方 Windows 文档，配置 `RUSTFLAGS` 或 config 中的 linker 路径。

> **注意：** 链接器配置与 `x86_64-pc-windows-gnu` / `msvc` target 强相关。改完后必须跑通 `cargo test` 与 `tauri build`。若链接失败，回退 config 即可。

### 5.4 推荐 `src-tauri/.cargo/config.toml` 模板

实施 P2 时可创建该文件（按环境删减）：

```toml
# Misaka-Tauri — Cargo 构建加速（P2）
# 文档：docs/guides/rust-build-test-optimization.md

[build]
# 并行 codegen（默认已开，显式写出便于团队统一）
jobs = 0   # 0 = CPU 核心数

# 若已安装 sccache，取消下一行注释：
# rustc-wrapper = "sccache"

[profile.dev]
# 开发时适度优化依赖，加快运行、有时也加快链接
# 首次编译略慢，增量体验更好 — 按团队偏好可选
# opt-level = 1

[profile.dev.package."*"]
# 只优化依赖 crate，自己的代码仍快速增量
# opt-level = 1

# --- Windows MSVC ---
[target.x86_64-pc-windows-msvc]
# 验证通过后再启用：
# linker = "rust-lld.exe"

# --- 若使用 GNU target（与诊断报告一致时）---
# [target.x86_64-pc-windows-gnu]
# linker = "D:\\soft\\msys64\\mingw64\\bin\\gcc.exe"
```

### 5.5 其他 P2 选项（按需）

| 手段 | 说明 |
|------|------|
| `cargo watch` | `cargo watch -x check` 保存即检查 |
| `mold` / `lld` | 见 §5.3 |
| CI `Swatinem/rust-cache` | 缓存 `target/`、registry |
| 不滥用 `CARGO_BUILD_JOBS=1` | 除非内存不足，否则保持并行 |

### 5.6 验收标准（P2）

- [ ] `cargo nextest run` 在本地与 CI 可用
- [ ] `sccache --show-stats` 显示命中（若启用）
- [ ] `.cargo/config.toml` 已提交或团队有统一安装文档
- [ ] 全量 `cargo test` 墙钟时间较 P0/P1 后再降一截

---

## 6. P3 — 架构级优化（中长期）

P3 改动面大，建议 **分 PR、可独立合并**，每步保持 `cargo test` 全绿。

### 6.1 目标架构：Cargo Workspace 拆分

**现状：** 单一 `misaka-x` crate 包含 DB、LLM、MCP、Tauri commands、Sidecar 全部逻辑。

**目标：** 薄 Tauri 壳 + 可独立测试的核心库。

```
Misaka-Tauri/
├── Cargo.toml                 # workspace root
├── crates/
│   ├── misaka-core/           # config, crypto, db, models
│   ├── misaka-llm/            # rig-core, providers, streaming
│   ├── misaka-mcp/            # rmcp, manager, config loader
│   └── misaka-tauri/          # 原 src-tauri：commands + lib.rs + build.rs
├── src/                       # 前端（不变）
└── agent/                     # Python（不变）
```

**依赖方向（单向，禁止循环）：**

```
misaka-tauri → misaka-llm, misaka-mcp, misaka-core
misaka-llm   → misaka-core
misaka-mcp   → misaka-core
```

**收益：**

- 改 `crypto` / `db` 时不重编 Tauri plugin 层
- 轻量 crate 的 `cargo test` 秒级反馈
- 更清晰边界，符合 Phase 4+ 扩展

**建议实施顺序：**

1. **PR-1：** 建 workspace，抽出 `misaka-core`（`config`, `crypto`, `db`）
2. **PR-2：** 抽出 `misaka-llm`（`services/llm/*`）
3. **PR-3：** 抽出 `misaka-mcp`（`services/mcp/*`）
4. **PR-4：** `src-tauri` 变薄，仅保留 Tauri glue
5. 每 PR 迁移对应 `tests/*.rs` 到各 crate 的 `tests/` 或模块内单元测试

### 6.2 收紧 Cargo features

基于当前代码实际用法，可逐步替换「全家桶」feature：

#### tokio（当前 `full` → 建议最小集）

代码中主要使用：`time`（sleep/timeout/select）、`sync`（watch/oneshot）、`process`（MCP child process）。Tauri 自带 async runtime，但仍需 tokio 类型与部分 API。

**建议起步（需在 PR 中 `cargo check` 验证补全）：**

```toml
tokio = { version = "1", features = ["rt", "time", "sync", "macros", "process"] }
```

若编译报缺 feature，按报错 **逐个添加**，不要回退到 `full`。

#### reqwest / rig-core / rmcp

- 保持 `default-features = false`（项目已对 reqwest、rig-core 这样做 ✅）
- 定期 `cargo tree -e features` 审查是否引入多余 TLS/backend

#### rusqlite

- `bundled` 便于分发，但编译贵；**不建议轻易去掉**
- 若 P3 后期考虑：开发 crate 用 `bundled`，发布脚本统一 — 复杂度高，优先级低

### 6.3 测试分层

| 层级 | 放哪里 | 适用 | 速度 |
|------|--------|------|------|
| 单元测试 | `src/**/mod.rs` 内 `#[cfg(test)]` | 纯函数、repo、序列化 | 最快 |
| 集成测试 | 各 crate 的 `tests/` | 模块协作、DB 文件 | 中 |
| E2E / Tauri | 少量 `misaka-tauri/tests/` | commands + `test-private` | 慢 |

**具体动作：**

1. 将 `crypto_tests`、`attachment_serde_tests` 等 **无 Tauri 依赖** 测试迁入 `misaka-core` 单元测试
2. repo 类测试保留集成测试，但放在 `misaka-core/tests/`
3. `chat_commands_tests`、`sidecar_tests` 留在 `misaka-tauri`，继续用 `test-private`
4. 长期将 27 个顶层集成测试 **收敛到 ~10 个** 或迁入子 crate

### 6.4 减少 test binary 数量（可选）

每个 `tests/foo.rs` 是一个独立可执行文件。合并同一领域的测试文件（如 4 个 `llm_*_tests.rs` → 1 个 `llm_tests.rs`）可减少链接次数，收益小于 workspace，但改动小。

### 6.5 `crate-type` 审视

```toml
crate-type = ["staticlib", "cdylib", "rlib"]
```

Tauri 桌面端通常 **rlib** 即可；`staticlib`/`cdylib` 多为 mobile 或未使用场景。确认 `tauri.conf.json` / 移动端需求后，若仅桌面 Windows，可评估减为：

```toml
crate-type = ["rlib"]
```

**必须在 `tauri build` 与全平台目标上验证后再合并。**

### 6.6 验收标准（P3）

- [ ] Workspace 拆分完成，依赖无环
- [ ] `misaka-core` 单独 `cargo test` 明显快于全仓
- [ ] tokio 等非 `full` features，全量测试通过
- [ ] 集成测试数量或总 test binary 数下降
- [ ] 文档与 CI 路径更新

---

## 7. 推荐日常命令速查（优化后）

```powershell
# ── 日常（P0 + P1）──
cd d:\code\Misaka-Tauri\src-tauri
cargo check
cargo test --test crypto_tests

# ── 提交前 ──
cargo test
# 或（P2）
cargo nextest run

# ── 需要 feature 的测试 ──
cargo test --features test-private --test chat_commands_tests

# ── 完整桌面应用 ──
cd d:\code\Misaka-Tauri
npm run tauri:dev

# ── 仅当编译诡异失败 ──
cd src-tauri
cargo clean
cargo check
```

---

## 8. 分阶段实施 checklist

复制到迭代计划或 Issue 中跟踪：

### P0（当天可完成）

- [x] 团队共识：日常不 `cargo clean`
- [x] 更新 `AGENTS.md`、`CLAUDE.md`、`README.md`、`DEVELOPMENT_STATUS.md`
- [ ] 验证增量 `cargo check` 第二次加速

### P1（1–2 天）

- [x] 推广 §4.2 测试映射表（AGENTS.md / README / DEVELOPMENT_STATUS）
- [ ] Defender 排除 `target/`（本机手动）
- [x] 优化 `post-edit-test.sh` 精准 `--test`

### P2（半天安装 + 验证）

- [ ] 安装 `cargo-nextest`、`sccache`
- [ ] 添加 `src-tauri/.cargo/config.toml`
- [ ] CI 接入 cache + nextest

### P3（多 PR，按 Phase 4 节奏穿插）

- [ ] PR-1：`misaka-core` 抽取
- [ ] PR-2：`misaka-llm` 抽取
- [ ] PR-3：`misaka-mcp` 抽取
- [ ] PR-4：tokio feature 收紧
- [ ] PR-5：测试迁移与合并
- [ ] PR-6：`crate-type` 评估

---

## 9. 预期效果（参考，非承诺）

在 **Windows + MSVC + SSD** 上，完成优化后大致预期：

| 阶段 | 增量 `cargo check` | 增量单测 `--test one` | 全量 `cargo test` |
|------|-------------------|------------------------|---------------------|
| 优化前（常 clean） | 5–8 min | 5–10 min | 8–15 min |
| P0 后 | 15–60 s | 30 s–2 min | 3–8 min |
| P0+P1 | 10–40 s | 20 s–1 min | 2–6 min |
| P0–P2 | 10–30 s | 15–45 s | 1.5–4 min |
| P0–P3 | 5–20 s（core） | 5–20 s（core） | 1–3 min |

冷编译（首次 clone / clean 后）仍会 **数分钟**，属 Tauri + bundled SQLite 正常现象；优化目标是 **缩短迭代循环**，不是消除首次编译。

---

## 10. 附录：当前集成测试文件清单

| 测试文件 | 大致覆盖 |
|----------|----------|
| `minimal.rs` | 冒烟 |
| `crypto_tests.rs` | 加解密 |
| `config_tests.rs` | 配置加载 |
| `database_tests.rs` | DB 初始化 |
| `db_migrations_tests.rs` | 迁移 |
| `session_repo_tests.rs` | 会话仓库 |
| `message_repo_tests.rs` | 消息仓库 |
| `workspace_repo_tests.rs` | 工作区仓库 |
| `tool_permission_repo_tests.rs` | 工具权限仓库 |
| `mcp_repo_tests.rs` | MCP 仓库 |
| `settings_command_tests.rs` | 设置命令 |
| `session_commands_tests.rs` | 会话命令 |
| `workspace_commands_tests.rs` | 工作区命令 |
| `chat_commands_tests.rs` | 聊天命令（test-private） |
| `models_command_tests.rs` | 模型命令 |
| `fs_explorer_tests.rs` | 文件浏览 |
| `router_configs_tests.rs` | 路由配置 |
| `llm_backend_tests.rs` | LLM backend |
| `llm_factory_tests.rs` | LLM factory |
| `llm_registry_tests.rs` | LLM registry |
| `llm_catalog_tests.rs` | LLM catalog |
| `streaming_tests.rs` | 流式 |
| `mcp_config_tests.rs` | MCP 配置 |
| `mcp_manager_tests.rs` | MCP manager |
| `mcp_types_tests.rs` | MCP 类型 |
| `sidecar_tests.rs` | Sidecar（test-private） |
| `attachment_serde_tests.rs` | 附件序列化 |

---

**维护说明：** 实施 P0 文档修订或 P3 workspace 后，请回写本文 §8 checklist 并更新文首 **Last reviewed** 日期。
