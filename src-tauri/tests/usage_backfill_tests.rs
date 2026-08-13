use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::repository::{ProfileRepo, SessionRepo, UsageRepo};
use misaka_x_lib::services::usage::backfill::backfill_legacy_usage;
use misaka_x_lib::services::usage::MeasurementSource;
use rusqlite::Connection;

fn setup() -> (Connection, String) {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();
    let profile = ProfileRepo::ensure_default(&conn).unwrap();
    SessionRepo::create(
        &conn,
        "session-1",
        Some("Legacy"),
        Some("legacy-model"),
        None,
    )
    .unwrap();
    (conn, profile.profile_id)
}

#[test]
fn backfill_is_idempotent_and_preserves_null_details() {
    let (conn, profile_id) = setup();
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, token_usage, model, created_at)
         VALUES ('message-1', 'session-1', 'assistant', 'legacy',
                 '{\"input_tokens\":10,\"output_tokens\":5,\"total_tokens\":15}',
                 'legacy-model', '2026-08-10 01:02:03')",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE sessions SET total_input_tokens = 13, total_output_tokens = 7
         WHERE id = 'session-1'",
        [],
    )
    .unwrap();

    let first = backfill_legacy_usage(&conn).unwrap();
    let replay = backfill_legacy_usage(&conn).unwrap();
    assert_eq!(first.message_events_inserted, 1);
    assert_eq!(first.residual_events_inserted, 1);
    assert_eq!(replay.message_events_inserted, 0);
    assert_eq!(replay.residual_events_inserted, 0);
    let events = UsageRepo::list_for_profile(&conn, &profile_id).unwrap();
    assert_eq!(events.len(), 2);
    let message = events
        .iter()
        .find(|event| event.message_id.is_some())
        .unwrap();
    assert_eq!(
        message.measurement_source,
        MeasurementSource::LegacyMigrated
    );
    assert_eq!(message.cache_read_tokens, None);
    let residual = events
        .iter()
        .find(|event| event.message_id.is_none())
        .unwrap();
    assert!(!residual.counts_toward_activity);
    assert!(!residual.counts_toward_trend);
    assert_eq!(residual.effective_model_id, None);
}

#[test]
fn bad_json_and_projection_below_message_sum_are_diagnostics_not_failures() {
    let (conn, _) = setup();
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, token_usage, created_at)
         VALUES ('bad', 'session-1', 'assistant', 'bad', '{oops', CURRENT_TIMESTAMP),
                ('large', 'session-1', 'assistant', 'large',
                 '{\"input_tokens\":10,\"output_tokens\":5,\"total_tokens\":15}',
                 CURRENT_TIMESTAMP)",
        [],
    )
    .unwrap();
    let diagnostics = backfill_legacy_usage(&conn).unwrap();
    assert_eq!(diagnostics.bad_json_count, 1);
    assert_eq!(diagnostics.message_events_inserted, 1);
    assert_eq!(diagnostics.projection_below_ledger_count, 1);
}

#[test]
fn existing_assistant_operation_is_not_backfilled_again() {
    let (conn, profile_id) = setup();
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, token_usage, created_at)
         VALUES ('message-1', 'session-1', 'assistant', 'new',
                 '{\"input_tokens\":1,\"output_tokens\":1,\"total_tokens\":2}',
                 CURRENT_TIMESTAMP)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO llm_usage_events (
            event_id, profile_id, operation_key, measurement_key, operation_kind,
            session_id, message_id, input_tokens, output_tokens, total_tokens,
            measurement_source, outcome, counts_toward_totals,
            counts_toward_activity, counts_toward_trend, occurred_at_utc,
            local_date, utc_offset_minutes, metadata_json
         ) VALUES (
            'existing', ?1, 'assistant:message-1', 'existing-measurement', 'chat',
            'session-1', 'message-1', 1, 1, 2, 'provider_reported', 'completed',
            1, 1, 1, '2026-08-13T00:00:00Z', '2026-08-13', 0, '{}'
         )",
        [profile_id],
    )
    .unwrap();
    let diagnostics = backfill_legacy_usage(&conn).unwrap();
    assert_eq!(diagnostics.existing_events_skipped, 1);
    assert_eq!(diagnostics.message_events_inserted, 0);
}
