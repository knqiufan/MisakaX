# MisakaX 项目结构说明

## 顶层目录

```
MisakaX/
├── .cargo/                        # Cargo 编译配置
├── .git/                          # Git 仓库元数据
├── .gitignore                     # Git 忽略规则
├── CLAUDE.md                      # AI 辅助开发指引
├── README.md                      # 项目说明文档
├── agent/                         # Python Sidecar 子项目
├── components.json                # shadcn/ui 配置文件
├── dist/                          # 前端构建产物
├── docs/                          # 项目文档（按主题分子目录）
│   ├── architecture/              # 架构与选型
│   ├── planning/                  # 阶段计划与总体规划
│   ├── research/                  # 技术调研
│   ├── project/                   # 项目结构说明等
│   ├── design/                    # UI / 设计报告
│   └── guides/                    # 学习指南
├── index.html                     # Vite 入口 HTML
├── node_modules/                  # 前端依赖包
├── package.json                   # 前端项目配置
├── package-lock.json              # 前端依赖锁定
├── src/                           # React 前端源码
├── src-tauri/                     # Tauri / Rust 后端
├── tsconfig.json                  # TypeScript 配置
├── tsconfig.node.json             # Node 端 TypeScript 配置
└── vite.config.ts                 # Vite 构建配置
```

---

## 一、`src-tauri/` — Rust 后端（Tauri 2.x）

### 根文件

| 文件 | 用途 |
|------|------|
| `Cargo.toml` | Rust 项目的依赖声明文件。定义了 Tauri 2.x 核心、7 个 Tauri 插件（shell / fs / http / notification / updater / dialog / clipboard-manager）、rusqlite（SQLite 数据库）、sqlite-vec（向量搜索）、tokio（异步运行时）、serde（序列化）等全部依赖 |
| `Cargo.lock` | 依赖版本锁定文件，确保编译可复现 |
| `build.rs` | Rust 构建脚本。调用 `tauri_build::build()` 生成 Tauri 所需的上下文代码和 `OUT_DIR` 环境变量 |
| `tauri.conf.json` | Tauri 应用的主配置文件。包含应用名称（MisakaX）、窗口大小（1280x800）、图标路径、构建命令等 |

### `src-tauri/capabilities/`

| 文件 | 用途 |
|------|------|
| `default.json` | Tauri 2.x 的权限 ACL 配置。声明前端可以调用的所有系统 API 权限：文件读写、HTTP 请求、Shell 命令、桌面通知、对话框、剪贴板等 |

### `src-tauri/gen/schemas/`

| 文件 | 用途 |
|------|------|
| `acl-manifests.json` | 各 Tauri 插件声明的权限清单（自动生成） |
| `capabilities.json` | 权限能力 Schema（自动生成） |
| `desktop-schema.json` | 桌面端配置 Schema（自动生成） |
| `windows-schema.json` | Windows 端配置 Schema（自动生成） |

### `src-tauri/icons/`

| 文件 | 用途 |
|------|------|
| `32x32.png` | 小尺寸应用图标（任务栏） |
| `128x128.png` | 中尺寸应用图标 |
| `128x128@2x.png` | 高 DPI（Retina）图标 |
| `icon.ico` | Windows ICO 格式图标 |

### `src-tauri/src/` — Rust 源码

| 文件 | 用途 |
|------|------|
| `main.rs` | **Tauri 应用入口**。调用 `misaka_x_lib::run()` 启动应用。包含 `#[cfg(test)]` 数据库初始化单元测试 |
| `lib.rs` | **核心启动逻辑**。注册所有 Tauri 插件、初始化日志（tracing）、创建配置目录、加载配置文件、初始化数据库、自动启动 Python Sidecar、注册 Tauri commands、定义 `AppState`（全局共享状态：数据库连接 + 配置 + Sidecar 进程句柄） |
| `config.rs` | **配置管理**。定义 `AppConfig` 结构体（语言、主题、默认模型、Sidecar 端口等）、默认值实现、路径函数（`~/.misakax/` 下各目录）、YAML 读写、目录创建 |
| `sidecar.rs` | **Python 进程管理**。`SidecarManager` 负责后台 Python 服务的完整生命周期：启动 `uvicorn` → 轮询 health 端点 → 超时保护。应用退出时自动 kill 子进程 |

### `src-tauri/src/db/`

| 文件 | 用途 |
|------|------|
| `mod.rs` | **数据库初始化**。打开 SQLite 连接 → 启用 WAL 模式 + 外键约束 → 加载 sqlite-vec 向量扩展 → 运行 Schema 迁移 |
| `migrations.rs` | **Schema 版本迁移**。管理数据库版本号（`_schema_version` 表），执行 v1 迁移：创建 `sessions` / `messages` / `settings` / `router_configs` / `tasks` / `knowledge_docs` 6 张核心表 + `messages_fts` / `knowledge_fts` 2 个全文搜索虚拟表 |
| `models.rs` | **数据模型**。定义 `Session` / `Message` / `Setting` / `RouterConfig` 四个结构体，派生 `Serialize` / `Deserialize` 用于 JSON 序列化 |

### `src-tauri/src/commands/`

| 文件 | 用途 |
|------|------|
| `mod.rs` | 命令模块注册 |
| `settings.rs` | **设置读写 API**。提供两个 Tauri commands：`get_settings`（获取全部配置为 JSON）、`update_setting`（按 key 更新单个配置项并持久化到 YAML） |

---

## 二、`src/` — React 前端（TypeScript + Vite 6）

| 文件 | 用途 |
|------|------|
| `index.css` | **Tailwind CSS v4 入口**。包含 `@import "tailwindcss"` 指令和 shadcn/ui 的 CSS 变量定义（Zinc 色彩体系 + New York 风格主题变量） |
| `main.tsx` | **React 入口**。使用 `ReactDOM.createRoot` 挂载 `<App />` 到 `<div id="root">` |
| `App.tsx` | **根组件**。当前显示 MisakaX 标题、副标题、shadcn/ui Button 组件。后续将扩展为完整的路由和布局框架 |
| `components/ui/button.tsx` | shadcn/ui Button 组件，基于 Radix UI + CVA 样式方案 |
| `lib/utils.ts` | shadcn/ui 工具函数，包含 `cn()` 类名合并函数（clsx + tailwind-merge） |
| `assets/` | 静态资源目录（图片、字体等） |

---

## 三、`agent/` — Python Sidecar（FastAPI + LangGraph）

| 文件 | 用途 |
|------|------|
| `pyproject.toml` | Python 项目元数据。声明项目名 `misaka-agent`、Python >= 3.11、核心依赖和可选依赖（agent: langgraph + langchain、memory: powermem、dev: pytest + ruff） |
| `requirements.txt` | Python 运行时依赖（FastAPI、uvicorn、pydantic、httpx） |
| `README.md` | Sidecar 快速启动说明 |
| `app/__init__.py` | Python 包标记 |
| `app/config.py` | **配置类**。使用 pydantic-settings 读取 `MISAKA_` 前缀环境变量（host / port / log_level / db_path） |
| `app/main.py` | **FastAPI 入口**。创建 FastAPI 实例，注册 health 路由，定义 startup / shutdown 生命周期钩子 |
| `app/routers/__init__.py` | 路由包标记 |
| `app/routers/health.py` | **健康检查端点**。`GET /health` 返回 `{"status": "ok", "service": "misaka-agent"}`，被 Rust 后端用于检测 Sidecar 就绪状态 |

---

## 四、`docs/` — 项目文档

顶层按主题分到子目录，避免所有 Markdown 扁平堆叠。

### `docs/architecture/`

| 文件 | 用途 |
|------|------|
| `MISAKAX_ARCHITECTURE_FINAL - DeepSeek-V4-Pro.md` | 最终技术架构选型文档 |
| `MISAKAX_ARCHITECTURE_SELECTION - Opus4.6.md` | 架构方案对比与选择理由 |
| `MISAKAX_TECH_SELECTION_REPORT.md` | 技术选型详细报告 |

### `docs/planning/`

| 文件 | 用途 |
|------|------|
| `MISAKAX_IMPLEMENTATION_PLAN - Opus4.6.md` | 整体实施方案 |
| `CLAW_IMPLEMENTATION_PLAN - Opus4.6.md` | CLAW / 相关业务实施方案 |
| `PHASE_0_DETAILED_PLAN.md` | Phase 0 详细执行方案 |
| `PHASE_1_DETAILED_PLAN.md` | Phase 1 详细执行方案 |
| `PHASE_2_DETAILED_PLAN.md` | Phase 2 详细执行方案 |

### `docs/research/`

| 文件 | 用途 |
|------|------|
| `RIG_DEEPAGENTS_RESEARCH.md` | Rig / DeepAgents 等技术调研 |

### `docs/project/`

| 文件 | 用途 |
|------|------|
| `PROJECT_STRUCTURE.md` | 仓库目录与各模块说明（本文件） |

### `docs/design/`

| 文件 | 用途 |
|------|------|
| `ui-design-report.md` | UI / 产品设计报告 |

### `docs/guides/`

| 文件 | 用途 |
|------|------|
| `rust-learning-faq-modules-and-lib.md` | Rust 模块与 `lib.rs` / `bin` 常见问题说明 |

---

## 五、根目录配置文件

| 文件 | 用途 |
|------|------|
| `.cargo/config.toml` | Cargo 工具链配置。指定 GNU 目标 linker（Rust 自带的 modern ld.exe）、CC/CXX 编译器（conda 的 MinGW-w64 GCC）、库搜索路径 |
| `.gitignore` | 忽略规则：`target/` / `node_modules/` / `dist/` / `__pycache__/` / `.env` 等 |
| `CLAUDE.md` | Claude Code 的 AI 辅助开发指引。包含架构图、目录表、开发命令、环境配置、技术决策和代码风格约定 |
| `README.md` | 项目顶级说明文档（中文） |
| `components.json` | shadcn/ui CLI 配置。记录组件风格（New York）、基础色（Zinc）、CSS 变量模式等元信息 |
| `index.html` | Vite 构建的 HTML 入口，包含 `<div id="root">` 和 `<script type="module" src="/src/main.tsx">` |
| `package.json` | 前端 NPM 项目配置。声明脚本（dev / build / tauri）、依赖（React 19 / Tailwind CSS v4 / shadcn/ui 相关 / Tauri API） |
| `package-lock.json` | 前端依赖的精确版本锁定 |
| `tsconfig.json` | TypeScript 主配置。包含 `@/*` → `./src/*` 路径别名（支持 `import { Button } from "@/components/ui/button"`） |
| `tsconfig.node.json` | Vite 配置文件专用 TypeScript 配置（composite + emitDeclarationOnly 模式） |
| `vite.config.ts` | Vite 构建配置。注册 React 和 Tailwind CSS v4 插件，设置 `@` 路径别名，配置开发服务器端口（1420）和 HMR |

---

## 六、运行时目录结构

应用启动后在用户主目录自动创建：

```
~/.misakax/
├── config.yaml           # 全局配置（YAML 格式，User-editable）
├── data/
│   └── misaka.db         # SQLite 数据库（WAL 模式，含向量索引）
├── skills/               # 用户自定义 Skills
├── managed/skills/       # 从市场安装的 Skills
├── plugins/              # 插件目录
├── models/               # 语音模型等本地大文件
└── logs/                 # 应用日志
```
