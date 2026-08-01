use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Emitter, State};

use crate::config;
use crate::services::skills::archive::inspect_archive;
use crate::services::skills::catalog;
use crate::services::skills::exporter::export_skill_dir;
use crate::services::skills::file_provider::{self, SkillFileProvider};
use crate::services::skills::installer;
use crate::services::skills::types::{
    InstallSource, RemoteSearchPage, RemoteSkill, RemoteSkillDetail, SkillActivationView,
    SkillDetail, SkillFilePage, SkillFilePreview, SkillInstallResult, SkillRecord, SkillRiskReport,
    SkillScanSummary, SkillSummary,
};
use crate::AppState;

#[tauri::command]
pub fn skills_list_installed(state: State<'_, AppState>) -> Result<Vec<SkillRecord>, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    installer::list_installed(&conn).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn skills_get_activation_view(
    state: State<'_, AppState>,
) -> Result<SkillActivationView, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    crate::services::skills::registry::activation_view(&conn, None)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn skills_get_detail(state: State<'_, AppState>, slug: String) -> Result<SkillDetail, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    installer::get_detail(&conn, &slug).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn skills_get_summary(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<SkillSummary, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    file_provider::get_summary(&conn, &skill_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn skills_list_files(
    state: State<'_, AppState>,
    skill_id: String,
    parent: Option<String>,
    cursor: Option<String>,
    limit: Option<u32>,
) -> Result<SkillFilePage, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    let (provider, _) =
        SkillFileProvider::from_registered(&conn, &skill_id).map_err(|error| error.to_string())?;
    let generation = crate::db::repository::SkillSourceRepo::generation(&conn)
        .map_err(|error| error.to_string())?;
    provider
        .list_files(generation, parent.as_deref(), cursor.as_deref(), limit)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn skills_read_file(
    state: State<'_, AppState>,
    skill_id: String,
    path: String,
    offset: Option<u64>,
    limit: Option<u32>,
) -> Result<SkillFilePreview, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    let (provider, _) =
        SkillFileProvider::from_registered(&conn, &skill_id).map_err(|error| error.to_string())?;
    let generation = crate::db::repository::SkillSourceRepo::generation(&conn)
        .map_err(|error| error.to_string())?;
    provider
        .read_file(generation, &path, offset.unwrap_or_default(), limit)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn skills_get_scan_summary(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<SkillScanSummary, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    file_provider::get_scan_summary(&conn, &skill_id).map_err(|error| error.to_string())
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
    let generation = crate::db::repository::SkillSourceRepo::generation(&conn)
        .map_err(|error| error.to_string())?;
    emit_change(&app, "installed", &result.skill.skill_id, Some(generation));
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
    catalog::remote_detail(&provider, &slug)
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
    emit_change(
        &app,
        "installed",
        &result.skill.skill_id,
        current_generation(&state),
    );
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
    emit_change(
        &app,
        "installed",
        &result.skill.skill_id,
        current_generation(&state),
    );
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
    identifier: String,
    enabled: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    let (skill_id, generation) =
        installer::set_enabled(&conn, &identifier, enabled).map_err(|error| error.to_string())?;
    emit_change(
        &app,
        if enabled { "enabled" } else { "disabled" },
        &skill_id,
        Some(generation),
    );
    Ok(())
}

#[tauri::command]
pub fn skills_uninstall(
    app: AppHandle,
    state: State<'_, AppState>,
    identifier: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    let (skill_id, generation) =
        installer::uninstall(&conn, &identifier).map_err(|error| error.to_string())?;
    emit_change(&app, "uninstalled", &skill_id, Some(generation));
    Ok(())
}

fn current_generation(state: &State<'_, AppState>) -> Option<u64> {
    let conn = state.db.lock().ok()?;
    crate::db::repository::SkillSourceRepo::generation(&conn).ok()
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

fn emit_change(app: &AppHandle, action: &str, skill_id: &str, generation: Option<u64>) {
    let _ = app.emit(
        "skills:changed",
        serde_json::json!({
            "action": action,
            "skill_id": skill_id,
            "generation": generation,
        }),
    );
}
