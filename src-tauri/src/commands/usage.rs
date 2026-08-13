use tauri::State;

use crate::db::repository::{ProfileRepo, UsageRepo};
use crate::services::usage::query::{get_dashboard, DashboardQuery};
use crate::services::usage::UsageDashboardV1;
use crate::AppState;

#[tauri::command]
pub fn usage_get_dashboard(
    state: State<'_, AppState>,
    activity_days: Option<u32>,
    trend_days: Option<u32>,
    max_series: Option<u32>,
) -> Result<UsageDashboardV1, String> {
    let defaults = DashboardQuery::default();
    let query = DashboardQuery {
        activity_days: activity_days.unwrap_or(defaults.activity_days),
        trend_days: trend_days.unwrap_or(defaults.trend_days),
        max_series: max_series.unwrap_or(defaults.max_series),
    };
    let db = state.db.lock().map_err(|error| error.to_string())?;
    get_dashboard(&db, query).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn usage_clear_history(state: State<'_, AppState>) -> Result<u64, String> {
    let db = state.db.lock().map_err(|error| error.to_string())?;
    let profile = ProfileRepo::get_current(&db).map_err(|error| error.to_string())?;
    let transaction = db
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let deleted = UsageRepo::clear_profile_history(&transaction, &profile.profile_id)
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "UPDATE sessions
             SET total_input_tokens = 0, total_output_tokens = 0,
                 updated_at = CURRENT_TIMESTAMP",
            [],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(deleted as u64)
}
