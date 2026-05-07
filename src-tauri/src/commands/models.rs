use serde::Serialize;
use tauri::State;

use crate::commands::router_configs::RouterConfigView;
use crate::crypto;
use crate::db::models::CreateCustomModel;
use crate::services::llm::registry::{ModelInfo, ModelRegistry};
use crate::AppState;

/// 按 Provider 分组的模型列表
#[derive(Debug, Serialize)]
pub struct ProviderModels {
    pub provider: RouterConfigView,
    pub models: Vec<ModelInfo>,
}

/// 列出所有可用模型（按 Provider 分组，合并内置 + 自定义）
///
/// 如果传入 `router_config_id`，仅返回该 Provider 的模型；
/// 否则返回所有已配置 Provider 的模型。
#[tauri::command]
pub fn list_available_models(
    state: State<'_, AppState>,
    router_config_id: Option<String>,
) -> Result<Vec<ProviderModels>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let configs = load_provider_views(&db, router_config_id.as_deref())?;

    let result = configs
        .into_iter()
        .map(|view| {
            let models = ModelRegistry::available_models(&db, &view.id, &view.provider)
                .unwrap_or_default();
            ProviderModels {
                provider: view,
                models,
            }
        })
        .collect();

    Ok(result)
}

/// 为指定 Provider 添加自定义模型
#[tauri::command]
pub fn add_custom_model(
    state: State<'_, AppState>,
    router_config_id: String,
    model: CreateCustomModel,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    verify_router_config_exists(&db, &router_config_id)?;

    let id = uuid::Uuid::new_v4().to_string();
    db.execute(
        "INSERT INTO custom_models (id, router_config_id, model_id, display_name,
         supports_vision, supports_thinking, max_tokens, context_window)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            id,
            router_config_id,
            model.model_id,
            model.display_name,
            model.supports_vision as i32,
            model.supports_thinking as i32,
            model.max_tokens,
            model.context_window,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(id)
}

/// 删除自定义模型（按数据库主键 id 删除）
#[tauri::command]
pub fn delete_custom_model(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let affected = db
        .execute("DELETE FROM custom_models WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    if affected == 0 {
        return Err(format!("Custom model not found: {}", id));
    }
    Ok(())
}

fn load_provider_views(
    db: &rusqlite::Connection,
    config_id: Option<&str>,
) -> Result<Vec<RouterConfigView>, String> {
    let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(id) = config_id {
        (
            "SELECT id, name, provider, api_key_encrypted, model, base_url, is_active, created_at
             FROM router_configs WHERE id = ?1",
            vec![Box::new(id.to_string())],
        )
    } else {
        (
            "SELECT id, name, provider, api_key_encrypted, model, base_url, is_active, created_at
             FROM router_configs ORDER BY created_at DESC",
            vec![],
        )
    };

    let mut stmt = db.prepare(sql).map_err(|e| e.to_string())?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            let api_key_encrypted: Option<String> = row.get(3)?;
            let masked = api_key_encrypted
                .as_deref()
                .and_then(|enc| crypto::decrypt(enc).ok())
                .map(|plain| crypto::mask_api_key(&plain))
                .unwrap_or_else(|| "***".to_string());

            Ok(RouterConfigView {
                id: row.get(0)?,
                name: row.get(1)?,
                provider: row.get(2)?,
                api_key_masked: masked,
                model: row.get(4)?,
                base_url: row.get(5)?,
                is_active: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn verify_router_config_exists(
    db: &rusqlite::Connection,
    config_id: &str,
) -> Result<(), String> {
    let exists: bool = db
        .query_row(
            "SELECT COUNT(*) FROM router_configs WHERE id = ?1",
            [config_id],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .map_err(|e| e.to_string())?;

    if !exists {
        return Err(format!("Router config not found: {}", config_id));
    }
    Ok(())
}
