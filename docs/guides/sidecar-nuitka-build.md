# Sidecar Nuitka 打包指南

> **用途：** 记录 MisakaX Python Sidecar 的 Nuitka 本地构建、验收指标与 Windows/Conda 排障经验。  
> **受众：** 维护者、发布构建执行者、AI 辅助开发。  
> **最后审阅 / Last reviewed:** 2026-07-09

---

## 1. 当前验收结果

| 指标 | 值 |
|------|-----|
| 验收日期 | 2026-07-09 |
| 环境 | Windows 11 / x86_64 / MSVC |
| Python | 3.11.11（Conda env: `misaka`） |
| Nuitka | 4.1.3 |
| 构建命令 | `conda run -n misaka python build_nuitka.py --clean` |
| 产物路径 | `agent/dist/misaka-agent.exe` |
| 体积 | 16.0 MB（16,749,568 bytes） |
| 冷启动到 `/health` 200 | 1.33s |
| 健康检查 | `status=ok`, `service=misaka-agent`, `version=0.1.0` |

结论：Phase 3 AC-15 / V27-V30 的本地打包验收已通过。产物体积低于 80 MB，本轮不需要额外瘦身。

---

## 2. 构建步骤

在仓库根目录进入 `agent/` 后执行：

```powershell
conda run -n misaka python build_nuitka.py --clean
```

成功时日志应包含：

```text
Build successful!
Artifact : D:\code\Misaka-Tauri\agent\dist\misaka-agent.exe
Size     : 16.0 MB
```

`build_nuitka.py` 当前使用 onefile 构建，并保留 `run.dist` / `run.build` 供排障。最终可执行文件位于 `agent/dist/misaka-agent.exe`。

---

## 3. 运行与健康检查

Sidecar 二进制不接收 CLI `--port` 参数，端口通过环境变量传入：

```powershell
$env:MISAKA_HOST = "127.0.0.1"
$env:MISAKA_PORT = "9531"
.\dist\misaka-agent.exe
```

另开终端检查：

```powershell
Invoke-RestMethod http://127.0.0.1:9531/health
```

预期响应：

```text
status         : ok
service        : misaka-agent
version        : 0.1.0
capabilities   : {health, info}
agent_ready    : False
```

---

## 4. Windows / Conda 排障记录

### 4.1 固定使用 `misaka` Python 3.11

不要使用 base 环境的 Python 3.13 构建本项目 Sidecar。项目目标环境是 Python 3.11.x，本次验收使用：

```powershell
conda run -n misaka python --version
```

### 4.2 OpenSSL DLL

Anaconda 的 `_ssl.pyd` 依赖 `libcrypto-3-x64.dll` / `libssl-3-x64.dll`。`build_nuitka.py` 会：

- 将 Conda DLL 目录加入 Nuitka 子进程 `PATH`
- 显式打包 `libcrypto-3-x64.dll` 与 `libssl-3-x64.dll`
- 在 `run.py` 冻结入口注册自身 DLL 目录，避免加载系统中的不匹配 OpenSSL

### 4.3 `click._winconsole`

在 Windows + Conda + Nuitka onefile 下，`click._winconsole` 的 ctypes 控制台包装可能导致进程在 Python 异常前以 `0xC0000409` 退出。Sidecar 不使用 Click 的交互式控制台能力，因此当前构建策略是：

- Nuitka 参数排除真实模块：`--nofollow-import-to=click._winconsole`
- `run.py` 在导入 `uvicorn` 前为冻结运行时注册 `click._winconsole` stub

这两个条件都需要保留，否则 onefile 可执行文件可能在启动阶段直接退出。

---

## 5. 与 Rust SidecarManager 的关系

`src-tauri/src/sidecar.rs` 会优先探测打包产物：

1. `agent/dist/misaka-agent.exe`（Windows）或 `agent/dist/misaka-agent`
2. 当前应用 exe 同级目录的 `misaka-agent(.exe)`

找到二进制时，Rust 使用 `MISAKA_HOST=127.0.0.1` 与 `MISAKA_PORT=<configured port>` 启动它；未找到时回退到开发模式的 `python <agent-dir>/run.py`，并通过相同环境变量传递 host 与 port。

因此：

- 开发环境未构建 `agent/dist/` 时，仍可使用 Python uvicorn 回退路径。
- 本地存在 `agent/dist/misaka-agent.exe` 时，Tauri 会优先验证冻结二进制路径。
- 发布捆绑到 `tauri.conf.json` `externalBin` 仍属于后续发布工作，不在 Phase 3 本轮范围内。
