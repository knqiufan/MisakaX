//! Model IPC commands — thin delegates over [`crate::services::model_probe`]
//! and the custom-model repository.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::models::{AdvancedConfig, CreateCustomModel, CustomModel};
use crate::db::repository::router_config_repo::RouterConfigView;
use crate::db::repository::{CustomModelRepo, RouterConfigRepo};
use crate::services::llm::registry::{ModelInfo, ModelRegistry};
use crate::services::model_probe::{self, ModelProbeParams};
use crate::AppState;

/// 按 Provider 分组的模型列表
#[derive(Debug, Serialize)]
pub struct ProviderModels {
    pub provider: RouterConfigView,
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
pub struct FetchProviderModelsRequest {
    pub api: String,
    pub vendor: String,
    pub base_url: Option<String>,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
pub struct FetchProviderModelsResult {
    pub models: Vec<ModelInfo>,
    pub source: String,
    pub warning: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TestModelRequest {
    pub api: String,
    pub vendor: Option<String>,
    pub base_url: Option<String>,
    pub api_key: String,
    pub model_id: String,
    pub advanced: Option<AdvancedConfig>,
}

#[derive(Debug, Serialize)]
pub struct ModelTestResult {
    pub success: bool,
    pub latency_ms: u128,
    pub message: String,
}

/// 列出所有可用模型（按 Provider 分组，仅启用的自定义模型）
#[tauri::command]
pub fn list_available_models(
    state: State<'_, AppState>,
    router_config_id: Option<String>,
) -> Result<Vec<ProviderModels>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let views = RouterConfigRepo::find_views_by_optional_id(&db, router_config_id.as_deref())
        .map_err(|e| e.to_string())?;

    views
        .into_iter()
        .map(|view| {
            let models = ModelRegistry::available_models(&db, &view.id, &view.provider)
                .map_err(|e| e.to_string())?;
            Ok(ProviderModels {
                provider: view,
                models,
            })
        })
        .collect()
}

/// 为指定 Provider 添加自定义模型
#[tauri::command]
pub fn add_custom_model(
    state: State<'_, AppState>,
    router_config_id: String,
    model: CreateCustomModel,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    RouterConfigRepo::verify_exists(&db, &router_config_id).map_err(|e| e.to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    CustomModelRepo::insert(&db, &id, &router_config_id, &model).map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
pub fn list_custom_models(
    state: State<'_, AppState>,
    router_config_id: String,
) -> Result<Vec<CustomModel>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    CustomModelRepo::list_by_router(&db, &router_config_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn replace_custom_models(
    state: State<'_, AppState>,
    router_config_id: String,
    models: Vec<CreateCustomModel>,
) -> Result<(), String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    RouterConfigRepo::verify_exists(&db, &router_config_id).map_err(|e| e.to_string())?;
    CustomModelRepo::replace_all(&mut db, &router_config_id, &models).map_err(|e| e.to_string())
}

/// 删除自定义模型
#[tauri::command]
pub fn delete_custom_model(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    CustomModelRepo::delete(&db, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_provider_models(
    request: FetchProviderModelsRequest,
) -> Result<FetchProviderModelsResult, String> {
    let api = model_probe::parse_api(&request.api)?;
    let (models, warning) = model_probe::fetch_models(
        api,
        &request.vendor,
        request.base_url.as_deref(),
        &request.api_key,
    )
    .await?;
    Ok(FetchProviderModelsResult {
        source: if warning.is_some() {
            "fallback"
        } else {
            "remote"
        }
        .to_string(),
        models,
        warning,
    })
}

#[tauri::command]
pub async fn test_model(request: TestModelRequest) -> Result<ModelTestResult, String> {
    let api = model_probe::parse_api(&request.api)?;
    let vendor = request.vendor.as_deref().unwrap_or(request.api.as_str());
    let params = ModelProbeParams {
        api,
        vendor,
        base_url: request.base_url.as_deref(),
        api_key: &request.api_key,
        model_id: &request.model_id,
        proxy: request.advanced.as_ref().and_then(|a| a.proxy.as_deref()),
        max_tokens: request.advanced.as_ref().and_then(|a| a.max_tokens),
    };

    let started = std::time::Instant::now();
    let result = model_probe::ping_model(&params).await;
    let latency_ms = started.elapsed().as_millis();

    Ok(match result {
        Ok(message) => ModelTestResult {
            success: true,
            latency_ms,
            message,
        },
        Err(message) => ModelTestResult {
            success: false,
            latency_ms,
            message,
        },
    })
}
