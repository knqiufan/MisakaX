use serde::Deserialize;
use tauri::State;

use crate::crypto;
use crate::db::repository::router_config_repo::RouterConfigView;
use crate::db::repository::RouterConfigRepo;
use crate::AppState;

// ─── 请求类型 ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateRouterConfig {
    pub name: String,
    pub provider: String,
    pub api_key: String,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub config_json: Option<String>,
    pub is_active: Option<bool>,
    pub api_compat: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRouterConfig {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub config_json: Option<String>,
    pub is_active: Option<bool>,
    pub api_compat: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub models: Option<Vec<String>>,
}

// ─── Commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_router_configs(
    state: State<'_, AppState>,
) -> Result<Vec<RouterConfigView>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    RouterConfigRepo::list_views(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_router_config(
    state: State<'_, AppState>,
    config: CreateRouterConfig,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let encrypted_key = crypto::encrypt(&config.api_key).map_err(|e| e.to_string())?;

    RouterConfigRepo::insert(
        &db,
        &id,
        &config.name,
        &config.provider,
        &encrypted_key,
        config.model.as_deref(),
        config.base_url.as_deref(),
        config.config_json.as_deref(),
        config.is_active.unwrap_or(false),
        config.api_compat.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
pub fn update_router_config(
    state: State<'_, AppState>,
    id: String,
    config: UpdateRouterConfig,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut updates: Vec<(&str, Box<dyn rusqlite::types::ToSql>)> = Vec::new();

    if let Some(ref name) = config.name {
        updates.push(("name", Box::new(name.clone())));
    }
    if let Some(ref provider) = config.provider {
        updates.push(("provider", Box::new(provider.clone())));
    }
    if let Some(ref api_key) = config.api_key {
        let encrypted = crypto::encrypt(api_key).map_err(|e| e.to_string())?;
        updates.push(("api_key_encrypted", Box::new(encrypted)));
    }
    if let Some(ref model) = config.model {
        updates.push(("model", Box::new(model.clone())));
    }
    if let Some(ref base_url) = config.base_url {
        updates.push(("base_url", Box::new(base_url.clone())));
    }
    if let Some(ref config_json) = config.config_json {
        updates.push(("config_json", Box::new(config_json.clone())));
    }
    if let Some(is_active) = config.is_active {
        updates.push(("is_active", Box::new(is_active as i32)));
    }
    if let Some(ref api_compat) = config.api_compat {
        updates.push(("api_compat", Box::new(api_compat.clone())));
    }

    RouterConfigRepo::update(&db, &id, &updates).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_router_config(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    RouterConfigRepo::delete(&db, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_router_connection(
    state: State<'_, AppState>,
    id: String,
) -> Result<ConnectionTestResult, String> {
    let (api_key, base_url, provider) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        RouterConfigRepo::find_connection_info(&db, &id)
            .map_err(|e| e.to_string())?
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

    let url = build_models_url(&provider, base_url.as_deref());

    let client = reqwest::Client::new();
    let mut request = client.get(&url).timeout(std::time::Duration::from_secs(10));

    request = match provider.as_str() {
        "anthropic" => request
            .header("x-api-key", &decrypted_key)
            .header("anthropic-version", "2023-06-01"),
        "google" => {
            let sep = if url.contains('?') { "&" } else { "?" };
            let url_with_key = format!("{}{}key={}", url, sep, decrypted_key);
            client
                .get(&url_with_key)
                .timeout(std::time::Duration::from_secs(10))
        }
        _ => request.header("Authorization", format!("Bearer {}", decrypted_key)),
    };

    let response = request.send().await.map_err(|e| e.to_string())?;

    if response.status().is_success() {
        Ok(ConnectionTestResult {
            success: true,
            message: "Connection successful".to_string(),
            models: None,
        })
    } else {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        Ok(ConnectionTestResult {
            success: false,
            message: format!("HTTP {}: {}", status, truncate(&body, 200)),
            models: None,
        })
    }
}

// ─── 内部辅助 ─────────────────────────────────────────────────────────

fn build_models_url(provider: &str, base_url: Option<&str>) -> String {
    match provider {
        "anthropic" => "https://api.anthropic.com/v1/models".to_string(),
        "google" => "https://generativelanguage.googleapis.com/v1/models".to_string(),
        _ => {
            let base = base_url.unwrap_or("https://api.openai.com");
            format!("{}/v1/models", base.trim_end_matches('/'))
        }
    }
}

fn truncate(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    match s.get(..max_bytes) {
        Some(valid) => valid,
        None => {
            let mut end = max_bytes;
            while end > 0 && !s.is_char_boundary(end) {
                end -= 1;
            }
            &s[..end]
        }
    }
}
