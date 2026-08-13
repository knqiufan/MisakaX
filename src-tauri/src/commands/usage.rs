use tauri::State;

use crate::db::repository::{ProfileRepo, UsageRepo};
use crate::services::usage::query::{get_dashboard, DashboardQuery};
use crate::services::usage::UsageDashboardV1;
use crate::AppState;

pub fn clear_usage_history(conn: &rusqlite::Connection, profile_id: &str) -> anyhow::Result<u64> {
    let transaction = conn.unchecked_transaction()?;
    let deleted = UsageRepo::clear_profile_history(&transaction, profile_id)?;
    transaction.execute(
        "DELETE FROM usage_rollup_state WHERE profile_id = ?1",
        [profile_id],
    )?;
    transaction.execute(
        "DELETE FROM usage_operation_rollups WHERE profile_id = ?1",
        [profile_id],
    )?;
    transaction.execute(
        "DELETE FROM usage_profile_rollups WHERE profile_id = ?1",
        [profile_id],
    )?;
    transaction.execute(
        "DELETE FROM usage_daily_rollups WHERE profile_id = ?1",
        [profile_id],
    )?;
    transaction.execute(
        "UPDATE sessions
         SET total_input_tokens = 0, total_output_tokens = 0,
             updated_at = CURRENT_TIMESTAMP",
        [],
    )?;
    transaction.commit()?;
    Ok(deleted as u64)
}

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
    let deleted =
        clear_usage_history(&db, &profile.profile_id).map_err(|error| error.to_string())?;
    tracing::info!(deleted_count = deleted, "Cleared local usage history");
    Ok(deleted)
}
