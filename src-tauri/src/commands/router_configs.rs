use serde::Deserialize;
use tauri::State;

use crate::crypto;
use crate::db::models::AdvancedConfig;
use crate::db::repository::router_config_repo::{RouterConfigView, UpdateBuilder};
use crate::db::repository::{CustomModelRepo, RouterConfigRepo};
use crate::services::llm::catalog::ProviderApi;
use crate::services::model_probe::{self, ConnectionTestOutcome};
use crate::AppState;

// ─── 请求类型 ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateRouterConfig {
    pub name: String,
    pub provider: String,
    pub vendor: Option<String>,
    pub api_key: String,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub config_json: Option<String>,
    pub advanced: Option<AdvancedConfig>,
    pub is_active: Option<bool>,
    pub api_compat: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRouterConfig {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub vendor: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub config_json: Option<String>,
    pub advanced: Option<AdvancedConfig>,
    pub is_active: Option<bool>,
    pub api_compat: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRouterConfigWithModels {
    pub config: CreateRouterConfig,
    pub models: Vec<crate::db::models::CreateCustomModel>,
}

#[derive(Debug, serde::Serialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub models: Option<Vec<String>>,
}

// ─── Commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_router_configs(state: State<'_, AppState>) -> Result<Vec<RouterConfigView>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    RouterConfigRepo::list_views(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_router_config(
    state: State<'_, AppState>,
    config: CreateRouterConfig,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    insert_router_config_record(&db, &config)
}

#[tauri::command]
pub fn create_router_config_with_models(
    state: State<'_, AppState>,
    request: CreateRouterConfigWithModels,
) -> Result<String, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    let tx = db.transaction().map_err(|e| e.to_string())?;
    let id = insert_router_config_record(&tx, &request.config)?;

    for model in &request.models {
        let model_id = uuid::Uuid::new_v4().to_string();
        CustomModelRepo::insert(&tx, &model_id, &id, model).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub fn update_router_config(
    state: State<'_, AppState>,
    id: String,
    config: UpdateRouterConfig,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let encrypted_key = config
        .api_key
        .as_deref()
        .map(crypto::encrypt)
        .transpose()
        .map_err(|e| e.to_string())?;

    let mut updates = UpdateBuilder::new();
    updates
        .set_opt("name", &config.name)
        .set_opt("provider", &config.provider)
        .set_opt("vendor", &config.vendor)
        .set_opt("api_key_encrypted", &encrypted_key)
        .set_opt("model", &config.model)
        .set_opt("base_url", &config.base_url)
        .set_opt("config_json", &config.config_json)
        .set_opt("api_compat", &config.api_compat);

    let advanced_json = serialize_advanced(config.advanced.as_ref())?;
    updates.set_opt("advanced_json", &advanced_json);

    if let Some(is_active) = config.is_active {
        updates.set("is_active", is_active as i32);
    }

    RouterConfigRepo::update(&db, &id, &updates.build()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reveal_router_api_key(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let config = RouterConfigRepo::find_by_id(&db, &id).map_err(|e| e.to_string())?;
    let encrypted = config
        .api_key_encrypted
        .ok_or_else(|| "No API key configured".to_string())?;
    crypto::decrypt(&encrypted).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_router_config(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    RouterConfigRepo::delete(&db, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_router_connection(
    state: State<'_, AppState>,
    id: String,
) -> Result<ConnectionTestResult, String> {
    let (api_key, base_url, provider, vendor) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        RouterConfigRepo::find_connection_info(&db, &id).map_err(|e| e.to_string())?
    };

    let decrypted_key = match api_key {
        Some(enc) => crypto::decrypt(&enc).map_err(|e| e.to_string())?,
        None => {
            return Ok(ConnectionTestResult {
                success: false,
                message: "No API key configured".to_string(),
                models: None,
            })
        }
    };

    let api = ProviderApi::parse(&provider)
        .ok_or_else(|| format!("Unsupported provider api: {provider}"))?;
    let vendor = vendor.as_deref().unwrap_or(provider.as_str());

    let outcome =
        model_probe::test_connection(api, vendor, base_url.as_deref(), &decrypted_key).await?;

    Ok(match outcome {
        ConnectionTestOutcome::Ok => ConnectionTestResult {
            success: true,
            message: "Connection successful".to_string(),
            models: None,
        },
        ConnectionTestOutcome::NoEndpoint => ConnectionTestResult {
            success: false,
            message: "Provider does not expose a model list endpoint".to_string(),
            models: None,
        },
        ConnectionTestOutcome::HttpError { status, body } => ConnectionTestResult {
            success: false,
            message: format!("HTTP {status}: {body}"),
            models: None,
        },
    })
}

// ─── 内部辅助 ─────────────────────────────────────────────────────────

fn serialize_advanced(advanced: Option<&AdvancedConfig>) -> Result<Option<String>, String> {
    advanced
        .map(serde_json::to_string)
        .transpose()
        .map_err(|e| e.to_string())
}

/// 加密 API key、序列化 advanced，写入 router_configs 记录（事务/连接均可）。
fn insert_router_config_record(
    conn: &rusqlite::Connection,
    config: &CreateRouterConfig,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let encrypted_key = crypto::encrypt(&config.api_key).map_err(|e| e.to_string())?;
    let advanced_json = serialize_advanced(config.advanced.as_ref())?;

    RouterConfigRepo::insert_with_metadata(
        conn,
        &id,
        &config.name,
        &config.provider,
        config.vendor.as_deref(),
        &encrypted_key,
        config.model.as_deref(),
        config.base_url.as_deref(),
        config.config_json.as_deref(),
        advanced_json.as_deref(),
        config.is_active.unwrap_or(false),
        config.api_compat.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    Ok(id)
}
