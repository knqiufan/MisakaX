#![cfg(feature = "test-private")]

use misaka_x_lib::commands::usage::clear_usage_history;
use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::models::NewUsageEvent;
use misaka_x_lib::db::repository::{ProfileRepo, SessionRepo, UsageRepo};
use misaka_x_lib::services::usage::{MeasurementSource, UsageOperationKind, UsageOutcome};
use rusqlite::Connection;

fn event(profile_id: &str, id: &str) -> NewUsageEvent {
    NewUsageEvent {
        event_id: id.into(),
        profile_id: profile_id.into(),
        operation_key: format!("operation:{id}"),
        measurement_key: format!("measurement:{id}"),
        operation_kind: UsageOperationKind::Chat,
        session_id: Some("session-1".into()),
        message_id: Some("message-1".into()),
        provider_config_id: None,
        provider_id: Some("provider".into()),
        vendor_id: None,
        selected_model_id: Some("model".into()),
        effective_model_id: Some("model".into()),
        model_display_name: Some("Model".into()),
        input_tokens: Some(10),
        output_tokens: Some(5),
        total_tokens: Some(15),
        cache_read_tokens: None,
        cache_creation_tokens: None,
        reasoning_tokens: None,
        measurement_source: MeasurementSource::ProviderReported,
        estimator_id: None,
        estimator_version: None,
        outcome: UsageOutcome::Completed,
        counts_toward_totals: true,
        counts_toward_activity: true,
        counts_toward_trend: true,
        occurred_at_utc: "2026-08-13T00:00:00Z".into(),
        local_date: "2026-08-13".into(),
        timezone_id: Some("Asia/Shanghai".into()),
        utc_offset_minutes: 480,
        metadata_json: "{}".into(),
        source_installation_id: None,
        source_event_id: None,
    }
}

fn setup() -> (Connection, String) {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();
    let profile = ProfileRepo::ensure_default(&conn).unwrap();
    SessionRepo::create(&conn, "session-1", Some("Session"), Some("model"), None).unwrap();
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, token_usage)
         VALUES ('message-1', 'session-1', 'assistant', 'kept', '{\"total_tokens\":15}')",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE sessions SET total_input_tokens = 10, total_output_tokens = 5 WHERE id = 'session-1'",
        [],
    )
    .unwrap();
    UsageRepo::insert_batch_idempotent(&conn, &[event(&profile.profile_id, "first")]).unwrap();
    (conn, profile.profile_id)
}

#[test]
fn clear_is_atomic_keeps_message_json_and_accepts_new_events_afterward() {
    let (conn, profile_id) = setup();
    assert_eq!(clear_usage_history(&conn, &profile_id).unwrap(), 1);
    assert!(UsageRepo::list_for_profile(&conn, &profile_id)
        .unwrap()
        .is_empty());
    let projections: (i64, i64) = conn
        .query_row(
            "SELECT total_input_tokens, total_output_tokens FROM sessions WHERE id = 'session-1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(projections, (0, 0));
    let token_json: Option<String> = conn
        .query_row(
            "SELECT token_usage FROM messages WHERE id = 'message-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(token_json.as_deref(), Some("{\"total_tokens\":15}"));
    assert_eq!(
        UsageRepo::insert_batch_idempotent(&conn, &[event(&profile_id, "second")])
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn projection_failure_rolls_back_ledger_deletion() {
    let (conn, profile_id) = setup();
    conn.execute_batch(
        "CREATE TRIGGER fail_usage_projection
         BEFORE UPDATE OF total_input_tokens ON sessions
         BEGIN SELECT RAISE(FAIL, 'projection failed'); END;",
    )
    .unwrap();
    assert!(clear_usage_history(&conn, &profile_id).is_err());
    assert_eq!(
        UsageRepo::list_for_profile(&conn, &profile_id)
            .unwrap()
            .len(),
        1
    );
    let input: i64 = conn
        .query_row(
            "SELECT total_input_tokens FROM sessions WHERE id = 'session-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(input, 10);
}
