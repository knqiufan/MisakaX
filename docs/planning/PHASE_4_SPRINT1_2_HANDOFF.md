# Phase 4 Sprint 1–2 出口与 Sprint 3 衔接

> **Purpose:** 记录 DeepAgents Sprint 1–2 落地结果，以及进入 Rust `4.5` 对话迁移前的前置条件。  
> **Audience:** 继续实现 Phase 4 Sprint 3 的开发者 / Agent  
> **Last reviewed:** 2026-07-09

## 已完成（Sprint 1–2）

| 能力 | 状态 | 关键路径 |
|------|------|----------|
| Sidecar Settings 扩展 | 完成 | `agent/app/config.py` |
| DeepAgents `/agent/chat` + `/agent/stream` | 完成（mock 单测覆盖） | `agent/app/routers/agent.py` |
| SSE 事件格式 | 完成 | `_sse` / `_format_sse_event` |
| working_dir 安全校验 | 完成 | `agent/app/utils.py` |
| PowerMem 最小桩 + tools | 完成 | `agent/app/memory.py`, `agent/app/tools.py` |
| SubAgent 配置 | 完成 | `agent/app/agent.py`, `agent/app/prompts.py` |
| Rust MCP HTTP Bridge | 完成 | `src-tauri/src/services/mcp_http_bridge.rs` |
| Sidecar 注入 `MISAKA_MCP_BRIDGE_URL` | 完成 | `src-tauri/src/sidecar.rs` |

## 出口验证（本机）

```powershell
python -m pytest agent/tests -q
# 60 passed, 1 skipped

cd src-tauri
cargo check
cargo test --test mcp_http_bridge_tests
# 4 passed
```

说明：Windows 下 axum `Router` 被 integration test 二进制直接链接时可能触发 `STATUS_ENTRYPOINT_NOT_FOUND`；因此 bridge 测试覆盖纯函数契约（校验、localhost 绑定、状态码映射、policy），HTTP serve 路径由 `cargo check` + 运行时启动覆盖。

## 进入 Sprint 3（4.5）前必须补齐

1. **`AppConfig.use_sidecar`**（默认 `true`）与降级开关。
2. **`chat.rs` 拆分** `send_via_sidecar()` / `send_via_rig()`；Sidecar 路径不要再注入 MCP prompt（Agent 走 `mcp_bridge`）。
3. **`parse_sidecar_sse`** 将 Python SSE（`token` / `tool_start` / `tool_end` / `done` / `error`）映射到现有 Tauri Event payload。
4. **Sidecar SSE 解析单测**（建议放在 `streaming_tests.rs` 或新测试文件），不要只靠手工验证。
5. **API Key 注入**（`MISAKA_ANTHROPIC_API_KEY` / `MISAKA_OPENAI_API_KEY`）可与 `4.8.3` 合并，但 live DeepAgents 冒烟前需要。

## 明确不在本轮范围

- 完整 HITL resume/deny（`4.10`）
- 记忆管理 UI（`4.9`）
- Nuitka 打包含 deepagents/powermem（`4.11`）
- 真实 LLM / 真实 MCP / 真实 PowerMem 端到端（integration/manual）
