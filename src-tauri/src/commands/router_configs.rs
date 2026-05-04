use crate::crypto;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct RouterConfigView {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_key_masked: String,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRouterConfig {
    pub name: String,
    pub provider: String,
    pub api_key: String,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub config_json: Option<String>,
    pub is_active: Option<bool>,
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
}

#[derive(Debug, Serialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub models: Option<Vec<String>>,
}

#[tauri::command]
pub fn list_router_configs(
    state: State<'_, AppState>,
) -> Result<Vec<RouterConfigView>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare(
            "SELECT id, name, provider, api_key_encrypted, model, base_url, is_active, created_at
             FROM router_configs ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
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

    let mut configs = Vec::new();
    for row in rows {
        configs.push(row.map_err(|e| e.to_string())?);
    }
    Ok(configs)
}

#[tauri::command]
pub fn create_router_config(
    state: State<'_, AppState>,
    config: CreateRouterConfig,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();

    let encrypted_key =
        crypto::encrypt(&config.api_key).map_err(|e| e.to_string())?;

    db.execute(
        "INSERT INTO router_configs (id, name, provider, api_key_encrypted, model, base_url, config_json, is_active)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            id,
            config.name,
            config.provider,
            encrypted_key,
            config.model,
            config.base_url,
            config.config_json,
            config.is_active.unwrap_or(false) as i32,
        ],
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

    let existing: bool = db
        .query_row(
            "SELECT COUNT(*) FROM router_configs WHERE id = ?1",
            [&id],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .map_err(|e| e.to_string())?;

    if !existing {
        return Err(format!("Router config not found: {}", id));
    }

    let mut updates = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref name) = config.name {
        updates.push("name = ?");
        params.push(Box::new(name.clone()));
    }
    if let Some(ref provider) = config.provider {
        updates.push("provider = ?");
        params.push(Box::new(provider.clone()));
    }
    if let Some(ref api_key) = config.api_key {
        let encrypted = crypto::encrypt(api_key).map_err(|e| e.to_string())?;
        updates.push("api_key_encrypted = ?");
        params.push(Box::new(encrypted));
    }
    if let Some(ref model) = config.model {
        updates.push("model = ?");
        params.push(Box::new(model.clone()));
    }
    if let Some(ref base_url) = config.base_url {
        updates.push("base_url = ?");
        params.push(Box::new(base_url.clone()));
    }
    if let Some(ref config_json) = config.config_json {
        updates.push("config_json = ?");
        params.push(Box::new(config_json.clone()));
    }
    if let Some(is_active) = config.is_active {
        updates.push("is_active = ?");
        params.push(Box::new(is_active as i32));
    }

    if updates.is_empty() {
        return Ok(());
    }

    params.push(Box::new(id));
    let sql = format!(
        "UPDATE router_configs SET {} WHERE id = ?",
        updates.join(", ")
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|p| p.as_ref()).collect();

    db.execute(&sql, param_refs.as_slice())
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_router_config(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let affected = db
        .execute("DELETE FROM router_configs WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    if affected == 0 {
        return Err(format!("Router config not found: {}", id));
    }
    Ok(())
}

#[tauri::command]
pub async fn test_router_connection(
    state: State<'_, AppState>,
    id: String,
) -> Result<ConnectionTestResult, String> {
    let (api_key, base_url, provider) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let row = db
            .query_row(
                "SELECT api_key_encrypted, base_url, provider FROM router_configs WHERE id = ?1",
                [&id],
                |row| {
                    Ok((
                        row.get::<_, Option<String>>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .map_err(|e| e.to_string())?;
        row
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
    let mut request = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10));

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

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
