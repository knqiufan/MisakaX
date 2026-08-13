use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::repository::{MessageRepo, ProfileRepo, SessionRepo, UsageRepo};
use misaka_x_lib::services::usage::collector::provider_capture;
use misaka_x_lib::services::usage::estimator::unavailable_measurement;
use misaka_x_lib::services::usage::finalize::{finalize_turn, FinalizeTurnRequest};
use misaka_x_lib::services::usage::{
    MeasurementSource, UsageCapture, UsageOperationKind, UsageOutcome,
};
use rusqlite::Connection;

fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();
    ProfileRepo::ensure_default(&conn).unwrap();
    conn.execute(
        "INSERT INTO router_configs (id, name, provider, vendor)
         VALUES ('router-1', 'Provider', 'openai', 'openai')",
        [],
    )
    .unwrap();
    SessionRepo::create(
        &conn,
        "session-1",
        Some("Test"),
        Some("router-1:model-a"),
        None,
    )
    .unwrap();
    MessageRepo::insert_assistant_placeholder(&conn, "message-1", "session-1", "model-a").unwrap();
    conn
}

fn request(captures: Vec<UsageCapture>) -> FinalizeTurnRequest {
    FinalizeTurnRequest {
        operation_key: "assistant:message-1".to_string(),
        operation_kind: UsageOperationKind::Chat,
        session_id: Some("session-1".to_string()),
        message_id: Some("message-1".to_string()),
        selected_model_id: Some("model-a".to_string()),
        effective_provider_config_id: Some("router-1".to_string()),
        effective_model_id: Some("model-a".to_string()),
        vendor_id: Some("openai".to_string()),
        content: "persisted answer".to_string(),
        thinking: String::new(),
        tool_calls_json: None,
        captures,
        was_aborted: false,
        stream_error: None,
        session_title: None,
    }
}

#[test]
fn replayed_finalize_is_idempotent_and_updates_projection_once() {
    let mut conn = setup();
    let capture = provider_capture(
        "rig:aggregate",
        Some("model-a".to_string()),
        Some(10),
        Some(5),
        Some(15),
        Some(2),
        Some(1),
        None,
    );
    let request = request(vec![capture]);

    let first = finalize_turn(&mut conn, &request).unwrap();
    let replay = finalize_turn(&mut conn, &request).unwrap();
    assert_eq!(first.inserted_count, 1);
    assert_eq!(replay.inserted_count, 0);

    let events = UsageRepo::find_by_operation_key(&conn, "assistant:message-1").unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].total_tokens, Some(15));
    let session = SessionRepo::find_by_id(&conn, "session-1").unwrap();
    assert_eq!(session.total_input_tokens, 10);
    assert_eq!(session.total_output_tokens, 5);
    let message = MessageRepo::find_recent(&conn, "session-1", 10)
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(message.status, "complete");
    let json = message.token_usage.unwrap();
    assert!(json.contains("provider_reported"));
    assert!(json.contains("cache_read_tokens"));
}

#[test]
fn provider_total_mismatch_is_preserved_as_authoritative_metadata() {
    let mut conn = setup();
    let request = request(vec![provider_capture(
        "rig:mismatch",
        Some("model-a".into()),
        Some(10),
        Some(5),
        Some(99),
        Some(7),
        Some(3),
        Some(2),
    )]);
    finalize_turn(&mut conn, &request).unwrap();
    let event = UsageRepo::find_by_operation_key(&conn, "assistant:message-1")
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(event.total_tokens, Some(99));
    assert_eq!(event.cache_read_tokens, Some(7));
    assert_eq!(event.cache_creation_tokens, Some(3));
    assert_eq!(event.reasoning_tokens, Some(2));
    assert!(event.metadata_json.contains("provider_total_mismatch"));
}

#[test]
fn distinct_sidecar_models_split_measurements_but_share_operation() {
    let mut conn = setup();
    let request = request(vec![
        provider_capture(
            "run-main",
            Some("model-a".into()),
            Some(4),
            Some(2),
            Some(6),
            None,
            None,
            None,
        ),
        provider_capture(
            "run-subagent",
            Some("model-b".into()),
            Some(7),
            Some(3),
            Some(10),
            None,
            None,
            None,
        ),
    ]);
    assert_eq!(
        finalize_turn(&mut conn, &request).unwrap().inserted_count,
        2
    );
    let events = UsageRepo::find_by_operation_key(&conn, "assistant:message-1").unwrap();
    assert_eq!(events.len(), 2);
    assert_ne!(events[0].measurement_key, events[1].measurement_key);
    assert_ne!(events[0].effective_model_id, events[1].effective_model_id);
}

#[test]
fn unavailable_usage_and_stream_error_still_stabilize_placeholder() {
    let mut conn = setup();
    let mut request = request(vec![UsageCapture {
        capture_id: "fallback:unavailable".to_string(),
        model: Some("model-a".to_string()),
        measurement: unavailable_measurement("provider failed before usage"),
    }]);
    request.content = "partial".to_string();
    request.stream_error = Some("stream failed".to_string());
    let outcome = finalize_turn(&mut conn, &request).unwrap();
    assert_eq!(outcome.outcome, UsageOutcome::Partial);
    let message = MessageRepo::find_recent(&conn, "session-1", 10)
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(message.content, "partial");
    assert_eq!(message.status, "error");
    assert!(message.token_usage.unwrap().contains("unavailable"));
}

#[test]
fn abort_partial_abort_without_usage_and_failed_before_call_stay_distinct() {
    let mut partial_conn = setup();
    let mut partial = request(vec![provider_capture(
        "rig:partial",
        Some("model-a".into()),
        Some(4),
        Some(2),
        Some(6),
        None,
        None,
        None,
    )]);
    partial.was_aborted = true;
    assert_eq!(
        finalize_turn(&mut partial_conn, &partial).unwrap().outcome,
        UsageOutcome::Aborted
    );
    let partial_event = UsageRepo::find_by_operation_key(&partial_conn, "assistant:message-1")
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(partial_event.total_tokens, Some(6));
    assert_eq!(
        partial_event.measurement_source,
        MeasurementSource::ProviderReported
    );

    let mut empty_abort_conn = setup();
    let mut empty_abort = request(vec![UsageCapture {
        capture_id: "fallback:unavailable".into(),
        model: Some("model-a".into()),
        measurement: unavailable_measurement("aborted before provider usage"),
    }]);
    empty_abort.was_aborted = true;
    empty_abort.content.clear();
    assert_eq!(
        finalize_turn(&mut empty_abort_conn, &empty_abort)
            .unwrap()
            .outcome,
        UsageOutcome::Aborted
    );
    let empty_abort_event =
        UsageRepo::find_by_operation_key(&empty_abort_conn, "assistant:message-1")
            .unwrap()
            .pop()
            .unwrap();
    assert_eq!(empty_abort_event.total_tokens, None);
    assert_eq!(
        empty_abort_event.measurement_source,
        MeasurementSource::Unavailable
    );

    let mut failed_conn = setup();
    let mut failed = request(vec![UsageCapture {
        capture_id: "fallback:unavailable".into(),
        model: Some("model-a".into()),
        measurement: unavailable_measurement("provider call did not start"),
    }]);
    failed.content.clear();
    failed.stream_error = Some("failed before call".into());
    assert_eq!(
        finalize_turn(&mut failed_conn, &failed).unwrap().outcome,
        UsageOutcome::Failed
    );
    let failed_event = UsageRepo::find_by_operation_key(&failed_conn, "assistant:message-1")
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(failed_event.total_tokens, None);
    assert_eq!(
        failed_event.measurement_source,
        MeasurementSource::Unavailable
    );
}

#[test]
fn title_usage_updates_title_and_totals_without_activity() {
    let mut conn = setup();
    let mut request = request(vec![provider_capture(
        "rig:title",
        Some("model-a".into()),
        Some(3),
        Some(2),
        Some(5),
        None,
        None,
        None,
    )]);
    request.operation_key = "session_title:one".to_string();
    request.operation_kind = UsageOperationKind::SessionTitle;
    request.message_id = None;
    request.session_title = Some("Generated title".to_string());
    finalize_turn(&mut conn, &request).unwrap();

    let event = UsageRepo::find_by_operation_key(&conn, "session_title:one")
        .unwrap()
        .pop()
        .unwrap();
    assert!(event.counts_toward_totals);
    assert!(!event.counts_toward_activity);
    assert!(event.counts_toward_trend);
    assert_eq!(
        SessionRepo::find_by_id(&conn, "session-1")
            .unwrap()
            .title
            .as_deref(),
        Some("Generated title")
    );
}

#[test]
fn model_probe_is_recorded_only_for_diagnostics() {
    let mut conn = setup();
    let mut request = request(vec![provider_capture(
        "probe",
        Some("model-a".into()),
        Some(1),
        Some(1),
        Some(2),
        None,
        None,
        None,
    )]);
    request.operation_key = "model_probe:one".into();
    request.operation_kind = UsageOperationKind::ModelProbe;
    request.message_id = None;
    finalize_turn(&mut conn, &request).unwrap();

    let event = UsageRepo::find_by_operation_key(&conn, "model_probe:one")
        .unwrap()
        .pop()
        .unwrap();
    assert!(!event.counts_toward_totals);
    assert!(!event.counts_toward_activity);
    assert!(!event.counts_toward_trend);
    let session = SessionRepo::find_by_id(&conn, "session-1").unwrap();
    assert_eq!(session.total_input_tokens, 0);
    assert_eq!(session.total_output_tokens, 0);
}

#[test]
fn regeneration_prunes_message_reference_but_keeps_old_usage_and_adds_new_operation() {
    let mut conn = setup();
    finalize_turn(
        &mut conn,
        &request(vec![provider_capture(
            "rig:first",
            Some("model-a".into()),
            Some(10),
            Some(5),
            Some(15),
            None,
            None,
            None,
        )]),
    )
    .unwrap();
    MessageRepo::delete_from(&conn, "session-1", "message-1").unwrap();

    MessageRepo::insert_assistant_placeholder(&conn, "message-2", "session-1", "model-a").unwrap();
    let mut regenerated = request(vec![provider_capture(
        "rig:regenerated",
        Some("model-a".into()),
        Some(7),
        Some(3),
        Some(10),
        None,
        None,
        None,
    )]);
    regenerated.operation_key = "assistant:message-2".into();
    regenerated.message_id = Some("message-2".into());
    finalize_turn(&mut conn, &regenerated).unwrap();

    let profile = ProfileRepo::get_current(&conn).unwrap();
    let events = UsageRepo::list_for_profile(&conn, &profile.profile_id).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(
        events
            .iter()
            .map(|event| event.total_tokens.unwrap())
            .sum::<u64>(),
        25
    );
    assert!(events
        .iter()
        .any(|event| event.operation_key == "assistant:message-1" && event.message_id.is_none()));
    assert!(events.iter().any(|event| {
        event.operation_key == "assistant:message-2"
            && event.message_id.as_deref() == Some("message-2")
    }));
}
