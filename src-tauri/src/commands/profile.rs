use serde::{Deserialize, Serialize};
use tauri::State;

use crate::config;
use crate::db::models::UserProfile;
use crate::db::repository::ProfileRepo;
use crate::services::profile_avatar::{read_avatar_data_url, remove_avatar, store_avatar};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ProfileUpdateRequest {
    pub display_name: Option<String>,
    pub timezone_id: Option<String>,
    pub week_start: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ProfileAvatarResponse {
    pub profile: UserProfile,
    pub avatar_data_url: String,
}

#[tauri::command]
pub fn profile_get_current(state: State<'_, AppState>) -> Result<UserProfile, String> {
    let db = state.db.lock().map_err(|error| error.to_string())?;
    ProfileRepo::get_current(&db).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn profile_update(
    state: State<'_, AppState>,
    request: ProfileUpdateRequest,
) -> Result<UserProfile, String> {
    let db = state.db.lock().map_err(|error| error.to_string())?;
    let current = ProfileRepo::get_current(&db).map_err(|error| error.to_string())?;
    if let Some(display_name) = request.display_name.as_deref() {
        let display_name = display_name.trim();
        let length = display_name.chars().count();
        if !(1..=40).contains(&length) {
            return Err("display_name must contain between 1 and 40 Unicode characters".into());
        }
        ProfileRepo::update_display_name(&db, &current.profile_id, display_name)
            .map_err(|error| error.to_string())?;
    }
    if request.timezone_id.is_some() || request.week_start.is_some() {
        let week_start = request.week_start.unwrap_or(current.week_start);
        if !matches!(week_start, 0 | 1) {
            return Err("week_start must be 0 (Sunday) or 1 (Monday)".into());
        }
        ProfileRepo::update_timezone_snapshot(
            &db,
            &current.profile_id,
            request
                .timezone_id
                .as_deref()
                .or(current.timezone_id.as_deref()),
            week_start,
        )
        .map_err(|error| error.to_string())?;
    }
    ProfileRepo::get_current(&db).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn profile_avatar_get(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let storage_root = config::profile_avatars_dir().map_err(|error| error.to_string())?;
    let db = state.db.lock().map_err(|error| error.to_string())?;
    let profile = ProfileRepo::get_current(&db).map_err(|error| error.to_string())?;
    let Some(storage_key) = profile.avatar_storage_key.as_deref() else {
        return Ok(None);
    };
    match read_avatar_data_url(&storage_root, storage_key) {
        Ok(data_url) => Ok(Some(data_url)),
        Err(_) => {
            tracing::warn!("Stored profile avatar is unavailable");
            Ok(None)
        }
    }
}

#[tauri::command]
pub fn profile_avatar_set(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ProfileAvatarResponse, String> {
    let storage_root = config::profile_avatars_dir().map_err(|error| error.to_string())?;
    let stored = store_avatar(std::path::Path::new(&file_path), &storage_root)
        .map_err(|error| error.to_string())?;
    let db = state.db.lock().map_err(|error| error.to_string())?;
    let current = ProfileRepo::get_current(&db).map_err(|error| error.to_string())?;
    let previous_key = current.avatar_storage_key.clone();
    let transaction = db
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let profile = match ProfileRepo::update_avatar(
        &transaction,
        &current.profile_id,
        &stored.storage_key,
        &stored.sha256,
    ) {
        Ok(profile) => profile,
        Err(error) => {
            let _ = remove_avatar(&storage_root, &stored.storage_key);
            return Err(error.to_string());
        }
    };
    if let Err(error) = transaction.commit() {
        let _ = remove_avatar(&storage_root, &stored.storage_key);
        return Err(error.to_string());
    }
    drop(db);
    if let Some(previous_key) = previous_key.as_deref() {
        if remove_avatar(&storage_root, previous_key).is_err() {
            tracing::warn!("Failed to remove replaced profile avatar");
        }
    }
    let avatar_data_url = read_avatar_data_url(&storage_root, &stored.storage_key)
        .map_err(|error| error.to_string())?;
    Ok(ProfileAvatarResponse {
        profile,
        avatar_data_url,
    })
}

#[tauri::command]
pub fn profile_avatar_clear(state: State<'_, AppState>) -> Result<UserProfile, String> {
    let storage_root = config::profile_avatars_dir().map_err(|error| error.to_string())?;
    let db = state.db.lock().map_err(|error| error.to_string())?;
    let current = ProfileRepo::get_current(&db).map_err(|error| error.to_string())?;
    let profile =
        ProfileRepo::clear_avatar(&db, &current.profile_id).map_err(|error| error.to_string())?;
    drop(db);
    if let Some(storage_key) = current.avatar_storage_key.as_deref() {
        if remove_avatar(&storage_root, storage_key).is_err() {
            tracing::warn!("Failed to remove cleared profile avatar");
        }
    }
    Ok(profile)
}
