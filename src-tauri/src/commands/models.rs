use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::models::{AdvancedConfig, CreateCustomModel, CustomModel};
use crate::db::repository::router_config_repo::RouterConfigView;
use crate::db::repository::{CustomModelRepo, RouterConfigRepo};
use crate::services::llm::catalog::{chat_url, google_generate_url, models_url, ProviderApi};
use crate::services::llm::registry::{ModelInfo, ModelRegistry};
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
    let api = parse_api(&request.api)?;
    let Some(url) = models_url(api, &request.vendor, request.base_url.as_deref()) else {
        return Ok(fallback_models(
            api,
            "Provider does not expose a model list endpoint",
        ));
    };

    let client = build_http_client(None)?;
    let response = models_request(&client, api, &url, &request.api_key)
        .send()
        .await
        .map_err(sanitize_request_error)?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status().as_u16()));
    }

    let body = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;
    Ok(FetchProviderModelsResult {
        models: parse_models_response(api, &body),
        source: "remote".to_string(),
        warning: None,
    })
}

#[tauri::command]
pub async fn test_model(request: TestModelRequest) -> Result<ModelTestResult, String> {
    let api = parse_api(&request.api)?;
    let vendor = request.vendor.as_deref().unwrap_or(request.api.as_str());
    let started = std::time::Instant::now();
    let result = send_model_ping(api, vendor, &request).await;
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

fn parse_api(api: &str) -> Result<ProviderApi, String> {
    ProviderApi::parse(api).ok_or_else(|| format!("Unsupported provider api: {api}"))
}

fn build_http_client(proxy: Option<&str>) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder().timeout(std::time::Duration::from_secs(10));
    if let Some(proxy) = proxy.filter(|value| !value.trim().is_empty()) {
        let proxy = reqwest::Proxy::all(proxy).map_err(|_| "Invalid proxy URL".to_string())?;
        builder = builder.proxy(proxy);
    }
    builder
        .build()
        .map_err(|_| "Failed to build HTTP client".to_string())
}

fn fallback_models(api: ProviderApi, warning: &str) -> FetchProviderModelsResult {
    FetchProviderModelsResult {
        models: ModelRegistry::builtin_models(api.as_str()),
        source: "fallback".to_string(),
        warning: Some(warning.to_string()),
    }
}

fn models_request<'a>(
    client: &'a reqwest::Client,
    api: ProviderApi,
    url: &'a str,
    api_key: &'a str,
) -> reqwest::RequestBuilder {
    match api {
        ProviderApi::Anthropic => client
            .get(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01"),
        ProviderApi::Google => client.get(url).query(&[("key", api_key)]),
        ProviderApi::OpenAi => client.get(url).bearer_auth(api_key),
    }
}

fn parse_models_response(api: ProviderApi, body: &serde_json::Value) -> Vec<ModelInfo> {
    match api {
        ProviderApi::Google => parse_google_models(body),
        _ => parse_data_models(body),
    }
}

fn parse_data_models(body: &serde_json::Value) -> Vec<ModelInfo> {
    body["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item["id"].as_str())
        .map(model_info_from_id)
        .collect()
}

fn parse_google_models(body: &serde_json::Value) -> Vec<ModelInfo> {
    body["models"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let raw_id = item["name"].as_str()?;
            let model_id = raw_id.trim_start_matches("models/");
            let display = item["displayName"].as_str().unwrap_or(model_id);
            Some(ModelInfo::builtin(model_id, display, true, false))
        })
        .collect()
}

fn model_info_from_id(model_id: &str) -> ModelInfo {
    ModelInfo::builtin(model_id, model_id, false, false)
}

async fn send_model_ping(
    api: ProviderApi,
    vendor: &str,
    request: &TestModelRequest,
) -> Result<String, String> {
    let proxy = request
        .advanced
        .as_ref()
        .and_then(|advanced| advanced.proxy.as_deref());
    let client = build_http_client(proxy)?;
    let response = match api {
        ProviderApi::OpenAi => send_openai_ping(&client, vendor, request).await?,
        ProviderApi::Anthropic => send_anthropic_ping(&client, vendor, request).await?,
        ProviderApi::Google => send_google_ping(&client, request).await?,
    };

    if response.status().is_success() {
        Ok("Connection successful".to_string())
    } else {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        Err(format!("HTTP {}: {}", status, truncate(&body, 200)))
    }
}

async fn send_openai_ping(
    client: &reqwest::Client,
    vendor: &str,
    request: &TestModelRequest,
) -> Result<reqwest::Response, String> {
    let url = chat_url(ProviderApi::OpenAi, vendor, request.base_url.as_deref())
        .ok_or_else(|| "Unsupported OpenAI endpoint".to_string())?;
    client
        .post(url)
        .bearer_auth(&request.api_key)
        .json(&openai_ping_body(request))
        .send()
        .await
        .map_err(sanitize_request_error)
}

async fn send_anthropic_ping(
    client: &reqwest::Client,
    vendor: &str,
    request: &TestModelRequest,
) -> Result<reqwest::Response, String> {
    let url = chat_url(ProviderApi::Anthropic, vendor, request.base_url.as_deref())
        .ok_or_else(|| "Unsupported Anthropic endpoint".to_string())?;
    client
        .post(url)
        .header("x-api-key", &request.api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&anthropic_ping_body(request))
        .send()
        .await
        .map_err(sanitize_request_error)
}

async fn send_google_ping(
    client: &reqwest::Client,
    request: &TestModelRequest,
) -> Result<reqwest::Response, String> {
    let url = google_generate_url(request.base_url.as_deref(), &request.model_id);
    client
        .post(url)
        .query(&[("key", &request.api_key)])
        .json(&serde_json::json!({ "contents": [{ "parts": [{ "text": "ping" }] }] }))
        .send()
        .await
        .map_err(sanitize_request_error)
}

fn openai_ping_body(request: &TestModelRequest) -> serde_json::Value {
    let advanced = request.advanced.clone().unwrap_or_default();
    serde_json::json!({
        "model": request.model_id,
        "messages": [{ "role": "user", "content": "ping" }],
        "max_tokens": advanced.max_tokens.unwrap_or(4),
        "temperature": 0.0
    })
}

fn anthropic_ping_body(request: &TestModelRequest) -> serde_json::Value {
    let advanced = request.advanced.clone().unwrap_or_default();
    serde_json::json!({
        "model": request.model_id,
        "messages": [{ "role": "user", "content": "ping" }],
        "max_tokens": advanced.max_tokens.unwrap_or(4),
        "temperature": 0.0
    })
}

fn truncate(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    s.get(..max_bytes).unwrap_or("")
}

fn sanitize_request_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        "Network request timed out".to_string()
    } else {
        "Network request failed".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_openai_models_response() {
        let body = serde_json::json!({
            "data": [{ "id": "gpt-4o" }, { "id": "gpt-4o-mini" }]
        });

        let models = parse_models_response(ProviderApi::OpenAi, &body);

        assert_eq!(models.len(), 2);
        assert_eq!(models[0].model_id, "gpt-4o");
    }

    #[test]
    fn parses_google_models_response() {
        let body = serde_json::json!({
            "models": [{ "name": "models/gemini-2.5-flash", "displayName": "Gemini Flash" }]
        });

        let models = parse_models_response(ProviderApi::Google, &body);

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].model_id, "gemini-2.5-flash");
        assert_eq!(models[0].display_name, "Gemini Flash");
    }

    #[test]
    fn fallback_models_uses_builtin_candidates() {
        let result = fallback_models(ProviderApi::Anthropic, "no endpoint");

        assert_eq!(result.source, "fallback");
        assert!(!result.models.is_empty());
        assert_eq!(result.warning, Some("no endpoint".to_string()));
    }

    #[test]
    fn build_http_client_rejects_invalid_proxy() {
        let result = build_http_client(Some("not a proxy url"));

        assert!(result.is_err());
    }
}
