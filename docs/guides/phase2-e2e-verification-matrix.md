# Phase 2 验证矩阵（代码路径 + 手工冒烟）

**用途：** 收尾验证 F-9 与《PHASE_2_DETAILED_PLAN》第 18 节 V-1～V-36 的对照记录。  
**受众：** 发布前自检、CI 无法覆盖的 E2E 项。  
**最后审阅 / Last reviewed:** 2026-05-10

## 图例

| 标记 | 含义 |
|------|------|
| **代码** | 逻辑已由单元测试或代码审阅确认（无真实 API 也能成立） |
| **手工** | 需本地运行 App + 有效 Provider / 网络 |

## 收尾验证 F-1～F-9

| ID | 项 | 结论 |
|----|----|------|
| F-1 | 工作目录冒烟 | **代码**：`WorkspaceBar` + `ChatPage`/`WorkspaceSelector` 路径；[`smoke-tests.test.ts`](../../src/__tests__/smoke-tests.test.ts) 覆盖 `extractDirName`。 |
| F-2 | 完整对话与持久化 | **代码**：`send_message` / `get_messages` / SQLite；**手工**：重启后会话与消息仍在（V-14）。 |
| F-3 | 三 Provider 对话 | **手工**：OpenAI / Anthropic / Gemini 各跑一轮（见 V-1～V-3）。 |
| F-4 | Claude 思维链 | **手工**：Claude thinking 模型 + `stream_thinking`（见 V-24～V-26）。 |
| F-5 | 停止 / 再生 / 多轮 | **代码**：`stop_generation`、`regenerate_message`、store `removeMessagesFrom`；[`smoke-tests.test.ts`](../../src/__tests__/smoke-tests.test.ts)。 |
| F-6 | 多模态 + 视觉模型 | **手工**：粘贴截图 + 视觉模型理解（见 V-30～V-32）。 |
| F-7 | 无效 Key / 友好错误 | **代码**：`stream_error` → `updateMessageError`；`ChatView` catch + `sonner`；`MessageItem` 错误气泡。 |
| F-8 | 长回复流式性能 | **代码**：1200 次 delta 累积测试；**手工**：主观感知大段生成是否卡顿。 |
| F-9 | 对照第 18 节清单 | 见下方 V-1～V-36 表。 |

## 第 18 节 V-1～V-36

### 18.1 核心对话（V-1～V-8）

| ID | 代码路径要点 | 验证方式 |
|----|----------------|-----------|
| V-1～V-3 | `RigBackend` + OpenAI/Anthropic/Gemini providers，`stream_token` | 手工 |
| V-4 | 历史 `get_messages` → `build_rig_chat_history` | 手工 |
| V-5 | `MessageItem` + `react-markdown` + `CodeBlock`（shiki） | 代码 + 手工 |
| V-6 | `CodeBlock` 复制 | 手工 |
| V-7～V-8 | `MessageList` `isAtBottom` + `scrollToBottom` | 代码（阈值逻辑）+ 手工 UI |

### 18.2 会话管理（V-9～V-14）

| ID | 代码路径要点 | 验证方式 |
|----|----------------|-----------|
| V-9～V-13 | `sessionsIpc`、`SessionPanel`、`SessionItem` | 手工为主 |
| V-14 | SQLite 持久化 | 手工 |

### 18.3 工作目录（V-15～V-21）

| ID | 代码路径要点 | 验证方式 |
|----|----------------|-----------|
| V-15～V-21 | `workspaceIpc`、`WorkspaceSelector`、`ChatPage`/`updateWorkingDir` | 手工 + F-1 路径测试 |

### 18.4 Token（V-22～V-23）

| ID | 代码路径要点 | 验证方式 |
|----|----------------|-----------|
| V-22 | `stream_complete.usage` → `token_usage` + `TokenBadge` | 代码 + 手工 |
| V-23 | `SessionRepo::update_stats` | 代码 + 手工 |

### 18.5 思维链（V-24～V-26）

| ID | 代码路径要点 | 验证方式 |
|----|----------------|-----------|
| V-24～V-26 | `stream_thinking`、`ThinkingBlock`、`collapsible` | 手工（Claude） |

### 18.6 控制流（V-27～V-29）

| ID | 代码路径要点 | 验证方式 |
|----|----------------|-----------|
| V-27 | `stop_generation` + `was_aborted` / `aborted` | 代码 + 手工 |
| V-28 | `regenerate_message` + `removeMessagesFrom` | 代码 + 手工 |
| V-29 | 单会话单流 `StreamRegistry` | 代码 + 手工 |

### 18.7 多模态（V-30～V-33）

| ID | 代码路径要点 | 验证方式 |
|----|----------------|-----------|
| V-30～V-33 | `MessageInput` 粘贴/拖拽/附件、`ImagePreview` | 代码 + 手工 |

### 18.8 错误处理（V-34～V-36）

| ID | 代码路径要点 | 验证方式 |
|----|----------------|-----------|
| V-34 | 无效 Key：`stream_error` / IPC `Err` → toast + 错误气泡 | 代码 + 手工 |
| V-35 | 网络超时由 Rig/reqwest 报错，同上路径 | 手工 |
| V-36 | 模型不存在 / 400 类错误文案透出 | 手工 |

---

**说明：** 标记为「代码」的项以仓库内实现与 Vitest 为准；标记「手工」的项仍需在打包或 `npm run tauri dev` 下由人跑通。
