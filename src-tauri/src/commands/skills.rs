use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Emitter, State};

use crate::config;
use crate::contracts::{AppErrorCode, AppErrorPayload};
use crate::services::skills::archive::inspect_archive;
use crate::services::skills::catalog;
use crate::services::skills::exporter::export_skill_dir;
use crate::services::skills::file_provider::{self, SkillFileProvider};
use crate::services::skills::installer;
use crate::services::skills::security;
use crate::services::skills::types::{
    InstallSource, RemoteSearchPage, RemoteSkill, RemoteSkillDetail, SkillActivationView,
    SkillApprovalOperation, SkillApprovalRecord, SkillFilePage, SkillFilePreview, SkillFinding,
    SkillFindingPage, SkillMigrationStatus, SkillRecord, SkillRiskReport, SkillScanOperation,
    SkillScanSummary, SkillSummary,
};
use crate::AppState;

type SkillCommandResult<T> = Result<T, AppErrorPayload>;

#[tauri::command]
pub fn skills_list_installed(state: State<'_, AppState>) -> Result<Vec<SkillRecord>, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    installer::list_installed(&conn).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn skills_get_activation_view(
    state: State<'_, AppState>,
) -> SkillCommandResult<SkillActivationView> {
    let conn = state.db.lock().map_err(skill_command_error)?;
    crate::services::skills::registry::activation_view(&conn, None).map_err(skill_command_error)
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
) -> SkillCommandResult<SkillScanSummary> {
    let conn = state.db.lock().map_err(skill_command_error)?;
    file_provider::get_scan_summary(&conn, &skill_id).map_err(skill_command_error)
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
) -> SkillCommandResult<SkillScanOperation> {
    let conn = state.db.lock().map_err(skill_command_error)?;
    let result = security::scan_and_install_archive(
        &conn,
        &PathBuf::from(path),
        local_source(),
        |scan_id, progress| emit_scan_progress(&app, scan_id, progress),
    )
    .map_err(skill_command_error)?;
    let generation =
        crate::db::repository::SkillSourceRepo::generation(&conn).map_err(skill_command_error)?;
    if let Some(skill) = &result.installed_skill {
        emit_change(&app, "installed", &skill.skill_id, Some(generation));
    }
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
) -> SkillCommandResult<SkillScanOperation> {
    let detail = catalog::remote_detail(&provider, &slug)
        .await
        .map_err(skill_command_error)?;
    let archive = temporary_archive_path().map_err(skill_command_error)?;
    let download =
        catalog::download_registry_archive(&provider, &slug, version.as_deref(), &archive).await;
    if let Err(error) = download {
        let _ = fs::remove_file(&archive);
        return Err(skill_command_error(error));
    }
    let source = registry_source(&detail, version);
    let result =
        install_remote_archive(&app, &state, &archive, source).map_err(skill_command_error)?;
    if let Some(skill) = &result.installed_skill {
        emit_change(
            &app,
            "installed",
            &skill.skill_id,
            current_generation(&state),
        );
    }
    Ok(result)
}

#[tauri::command]
pub async fn skills_import_modelscope(
    app: AppHandle,
    state: State<'_, AppState>,
    reference: String,
) -> SkillCommandResult<SkillScanOperation> {
    let archive = temporary_archive_path().map_err(skill_command_error)?;
    let remote = catalog::download_modelscope_archive(&reference, &archive)
        .await
        .map_err(skill_command_error)?;
    let source = modelscope_source(&remote);
    let result =
        install_remote_archive(&app, &state, &archive, source).map_err(skill_command_error)?;
    if let Some(skill) = &result.installed_skill {
        emit_change(
            &app,
            "installed",
            &skill.skill_id,
            current_generation(&state),
        );
    }
    Ok(result)
}

#[tauri::command]
pub fn skills_list_findings(
    state: State<'_, AppState>,
    scan_id: String,
    severity: Option<String>,
    cursor: Option<String>,
    limit: Option<u32>,
) -> SkillCommandResult<SkillFindingPage> {
    let conn = state.db.lock().map_err(skill_command_error)?;
    crate::db::repository::SkillSecurityRepo::list_findings(
        &conn,
        &scan_id,
        severity.as_deref(),
        cursor.as_deref(),
        limit.unwrap_or(50),
    )
    .map_err(skill_command_error)
}

#[tauri::command]
pub fn skills_get_finding(
    state: State<'_, AppState>,
    scan_id: String,
    finding_id: String,
) -> SkillCommandResult<SkillFinding> {
    let conn = state.db.lock().map_err(skill_command_error)?;
    crate::db::repository::SkillSecurityRepo::get_finding(&conn, &scan_id, &finding_id)
        .map_err(skill_command_error)
}

#[tauri::command]
pub fn skills_list_approvals(
    state: State<'_, AppState>,
    scan_id: String,
) -> SkillCommandResult<Vec<SkillApprovalRecord>> {
    let conn = state.db.lock().map_err(skill_command_error)?;
    crate::db::repository::SkillSecurityRepo::list_approvals(&conn, &scan_id)
        .map_err(skill_command_error)
}

#[tauri::command]
pub fn skills_rescan(
    app: AppHandle,
    state: State<'_, AppState>,
    skill_id: String,
) -> SkillCommandResult<SkillScanOperation> {
    let conn = state.db.lock().map_err(skill_command_error)?;
    installer::list_installed(&conn).map_err(skill_command_error)?;
    let record = crate::db::repository::SkillSourceRepo::find(&conn, &skill_id)
        .map_err(skill_command_error)?
        .ok_or_else(|| {
            skill_command_error("SKILL_SCAN_REQUIRED: Skill source is not registered")
        })?;
    let result = security::scan_existing_source(&conn, &record, |scan_id, progress| {
        emit_scan_progress(&app, scan_id, progress)
    })
    .map_err(skill_command_error)?;
    emit_change(
        &app,
        "scan_completed",
        &skill_id,
        crate::db::repository::SkillSourceRepo::generation(&conn).ok(),
    );
    Ok(result)
}

#[tauri::command]
pub fn skills_cancel_scan(state: State<'_, AppState>, scan_id: String) -> SkillCommandResult<bool> {
    let signalled = security::coordinator().cancel(&scan_id);
    let conn = state.db.lock().map_err(skill_command_error)?;
    security::cancel_scan(&conn, &scan_id)
        .map(|persisted| signalled || persisted)
        .map_err(skill_command_error)
}

#[tauri::command]
pub fn skills_approve_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    scan_id: String,
    actor: String,
    reason: String,
    expires_at: Option<String>,
) -> SkillCommandResult<SkillApprovalOperation> {
    let conn = state.db.lock().map_err(skill_command_error)?;
    let (approval, operation) =
        security::approve_scan(&conn, &scan_id, &actor, &reason, expires_at.as_deref())
            .map_err(skill_command_error)?;
    if let Some(skill) = &operation.installed_skill {
        emit_change(
            &app,
            "scan_approved",
            &skill.skill_id,
            crate::db::repository::SkillSourceRepo::generation(&conn).ok(),
        );
    }
    Ok(SkillApprovalOperation {
        approval,
        operation,
    })
}

#[tauri::command]
pub fn skills_reject_scan(
    state: State<'_, AppState>,
    scan_id: String,
    actor: String,
    reason: String,
) -> SkillCommandResult<SkillApprovalRecord> {
    let conn = state.db.lock().map_err(skill_command_error)?;
    security::reject_scan(&conn, &scan_id, &actor, &reason).map_err(skill_command_error)
}

#[tauri::command]
pub fn skills_revoke_approval(
    app: AppHandle,
    state: State<'_, AppState>,
    approval_id: String,
    skill_id: String,
) -> SkillCommandResult<()> {
    let conn = state.db.lock().map_err(skill_command_error)?;
    let generation =
        security::revoke_approval(&conn, &approval_id, &skill_id).map_err(skill_command_error)?;
    emit_change(&app, "approval_revoked", &skill_id, Some(generation));
    Ok(())
}

#[tauri::command]
pub fn skills_export_scan(
    state: State<'_, AppState>,
    scan_id: String,
    format: String,
    destination: String,
) -> SkillCommandResult<()> {
    let conn = state.db.lock().map_err(skill_command_error)?;
    security::export_scan(&conn, &scan_id, &format, &PathBuf::from(destination))
        .map_err(skill_command_error)
}

#[tauri::command]
pub fn skills_get_scan_privacy_defaults() -> serde_json::Value {
    security::privacy_defaults()
}

#[tauri::command]
pub fn skills_get_migration_status(app: AppHandle) -> SkillCommandResult<SkillMigrationStatus> {
    security::migration::status(&app).map_err(skill_command_error)
}

#[tauri::command]
pub fn skills_retry_migration_scan(app: AppHandle) -> SkillCommandResult<SkillMigrationStatus> {
    security::migration::retry(app).map_err(skill_command_error)
}

#[tauri::command]
pub fn skills_export_installed(
    state: State<'_, AppState>,
    skill_id: String,
    destination: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    installer::list_installed(&conn).map_err(|error| error.to_string())?;
    let skill = crate::db::repository::SkillSourceRepo::find(&conn, &skill_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Skill is not installed".to_string())?;
    if skill.is_external {
        return Err("External Skill sources cannot be exported as managed packages".to_string());
    }
    export_skill_dir(
        &PathBuf::from(skill.installed_path),
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
    skill_id: String,
    enabled: bool,
) -> SkillCommandResult<()> {
    let conn = state.db.lock().map_err(skill_command_error)?;
    let (skill_id, generation) =
        installer::set_enabled(&conn, &skill_id, enabled).map_err(skill_command_error)?;
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
    skill_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    let (skill_id, generation) =
        installer::uninstall(&conn, &skill_id).map_err(|error| error.to_string())?;
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
    app: &AppHandle,
    state: &State<'_, AppState>,
    archive: &PathBuf,
    source: InstallSource,
) -> anyhow::Result<SkillScanOperation> {
    let conn = state
        .db
        .lock()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let result = security::scan_and_install_archive(&conn, archive, source, |scan_id, progress| {
        emit_scan_progress(app, scan_id, progress)
    });
    let _ = fs::remove_file(archive);
    result
}

fn temporary_archive_path() -> anyhow::Result<PathBuf> {
    let directory = config::skills_quarantine_dir()?
        .join("incoming")
        .join(uuid::Uuid::new_v4().to_string());
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

fn emit_scan_progress(app: &AppHandle, scan_id: &str, progress: u8) {
    let _ = app.emit(
        "skills:scan-progress",
        serde_json::json!({
            "scan_id": scan_id,
            "progress": progress,
        }),
    );
}

fn skill_command_error(error: impl ToString) -> AppErrorPayload {
    let message = error.to_string();
    let (code, message_key, retryable) = if message.contains("SKILL_DISABLED")
        || message.to_ascii_lowercase().contains("is disabled")
    {
        (AppErrorCode::SkillDisabled, "errors.skillDisabled", false)
    } else if message.contains("SKILL_APPROVAL_EXPIRED") {
        (
            AppErrorCode::SkillApprovalExpired,
            "errors.skillApprovalExpired",
            false,
        )
    } else if message.contains("SKILL_SCAN_STALE") {
        (AppErrorCode::SkillScanStale, "errors.skillScanStale", true)
    } else if message.contains("SKILL_REVIEW_REQUIRED") {
        (
            AppErrorCode::SkillReviewRequired,
            "errors.skillReviewRequired",
            false,
        )
    } else if message.contains("SKILL_POLICY_BLOCKED") {
        (
            AppErrorCode::SkillPolicyBlocked,
            "errors.skillPolicyBlocked",
            false,
        )
    } else if message.contains("SCAN_CANCELLED") {
        (
            AppErrorCode::SkillScanCancelled,
            "errors.skillScanCancelled",
            true,
        )
    } else if message.contains("SCAN_TIMEOUT") || message.contains("SCAN_QUEUE_TIMEOUT") {
        (
            AppErrorCode::SkillScanTimeout,
            "errors.skillScanTimeout",
            true,
        )
    } else if message.contains("SKILL_QUARANTINE_QUOTA") {
        (
            AppErrorCode::SkillQuarantineQuota,
            "errors.skillQuarantineQuota",
            true,
        )
    } else if message.contains("SKILL_SCAN_REQUIRED") {
        (
            AppErrorCode::SkillScanRequired,
            "errors.skillScanRequired",
            true,
        )
    } else if message.contains("path") || message.contains("archive") {
        (
            AppErrorCode::SkillPathInvalid,
            "errors.skillPathInvalid",
            false,
        )
    } else {
        (AppErrorCode::InternalError, "errors.internal", false)
    };
    AppErrorPayload {
        code,
        message_key: message_key.to_string(),
        params: BTreeMap::new(),
        retryable,
        correlation_id: uuid::Uuid::new_v4().to_string(),
    }
}
