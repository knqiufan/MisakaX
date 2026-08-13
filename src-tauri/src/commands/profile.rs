use serde::Deserialize;
use tauri::State;

use crate::db::models::UserProfile;
use crate::db::repository::ProfileRepo;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ProfileUpdateRequest {
    pub display_name: Option<String>,
    pub timezone_id: Option<String>,
    pub week_start: Option<i32>,
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
