use serde::Deserialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::config;
use crate::db::repository::MessageBlockRepo;
use crate::services::artifacts::{
    ArtifactMetadata, ArtifactOrigin, ArtifactPreview, ArtifactService, ContentSafetyPolicy,
    ExportOutcome,
};
use crate::services::content::{ContentBlock, MessageBlockService};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ArtifactRegisterRequest {
    pub session_id: String,
    pub origin_message_id: Option<String>,
    pub origin_kind: ArtifactOrigin,
    pub display_name: String,
    pub media_type: String,
    /// Transport-only ingress. It is decoded immediately and never persisted in
    /// SQLite or returned by metadata commands.
    pub bytes_base64: String,
}

fn service() -> Result<ArtifactService, String> {
    ArtifactService::new(
        config::artifacts_dir().map_err(|error| error.to_string())?,
        ContentSafetyPolicy::default(),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn artifact_register(
    state: State<'_, AppState>,
    request: ArtifactRegisterRequest,
) -> Result<ArtifactMetadata, String> {
    if !state.feature_flags.rich_content_write {
        return Err("CONTENT_BLOCK_UNSUPPORTED".to_string());
    }
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    service()?
        .register_base64(
            &conn,
            request.session_id,
            request.origin_message_id,
            request.origin_kind,
            request.display_name,
            request.media_type,
            &request.bytes_base64,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn artifact_get_metadata(
    state: State<'_, AppState>,
    session_id: String,
    artifact_id: String,
) -> Result<ArtifactMetadata, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    service()?
        .metadata(&conn, &session_id, &artifact_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn artifact_get_preview(
    state: State<'_, AppState>,
    session_id: String,
    artifact_id: String,
) -> Result<ArtifactPreview, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    service()?
        .preview(&conn, &session_id, &artifact_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn artifact_read_preview_base64(
    state: State<'_, AppState>,
    session_id: String,
    artifact_id: String,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    service()?
        .read_preview_base64(&conn, &session_id, &artifact_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn artifact_export(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    artifact_id: String,
) -> Result<ExportOutcome, String> {
    let metadata = {
        let conn = state.db.lock().map_err(|error| error.to_string())?;
        service()?
            .metadata(&conn, &session_id, &artifact_id)
            .map_err(|error| error.to_string())?
    };
    let app_for_dialog = app.clone();
    let target = tauri::async_runtime::spawn_blocking(move || {
        app_for_dialog
            .dialog()
            .file()
            .set_title("Save artifact")
            .set_file_name(metadata.display_name)
            .blocking_save_file()
            .and_then(|path| path.into_path().ok())
    })
    .await
    .map_err(|error| error.to_string())?;
    let Some(target) = target else {
        return Ok(ExportOutcome {
            status: "cancelled".to_string(),
            file_name: None,
        });
    };
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    service()?
        .export_to_path(&conn, &session_id, &artifact_id, &target)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn artifact_delete_or_expire(
    state: State<'_, AppState>,
    session_id: String,
    artifact_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    service()?
        .expire(&conn, &session_id, &artifact_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn append_content_block(state: State<'_, AppState>, block: ContentBlock) -> Result<(), String> {
    if !state.feature_flags.rich_content_write {
        return Err("CONTENT_BLOCK_UNSUPPORTED".to_string());
    }
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    MessageBlockService::append(&conn, &block, service()?.policy())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_message_blocks(
    state: State<'_, AppState>,
    message_id: String,
) -> Result<Vec<ContentBlock>, String> {
    if !state.feature_flags.rich_content_render {
        return Ok(Vec::new());
    }
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    MessageBlockRepo::find_by_message(&conn, &message_id).map_err(|error| error.to_string())
}
