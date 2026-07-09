//! MCP 工具调用审批服务
//!
//! 将工具调用的权限判定（policy 查询 + 用户审批弹窗）抽取为可复用模块，
//! 供手动 IPC（`mcp_call_tool`）与对话内 Rig 工具循环（`tool_loop`）共用同一路径。
//!
//! 审批 UX 复用现有前端 `mcp:tool_call_request` 事件 + `ToolApprovalDialog`。

use std::sync::Mutex;
use std::time::Duration;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

use crate::db::repository::ToolPermissionRepo;

use super::manager::McpManager;

/// 全局待审批请求注册表
///
/// key 为 `request_id`，value 为等待前端响应的 oneshot 发送端。
static PENDING_APPROVALS: std::sync::LazyLock<
    dashmap::DashMap<String, oneshot::Sender<ToolCallApprovalResponse>>,
> = std::sync::LazyLock::new(dashmap::DashMap::new);

/// 审批请求 payload（emit 给前端 `ToolApprovalDialog`）
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallRequestPayload {
    pub request_id: String,
    pub server_id: String,
    pub server_name: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

/// 前端审批响应（approve/deny command 回传）
#[derive(Debug, Clone, Deserialize)]
pub struct ToolCallApprovalResponse {
    pub approved: bool,
    pub remember: bool,
}

/// policy 判定结果（纯函数产出，便于单测）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    /// 直接放行
    Allow,
    /// 直接拒绝
    Deny,
    /// 需要弹窗询问用户
    Ask,
}

/// 根据存储的 policy 字符串判定分支
///
/// - `Some("allow")` → Allow
/// - `Some("deny")` → Deny
/// - `None` / `Some("ask")` / 其他 → Ask
pub fn decide_from_policy(policy: Option<&str>) -> PolicyDecision {
    match policy {
        Some("allow") => PolicyDecision::Allow,
        Some("deny") => PolicyDecision::Deny,
        _ => PolicyDecision::Ask,
    }
}

/// 判定某次工具调用是否被允许执行
///
/// 返回 `Ok(true)` 放行、`Ok(false)` 拒绝，`Err` 仅表示系统级错误（锁失败/超时/emit 失败）。
/// 供 `mcp_call_tool` 与 Rig 工具循环共用。
pub async fn ensure_tool_allowed(
    app: &AppHandle,
    db: &Mutex<Connection>,
    manager: &McpManager,
    server_id: &str,
    tool_name: &str,
    arguments: &serde_json::Value,
) -> Result<bool, String> {
    let policy = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        ToolPermissionRepo::find_policy(&conn, server_id, tool_name).map_err(|e| e.to_string())?
    };

    match decide_from_policy(policy.as_deref()) {
        PolicyDecision::Allow => Ok(true),
        PolicyDecision::Deny => Ok(false),
        PolicyDecision::Ask => {
            request_user_approval(app, db, manager, server_id, tool_name, arguments).await
        }
    }
}

/// 弹窗询问用户，并在 60s 内等待响应；`remember` 时落库 policy
async fn request_user_approval(
    app: &AppHandle,
    db: &Mutex<Connection>,
    manager: &McpManager,
    server_id: &str,
    tool_name: &str,
    arguments: &serde_json::Value,
) -> Result<bool, String> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = oneshot::channel::<ToolCallApprovalResponse>();

    PENDING_APPROVALS.insert(request_id.clone(), tx);

    let server_name = manager
        .server_info(server_id)
        .map(|info| info.name.clone())
        .unwrap_or_else(|| server_id.to_string());

    let payload = ToolCallRequestPayload {
        request_id: request_id.clone(),
        server_id: server_id.to_string(),
        server_name,
        tool_name: tool_name.to_string(),
        arguments: arguments.clone(),
    };

    app.emit("mcp:tool_call_request", &payload)
        .map_err(|e| e.to_string())?;

    let response = tokio::time::timeout(Duration::from_secs(60), rx)
        .await
        .map_err(|_| {
            PENDING_APPROVALS.remove(&request_id);
            "Tool call approval timed out (60s)".to_string()
        })?
        .map_err(|_| "Approval channel closed".to_string())?;

    persist_remembered_policy(db, server_id, tool_name, &response)?;

    Ok(response.approved)
}

/// 用户勾选「始终允许/拒绝」时，将结果写入 `tool_permissions`
fn persist_remembered_policy(
    db: &Mutex<Connection>,
    server_id: &str,
    tool_name: &str,
    response: &ToolCallApprovalResponse,
) -> Result<(), String> {
    if !response.remember {
        return Ok(());
    }
    let policy = if response.approved { "allow" } else { "deny" };
    let conn = db.lock().map_err(|e| e.to_string())?;
    ToolPermissionRepo::upsert_policy(&conn, server_id, tool_name, policy)
        .map_err(|e| e.to_string())
}

/// 前端 approve/deny command 调用：向等待中的调用方发送审批结果
pub fn complete_approval(request_id: &str, approved: bool, remember: bool) -> Result<(), String> {
    let entry = PENDING_APPROVALS
        .remove(request_id)
        .ok_or_else(|| "No pending approval found for this request".to_string())?;

    let _ = entry
        .1
        .send(ToolCallApprovalResponse { approved, remember });

    Ok(())
}
