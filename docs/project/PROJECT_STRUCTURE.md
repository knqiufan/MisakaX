# MisakaX 项目结构说明

> **最后审阅 / Last reviewed:** 2026-08-01
> **开发进度与续做入口：** 见同目录 [`DEVELOPMENT_STATUS.md`](./DEVELOPMENT_STATUS.md)（Phase 3 主体完成 → **Phase 4 DeepAgents 迁移** 为下一步）。

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

| 文件 / 目录 | 用途 |
|-------------|------|
| `main.rs` | Tauri 应用入口，调用 `misaka_x_lib::run()` |
| `lib.rs` | 插件注册、DB 初始化、Sidecar / MCP 预热、`AppState`、`invoke_handler`（~50+ commands） |
| `config.rs` | `AppConfig`、YAML 读写、`~/.misakax/` 路径 |
| `crypto.rs` | API Key AES-GCM 加密 |
| `sidecar.rs` | `SidecarManager`：Sidecar 启动、健康检查、状态事件 |

### `src-tauri/src/db/`

| 文件 / 目录 | 用途 |
|-------------|------|
| `mod.rs` | SQLite 初始化、WAL、sqlite-vec 加载、运行迁移 |
| `migrations.rs` | Schema **v1–v13**；v11 stable SkillId/activation，v12 scan/finding/approval，v13 存量扫描与旧表清理 |
| `models.rs` | Session / Message / RouterConfig 等数据模型 |
| `repository/` | 会话、消息、Provider、MCP、Workspace、Settings，以及 `skill_source_repo` / `skill_security_repo`；Skills 只写 `skill_sources` |

### `src-tauri/src/commands/`

| 模块 | 用途 |
|------|------|
| `settings.rs` | 设置与 `AppConfig` CRUD |
| `router_configs.rs` | Provider / API Key 管理、连接测试 |
| `models.rs` | 可用模型列表、自定义模型、拉取 Provider 模型 |
| `chat.rs` | `send_message`、`stop_generation`、`regenerate_message`、`get_messages` |
| `session.rs` | 会话 CRUD、搜索、导入导出、工作目录 |
| `workspace.rs` | 工作目录浏览、最近目录、按会话读取只读 Workspace/Git context |
| `fs_explorer.rs` | 工作区文件读写、在资源管理器中Reveal |
| `mcp.rs` | MCP Server 连接、工具调用、权限审批 |
| `sidecar.rs` | Sidecar 状态查询、重启 |
| `skills.rs` | Skills inventory、按需文件、扫描/审批、stable SkillId 开关、v13 迁移状态与失败重试 |

### `src-tauri/src/services/`

| 目录 | 用途 |
|------|------|
| `llm/` | Rig 过渡后端：`backend`、`streaming`、`factory`、`registry`、多 Provider |
| `mcp/` | rmcp `McpManager`、配置加载、类型定义 |
| `sidecar_client.rs` | Rust → Sidecar HTTP 客户端 |
| `mcp_bridge.rs` | MCP 桥接（供 Agent / Tool 复用） |
| `skills/` | 多来源 registry、安装/文件提供器、quarantine、离线 scanner、policy、migration 与 watcher |
| `workspace/` | canonical workspace、只读 `VcsProvider`/Git CLI、single-flight cache、generation 与 HEAD/ref watcher |

---

## 二、`src/` — React 前端（TypeScript + Vite 6）

| 路径 | 用途 |
|------|------|
| `App.tsx` | 根组件：`AppShell` + `ErrorBoundary` + Toaster |
| `main.tsx` | React 入口 |
| `index.css` | Tailwind CSS v4 + 设计 token |
| `components/layout/` | `AppShell`、`Sidebar`、`ContentArea` |
| `components/chat/` | `ChatView`、会话、Composer、只读 WorkspaceContext badge、WorkspacePanel、Explorer/Monaco 与消息工具日志入口 |
| `components/ui/` | shadcn/ui 组件 |
| `pages/` | `ChatPage`、`SettingsPage`；Skills 领域 UI 位于 Settings，Knowledge / Dashboard 仍为占位 |
| `components/skills/` | Skills 仓库双栏、按需文件预览、安全报告、迁移进度与安装/卸载 Dialog |
| `stores/` | Zustand：`chat-store`、`settings-store`、`theme-store`、独立 `workspace-panel-store` 与 `workspace-explorer-store` 等 |
| `lib/ipc/` | Tauri IPC 封装（chat、session、mcp、settings…） |
| `lib/providers/` | Provider 目录与 catalog |
| `locales/` | i18n（zh-CN / en） |
| `hooks/` | `use-stream-listener`、`use-ipc`、`use-sidecar-status`、`use-workspace-context` |
| `__tests__/` | Vitest 单元测试 |

---

## 三、`agent/` — Python Sidecar（FastAPI）

| 文件 | 用途 |
|------|------|
| `pyproject.toml` | 项目元数据；optional：`agent`（langgraph）、`memory`（powermem）、`dev`（pytest） |
| `requirements.txt` | 运行时核心依赖 |
| `build_nuitka.py` | Sidecar Nuitka 打包脚本（Phase 3 验收项） |
| `app/main.py` | FastAPI 入口，注册 health / info / agent 路由 |
| `app/config.py` | pydantic-settings（`MISAKA_` 环境变量） |
| `app/models.py` | Sidecar 请求 / 响应模型 |
| `app/routers/health.py` | `GET /health` — Sidecar 就绪检测 |
| `app/routers/info.py` | 服务信息 |
| `app/routers/agent.py` | `/agent/chat`、`/agent/stream` — **501 占位，Phase 4 实现 DeepAgents** |
| `tests/` | pytest（health、info、agent 占位端点） |

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
| `PHASE_3_DETAILED_PLAN.md` | Phase 3 完整方案（背景设计） |
| `PHASE_3_REMAINING_TODO.md` | **Phase 3 未完成项执行清单（续做入口）** |
| `PHASE_4_DETAILED_PLAN.md` | Phase 4 DeepAgents 迁移（Phase 3 完成后） |

### `docs/research/`

| 文件 | 用途 |
|------|------|
| `RIG_DEEPAGENTS_RESEARCH.md` | Rig / DeepAgents 等技术调研 |

### `docs/project/`

| 文件 | 用途 |
|------|------|
| `DEVELOPMENT_STATUS.md` | **开发进度、已完成 / 未完成项、续做入口（优先阅读）** |
| `PROJECT_STRUCTURE.md` | 仓库目录与各模块说明（本文件） |
| `CODE_ARCHITECTURE_REVIEW.md` | 代码架构评审记录 |

### `docs/design/`

| 文件 | 用途 |
|------|------|
| `frontend-ui-guidelines.md` | 全局动效、色彩、桌面 Agent 风格 |
| `shell-and-workspace-ui-spec.md` | Shell / 会话栏 / 工作区规范 |
| `button-menu-design-spec.md` | 按钮、菜单、弹层规范 |
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
