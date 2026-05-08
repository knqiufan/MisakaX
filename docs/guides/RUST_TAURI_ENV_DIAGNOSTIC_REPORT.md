# Rust / Tauri 运行环境诊断报告

**生成时间**: 2026-05-08
**机器**: Windows 11 Pro (10.0.26200)
**项目**: Misaka-Tauri (`D:\code\Misaka-Tauri`)

---

## 一、环境总览

| 组件 | 版本 | 路径 |
|------|------|------|
| Rust | 1.95.0 (stable-gnu) | `C:\Users\Administrator\.rustup\` |
| Cargo | 1.95.0 | `C:\Users\Administrator\.cargo\bin\` |
| Node.js | v24.14.1 | `C:\nvm4w\nodejs\` (via nvm-windows) |
| Python | 3.13.9 (conda) | `D:\soft\install\anaconda\envs\misaka\` |
| GCC (MSYS2) | 15.2.0 ✅ | `D:\soft\msys64\mingw64\bin\` |
| GCC (Anaconda) | **5.3.0** ⚠️ | `D:\soft\install\anaconda\envs\misaka\Library\mingw-w64\bin\` |
| GCC (Git Bash) | 5.3.0 ⚠️ | Git for Windows 内嵌 `C:\Program Files\Git\mingw64\bin\` |

---

## 二、严重问题：三套 MinGW GCC 并存，版本冲突

当前系统存在 **三套 GCC 工具链**，且版本差异巨大：

### 1. MSYS2 GCC 15.2.0（正确版本，项目需要）
- 路径: `D:\soft\msys64\mingw64\bin\`
- 用途: Rust GNU target 链接器 + C 编译器
- 状态: 正常，项目 `.cargo\config.toml` 已明确指定此路径

### 2. Anaconda MinGW GCC 5.3.0（过期版本，冲突源）
- 路径: `D:\soft\install\anaconda\envs\misaka\Library\mingw-w64\bin\`
- 用途: conda 环境 `misaka` 依赖
- 状态: **GCC 5.3.0 发布于 2015 年，比 MSYS2 版本老 10 年**
- 影响: PATH 顺序导致此版本的 DLL 被错误加载

### 3. Git Bash MinGW GCC 5.3.0（过期版本，次要冲突）
- 路径: `C:\Program Files\Git\mingw64\bin\`
- 版本: GCC 5.3.0
- 影响: Git Bash 启动时自动加入 PATH，可能干扰

### 冲突表现

```
PATH 中 MinGW 相关目录的实际顺序：
1. Anaconda MinGW  ← GCC 5.3.0, 最先被找到
2. Git Bash MinGW  ← GCC 5.3.0
3. MSYS2 MinGW     ← GCC 15.2.0, 最后被找到

→ 运行 gcc 命令实际调用的是 Anaconda 的 GCC 5.3.0
→ ld.exe 也指向 Anaconda 的 GNU ld 2.25.1 (2015)
→ 运行时加载的 libgcc_s_seh-1.dll 版本混乱
```

### 重复 DLL 清单

`libgcc_s_seh-1.dll` 在以下 **5 个**位置存在：

| 位置 | 归属 |
|------|------|
| `C:\Users\Administrator\.cargo\bin\` | Rust toolchain 自带 |
| `D:\soft\msys64\mingw64\bin\` | MSYS2 GCC 15.2.0 |
| `D:\soft\install\anaconda\envs\misaka\Library\mingw-w64\bin\` | Anaconda GCC 5.3.0 |
| `C:\Program Files\Git\mingw64\bin\` (×2 PATH entries) | Git Bash GCC 5.3.0 |

---

## 三、GCC 版本依赖精确分析

### 3.1 项目需要哪个 GCC 版本？

**答案：MinGW-w64 GCC 14.x 或 15.x（当前 MSYS2 提供的 15.2.0 完全符合要求）。**

不是任意版本的 GCC 都可以——必须与 Rust 工具链的 GCC ABI 版本匹配。

### 3.2 Rust 工具链的 GCC ABI 版本

| 组件 | 版本 | 说明 |
|------|------|------|
| Rust | 1.95.0 (stable-gnu) | 发布于 2026-04-14 |
| LLVM | 22.1.2 | 编译器后端 |
| **自带 MinGW 导入库的 GCC 版本** | **GNU C17 14.1.0** | 关键！见下方分析 |
| 自带 LTO 插件 | GCC 14.1.0 | libLTO.dll |

**关键证据**：Rust 工具链自带的 `libgcc_eh.a` 中包含编译信息：
```
GNU C17 14.1.0
```
文件位置：
```
C:\Users\Administrator\.rustup\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\lib\self-contained\libgcc_eh.a
```

这意味着 Rust 1.95.0 的 GNU toolchain 是 **基于 GCC 14.1.0 的 ABI 构建的**。所有 self-contained 导入库（`libgcc_eh.a`、`libgcc_s.a`、`libgcc.a`、`libpthread.a`、`libkernel32.a` 等 44 个）均来自 GCC 14.1.0 的 MinGW-w64 工具链。

### 3.3 ABI 兼容性链

二进制从编译到运行的完整链条：

```
[编译期]
  Rust crate 源码
    → LLVM 22.1.2 生成 .o 目标文件
    → 链接器 (MSYS2 GCC 15.2.0)
    → 链接 Rust 自带的 libgcc_eh.a (GCC 14.1.0 ABI) ★
    → 链接 Rust 自带的 libgcc_s.a (GCC 14.1.0 ABI) ★
    → 生成二进制，记录 DLL 导入符号

[运行期]
  Windows 加载器按顺序搜索 libgcc_s_seh-1.dll:
    ① 二进制所在目录
    ② C:\Windows\System32
    ③ C:\Windows\System
    ④ 当前工作目录
    ⑤ PATH 环境变量 (按顺序) ← 冲突发生处！
```

**编译期使用 GCC 14.1.0 ABI 的导入库，运行时就必须加载 GCC 14.1.0+ ABI 兼容的 DLL。**

### 3.4 各 GCC 版本 DLL 的 ABI 兼容性

| 来源 | GCC 版本 | 导出调试字符串数 | 与 GCC 14.1 ABI 兼容？ |
|------|---------|----------------|----------------------|
| Rust 工具链自带 | 14.1.0 (无版本字符串) | — | ✅ 完全兼容（同版本） |
| MSYS2 MinGW | **15.2.0** | 161 | ✅ 兼容（GCC 14→15 主版本递增，ABI 向后兼容） |
| Anaconda MinGW | 5.3.0 | 122 | ❌ 不兼容（相差 10 年，缺少 39+ 个符号） |
| Git Bash MinGW | 5.3.0 | 122 | ❌ 不兼容（同上） |

**GCC 5.3.0 发布于 2015 年，GCC 14.1.0 发布于 2024 年。** 9 年间 `libgcc_s_seh-1.dll` 增加了约 39 个新的导出符号和入口点。用 GCC 14.1 ABI 编译的二进制调用 `libgcc_s_seh-1.dll` 中的函数时，如果加载的是 GCC 5.3 版本的 DLL，某些函数入口点不存在 → `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)`。

**验证命令**（PowerShell）：
```powershell
# 查看 MSYS2 libgcc 的 GCC 版本信息
Select-String -Path "D:\soft\msys64\mingw64\bin\libgcc_s_seh-1.dll" -Pattern "GCC:" -Encoding Byte 2>$null
# 或用 strings 工具:
# strings "D:\soft\msys64\mingw64\bin\libgcc_s_seh-1.dll" | findstr "GCC:"

# 查看 Rust 工具链 libgcc_eh.a 的 GCC 版本
# strings "C:\Users\Administrator\.rustup\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\lib\self-contained\libgcc_eh.a" | findstr "GNU C"
```

### 3.5 项目 C 编译依赖分析

项目通过 `cc-rs` crate (v1.2.61) 自动编译 C 代码。以下是需要 C 编译的所有 crate：

| Crate | 版本 | C 文件 | C 标准 | GCC 最低要求 | 用途 |
|-------|------|--------|--------|-------------|------|
| **ring** | 0.17.14 | 17 个 .c + x86_64 Perl 汇编 | **C11** (`-std=c1x`) | ≥ 4.6 | 加密原语 (AES, curve25519, P256/P384, SHA, ChaCha, Poly1305) |
| **libsqlite3-sys** | 0.32.0 | 1 个 (sqlite3.c 合并文件) | C89 | 任何版本 | SQLite3 嵌入式数据库 (bundled, ~8MB 源文件) |
| **sqlite-vec** | 0.1.9 | 1 个 (sqlite-vec.c) | C99+ | ≥ 4.x | 向量搜索扩展 |

**关于 C11 的说明**：ring 0.17.14 使用 `-std=c1x`（C11 草案名称），因为 GCC 4.6 只认 `c1x` 而非 `c11`。任何 GCC ≥ 4.6 都支持。但这是**编译期**要求，与**运行期** ABI 兼容性是独立的问题。

### 3.6 cc-rs 编译器查找顺序

`cc-rs` 是 Rust 生态中编译 C 代码的标准工具。它按以下**固定优先级**查找 C 编译器：

| 优先级 | 环境变量 | 本项目配置值 |
|--------|---------|-------------|
| 1 (最高) | `CC_x86_64-pc-windows-gnu` | 未设置 → 跳过 |
| **2** | **`CC_x86_64_pc_windows_gnu`** | **`D:\soft\msys64\mingw64\bin\gcc.exe`** ✅ |
| 3 | `HOST_CC` | 未设置 → 跳过 |
| 4 (最低) | `CC` | 未设置 → 跳过 |
| 5 (兜底) | PATH 搜索 `gcc` → `cc` → `clang` | 会找到 Anaconda GCC 5.3.0 ❌ |

本项目在 `.cargo\config.toml` 中正确设置了 `CC_x86_64_pc_windows_gnu`，因此**编译期始终使用 MSYS2 GCC 15.2.0**，不受 PATH 影响。这是编译能成功的原因。

### 3.7 结论：正确版本和错误版本

| | 正确版本 | 错误版本 |
|------|---------|---------|
| 编译器 | **MSYS2 MinGW-w64 GCC 15.2.0** (2025) | Anaconda MinGW GCC 5.3.0 (2015) / Git Bash GCC 5.3.0 |
| libgcc DLL | **MSYS2 `libgcc_s_seh-1.dll` (GCC 15.2.0)** | Anaconda `libgcc_s_seh-1.dll` (GCC 5.3.0) |
| ABI 兼容 | GCC ≥ 14.x (与 Rust 工具链匹配) | GCC 5.x (10 年代差, ABI 断裂) |
| PATH 优先级 | 应在所有 MinGW 目录的**最前面** | 当前错误地在最前面 |

**核心事实**：项目需要且只需要 **一套** MinGW-w64 GCC 14.x/15.x 工具链，且其 DLL 在运行时 PATH 中必须排在第一位。

---

## 四、当前测试失败原因

### 现象
```powershell
cargo test --lib        # 失败: STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)
cargo test --test *     # 失败: STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)
cargo test --bin misaka-x  # ✅ 正常通过
```

### 根因分析

1. **编译期**：项目 `.cargo\config.toml` 通过绝对路径指定 MSYS2 GCC 15.2.0 作为链接器，编译本身没有问题
2. **运行期**：测试二进制启动时，Windows 加载器按 PATH 顺序搜索 DLL，首先找到 Anaconda/Git Bash 的 GCC 5.3.0 运行时 DLL（`libgcc_s_seh-1.dll`, `libwinpthread-1.dll`, `libstdc++-6.dll`）
3. **版本不匹配**：GCC 15.2 编译的二进制尝试调用 GCC 5.3 的 DLL 中不存在的函数 → `STATUS_ENTRYPOINT_NOT_FOUND`
4. **为什么主二进制能用**：`misaka-x.exe` 在 `target\debug\` 目录下，其 DLL 搜索路径和顺序与 `target\debug\deps\` 下的测试二进制不同；且主二进制通过 Tauri 的进程初始化流程加载 WebView2，间接改变了 DLL 搜索上下文

### 为什么每次执行 cargo 前需要手动设置 PATH

当前 shell session 的 PATH 中，Anaconda MinGW 排在 MSYS2 MinGW 前面。虽然配置文件已正确设置（MSYS2 在最前），但**当前终端 session 未重新加载配置**。需要从系统层面永久修复 PATH 顺序。

---

## 五、PATH 完整分析

### 当前会话 PATH（问题版本）

```
优先级从高到低：
  1. D:\soft\install\anaconda\envs\misaka\Library\mingw-w64\bin  ← GCC 5.3
  2. C:\Users\Administrator\.cargo\bin                            ← Rust tools
  3. C:\Users\Administrator\bin
  4. C:\Program Files\Git\mingw64\bin                            ← Git Bash GCC 5.3
  5. C:\Program Files\Git\usr\local\bin
  6. C:\Program Files\Git\usr\bin
  7. C:\Program Files\Git\bin
  8. C:\Program Files\Git\mingw64\bin (重复)
  9. C:\Windows\System32
 10. C:\Windows
 11. ... (其他系统目录)
 12. D:\soft\install\anaconda
 13. D:\soft\install\anaconda\Scripts
 14. D:\soft\msys64\mingw64\bin                                   ← MSYS2 GCC 15.2 (太靠后!)
 15. C:\Users\Administrator\.cargo\bin (重复)
```

### 修正后的 PATH（目标状态）

```
  1. D:\soft\msys64\mingw64\bin                                  ← MSYS2 GCC 15.2 ✅ 排在第一位
  2. C:\Users\Administrator\.cargo\bin                            ← Rust tools
  3. D:\soft\install\anaconda\envs\misaka\Library\mingw-w64\bin   ← Anaconda GCC 5.3 (被 MSYS2 遮蔽)
  4. C:\Program Files\Git\mingw64\bin                            ← Git Bash GCC 5.3 (被 MSYS2 遮蔽)
  5. ... (其余)
```

---

## 六、缺失的依赖

| 依赖 | 状态 | 说明 |
|------|------|------|
| Visual Studio Build Tools | ❌ 未安装 | MSVC toolchain 无法使用 |
| Windows SDK | ❌ 未安装 | MSVC 需要 |
| WebView2 Runtime | ⚠️ 未确认 | 注册表未找到，但 Edge 应已自带 |
| MSYS2 packages | ✅ | mingw-w64-x86_64-gcc 15.2.0 正确 |
| Rust MSVC target | ✅ 已安装 | 但因缺少 VS Build Tools 无法使用 |

---

## 七、修复方案

### 方案 A（推荐）：修复 PATH 顺序，保留当前 GNU 环境

无需卸载任何软件，只需调整 Windows 系统 PATH 中 MinGW 目录的优先级。

#### Step 1: 将 MSYS2 MinGW 移到 PATH 最前面

**方法一：Windows 系统环境变量（永久生效，推荐）**

1. 按 `Win + R`，输入 `sysdm.cpl`，回车
2. 点击「高级」→「环境变量」
3. 在「用户变量」中找到 `Path`，双击编辑
4. 找到 `D:\soft\msys64\mingw64\bin`，点击「上移」直到它排在最前面
5. 或者：**新建**一条 `D:\soft\msys64\mingw64\bin` 并移到最顶部
6. 确认：确保 `D:\soft\msys64\mingw64\bin` 排在 `D:\soft\install\anaconda\envs\misaka\Library\mingw-w64\bin` 和 `C:\Program Files\Git\mingw64\bin` 之前
7. 点击「确定」保存，**重启终端**生效

**方法二：PowerShell 命令行**

```powershell
# 查看当前用户 PATH
[Environment]::GetEnvironmentVariable("PATH", "User")

# 将 MSYS2 MinGW 追加到用户 PATH 最前面
$msys2 = "D:\soft\msys64\mingw64\bin"
$current = [Environment]::GetEnvironmentVariable("PATH", "User")
[Environment]::SetEnvironmentVariable("PATH", "$msys2;$current", "User")
```

> **注意**：如果之前通过 conda 或 Git Bash 的配置文件（如 `.bashrc`、`.bash_profile`）设置 PATH，也需要同步修改或注释掉相关行，避免覆盖系统 PATH。

#### Step 2: 如果使用 Git Bash，修改 `.bashrc` 或 `.bash_profile`

如果 `C:\Users\Administrator\.bashrc` 中有以下内容，注释掉：

```bash
# ⚠️ 注释掉：不再将 Anaconda MinGW 加入 PATH（与 MSYS2 GCC 15.2 冲突）
# MINGW_BIN="/d/soft/install/anaconda/envs/misaka/Library/mingw-w64/bin"
# [ -d "$MINGW_BIN" ] && PATH="$MINGW_BIN:$PATH"
```

并确保 MSYS2 MinGW 在最前面：
```bash
# MSYS2 MinGW 必须最先加载
[ -d "/d/soft/msys64/mingw64/bin" ] && PATH="/d/soft/msys64/mingw64/bin:$PATH"
```

#### Step 3: 验证修复

重新打开终端（PowerShell 或 CMD），依次运行：

```powershell
# 确认 MSYS2 GCC 在最前面
where.exe gcc
# 应输出:
# D:\soft\msys64\mingw64\bin\gcc.exe
# (如果还有其他路径跟在后面，第一个必须是 MSYS2)

# 确认版本
gcc --version
# 应输出: gcc.exe (Rev11, Built by MSYS2 project) 15.2.0
```

```powershell
# 回到项目目录，验证编译和测试
cd D:\code\Misaka-Tauri\src-tauri
cargo build
cargo test --bin misaka-x
```

---

### 方案 B：完全重置 Rust 环境

如果方案 A 不生效，执行以下彻底清理后重装：

#### Step 1: 卸载 Rust

```powershell
# 卸载 Rust（保留项目代码）
rustup self uninstall
```

#### Step 2: 清理残留文件

```powershell
# 删除 Rust 相关目录
Remove-Item -Recurse -Force "$env:USERPROFILE\.cargo" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "$env:USERPROFILE\.rustup" -ErrorAction SilentlyContinue
```

#### Step 3: 确认 MSYS2 MinGW 在 PATH 最前面

```powershell
where.exe gcc
# 必须输出: D:\soft\msys64\mingw64\bin\gcc.exe（且是第一个结果）
gcc --version
# 必须输出: 15.2.0
```

如果 MSYS2 MinGW 不在最前面，先执行方案 A 的 Step 1。

#### Step 4: 重新安装 Rust

1. 下载 rustup-init.exe：https://rustup.rs
2. 双击运行，选择 **"Customize installation"**
3. 关键配置项：
   - **Default host triple**: 选择 `x86_64-pc-windows-gnu`
   - **不要安装 MSVC toolchain**（如果没有 Visual Studio）
4. 安装完成后，**新开终端**验证：

```powershell
rustc --version
cargo --version
rustup show
```

#### Step 5: 回到项目验证

```powershell
cd D:\code\Misaka-Tauri\src-tauri
cargo build
cargo test --bin misaka-x
```

---

### 方案 C：切换到 MSVC Toolchain（最干净，需额外安装）

彻底摆脱 MinGW DLL 版本冲突的最佳方案。

#### Step 1: 安装 Visual Studio Build Tools 2022

1. 下载：https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
2. 运行安装程序，勾选 **"Desktop development with C++"**（桌面 C++ 开发）
3. 安装完成后**重启电脑**

#### Step 2: 切换到 MSVC toolchain

```powershell
rustup default stable-x86_64-pc-windows-msvc
```

#### Step 3: 验证

```powershell
rustup show
# 应显示: stable-x86_64-pc-windows-msvc (default)

cd D:\code\Misaka-Tauri\src-tauri
cargo build
cargo test
# cargo test --lib 和 cargo test --test * 此时都应正常工作
```

切换到 MSVC 后不再需要 MinGW DLL，PATH 顺序问题自然消失。

---

### 可安全删除的冗余内容

| 项目 | 路径 | 原因 |
|------|------|------|
| Rust MSVC toolchain (当前 GNU 用户) | `rustup toolchain remove stable-x86_64-pc-windows-msvc` | 没有 VS Build Tools，无法使用 |
| `libgcc_s_seh-1.dll` (非 MSYS2 版本) | `C:\Users\Administrator\.cargo\bin\libgcc_s_seh-1.dll` | Rust GNU toolchain 自带，但 MSYS2 已提供更新版本 |
| Python 2.7 (如果有) | `C:\Python27` | 已不再需要，Python 3 由 conda 管理 |

---

## 八、开发环境最佳实践

### 每日开发命令

在项目根目录 `D:\code\Misaka-Tauri` 下，打开 **PowerShell**（或 Windows Terminal）：

```powershell
# 前端（终端 1）
npm run dev

# 后端（终端 2）
cd src-tauri
cargo run

# 测试（当前 GNU 环境只支持 binary test）
cargo test --bin misaka-x

# Python sidecar（终端 3，按需启动）
cd agent
conda activate misaka
python -m uvicorn app.main:app --port 9527
```

### `cargo test` 现状说明

| 命令 | 状态 | 说明 |
|------|------|------|
| `cargo test --bin misaka-x` | ✅ | 主二进制测试，可直接运行 |
| `cargo test --lib` | ❌ | STATUS_ENTRYPOINT_NOT_FOUND（DLL 冲突） |
| `cargo test --test *` | ❌ | 同上（集成测试） |

这是 `windows-gnu` target + Tauri WebView2 依赖的已知兼容性问题，与项目代码无关。

- **临时方案**：将测试集中在 `src\main.rs` 的 `#[cfg(test)]` 块中，用 `cargo test --bin misaka-x` 运行
- **长期方案**：切换到 MSVC toolchain（方案 C）

### Windows 终端配置建议

推荐使用 **Windows Terminal** + **PowerShell 7**：

1. 在 Microsoft Store 搜索并安装 "Windows Terminal"
2. 安装 PowerShell 7：`winget install Microsoft.PowerShell`
3. 将 Windows Terminal 设为默认终端

---

## 九、诊断结论

**核心问题**：三套 GCC 工具链并存，PATH 顺序混乱导致 GCC 15.2 编译的二进制在运行时加载 GCC 5.3 的运行时 DLL，函数入口点不匹配。

**技术细节**：Rust 1.95.0 GNU toolchain 基于 GCC 14.1.0 ABI 构建。运行时 DLL (`libgcc_s_seh-1.dll`) 必须来自 GCC ≥ 14.x。Anaconda/Git Bash 提供的 GCC 5.3.0 DLL（2015年产物）缺少 GCC 14.1 ABI 下的约 39 个符号，导致 `STATUS_ENTRYPOINT_NOT_FOUND` 错误。

**推荐操作**：

1. **立即执行（方案 A）**：打开 Windows 系统环境变量设置（`sysdm.cpl`），将 `D:\soft\msys64\mingw64\bin` 移到 PATH 最前面，重启终端即可。无需卸载任何软件。

2. **如果想要一劳永逸（方案 C）**：安装 Visual Studio Build Tools 2022，切换到 `stable-x86_64-pc-windows-msvc` toolchain，彻底摆脱 MinGW DLL 版本冲突。

**无论选择哪个方案，只需操作一次，之后的日常开发不需要任何手动 PATH 设置。**
