//! Local HTTP bridge exposing MCP tools to the Python Sidecar.
//!
//! Binds only to `127.0.0.1` and reuses `McpManager` + approval policy.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::services::mcp::approval::{decide_from_policy, ensure_tool_allowed, PolicyDecision};
use crate::services::mcp::McpManager;
use crate::services::mcp_bridge::McpToolBridge;
use crate::AppState;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CallToolRequest {
    pub server_id: String,
    pub tool_name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CallToolSuccess {
    pub result: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BridgeError {
    pub error: String,
}

#[derive(Clone)]
pub struct BridgeState {
    pub manager: Arc<McpManager>,
    /// When set, tool calls go through the same approval path as Tauri IPC.
    pub app: Option<AppHandle>,
}

/// Always bind the bridge to localhost only.
pub fn bridge_bind_addr(port: u16) -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], port))
}

pub fn validate_call_tool_request(req: &CallToolRequest) -> Result<(), String> {
    if req.server_id.trim().is_empty() {
        return Err("server_id is required".to_string());
    }
    if req.tool_name.trim().is_empty() {
        return Err("tool_name is required".to_string());
    }
    Ok(())
}

/// Pure helper used by tests to assert Ask policies are not silently allowed.
pub fn http_policy_decision(policy: Option<&str>) -> PolicyDecision {
    decide_from_policy(policy)
}

/// Map manager/call failures to HTTP status codes without pulling axum into unit tests.
pub fn map_call_failure_status(kind: CallFailureKind) -> u16 {
    match kind {
        CallFailureKind::BadRequest => 400,
        CallFailureKind::Forbidden => 403,
        CallFailureKind::Upstream => 502,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallFailureKind {
    BadRequest,
    Forbidden,
    Upstream,
}

fn build_router(state: BridgeState) -> Router {
    Router::new()
        .route("/mcp/list_tools", get(list_tools))
        .route("/mcp/call_tool", post(call_tool))
        .with_state(state)
}

pub async fn serve(app: AppHandle, port: u16) -> Result<(), String> {
    let (manager, sidecar) = {
        let state = app.state::<AppState>();
        (Arc::clone(&state.mcp_manager), Arc::clone(&state.sidecar))
    };
    let state = BridgeState {
        manager,
        app: Some(app),
    };
    let addr = bridge_bind_addr(port);
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        sidecar.set_mcp_bridge_ready(false);
        let pids = crate::sidecar_ownership::listening_pids_on_port(port);
        let pid_hint = pids
            .first()
            .map(|p| format!(" (listener PID {p})"))
            .unwrap_or_default();
        format!(
            "MCP HTTP bridge bind failed on {addr}{pid_hint}: {e}. \
             Stop the unmanaged listener or free port {port}; MisakaX will not kill unknown processes."
        )
    })?;

    sidecar.set_mcp_bridge_ready(true);
    tracing::info!(%addr, "MCP HTTP bridge listening");
    let result = axum::serve(listener, build_router(state))
        .await
        .map_err(|e| format!("MCP HTTP bridge server error: {e}"));
    sidecar.set_mcp_bridge_ready(false);
    result
}

/// Test/helper entrypoint that skips UI approval and only uses McpManager.
pub async fn serve_manager_only(manager: Arc<McpManager>, port: u16) -> Result<(), String> {
    let state = BridgeState { manager, app: None };
    let addr = bridge_bind_addr(port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("MCP HTTP bridge bind failed on {addr}: {e}"))?;
    axum::serve(listener, build_router(state))
        .await
        .map_err(|e| format!("MCP HTTP bridge server error: {e}"))
}

async fn list_tools(State(state): State<BridgeState>) -> impl IntoResponse {
    let tools = state.manager.list_all_tools();
    (StatusCode::OK, Json(serde_json::json!({ "tools": tools })))
}

async fn call_tool(
    State(state): State<BridgeState>,
    Json(req): Json<CallToolRequest>,
) -> impl IntoResponse {
    if let Err(msg) = validate_call_tool_request(&req) {
        return error_response(StatusCode::BAD_REQUEST, msg);
    }

    if let Some(app) = state.app.as_ref() {
        match enforce_approval(app, &req).await {
            Ok(true) => {}
            Ok(false) => {
                return error_response(StatusCode::FORBIDDEN, "Tool call denied".to_string());
            }
            Err(err) => {
                return error_response(StatusCode::FORBIDDEN, err);
            }
        }
    }

    match state
        .manager
        .call_tool(&req.server_id, &req.tool_name, req.arguments.clone())
        .await
    {
        Ok(result) => {
            let text = McpToolBridge::format_tool_result(&result);
            (StatusCode::OK, Json(serde_json::json!({ "result": text }))).into_response()
        }
        Err(err) => error_response(StatusCode::BAD_GATEWAY, err.to_string()),
    }
}

async fn enforce_approval(app: &AppHandle, req: &CallToolRequest) -> Result<bool, String> {
    let state = app.state::<AppState>();
    ensure_tool_allowed(
        app,
        &state.db,
        &state.mcp_manager,
        &req.server_id,
        &req.tool_name,
        &req.arguments,
    )
    .await
}

fn error_response(status: StatusCode, message: String) -> axum::response::Response {
    (status, Json(BridgeError { error: message })).into_response()
}
