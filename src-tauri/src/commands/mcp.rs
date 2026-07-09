use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::db::repository::{McpServerRepo, ToolPermission, ToolPermissionRepo};
use crate::services::mcp::approval::{complete_approval, ensure_tool_allowed};
use crate::services::mcp::{McpServerConfig, McpServerInfo, McpToolInfo};
use crate::AppState;

// ─── 查询类 Commands ──────────────────────────────────────────────────

#[tauri::command]
pub fn mcp_list_servers(state: State<'_, AppState>) -> Vec<McpServerInfo> {
    state.mcp_manager.list_servers()
}

#[tauri::command]
pub fn mcp_list_tools(state: State<'_, AppState>) -> Vec<McpToolInfo> {
    state.mcp_manager.list_all_tools()
}

// ─── 连接管理 Commands ────────────────────────────────────────────────

#[tauri::command]
pub async fn mcp_connect_server(
    state: State<'_, AppState>,
    app: AppHandle,
    config: McpServerConfig,
) -> Result<McpServerInfo, String> {
    let manager = Arc::clone(&state.mcp_manager);

    manager
        .connect(config.clone())
        .await
        .map_err(|e| e.to_string())?;

    let info = manager
        .server_info(&config.id)
        .unwrap_or_else(|| McpServerInfo {
            id: config.id.clone(),
            name: config.name.clone(),
            transport_type: "unknown".to_string(),
            status: crate::services::mcp::McpServerStatus::Disconnected,
            tools_count: 0,
            auto_connect: config.auto_connect,
        });

    let _ = app.emit("mcp:server_status", &info);
    Ok(info)
}

#[tauri::command]
pub async fn mcp_disconnect_server(
    state: State<'_, AppState>,
    app: AppHandle,
    server_id: String,
) -> Result<(), String> {
    let manager = Arc::clone(&state.mcp_manager);

    manager
        .disconnect(&server_id)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(info) = manager.server_info(&server_id) {
        let _ = app.emit("mcp:server_status", &info);
    }

    Ok(())
}

#[tauri::command]
pub async fn mcp_restart_server(
    state: State<'_, AppState>,
    app: AppHandle,
    server_id: String,
) -> Result<McpServerInfo, String> {
    let manager = Arc::clone(&state.mcp_manager);

    manager
        .reconnect(&server_id)
        .await
        .map_err(|e| e.to_string())?;

    let info = manager
        .server_info(&server_id)
        .ok_or_else(|| "Server not found after restart".to_string())?;

    let _ = app.emit("mcp:server_status", &info);
    Ok(info)
}

// ─── 工具调用 Command ─────────────────────────────────────────────────

#[tauri::command]
pub async fn mcp_call_tool(
    state: State<'_, AppState>,
    app: AppHandle,
    server_id: String,
    tool_name: String,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let allowed = ensure_tool_allowed(
        &app,
        &state.db,
        &state.mcp_manager,
        &server_id,
        &tool_name,
        &arguments,
    )
    .await?;

    if !allowed {
        return Err("Tool call denied".to_string());
    }

    let manager = Arc::clone(&state.mcp_manager);
    let result = manager
        .call_tool(&server_id, &tool_name, arguments)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(&result).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mcp_approve_tool_call(request_id: String, remember: bool) -> Result<(), String> {
    complete_approval(&request_id, true, remember)
}

#[tauri::command]
pub fn mcp_deny_tool_call(request_id: String, remember: bool) -> Result<(), String> {
    complete_approval(&request_id, false, remember)
}

// ─── 权限管理 Commands ────────────────────────────────────────────────

#[tauri::command]
pub fn mcp_list_permissions(state: State<'_, AppState>) -> Result<Vec<ToolPermission>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ToolPermissionRepo::list_all(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mcp_reset_permission(
    state: State<'_, AppState>,
    server_id: String,
    tool_name: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    ToolPermissionRepo::reset(&db, &server_id, &tool_name).map_err(|e| e.to_string())
}

// ─── 配置管理 Commands ────────────────────────────────────────────────

#[tauri::command]
pub fn mcp_add_server_config(
    state: State<'_, AppState>,
    config: McpServerConfig,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let transport_json = serde_json::to_string(&config.transport).map_err(|e| e.to_string())?;
    let env_json = if config.env.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&config.env).map_err(|e| e.to_string())?)
    };

    let record = crate::db::repository::McpServerRecord {
        id: config.id.clone(),
        name: config.name.clone(),
        transport_json,
        auto_connect: config.auto_connect,
        env_json,
        created_at: String::new(),
        updated_at: String::new(),
    };

    McpServerRepo::insert(&db, &record).map_err(|e| e.to_string())?;

    state.mcp_manager.add_server_config(config);

    Ok(())
}

#[tauri::command]
pub async fn mcp_remove_server_config(
    state: State<'_, AppState>,
    app: AppHandle,
    server_id: String,
) -> Result<(), String> {
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let _ = McpServerRepo::delete(&db, &server_id);
    }

    let manager = Arc::clone(&state.mcp_manager);
    manager
        .remove_server(&server_id)
        .await
        .map_err(|e| e.to_string())?;

    let _ = app.emit(
        "mcp:server_status",
        &McpServerInfo {
            id: server_id,
            name: String::new(),
            transport_type: String::new(),
            status: crate::services::mcp::McpServerStatus::Disconnected,
            tools_count: 0,
            auto_connect: false,
        },
    );

    Ok(())
}
