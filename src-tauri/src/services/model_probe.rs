//! Provider model-list fetching and connection probing over HTTP.
//!
//! Used by `commands::models` for `fetch_provider_models` / `test_model`.
//! Deliberately free of IPC DTO types — callers pass primitives via
//! [`ModelProbeParams`].

use crate::services::llm::catalog::{chat_url, google_generate_url, models_url, ProviderApi};
use crate::services::llm::registry::{ModelInfo, ModelRegistry};

/// Parameters for a single model connection probe (vendor-agnostic).
#[derive(Debug, Clone)]
pub struct ModelProbeParams<'a> {
    pub api: ProviderApi,
    pub vendor: &'a str,
    pub base_url: Option<&'a str>,
    pub api_key: &'a str,
    pub model_id: &'a str,
    pub proxy: Option<&'a str>,
    pub max_tokens: Option<i32>,
}

pub(crate) fn parse_api(api: &str) -> Result<ProviderApi, String> {
    ProviderApi::parse(api).ok_or_else(|| format!("Unsupported provider api: {api}"))
}

pub(crate) fn build_http_client(proxy: Option<&str>) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder().timeout(std::time::Duration::from_secs(10));
    if let Some(proxy) = proxy.filter(|value| !value.trim().is_empty()) {
        let proxy = reqwest::Proxy::all(proxy).map_err(|_| "Invalid proxy URL".to_string())?;
        builder = builder.proxy(proxy);
    }
    builder
        .build()
        .map_err(|_| "Failed to build HTTP client".to_string())
}

/// Fetch the remote model list.
///
/// Returns `(models, warning)`. When the provider exposes no list endpoint,
/// built-in candidates are returned and `warning` is set (no network call).
pub(crate) async fn fetch_models(
    api: ProviderApi,
    vendor: &str,
    base_url: Option<&str>,
    api_key: &str,
) -> Result<(Vec<ModelInfo>, Option<String>), String> {
    let Some(url) = models_url(api, vendor, base_url) else {
        return Ok(fallback_models(
            api,
            "Provider does not expose a model list endpoint",
        ));
    };

    let client = build_http_client(None)?;
    let response = models_request(&client, api, &url, api_key)
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
    Ok((parse_models_response(api, &body), None))
}

/// Probe a single model endpoint; returns a human-readable status message.
pub(crate) async fn ping_model(params: &ModelProbeParams<'_>) -> Result<String, String> {
    let client = build_http_client(params.proxy)?;
    let response = match params.api {
        ProviderApi::OpenAi => send_openai_ping(&client, params).await?,
        ProviderApi::Anthropic => send_anthropic_ping(&client, params).await?,
        ProviderApi::Google => send_google_ping(&client, params).await?,
    };

    if response.status().is_success() {
        Ok("Connection successful".to_string())
    } else {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        Err(format!("HTTP {}: {}", status, truncate(&body, 200)))
    }
}

/// Outcome of probing a provider's model-list endpoint for connectivity.
///
/// Network/transport failures (timeout, DNS, client build) are returned as
/// `Err`; HTTP-level results are variants so callers can report them as a
/// structured test result rather than a hard error.
#[derive(Debug)]
pub(crate) enum ConnectionTestOutcome {
    /// Endpoint reachable, HTTP 2xx.
    Ok,
    /// Provider exposes no model-list endpoint to probe.
    NoEndpoint,
    /// Endpoint responded with a non-success HTTP status.
    HttpError { status: u16, body: String },
}

/// Probe a provider's model-list endpoint for connectivity.
pub(crate) async fn test_connection(
    api: ProviderApi,
    vendor: &str,
    base_url: Option<&str>,
    api_key: &str,
) -> Result<ConnectionTestOutcome, String> {
    let Some(url) = models_url(api, vendor, base_url) else {
        return Ok(ConnectionTestOutcome::NoEndpoint);
    };

    let client = build_http_client(None)?;
    let response = models_request(&client, api, &url, api_key)
        .send()
        .await
        .map_err(sanitize_request_error)?;

    if response.status().is_success() {
        Ok(ConnectionTestOutcome::Ok)
    } else {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        Ok(ConnectionTestOutcome::HttpError {
            status,
            body: truncate(&body, 200).to_string(),
        })
    }
}

fn fallback_models(api: ProviderApi, warning: &str) -> (Vec<ModelInfo>, Option<String>) {
    (
        ModelRegistry::builtin_models(api.as_str()),
        Some(warning.to_string()),
    )
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

/// OpenAI and Anthropic share the same minimal ping body shape.
fn ping_body(params: &ModelProbeParams<'_>) -> serde_json::Value {
    serde_json::json!({
        "model": params.model_id,
        "messages": [{ "role": "user", "content": "ping" }],
        "max_tokens": params.max_tokens.unwrap_or(4),
        "temperature": 0.0
    })
}

async fn send_openai_ping(
    client: &reqwest::Client,
    params: &ModelProbeParams<'_>,
) -> Result<reqwest::Response, String> {
    let url = chat_url(ProviderApi::OpenAi, params.vendor, params.base_url)
        .ok_or_else(|| "Unsupported OpenAI endpoint".to_string())?;
    client
        .post(url)
        .bearer_auth(params.api_key)
        .json(&ping_body(params))
        .send()
        .await
        .map_err(sanitize_request_error)
}

async fn send_anthropic_ping(
    client: &reqwest::Client,
    params: &ModelProbeParams<'_>,
) -> Result<reqwest::Response, String> {
    let url = chat_url(ProviderApi::Anthropic, params.vendor, params.base_url)
        .ok_or_else(|| "Unsupported Anthropic endpoint".to_string())?;
    client
        .post(url)
        .header("x-api-key", params.api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&ping_body(params))
        .send()
        .await
        .map_err(sanitize_request_error)
}

async fn send_google_ping(
    client: &reqwest::Client,
    params: &ModelProbeParams<'_>,
) -> Result<reqwest::Response, String> {
    let url = google_generate_url(params.base_url, params.model_id);
    client
        .post(url)
        .query(&[("key", params.api_key)])
        .json(&serde_json::json!({ "contents": [{ "parts": [{ "text": "ping" }] }] }))
        .send()
        .await
        .map_err(sanitize_request_error)
}

fn truncate(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
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
        let (models, warning) = fallback_models(ProviderApi::Anthropic, "no endpoint");

        assert!(!models.is_empty());
        assert_eq!(warning, Some("no endpoint".to_string()));
    }

    #[test]
    fn build_http_client_rejects_invalid_proxy() {
        let result = build_http_client(Some("not a proxy url"));

        assert!(result.is_err());
    }
}
