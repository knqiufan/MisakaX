use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tauri::State;

use crate::config;
use crate::db::models::{
    ExportData, ExportProfileMetadata, ExportSession, ExportUsageEvent, ImportResult,
    MessageSearchResult, NewUsageEvent, Session, UsageEvent,
};
use crate::db::repository::{
    ArtifactRepo, MessageBlockRepo, MessageRepo, ProfileRepo, SessionRepo, SettingsRepo, UsageRepo,
    WorkspaceRepo,
};
use crate::services::artifacts::{ArtifactService, ContentSafetyPolicy, RetentionState};
use crate::services::usage::backfill::backfill_legacy_usage;
use crate::AppState;

pub const WORKSPACE_KIND_DEFAULT: &str = "default";
pub const WORKSPACE_KIND_CUSTOM: &str = "custom";
pub const EXPORT_DATA_VERSION: u32 = 2;
const INSTALLATION_ID_SETTING_KEY: &str = "installation.id";

/// Resolved working directory + kind for session create/update.
pub struct ResolvedWorkspace {
    pub path: String,
    pub kind: String,
}

/// Resolve an explicit path or fall back to the app-managed default workspace.
pub fn resolve_workspace(dir: Option<&str>) -> Result<ResolvedWorkspace, String> {
    match dir {
        None | Some("") => {
            let path = ensure_default_workspace_path()?;
            Ok(ResolvedWorkspace {
                path,
                kind: WORKSPACE_KIND_DEFAULT.to_string(),
            })
        }
        Some(path) => {
            let canonical = validate_existing_dir(path)?;
            Ok(ResolvedWorkspace {
                path: canonical,
                kind: WORKSPACE_KIND_CUSTOM.to_string(),
            })
        }
    }
}

fn ensure_default_workspace_path() -> Result<String, String> {
    let path = config::default_workspace_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

fn validate_existing_dir(path: &str) -> Result<String, String> {
    let p = Path::new(path);
    if !p.exists() || !p.is_dir() {
        return Err(format!(
            "Working directory does not exist or is not a directory: {}",
            path
        ));
    }
    match std::fs::canonicalize(p) {
        Ok(canonical) => Ok(strip_windows_unc_prefix(canonical)),
        Err(_) => Ok(path.to_string()),
    }
}

fn strip_windows_unc_prefix(path: PathBuf) -> String {
    let s = path.to_string_lossy().to_string();
    // Windows canonicalize yields \\?\C:\...
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        s
    }
}

// ─── create_session Command ──────────────────────────────────────────

#[tauri::command]
pub fn create_session(
    state: State<'_, AppState>,
    title: Option<String>,
    model: Option<String>,
    working_directory: Option<String>,
) -> Result<Session, String> {
    let workspace = resolve_workspace(working_directory.as_deref())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();

    let session = SessionRepo::create_with_workspace(
        &conn,
        &id,
        title.as_deref(),
        model.as_deref(),
        Some(workspace.path.as_str()),
        &workspace.kind,
    )
    .map_err(|e| e.to_string())?;

    WorkspaceRepo::restore_workspace(&conn, &workspace.path).map_err(|e| e.to_string())?;

    record_directory_usage_internal(&conn, &workspace.path);

    Ok(session)
}

// ─── list_sessions Command ───────────────────────────────────────────

#[tauri::command]
pub fn list_sessions(
    state: State<'_, AppState>,
    status: Option<String>,
) -> Result<Vec<Session>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::list(&conn, status.as_deref()).map_err(|e| e.to_string())
}

// ─── update_session Command ──────────────────────────────────────────

#[tauri::command]
pub fn update_session(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    model: Option<String>,
    pinned: Option<bool>,
    status: Option<String>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::update(
        &conn,
        &id,
        title.as_deref(),
        model.as_deref(),
        pinned,
        status.as_deref(),
    )
    .map_err(|e| e.to_string())
}

// ─── delete_session Command ──────────────────────────────────────────

#[tauri::command]
pub async fn delete_session(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let _binding_guard = state.workspace_terminal_guard.lock().await;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let artifact_service = ArtifactService::new(
        config::artifacts_dir().map_err(|e| e.to_string())?,
        ContentSafetyPolicy::default(),
    )
    .map_err(|e| e.to_string())?;
    artifact_service
        .expire_session(&conn, &id)
        .map_err(|e| e.to_string())?;
    SessionRepo::delete(&conn, &id).map_err(|e| e.to_string())?;
    drop(conn);
    state.terminal_manager.kill_chat_session(&id);
    state.workspace_context.unbind_session(&id);
    Ok(())
}

// ─── search_sessions Command ─────────────────────────────────────────

#[tauri::command]
pub fn search_sessions(state: State<'_, AppState>, query: String) -> Result<Vec<Session>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::search(&conn, &query).map_err(|e| e.to_string())
}

// ─── update_session_working_dir Command ──────────────────────────────

#[tauri::command]
pub async fn update_session_working_dir(
    state: State<'_, AppState>,
    session_id: String,
    working_directory: Option<String>,
) -> Result<(), String> {
    let _binding_guard = state.workspace_terminal_guard.lock().await;
    let workspace = resolve_workspace(working_directory.as_deref())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    SessionRepo::update_working_directory_with_kind(
        &conn,
        &session_id,
        Some(workspace.path.as_str()),
        &workspace.kind,
    )
    .map_err(|e| e.to_string())?;

    record_directory_usage_internal(&conn, &workspace.path);
    drop(conn);
    state.workspace_context.unbind_session(&session_id);
    Ok(())
}

// ─── get_session Command ─────────────────────────────────────────────

#[tauri::command]
pub fn get_session(state: State<'_, AppState>, session_id: String) -> Result<Session, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::find_by_id(&conn, &session_id).map_err(|e| e.to_string())
}

// ─── pin_session Command ─────────────────────────────────────────────

#[tauri::command]
pub fn pin_session(state: State<'_, AppState>, id: String, pinned: bool) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::pin_session(&conn, &id, pinned).map_err(|e| e.to_string())
}

// ─── archive_session Command ────────────────────────────────────────

#[tauri::command]
pub fn archive_session(
    state: State<'_, AppState>,
    id: String,
    archived: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if archived {
        SessionRepo::archive_session(&conn, &id)
    } else {
        SessionRepo::unarchive_session(&conn, &id)
    }
    .map_err(|e| e.to_string())
}

// ─── set_session_group Command ──────────────────────────────────────

#[tauri::command]
pub fn set_session_group(
    state: State<'_, AppState>,
    id: String,
    group: Option<String>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::set_group(&conn, &id, group.as_deref()).map_err(|e| e.to_string())
}

// ─── list_session_groups Command ────────────────────────────────────

#[tauri::command]
pub fn list_session_groups(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SessionRepo::list_groups(&conn).map_err(|e| e.to_string())
}

// ─── search_messages Command (FTS5) ─────────────────────────────────

#[tauri::command]
pub fn search_messages(
    state: State<'_, AppState>,
    query: String,
    session_id: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<MessageSearchResult>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    MessageRepo::search_fts(&conn, &query, session_id.as_deref(), limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

// ─── export_sessions Command ────────────────────────────────────────

#[tauri::command]
pub fn export_sessions(
    state: State<'_, AppState>,
    session_ids: Vec<String>,
    file_path: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    export_sessions_to_file(&conn, &session_ids, Path::new(&file_path))
}

pub fn export_sessions_to_file(
    conn: &rusqlite::Connection,
    session_ids: &[String],
    file_path: &Path,
) -> Result<(), String> {
    let mut export_sessions = Vec::with_capacity(session_ids.len());
    let mut usage_events = Vec::new();
    for sid in session_ids {
        let session = SessionRepo::find_by_id(conn, sid).map_err(|e| e.to_string())?;
        let mut messages =
            MessageRepo::find_recent(conn, sid, u32::MAX).map_err(|e| e.to_string())?;
        for message in &mut messages {
            message.blocks =
                MessageBlockRepo::find_by_message(conn, &message.id).map_err(|e| e.to_string())?;
        }
        export_sessions.push(ExportSession { session, messages });
        usage_events.extend(UsageRepo::list_for_session(conn, sid).map_err(|e| e.to_string())?);
    }

    let installation_id = installation_id(conn)?;
    let profile = ProfileRepo::ensure_default(conn).map_err(|error| error.to_string())?;

    let export_data = ExportData {
        version: EXPORT_DATA_VERSION,
        exported_at: chrono::Utc::now().to_rfc3339(),
        app: "MisakaX".to_string(),
        sessions: export_sessions,
        artifact_manifest: ArtifactRepo::find_by_sessions(conn, session_ids)
            .map_err(|e| e.to_string())?,
        profile: Some(ExportProfileMetadata {
            display_name: profile.display_name,
            timezone_mode: profile.timezone_mode,
            timezone_id: profile.timezone_id,
            week_start: profile.week_start,
        }),
        usage_events: usage_events
            .into_iter()
            .map(|event| export_usage_event(event, &installation_id))
            .collect(),
    };

    let json = serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string())?;
    std::fs::write(file_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

// ─── import_sessions Command ────────────────────────────────────────

#[tauri::command]
pub fn import_sessions(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let result = import_sessions_from_file(&conn, Path::new(&file_path))?;
    // Normalize any imported sessions that still lack a working directory.
    if let Ok(default_dir) = ensure_default_workspace_path() {
        let _ = SessionRepo::backfill_null_workspaces(&conn, &default_dir);
        record_directory_usage_internal(&conn, &default_dir);
    }
    Ok(result)
}

pub fn import_sessions_from_file(
    conn: &rusqlite::Connection,
    file_path: &Path,
) -> Result<ImportResult, String> {
    let json = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let data: ExportData = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    if data.version == 0 || data.version > EXPORT_DATA_VERSION {
        return Err(format!("Unsupported export version {}", data.version));
    }
    let mut imported = 0u32;
    let mut skipped = 0u32;
    let mut errors: Vec<String> = Vec::new();

    for es in &data.sessions {
        let exists = SessionRepo::find_by_id(conn, &es.session.id).is_ok();
        if exists {
            skipped += 1;
            continue;
        }

        if let Err(e) = import_single_session(conn, es) {
            errors.push(format!("Session {}: {}", es.session.id, e));
            continue;
        }
        imported += 1;
    }

    for mut artifact in data.artifact_manifest {
        if artifact.owner_session_id.is_empty()
            || ArtifactRepo::find_by_id(conn, &artifact.artifact_id).is_ok()
        {
            continue;
        }
        // A session export intentionally contains a manifest, not binary bytes.
        // Imported records stay inspectable but cannot be read until a future
        // import flow transfers verified content into the artifact store.
        artifact.retention_state = RetentionState::Expired;
        let _ = ArtifactRepo::insert(conn, &artifact);
    }

    let profile = ProfileRepo::ensure_default(conn).map_err(|error| error.to_string())?;
    let usage_count = data.usage_events.len();
    let usage_batch = data
        .usage_events
        .into_iter()
        .map(|event| import_usage_event(conn, &profile.profile_id, event, data.version))
        .collect::<Result<Vec<_>, _>>()?;
    let usage_inserted = UsageRepo::insert_batch_idempotent(conn, &usage_batch)
        .map_err(|error| error.to_string())?
        .len();
    let legacy_backfill = backfill_legacy_usage(conn).map_err(|error| error.to_string())?;
    let legacy_inserted =
        legacy_backfill.message_events_inserted + legacy_backfill.residual_events_inserted;
    let total_usage_inserted = usage_inserted.saturating_add(legacy_inserted as usize);

    Ok(ImportResult {
        imported_count: imported,
        skipped_count: skipped,
        errors,
        usage_imported_count: total_usage_inserted as u32,
        usage_skipped_count: usage_count.saturating_sub(usage_inserted) as u32,
    })
}

fn installation_id(conn: &rusqlite::Connection) -> Result<String, String> {
    if let Some(installation_id) =
        SettingsRepo::get(conn, INSTALLATION_ID_SETTING_KEY).map_err(|error| error.to_string())?
    {
        if uuid::Uuid::parse_str(&installation_id).is_ok() {
            return Ok(installation_id);
        }
    }
    let installation_id = uuid::Uuid::new_v4().to_string();
    SettingsRepo::set(conn, INSTALLATION_ID_SETTING_KEY, &installation_id)
        .map_err(|error| error.to_string())?;
    Ok(installation_id)
}

fn export_usage_event(event: UsageEvent, local_installation_id: &str) -> ExportUsageEvent {
    ExportUsageEvent {
        source_installation_id: event
            .source_installation_id
            .unwrap_or_else(|| local_installation_id.to_string()),
        source_event_id: event.source_event_id.unwrap_or(event.event_id),
        operation_key: event.operation_key,
        operation_kind: event.operation_kind,
        session_id: event.session_id,
        message_id: event.message_id,
        provider_config_id: event.provider_config_id,
        provider_id: event.provider_id,
        vendor_id: event.vendor_id,
        selected_model_id: event.selected_model_id,
        effective_model_id: event.effective_model_id,
        model_display_name: event.model_display_name,
        input_tokens: event.input_tokens,
        output_tokens: event.output_tokens,
        total_tokens: event.total_tokens,
        cache_read_tokens: event.cache_read_tokens,
        cache_creation_tokens: event.cache_creation_tokens,
        reasoning_tokens: event.reasoning_tokens,
        measurement_source: event.measurement_source,
        estimator_id: event.estimator_id,
        estimator_version: event.estimator_version,
        outcome: event.outcome,
        counts_toward_totals: event.counts_toward_totals,
        counts_toward_activity: event.counts_toward_activity,
        counts_toward_trend: event.counts_toward_trend,
        occurred_at_utc: event.occurred_at_utc,
        local_date: event.local_date,
        timezone_id: event.timezone_id,
        utc_offset_minutes: event.utc_offset_minutes,
    }
}

fn import_usage_event(
    conn: &rusqlite::Connection,
    profile_id: &str,
    event: ExportUsageEvent,
    export_version: u32,
) -> Result<NewUsageEvent, String> {
    if event.source_installation_id.trim().is_empty()
        || event.source_event_id.trim().is_empty()
        || event.source_installation_id.len() > 128
        || event.source_event_id.len() > 256
    {
        return Err("Invalid usage event origin identifiers".into());
    }
    let session_id = existing_reference(conn, "sessions", "id", event.session_id.as_deref())?;
    let message_id = existing_reference(conn, "messages", "id", event.message_id.as_deref())?;
    let operation_key = scoped_import_key(
        "operation",
        &event.source_installation_id,
        &event.operation_key,
    );
    let measurement_key = scoped_import_key(
        "measurement",
        &event.source_installation_id,
        &event.source_event_id,
    );
    let metadata_json = serde_json::json!({
        "imported": true,
        "source_export_version": export_version,
        "source_measurement_quality": event.measurement_source.as_str(),
        "legacy_migrated": event.measurement_source == crate::services::usage::MeasurementSource::LegacyMigrated,
    })
    .to_string();
    Ok(NewUsageEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        profile_id: profile_id.to_string(),
        operation_key,
        measurement_key,
        operation_kind: event.operation_kind,
        session_id,
        message_id,
        provider_config_id: event.provider_config_id,
        provider_id: event.provider_id,
        vendor_id: event.vendor_id,
        selected_model_id: event.selected_model_id,
        effective_model_id: event.effective_model_id,
        model_display_name: event.model_display_name,
        input_tokens: event.input_tokens,
        output_tokens: event.output_tokens,
        total_tokens: event.total_tokens,
        cache_read_tokens: event.cache_read_tokens,
        cache_creation_tokens: event.cache_creation_tokens,
        reasoning_tokens: event.reasoning_tokens,
        measurement_source: event.measurement_source,
        estimator_id: event.estimator_id,
        estimator_version: event.estimator_version,
        outcome: event.outcome,
        counts_toward_totals: event.counts_toward_totals,
        counts_toward_activity: event.counts_toward_activity,
        counts_toward_trend: event.counts_toward_trend,
        occurred_at_utc: event.occurred_at_utc,
        local_date: event.local_date,
        timezone_id: event.timezone_id,
        utc_offset_minutes: event.utc_offset_minutes,
        metadata_json,
        source_installation_id: Some(event.source_installation_id),
        source_event_id: Some(event.source_event_id),
    })
}

fn scoped_import_key(namespace: &str, first: &str, second: &str) -> String {
    let mut digest = Sha256::new();
    for part in [namespace, first, second] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part.as_bytes());
    }
    format!("import:{namespace}:{:x}", digest.finalize())
}

fn existing_reference(
    conn: &rusqlite::Connection,
    table: &str,
    column: &str,
    value: Option<&str>,
) -> Result<Option<String>, String> {
    let Some(value) = value else { return Ok(None) };
    let exists = conn
        .query_row(
            &format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE {column} = ?1)"),
            [value],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| error.to_string())?;
    Ok(exists.then(|| value.to_string()))
}

/// Idempotent backfill for sessions with NULL/empty working directories.
#[tauri::command]
pub fn backfill_session_workspaces(state: State<'_, AppState>) -> Result<usize, String> {
    let default_dir = ensure_default_workspace_path()?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let count =
        SessionRepo::backfill_null_workspaces(&conn, &default_dir).map_err(|e| e.to_string())?;
    if count > 0 {
        record_directory_usage_internal(&conn, &default_dir);
    }
    Ok(count)
}

// ─── 内部辅助 ────────────────────────────────────────────────────────

fn import_single_session(conn: &rusqlite::Connection, es: &ExportSession) -> Result<(), String> {
    let mut session = es.session.clone();
    if session
        .working_directory
        .as_deref()
        .unwrap_or("")
        .is_empty()
    {
        let default_dir = ensure_default_workspace_path()?;
        session.working_directory = Some(default_dir.clone());
        session.workspace_kind = WORKSPACE_KIND_DEFAULT.to_string();
        let name = Path::new(&default_dir)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workspace")
            .to_string();
        session.project_name = Some(name);
    } else if session.workspace_kind.is_empty() {
        session.workspace_kind = WORKSPACE_KIND_CUSTOM.to_string();
    }

    SessionRepo::import(conn, &session).map_err(|e| e.to_string())?;

    for m in &es.messages {
        MessageRepo::import(conn, m).map_err(|e| e.to_string())?;
        for block in &m.blocks {
            MessageBlockRepo::insert(conn, block).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn record_directory_usage_internal(conn: &rusqlite::Connection, dir: &str) {
    let id = uuid::Uuid::new_v4().to_string();
    let display_name = Path::new(dir)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    if let Err(e) = WorkspaceRepo::record_usage(conn, &id, dir, display_name.as_deref()) {
        tracing::warn!("Failed to record directory usage: {}", e);
    }
}
