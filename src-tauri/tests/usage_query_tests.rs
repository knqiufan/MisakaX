use chrono::NaiveDate;
use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::models::NewUsageEvent;
use misaka_x_lib::db::repository::{ProfileRepo, UsageRepo};
use misaka_x_lib::services::usage::query::{get_dashboard_at, DashboardQuery};
use misaka_x_lib::services::usage::{
    local_date_sequence, week_bucket_start, MeasurementSource, UsageOperationKind, UsageOutcome,
};
use rusqlite::Connection;

fn date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
}

fn setup() -> (Connection, String) {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();
    let profile = ProfileRepo::ensure_default(&conn).unwrap();
    (conn, profile.profile_id)
}

#[allow(clippy::too_many_arguments)]
fn event(
    profile_id: &str,
    id: &str,
    local_date: &str,
    model: Option<&str>,
    total: Option<u64>,
    source: MeasurementSource,
    activity: bool,
    trend: bool,
) -> NewUsageEvent {
    NewUsageEvent {
        event_id: id.to_string(),
        profile_id: profile_id.to_string(),
        operation_key: format!("operation:{id}"),
        measurement_key: format!("measurement:{id}"),
        operation_kind: UsageOperationKind::Chat,
        session_id: None,
        message_id: None,
        provider_config_id: model.map(|_| format!("provider-{id}")),
        provider_id: model.map(|_| "openai".to_string()),
        vendor_id: None,
        selected_model_id: model.map(ToString::to_string),
        effective_model_id: model.map(ToString::to_string),
        model_display_name: model.map(ToString::to_string),
        input_tokens: total.map(|value| value / 2),
        output_tokens: total.map(|value| value - value / 2),
        total_tokens: total,
        cache_read_tokens: None,
        cache_creation_tokens: None,
        reasoning_tokens: None,
        measurement_source: source,
        estimator_id: source.is_estimated().then(|| "fixture".into()),
        estimator_version: source.is_estimated().then(|| "1".into()),
        outcome: UsageOutcome::Completed,
        counts_toward_totals: true,
        counts_toward_activity: activity,
        counts_toward_trend: trend,
        occurred_at_utc: format!("{local_date}T06:00:00Z"),
        local_date: local_date.to_string(),
        timezone_id: Some("Asia/Shanghai".into()),
        utc_offset_minutes: 480,
        metadata_json: "{}".into(),
        source_installation_id: None,
        source_event_id: None,
    }
}

#[test]
fn dashboard_has_fixed_ranges_quality_totals_and_unknown_semantics() {
    let (conn, profile_id) = setup();
    let events = vec![
        event(
            &profile_id,
            "exact",
            "2026-08-13",
            Some("model-a"),
            Some(10),
            MeasurementSource::ProviderReported,
            true,
            true,
        ),
        event(
            &profile_id,
            "estimated",
            "2026-08-12",
            Some("model-a"),
            Some(7),
            MeasurementSource::HeuristicEstimated,
            true,
            true,
        ),
        event(
            &profile_id,
            "unknown",
            "2026-08-11",
            Some("model-a"),
            None,
            MeasurementSource::Unavailable,
            true,
            true,
        ),
        event(
            &profile_id,
            "legacy-residual",
            "2020-01-01",
            None,
            Some(5),
            MeasurementSource::LegacyMigrated,
            false,
            false,
        ),
    ];
    UsageRepo::insert_batch_idempotent(&conn, &events).unwrap();

    let dashboard = get_dashboard_at(&conn, DashboardQuery::default(), date("2026-08-13")).unwrap();
    assert_eq!(dashboard.daily_activity.len(), 365);
    assert_eq!(dashboard.model_series[0].points.len(), 30);
    assert_eq!(dashboard.overview.total_tokens, "22");
    assert_eq!(dashboard.overview.exact_tokens, "10");
    assert_eq!(dashboard.overview.estimated_tokens, "7");
    assert_eq!(dashboard.overview.legacy_tokens, "5");
    assert_eq!(dashboard.overview.unknown_operation_count, 1);
    assert_eq!(dashboard.overview.total_days, 3);
    assert_eq!(dashboard.overview.current_streak, 3);
    let unknown_day = dashboard
        .daily_activity
        .iter()
        .find(|day| day.local_date == "2026-08-11")
        .unwrap();
    assert_eq!(unknown_day.total_tokens, None);
    assert_eq!(unknown_day.operation_count, 1);
    assert_eq!(unknown_day.quality.unknown_operation_count, 1);
    let no_call = dashboard.model_series[0]
        .points
        .iter()
        .find(|point| point.local_date == "2026-08-10")
        .unwrap();
    assert_eq!(no_call.total_tokens.as_deref(), Some("0"));
}

#[test]
fn trend_keeps_provider_boundaries_and_collapses_after_top_five() {
    let (conn, profile_id) = setup();
    let mut events = Vec::new();
    for index in 0..7 {
        events.push(event(
            &profile_id,
            &format!("model-{index}"),
            "2026-08-13",
            Some(&format!("same-name-{index}")),
            Some(100 - index),
            MeasurementSource::ProviderReported,
            true,
            true,
        ));
    }
    UsageRepo::insert_batch_idempotent(&conn, &events).unwrap();
    let dashboard = get_dashboard_at(&conn, DashboardQuery::default(), date("2026-08-13")).unwrap();
    assert_eq!(dashboard.model_series.len(), 5);
    assert!(dashboard.other_series.is_some());
    assert_eq!(dashboard.other_series.unwrap().points.len(), 30);
}

#[test]
fn date_helpers_cover_leap_year_year_boundary_and_week_starts() {
    let dates = local_date_sequence(date("2024-03-01"), 3);
    assert_eq!(
        dates,
        vec![date("2024-02-28"), date("2024-02-29"), date("2024-03-01")]
    );
    let dates = local_date_sequence(date("2026-01-01"), 2);
    assert_eq!(dates, vec![date("2025-12-31"), date("2026-01-01")]);
    assert_eq!(week_bucket_start(date("2026-08-13"), 1), date("2026-08-10"));
    assert_eq!(week_bucket_start(date("2026-08-13"), 0), date("2026-08-09"));
}

#[test]
fn query_limits_return_stable_errors() {
    let (conn, _) = setup();
    let error = get_dashboard_at(
        &conn,
        DashboardQuery {
            activity_days: 367,
            trend_days: 30,
            max_series: 5,
        },
        date("2026-08-13"),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("activity_days"));
}

#[test]
fn lazy_rollups_incrementally_refresh_and_remain_rebuildable_from_the_ledger() {
    let (conn, profile_id) = setup();
    let first = event(
        &profile_id,
        "mixed-known",
        "2026-08-13",
        Some("model-a"),
        Some(10),
        MeasurementSource::ProviderReported,
        true,
        true,
    );
    UsageRepo::insert_batch_idempotent(&conn, &[first]).unwrap();
    let initial = get_dashboard_at(&conn, DashboardQuery::default(), date("2026-08-13")).unwrap();
    assert_eq!(initial.overview.total_tokens, "10");

    let mut unknown = event(
        &profile_id,
        "mixed-unknown",
        "2026-08-13",
        Some("model-a"),
        None,
        MeasurementSource::Unavailable,
        true,
        true,
    );
    unknown.operation_key = "operation:mixed-known".into();
    UsageRepo::insert_batch_idempotent(&conn, &[unknown]).unwrap();
    let refreshed = get_dashboard_at(&conn, DashboardQuery::default(), date("2026-08-13")).unwrap();
    assert_eq!(refreshed.overview.total_tokens, "10");
    assert_eq!(refreshed.overview.unknown_operation_count, 1);
    let today = refreshed.daily_activity.last().unwrap();
    assert_eq!(today.operation_count, 1);
    assert_eq!(today.total_tokens.as_deref(), Some("10"));
    assert_eq!(today.quality.unknown_operation_count, 1);

    conn.execute("DELETE FROM usage_rollup_state", []).unwrap();
    conn.execute("DELETE FROM usage_operation_rollups", [])
        .unwrap();
    conn.execute("DELETE FROM usage_profile_rollups", [])
        .unwrap();
    conn.execute("DELETE FROM usage_daily_rollups", []).unwrap();
    let rebuilt = get_dashboard_at(&conn, DashboardQuery::default(), date("2026-08-13")).unwrap();
    assert_eq!(rebuilt.overview, refreshed.overview);
    assert_eq!(rebuilt.daily_activity, refreshed.daily_activity);
}
