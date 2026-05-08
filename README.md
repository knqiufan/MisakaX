# MisakaX

<p align="center">
  <strong>现代化桌面 AI Agent 客户端</strong>
</p>

<p align="center">
  基于 Tauri 2.x + React 19 + Python Sidecar 构建的跨平台桌面 AI 助手
</p>

---

## 项目概况

MisakaX 是一个开源的桌面 AI Agent 客户端，采用三层架构设计：

- **Rust 后端**负责高性能计算密集型任务（数据库、文件系统、MCP 协议、LLM 直接调用）
- **React 前端**提供现代化的用户界面（会话管理、任务跟踪、知识库浏览）
- **Python Sidecar**处理 AI Agent 编排（LangGraph）和长期记忆管理（PowerMem）

项目处于 **Phase 0（基础框架）** 阶段，已完成项目骨架搭建、数据库 Schema 设计、配置管理系统和前端 UI 框架集成。

## 已实现功能

- 跨平台桌面应用框架（Tauri 2.x，Windows / macOS / Linux）
- SQLite 数据库初始化与 Schema 迁移（WAL 模式、FTS5 全文搜索、sqlite-vec 向量扩展）
- 配置管理系统（`~/.misakax/config.yaml` 持久化存储）
- 插件体系（文件系统、HTTP、通知、对话框、剪贴板、Shell 调用）
- Python Sidecar 健康检查端点（FastAPI，端口 9527）
- React 19 + Tailwind CSS v4 + shadcn/ui 前端 UI 框架

## 技术架构

```
┌────────────────────────────────────────────────┐
│  React 19 (TypeScript) + Tailwind CSS v4 +    │
│  shadcn/ui (New York / Zinc)                   │
│  → Vite 6 开发服务器 (:1420)                   │
├────────────────────────────────────────────────┤
│  Tauri 2.x (Rust)                              │
│  - SQLite (WAL) + sqlite-vec 向量搜索 + FTS5  │
│  - 配置管理 (~/.misakax/config.yaml)            │
│  - 插件系统 (shell, fs, http, dialog 等)       │
├────────────────────────────────────────────────┤
│  Python Sidecar (:9527)                        │
│  - FastAPI + uvicorn                           │
│  - LangGraph Agent 编排（规划中）               │
│  - PowerMem 记忆引擎（规划中）                  │
└────────────────────────────────────────────────┘
```

## 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 桌面框架 | Tauri 2.x | Rust 驱动，内存占用低 |
| 前端 | React 19 + TypeScript + Vite 6 | 现代化 Web 技术栈 |
| UI | Tailwind CSS v4 + shadcn/ui | CSS-first，组件优先 |
| 后端 | Rust (tokio, rusqlite, serde) | 异步运行时 + 序列化 |
| 数据库 | SQLite + sqlite-vec + FTS5 | 结构化 + 向量 + 全文搜索一体 |
| AI 编排 | LangGraph（规划中） | 复杂 Agent 工作流 |
| 记忆引擎 | PowerMem（规划中） | 长期上下文记忆 |

## 快速开始

### 环境要求

| 工具 | 最低版本 | 用途 |
|------|---------|------|
| Rust | 1.82+ | Tauri 后端编译 |
| Node.js | 20+ | 前端开发 |
| Python | 3.11.x | Sidecar 运行 |
| pnpm / npm | 10+ | 包管理 |

Windows 用户还需要：
- MinGW-w64 或 MSVC 构建工具
- WebView2 运行时（Windows 11 已内置）

### 克隆项目

```bash
git clone https://github.com/your-org/misaka-x.git
cd misaka-x
```

### 安装依赖

```bash
# 前端依赖
npm install

# Python Sidecar 依赖
cd agent && pip install -r requirements.txt
```

### 启动开发环境

```bash
# 启动完整桌面应用（前端 + Rust 后端）
npm run tauri dev

# 仅启动前端（浏览器调试）
npm run dev

# 启动 Python Sidecar（另一个终端）
cd agent && python -m uvicorn app.main:app --host 127.0.0.1 --port 9527
```

### 构建

```bash
# 前端生产构建
npm run build

# 完整应用打包
npm run tauri build
```

## 目录结构

```
misaka-x/
├── src-tauri/                     # Rust 后端
│   ├── src/
│   │   ├── main.rs                # Tauri 入口
│   │   ├── lib.rs                 # 模块导出 + 应用初始化
│   │   ├── config.rs              # 配置管理（YAML 持久化）
│   │   ├── db/
│   │   │   ├── mod.rs             # 数据库初始化 + sqlite-vec 加载
│   │   │   ├── migrations.rs      # Schema 版本迁移
│   │   │   └── models.rs          # 数据模型定义
│   │   └── commands/
│   │       ├── mod.rs             # 命令注册
│   │       └── settings.rs        # 设置读写 API
│   ├── Cargo.toml                 # Rust 依赖声明
│   ├── tauri.conf.json            # Tauri 应用配置
│   ├── capabilities/
│   │   └── default.json           # 权限 ACL
│   └── icons/                     # 应用图标
├── src/                           # React 前端
│   ├── App.tsx                    # 根组件
│   ├── main.tsx                   # 入口文件
│   ├── index.css                  # Tailwind CSS 入口
│   ├── components/ui/             # shadcn/ui 组件
│   └── lib/utils.ts               # shadcn/ui 工具函数
├── agent/                         # Python Sidecar
│   ├── app/
│   │   ├── main.py                # FastAPI 应用入口
│   │   ├── config.py              # Sidecar 配置（pydantic-settings）
│   │   └── routers/health.py      # 健康检查端点
│   ├── pyproject.toml             # Python 项目配置
│   └── requirements.txt           # Python 依赖
├── docs/                          # 项目文档
│   ├── architecture/              # 架构与选型文档
│   ├── planning/                  # 阶段计划与总体规划
│   ├── research/                  # 技术调研
│   ├── project/                   # 项目结构等说明文档
│   ├── design/                    # UI / 视觉设计文档
│   └── guides/                    # 学习与入门指南
├── .gitignore
├── CLAUDE.md                      # AI 辅助开发指引
├── package.json                   # 前端依赖与脚本
├── vite.config.ts                 # Vite 配置
└── README.md                      # 项目说明
```

## Rust 开发命令

```bash
cd src-tauri

# 编译检查（不生成二进制文件，速度快）
cargo check

# 运行测试
cargo test

# 查看依赖树
cargo tree --depth 0

# 格式化代码
cargo fmt

# Clippy 静态检查
cargo clippy
```

## 运行时数据目录

应用运行后会在用户主目录创建以下结构：

```
~/.misakax/
├── config.yaml           # 全局配置（主题、语言、默认模型等）
├── data/
│   └── misaka.db         # SQLite 数据库（WAL 模式）
├── skills/               # 用户自定义 Skills
├── managed/skills/       # 市场安装的 Skills
├── plugins/              # 插件目录
├── models/               # 语音模型等大文件
└── logs/                 # 应用日志
```

## 路线图

- [x] **Phase 0** — 项目骨架、数据库 Schema、配置系统、UI 框架
- [ ] **Phase 1** — 会话管理 UI、LLM 对话功能、消息流式渲染
- [ ] **Phase 2** — MCP 协议集成、Skills 系统、文件系统交互
- [ ] **Phase 3** — LangGraph Agent 编排、PowerMem 长期记忆
- [ ] **Phase 4** — 插件市场、语音模型、多语言支持

详细规划参见 [`docs/planning/PHASE_0_DETAILED_PLAN.md`](docs/planning/PHASE_0_DETAILED_PLAN.md) 及后续文档。

## 贡献指南

本项目处于早期开发阶段，欢迎 Issue 和 PR。

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

提交信息遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范。

## 许可证

本项目采用 [MIT License](LICENSE) 开源。
