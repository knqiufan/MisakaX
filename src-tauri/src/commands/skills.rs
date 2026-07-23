use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Emitter, State};

use crate::config;
use crate::services::skills::archive::{extract_archive, inspect_archive};
use crate::services::skills::catalog;
use crate::services::skills::exporter::export_skill_dir;
use crate::services::skills::installer;
use crate::services::skills::types::{
    InstallSource, RemoteSearchPage, RemoteSkill, RemoteSkillDetail, SkillDetail,
    SkillInstallResult, SkillRecord, SkillRiskReport,
};
use crate::AppState;

#[tauri::command]
pub fn skills_list_installed(state: State<'_, AppState>) -> Result<Vec<SkillRecord>, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    installer::list_installed(&conn).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn skills_get_detail(state: State<'_, AppState>, slug: String) -> Result<SkillDetail, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    installer::get_detail(&conn, &slug).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn skills_inspect_archive(
    path: String,
) -> Result<crate::services::skills::types::ArchiveInspection, String> {
    inspect_archive(&PathBuf::from(path)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn skills_install_archive(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<SkillInstallResult, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    let result = installer::install_local_archive(&conn, &PathBuf::from(path), local_source())
        .map_err(|error| error.to_string())?;
    emit_change(&app, "installed", &result.skill.slug);
    Ok(result)
}

#[tauri::command]
pub async fn skills_search_remote(
    provider: String,
    query: String,
    limit: Option<u32>,
) -> Result<RemoteSearchPage, String> {
    catalog::search_registry(&provider, &query, limit.unwrap_or(20))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn skills_get_remote_detail(
    provider: String,
    slug: String,
) -> Result<RemoteSkillDetail, String> {
    preview_remote_detail(&provider, &slug)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn skills_install_remote(
    app: AppHandle,
    state: State<'_, AppState>,
    provider: String,
    slug: String,
    version: Option<String>,
) -> Result<SkillInstallResult, String> {
    let detail = catalog::remote_detail(&provider, &slug)
        .await
        .map_err(|error| error.to_string())?;
    let archive = temporary_archive_path().map_err(|error| error.to_string())?;
    let download =
        catalog::download_registry_archive(&provider, &slug, version.as_deref(), &archive).await;
    if let Err(error) = download {
        let _ = fs::remove_file(&archive);
        return Err(error.to_string());
    }
    let source = registry_source(&detail, version);
    let result =
        install_remote_archive(&state, &archive, source).map_err(|error| error.to_string())?;
    emit_change(&app, "installed", &result.skill.slug);
    Ok(result)
}

#[tauri::command]
pub async fn skills_import_modelscope(
    app: AppHandle,
    state: State<'_, AppState>,
    reference: String,
) -> Result<SkillInstallResult, String> {
    let archive = temporary_archive_path().map_err(|error| error.to_string())?;
    let remote = catalog::download_modelscope_archive(&reference, &archive)
        .await
        .map_err(|error| error.to_string())?;
    let source = modelscope_source(&remote);
    let result =
        install_remote_archive(&state, &archive, source).map_err(|error| error.to_string())?;
    emit_change(&app, "installed", &result.skill.slug);
    Ok(result)
}

#[tauri::command]
pub fn skills_export_installed(
    state: State<'_, AppState>,
    slug: String,
    destination: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    let detail = installer::get_detail(&conn, &slug).map_err(|error| error.to_string())?;
    export_skill_dir(
        &PathBuf::from(detail.skill.installed_path),
        &PathBuf::from(destination),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn skills_download_remote(
    provider: String,
    slug: String,
    version: Option<String>,
    destination: String,
) -> Result<(), String> {
    catalog::download_registry_archive(
        &provider,
        &slug,
        version.as_deref(),
        &PathBuf::from(destination),
    )
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn skills_set_enabled(
    app: AppHandle,
    state: State<'_, AppState>,
    slug: String,
    enabled: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    installer::set_enabled(&conn, &slug, enabled).map_err(|error| error.to_string())?;
    emit_change(&app, if enabled { "enabled" } else { "disabled" }, &slug);
    Ok(())
}

#[tauri::command]
pub fn skills_uninstall(
    app: AppHandle,
    state: State<'_, AppState>,
    slug: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    installer::uninstall(&conn, &slug).map_err(|error| error.to_string())?;
    emit_change(&app, "uninstalled", &slug);
    Ok(())
}

fn local_source() -> InstallSource {
    InstallSource {
        kind: "local".to_string(),
        reference: None,
        url: None,
        version: None,
        remote_risk: SkillRiskReport::default(),
    }
}

fn registry_source(detail: &RemoteSkillDetail, version: Option<String>) -> InstallSource {
    InstallSource {
        kind: detail.skill.provider.clone(),
        reference: Some(detail.skill.slug.clone()),
        url: Some(detail.skill.source_url.clone()),
        version: version.or_else(|| detail.skill.version.clone()),
        remote_risk: detail.risk.clone(),
    }
}

fn modelscope_source(remote: &RemoteSkill) -> InstallSource {
    InstallSource {
        kind: "modelscope".to_string(),
        reference: Some(remote.slug.clone()),
        url: Some(remote.source_url.clone()),
        version: remote.version.clone(),
        remote_risk: SkillRiskReport::default(),
    }
}

fn install_remote_archive(
    state: &State<'_, AppState>,
    archive: &PathBuf,
    source: InstallSource,
) -> anyhow::Result<SkillInstallResult> {
    let conn = state
        .db
        .lock()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let result = installer::install_local_archive(&conn, archive, source);
    let _ = fs::remove_file(archive);
    result
}

fn temporary_archive_path() -> anyhow::Result<PathBuf> {
    let directory = config::skills_staging_dir()?.join("downloads");
    fs::create_dir_all(&directory)?;
    Ok(directory.join(format!("{}.zip", uuid::Uuid::new_v4())))
}

async fn preview_remote_detail(provider: &str, slug: &str) -> anyhow::Result<RemoteSkillDetail> {
    let mut detail = catalog::remote_detail(provider, slug).await?;
    let archive = temporary_archive_path()?;
    catalog::download_registry_archive(provider, slug, detail.skill.version.as_deref(), &archive)
        .await?;
    let preview = config::skills_preview_cache_dir()?.join(uuid::Uuid::new_v4().to_string());
    let inspection = extract_archive(&archive, &preview)?;
    detail.manifest = Some(inspection.manifest);
    detail.files = inspection.files;
    detail.skill_markdown = Some(fs::read_to_string(preview.join("SKILL.md"))?);
    detail.risk = merge_preview_risk(detail.risk, inspection.risk);
    let _ = fs::remove_file(archive);
    let _ = fs::remove_dir_all(preview);
    Ok(detail)
}

fn merge_preview_risk(mut remote: SkillRiskReport, preview: SkillRiskReport) -> SkillRiskReport {
    remote.has_scripts |= preview.has_scripts;
    remote.has_binary_files |= preview.has_binary_files;
    remote.has_allowed_tools |= preview.has_allowed_tools;
    remote.notes.extend(preview.notes);
    remote
}

fn emit_change(app: &AppHandle, action: &str, slug: &str) {
    let _ = app.emit(
        "skills:changed",
        serde_json::json!({ "action": action, "slug": slug }),
    );
}
