use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use reqwest::{Client, Url};
use serde_json::Value;

use super::types::{RemoteSearchPage, RemoteSkill, RemoteSkillDetail, SkillRiskReport};

const SKILLHUB_API: &str = "https://api.skillhub.cn";
const CLAWHUB_API: &str = "https://clawhub.ai";

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
    let (skillhub, clawhub) = tokio::join!(
        search_one_registry("skillhub", query, limit),
        search_one_registry("clawhub", query, limit),
    );
    match (skillhub, clawhub) {
        (Ok(mut primary), Ok(secondary)) => {
            primary.items.extend(secondary.items);
            primary.items.truncate(limit.min(50) as usize);
            Ok(primary)
        }
        (Ok(page), Err(_)) | (Err(_), Ok(page)) => Ok(page),
        (Err(skillhub), Err(clawhub)) => {
            bail!("All Skills registries are unavailable. SkillHub: {skillhub}; ClawHub: {clawhub}")
        }
    }
}

async fn search_registry_at(
    provider: &str,
    query: &str,
    limit: u32,
    api_base: &str,
) -> Result<RemoteSearchPage> {
    let response = if provider == "skillhub" {
        let maximum = limit.min(50).to_string();
        let request = if query.trim().is_empty() {
            client()?.get(format!("{api_base}/api/v1/showcase/recommended"))
        } else {
            client()?.get(format!("{api_base}/api/skills")).query(&[
                ("page", "1"),
                ("pageSize", maximum.as_str()),
                ("keyword", query.trim()),
                ("sortBy", "score"),
                ("order", "desc"),
            ])
        };
        send_with_retry(request).await?
    } else if query.trim().is_empty() {
        let maximum = limit.min(50).to_string();
        send_with_retry(
            client()?
                .get(format!("{api_base}/api/v1/skills"))
                .query(&[("limit", maximum.as_str()), ("sort", "trending")]),
        )
        .await?
    } else {
        let maximum = limit.min(50).to_string();
        send_with_retry(client()?.get(format!("{api_base}/api/v1/search")).query(&[
            ("q", query),
            ("limit", maximum.as_str()),
            ("nonSuspiciousOnly", "true"),
        ]))
        .await?
    };
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
    let response = send_with_retry(client()?.get(url)).await?;
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
    let response = send_with_retry(
        client()?
            .get(format!("{api_base}/api/v1/download"))
            .query(&[
                ("slug", slug),
                ("version", version.unwrap_or_default()),
                ("tag", if version.is_some() { "" } else { "latest" }),
            ]),
    )
    .await?;
    write_download(response, destination).await
}

pub async fn download_modelscope_archive(
    reference: &str,
    destination: &Path,
) -> Result<RemoteSkill> {
    let slug = parse_modelscope_reference(reference)?;
    let url = format!("https://www.modelscope.cn/skills/{slug}/archive/zip/master");
    let response = send_with_retry(client()?.get(&url)).await?;
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
        // SkillHub's web host now serves its SPA for arbitrary paths,
        // including the former discovery URL. Its public API host is stable.
        "skillhub" => Ok(SKILLHUB_API.to_string()),
        _ => bail!("Unknown skills registry provider '{provider}'"),
    }
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

async fn send_with_retry(request: reqwest::RequestBuilder) -> Result<reqwest::Response> {
    const MAX_ATTEMPTS: u32 = 3;
    for attempt in 0..MAX_ATTEMPTS {
        let retryable = request
            .try_clone()
            .context("Cannot clone Skills registry request for retry")?;
        match retryable.send().await {
            Ok(response)
                if !is_transient_status(response.status()) || attempt + 1 == MAX_ATTEMPTS =>
            {
                return Ok(response)
            }
            Ok(response) => {
                let delay = retry_delay_seconds(&response, attempt);
                tokio::time::sleep(Duration::from_secs(delay)).await;
            }
            Err(error) if attempt + 1 == MAX_ATTEMPTS => {
                return Err(error).context("Skills registry request failed after retries")
            }
            Err(_) => tokio::time::sleep(Duration::from_secs(1_u64 << attempt)).await,
        }
    }
    unreachable!("the retry loop always returns")
}

fn is_transient_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status == reqwest::StatusCode::SERVICE_UNAVAILABLE
        || status == reqwest::StatusCode::BAD_GATEWAY
        || status == reqwest::StatusCode::GATEWAY_TIMEOUT
}

fn retry_delay_seconds(response: &reqwest::Response, attempt: u32) -> u64 {
    response
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(|seconds| seconds.clamp(1, 5))
        .unwrap_or_else(|| 1_u64 << attempt)
}

fn parse_search_page(provider: &str, payload: Value) -> Result<RemoteSearchPage> {
    let items = payload
        .get("results")
        .or_else(|| payload.get("items"))
        .or_else(|| payload.get("skills"))
        .or_else(|| payload.get("data").and_then(|data| data.get("skills")))
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
        .or_else(|| {
            value
                .get("namespace")
                .and_then(|namespace| namespace.get("handle"))
        })
        .or_else(|| value.get("ownerName"))
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
        .or_else(|| value.get("summary_zh"))
        .or_else(|| value.get("description"))
        .or_else(|| value.get("description_zh"))
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
            .or_else(|| value.get("tags").and_then(|tags| tags.get("latest")))
            .or_else(|| {
                value
                    .get("latestVersion")
                    .and_then(|latest| latest.get("version"))
            })
            .and_then(Value::as_str)
            .map(str::to_string),
        owner,
        source_url: value
            .get("url")
            .or_else(|| value.get("sourceUrl"))
            .or_else(|| value.get("homepage"))
            .or_else(|| value.get("upstream_url"))
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
    for part in skill_identifier_parts(slug)? {
        segments.push(part);
    }
    drop(segments);
    Ok(url)
}

fn skill_identifier_parts(slug: &str) -> Result<Vec<&str>> {
    let parts = slug
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() || parts.len() > 2 || parts.iter().any(|part| *part == "." || *part == "..")
    {
        bail!("Skills registry returned an invalid Skill identifier")
    }
    Ok(parts)
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{parse_search_page, skill_identifier_parts};

    #[test]
    fn parses_skillhub_search_envelope() {
        let page = parse_search_page(
            "skillhub",
            json!({
                "code": 0,
                "data": {
                    "skills": [{
                        "slug": "find-skill-skillhub",
                        "name": "Find Skill",
                        "description_zh": "搜索可用技能",
                        "latestVersion": { "version": "1.2.3" }
                    }]
                }
            }),
        )
        .expect("SkillHub's documented response envelope should be supported");

        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].slug, "find-skill-skillhub");
        assert_eq!(page.items[0].summary, "搜索可用技能");
        assert_eq!(page.items[0].version.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn preserves_owner_scoped_clawhub_identifier() {
        assert_eq!(
            skill_identifier_parts("@openclaw/web-search").unwrap(),
            vec!["@openclaw", "web-search"]
        );
    }

    #[test]
    fn rejects_path_traversal_in_remote_identifier() {
        assert!(skill_identifier_parts("owner/../skill").is_err());
    }
}
