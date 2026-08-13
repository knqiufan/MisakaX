use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::models::NewUsageEvent;
use misaka_x_lib::db::repository::{ProfileRepo, SessionRepo, UsageRepo};
use misaka_x_lib::services::usage::{MeasurementSource, UsageOperationKind, UsageOutcome};
use rusqlite::Connection;

fn setup() -> (Connection, String) {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();
    let profile = ProfileRepo::ensure_default(&conn).unwrap();
    SessionRepo::create(&conn, "session-1", Some("Test"), Some("model-a"), None).unwrap();
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, model)
         VALUES ('message-1', 'session-1', 'assistant', 'hello', 'model-a')",
        [],
    )
    .unwrap();
    (conn, profile.profile_id)
}

fn event(profile_id: &str, event_id: &str, measurement_key: &str) -> NewUsageEvent {
    NewUsageEvent {
        event_id: event_id.to_string(),
        profile_id: profile_id.to_string(),
        operation_key: "assistant:message-1".to_string(),
        measurement_key: measurement_key.to_string(),
        operation_kind: UsageOperationKind::Chat,
        session_id: Some("session-1".to_string()),
        message_id: Some("message-1".to_string()),
        provider_config_id: Some("router-1".to_string()),
        provider_id: Some("openai".to_string()),
        vendor_id: Some("openai".to_string()),
        selected_model_id: Some("model-a".to_string()),
        effective_model_id: Some("model-a".to_string()),
        model_display_name: Some("Model A".to_string()),
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
        occurred_at_utc: "2026-08-13T06:00:00Z".to_string(),
        local_date: "2026-08-13".to_string(),
        timezone_id: Some("Asia/Shanghai".to_string()),
        utc_offset_minutes: 480,
        metadata_json: "{}".to_string(),
        source_installation_id: None,
        source_event_id: None,
    }
}

#[test]
fn insert_batch_returns_only_new_measurements_and_replay_is_idempotent() {
    let (conn, profile_id) = setup();
    let first = event(&profile_id, "event-1", "assistant:message-1:model:a");

    let inserted = UsageRepo::insert_batch_idempotent(&conn, &[first.clone()]).unwrap();
    let replayed = UsageRepo::insert_batch_idempotent(&conn, &[first]).unwrap();
    assert_eq!(inserted.len(), 1);
    assert!(replayed.is_empty());

    let stored = UsageRepo::find_by_operation_key(&conn, "assistant:message-1").unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].total_tokens, Some(15));
    assert_eq!(
        stored[0].measurement_source,
        MeasurementSource::ProviderReported
    );
}

#[test]
fn null_usage_and_sqlite_i64_max_round_trip_without_precision_loss() {
    let (conn, profile_id) = setup();
    let mut unknown = event(&profile_id, "event-unknown", "unknown");
    unknown.input_tokens = None;
    unknown.output_tokens = None;
    unknown.total_tokens = None;
    unknown.measurement_source = MeasurementSource::Unavailable;

    let mut large = event(&profile_id, "event-large", "large");
    large.input_tokens = Some(i64::MAX as u64);
    large.output_tokens = Some(0);
    large.total_tokens = Some(i64::MAX as u64);

    UsageRepo::insert_batch_idempotent(&conn, &[unknown, large]).unwrap();
    assert_eq!(
        UsageRepo::find_by_measurement_key(&conn, "unknown")
            .unwrap()
            .unwrap()
            .total_tokens,
        None
    );
    assert_eq!(
        UsageRepo::find_by_measurement_key(&conn, "large")
            .unwrap()
            .unwrap()
            .total_tokens,
        Some(i64::MAX as u64)
    );

    let mut overflow = event(&profile_id, "event-overflow", "overflow");
    overflow.total_tokens = Some(i64::MAX as u64 + 1);
    assert!(UsageRepo::insert_batch_idempotent(&conn, &[overflow]).is_err());
}

#[test]
fn one_operation_can_keep_multiple_model_measurements() {
    let (conn, profile_id) = setup();
    let first = event(&profile_id, "event-a", "operation:model:a");
    let mut second = event(&profile_id, "event-b", "operation:model:b");
    second.effective_model_id = Some("model-b".to_string());
    second.model_display_name = Some("Model B".to_string());
    second.total_tokens = Some(20);

    let inserted = UsageRepo::insert_batch_idempotent(&conn, &[first, second]).unwrap();
    assert_eq!(inserted.len(), 2);
    let stored = UsageRepo::find_by_operation_key(&conn, "assistant:message-1").unwrap();
    assert_eq!(stored.len(), 2);
    assert_eq!(stored[1].effective_model_id.as_deref(), Some("model-b"));
}

#[test]
fn provider_snapshot_survives_router_config_deletion() {
    let (conn, profile_id) = setup();
    conn.execute(
        "INSERT INTO router_configs (id, name, provider) VALUES ('router-1', 'OpenAI', 'openai')",
        [],
    )
    .unwrap();
    UsageRepo::insert_batch_idempotent(&conn, &[event(&profile_id, "event-1", "snapshot")])
        .unwrap();
    conn.execute("DELETE FROM router_configs WHERE id = 'router-1'", [])
        .unwrap();

    let stored = UsageRepo::find_by_measurement_key(&conn, "snapshot")
        .unwrap()
        .unwrap();
    assert_eq!(stored.provider_config_id.as_deref(), Some("router-1"));
    assert_eq!(stored.model_display_name.as_deref(), Some("Model A"));
}

#[test]
fn deleting_session_and_message_only_clears_ledger_weak_references() {
    let (conn, profile_id) = setup();
    UsageRepo::insert_batch_idempotent(&conn, &[event(&profile_id, "event-1", "weak-refs")])
        .unwrap();
    SessionRepo::delete(&conn, "session-1").unwrap();

    let stored = UsageRepo::find_by_measurement_key(&conn, "weak-refs")
        .unwrap()
        .unwrap();
    assert!(stored.session_id.is_none());
    assert!(stored.message_id.is_none());
    assert_eq!(stored.total_tokens, Some(15));
}

#[test]
fn clearing_history_requires_and_obeys_an_explicit_transaction() {
    let (conn, profile_id) = setup();
    UsageRepo::insert_batch_idempotent(&conn, &[event(&profile_id, "event-1", "clear-me")])
        .unwrap();

    let transaction = conn.unchecked_transaction().unwrap();
    assert_eq!(
        UsageRepo::clear_profile_history(&transaction, &profile_id).unwrap(),
        1
    );
    transaction.rollback().unwrap();
    assert_eq!(
        UsageRepo::list_for_profile(&conn, &profile_id)
            .unwrap()
            .len(),
        1
    );

    let transaction = conn.unchecked_transaction().unwrap();
    assert_eq!(
        UsageRepo::clear_profile_history(&transaction, &profile_id).unwrap(),
        1
    );
    transaction.commit().unwrap();
    assert!(UsageRepo::list_for_profile(&conn, &profile_id)
        .unwrap()
        .is_empty());
}

#[test]
fn bound_profile_parameters_do_not_change_query_scope() {
    let (conn, profile_id) = setup();
    UsageRepo::insert_batch_idempotent(&conn, &[event(&profile_id, "event-1", "bound")]).unwrap();
    assert!(UsageRepo::list_for_profile(&conn, "' OR 1=1 --")
        .unwrap()
        .is_empty());
}
