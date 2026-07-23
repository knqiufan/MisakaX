use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use reqwest::{Client, Url};
use serde::Deserialize;
use serde_json::Value;

use super::types::{RemoteSearchPage, RemoteSkill, RemoteSkillDetail, SkillRiskReport};

const SKILLHUB_SITE: &str = "https://skillhub.cn";
const CLAWHUB_API: &str = "https://clawhub.ai";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveryDocument {
    api_base: String,
}

pub async fn search_registry(provider: &str, query: &str, limit: u32) -> Result<RemoteSearchPage> {
    if provider == "all" {
        return search_preferred_registry(query, limit).await;
    }
    search_one_registry(provider, query, limit).await
}

async fn search_one_registry(provider: &str, query: &str, limit: u32) -> Result<RemoteSearchPage> {
    let api_base = resolve_registry(provider).await?;
    search_registry_at(provider, query, limit, &api_base).await
}

async fn search_preferred_registry(query: &str, limit: u32) -> Result<RemoteSearchPage> {
    match search_one_registry("skillhub", query, limit).await {
        Ok(page) if !page.items.is_empty() => Ok(page),
        Ok(_) | Err(_) => search_one_registry("clawhub", query, limit).await,
    }
}

async fn search_registry_at(
    provider: &str,
    query: &str,
    limit: u32,
    api_base: &str,
) -> Result<RemoteSearchPage> {
    let response = client()?
        .get(format!("{api_base}/api/v1/search"))
        .query(&[
            ("q", query),
            ("limit", &limit.min(50).to_string()),
            ("nonSuspiciousOnly", "true"),
        ])
        .send()
        .await
        .context("Skills registry search request failed")?;
    ensure_success(response)
        .await?
        .json::<Value>()
        .await
        .context("Skills registry search response is invalid")
        .and_then(|payload| parse_search_page(provider, payload))
}

pub async fn remote_detail(provider: &str, slug: &str) -> Result<RemoteSkillDetail> {
    let api_base = resolve_registry(provider).await?;
    let url = skill_url(&api_base, slug)?;
    let response = client()?
        .get(url)
        .send()
        .await
        .context("Skills registry detail request failed")?;
    let payload = ensure_success(response)
        .await?
        .json::<Value>()
        .await
        .context("Skills registry detail response is invalid")?;
    parse_remote_detail(provider, slug, payload)
}

pub async fn download_registry_archive(
    provider: &str,
    slug: &str,
    version: Option<&str>,
    destination: &Path,
) -> Result<()> {
    let api_base = resolve_registry(provider).await?;
    let response = client()?
        .get(format!("{api_base}/api/v1/download"))
        .query(&[
            ("slug", slug),
            ("version", version.unwrap_or_default()),
            ("tag", if version.is_some() { "" } else { "latest" }),
        ])
        .send()
        .await
        .context("Skills registry download request failed")?;
    write_download(response, destination).await
}

pub async fn download_modelscope_archive(
    reference: &str,
    destination: &Path,
) -> Result<RemoteSkill> {
    let slug = parse_modelscope_reference(reference)?;
    let url = format!("https://www.modelscope.cn/skills/{slug}/archive/zip/master");
    let response = client()?
        .get(&url)
        .send()
        .await
        .context("ModelScope download request failed")?;
    write_download(response, destination).await?;
    Ok(RemoteSkill {
        provider: "modelscope".to_string(),
        slug: slug.clone(),
        display_name: slug.clone(),
        summary: "Imported from ModelScope Skills.".to_string(),
        version: Some("master".to_string()),
        owner: slug.split('/').next().map(str::to_string),
        source_url: format!("https://www.modelscope.cn/skills/{slug}"),
        topics: vec!["ModelScope".to_string()],
        suspicious: false,
    })
}

async fn resolve_registry(provider: &str) -> Result<String> {
    match provider {
        "clawhub" => Ok(CLAWHUB_API.to_string()),
        "skillhub" => discover_registry(SKILLHUB_SITE).await,
        _ => bail!("Unknown skills registry provider '{provider}'"),
    }
}

async fn discover_registry(site: &str) -> Result<String> {
    let response = client()?
        .get(format!("{site}/.well-known/clawhub.json"))
        .send()
        .await
        .context("Skills registry discovery failed")?;
    let document = ensure_success(response)
        .await?
        .json::<DiscoveryDocument>()
        .await
        .context("Skills registry discovery response is invalid")?;
    if document.api_base.trim().is_empty() {
        bail!("Skills registry discovery returned an empty API base")
    }
    Ok(document.api_base.trim_end_matches('/').to_string())
}

fn client() -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("MisakaX-Skills/0.1")
        .build()
        .context("Cannot create skills registry HTTP client")
}

async fn ensure_success(response: reqwest::Response) -> Result<reqwest::Response> {
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status();
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let body = response.text().await.unwrap_or_default();
    let retry_hint = retry_after
        .map(|value| format!(" Retry after {value}."))
        .unwrap_or_default();
    bail!("Skills registry returned HTTP {status}: {body}{retry_hint}")
}

fn parse_search_page(provider: &str, payload: Value) -> Result<RemoteSearchPage> {
    let items = payload
        .get("results")
        .or_else(|| payload.get("items"))
        .and_then(Value::as_array)
        .context("Skills registry response has no search results")?;
    let skills = items
        .iter()
        .filter_map(|item| parse_remote_skill(provider, item).ok())
        .collect();
    Ok(RemoteSearchPage {
        items: skills,
        next_cursor: payload
            .get("nextCursor")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn parse_remote_detail(
    provider: &str,
    requested_slug: &str,
    payload: Value,
) -> Result<RemoteSkillDetail> {
    let skill_value = payload.get("skill").unwrap_or(&payload);
    let mut skill = parse_remote_skill(provider, skill_value)?;
    if skill.slug.is_empty() {
        skill.slug = requested_slug.to_string();
    }
    let null = Value::Null;
    let latest = payload.get("latestVersion").unwrap_or(&null);
    skill.version = latest
        .get("version")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or(skill.version);
    let moderation = payload.get("moderation").unwrap_or(&null);
    let risk = registry_risk(moderation, skill_value);
    Ok(RemoteSkillDetail {
        skill,
        changelog: latest
            .get("changelog")
            .and_then(Value::as_str)
            .map(str::to_string),
        license: skill_value
            .get("license")
            .and_then(Value::as_str)
            .map(str::to_string),
        compatibility: skill_value
            .get("compatibility")
            .and_then(Value::as_str)
            .map(str::to_string),
        risk,
        manifest: None,
        files: Vec::new(),
        skill_markdown: None,
    })
}

fn parse_remote_skill(provider: &str, value: &Value) -> Result<RemoteSkill> {
    let slug = value
        .get("slug")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let owner = value
        .get("ownerHandle")
        .or_else(|| value.get("owner").and_then(|owner| owner.get("handle")))
        .and_then(Value::as_str)
        .map(str::to_string);
    if slug.is_empty() {
        bail!("Skills registry entry has no slug")
    }
    let display_name = value
        .get("displayName")
        .or_else(|| value.get("name"))
        .and_then(Value::as_str)
        .unwrap_or(&slug)
        .to_string();
    let summary = value
        .get("summary")
        .or_else(|| value.get("description"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    Ok(RemoteSkill {
        provider: provider.to_string(),
        slug: slug.clone(),
        display_name,
        summary,
        version: value
            .get("version")
            .and_then(Value::as_str)
            .map(str::to_string),
        owner,
        source_url: value
            .get("url")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("https://{provider}/skills/{slug}")),
        topics: value
            .get("topics")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        suspicious: value
            .get("moderation")
            .and_then(|moderation| moderation.get("isSuspicious"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn registry_risk(moderation: &Value, skill: &Value) -> SkillRiskReport {
    let suspicious = moderation
        .get("isSuspicious")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let blocked = moderation
        .get("isMalwareBlocked")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut notes = Vec::new();
    if suspicious || blocked {
        notes.push("The registry marked this skill for security review.".to_string());
    }
    SkillRiskReport {
        remote_scan_status: moderation
            .get("verdict")
            .and_then(Value::as_str)
            .map(str::to_string),
        has_scripts: skill
            .get("hasScripts")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        has_binary_files: false,
        has_allowed_tools: false,
        notes,
    }
}

fn skill_url(api_base: &str, slug: &str) -> Result<Url> {
    let mut url = Url::parse(api_base).context("Skills registry API base is invalid")?;
    let mut segments = url
        .path_segments_mut()
        .map_err(|_| anyhow::anyhow!("Skills registry API base cannot accept a path"))?;
    segments.push("api");
    segments.push("v1");
    segments.push("skills");
    segments.push(slug);
    drop(segments);
    Ok(url)
}

async fn write_download(response: reqwest::Response, destination: &Path) -> Result<()> {
    let response = ensure_success(response).await?;
    let content_length = response.content_length().unwrap_or(0);
    if content_length > 10 * 1024 * 1024 {
        bail!("Remote skill archive exceeds the 10 MiB size limit")
    }
    let bytes = response
        .bytes()
        .await
        .context("Cannot read remote skill archive")?;
    if bytes.len() > 10 * 1024 * 1024 {
        bail!("Remote skill archive exceeds the 10 MiB size limit")
    }
    std::fs::write(destination, bytes).context("Cannot save remote skill archive")
}

fn parse_modelscope_reference(reference: &str) -> Result<String> {
    let without_host = reference
        .trim()
        .trim_start_matches("https://modelscope.cn/skills/")
        .trim_start_matches("https://www.modelscope.cn/skills/")
        .trim_start_matches('@');
    let parts = without_host
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() != 2
        || parts
            .iter()
            .any(|part| part.contains("..") || part.contains('\\'))
    {
        bail!("ModelScope skill reference must be owner/skill")
    }
    Ok(format!("{}/{}", parts[0], parts[1]))
}
