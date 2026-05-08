use serde::Serialize;
use tauri::State;

use crate::db::models::CreateCustomModel;
use crate::db::repository::router_config_repo::RouterConfigView;
use crate::db::repository::{CustomModelRepo, RouterConfigRepo};
use crate::services::llm::registry::{ModelInfo, ModelRegistry};
use crate::AppState;

/// 按 Provider 分组的模型列表
#[derive(Debug, Serialize)]
pub struct ProviderModels {
    pub provider: RouterConfigView,
    pub models: Vec<ModelInfo>,
}

/// 列出所有可用模型（按 Provider 分组，合并内置 + 自定义）
#[tauri::command]
pub fn list_available_models(
    state: State<'_, AppState>,
    router_config_id: Option<String>,
) -> Result<Vec<ProviderModels>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let views = RouterConfigRepo::find_views_by_optional_id(
        &db,
        router_config_id.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    let result = views
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

    RouterConfigRepo::verify_exists(&db, &router_config_id)
        .map_err(|e| e.to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    CustomModelRepo::insert(&db, &id, &router_config_id, &model)
        .map_err(|e| e.to_string())?;

    Ok(id)
}

/// 删除自定义模型
#[tauri::command]
pub fn delete_custom_model(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    CustomModelRepo::delete(&db, &id).map_err(|e| e.to_string())
}
