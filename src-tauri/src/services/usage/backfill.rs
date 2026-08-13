use anyhow::Result;
use chrono::{NaiveDateTime, SecondsFormat};
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use serde_json::{json, Value};

use crate::db::models::NewUsageEvent;
use crate::db::repository::{ProfileRepo, UsageRepo};

use super::{MeasurementSource, UsageOperationKind, UsageOutcome};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LegacyBackfillDiagnostics {
    pub message_events_inserted: u64,
    pub residual_events_inserted: u64,
    pub existing_events_skipped: u64,
    pub bad_json_count: u64,
    pub invalid_date_count: u64,
    pub projection_below_ledger_count: u64,
}

pub fn backfill_legacy_usage(conn: &Connection) -> Result<LegacyBackfillDiagnostics> {
    let profile = ProfileRepo::get_current(conn)?;
    let tx = conn.unchecked_transaction()?;
    let mut diagnostics = LegacyBackfillDiagnostics::default();
    let mut statement = tx.prepare(
        "SELECT id, session_id, token_usage, model, created_at
         FROM messages
         WHERE role = 'assistant' AND token_usage IS NOT NULL
         ORDER BY created_at, id",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;

    for row in rows {
        let (message_id, session_id, raw_usage, model, created_at) = row?;
        let already_recorded: Option<i64> = tx
            .query_row(
                "SELECT 1 FROM llm_usage_events
                 WHERE message_id = ?1 OR operation_key = ?2 LIMIT 1",
                rusqlite::params![message_id, format!("assistant:{message_id}")],
                |row| row.get(0),
            )
            .optional()?;
        if already_recorded.is_some() {
            diagnostics.existing_events_skipped += 1;
            continue;
        }

        let usage: Value = match serde_json::from_str(&raw_usage) {
            Ok(value) => value,
            Err(_) => {
                diagnostics.bad_json_count += 1;
                continue;
            }
        };
        let input = token_field(&usage, "input_tokens");
        let output = token_field(&usage, "output_tokens");
        let total = token_field(&usage, "total_tokens").or_else(|| {
            input
                .zip(output)
                .and_then(|(left, right)| left.checked_add(right))
        });
        if input.is_none() && output.is_none() && total.is_none() {
            diagnostics.bad_json_count += 1;
            continue;
        }
        let Some((occurred_at, local_date)) = normalize_legacy_timestamp(&created_at) else {
            diagnostics.invalid_date_count += 1;
            continue;
        };

        let event = NewUsageEvent {
            event_id: format!("legacy-message-{message_id}"),
            profile_id: profile.profile_id.clone(),
            operation_key: format!("legacy:message:{message_id}"),
            measurement_key: format!("legacy:message:{message_id}:usage"),
            operation_kind: UsageOperationKind::LegacyBackfill,
            session_id: Some(session_id),
            message_id: Some(message_id),
            provider_config_id: None,
            provider_id: None,
            vendor_id: None,
            selected_model_id: model.clone(),
            effective_model_id: model.clone(),
            model_display_name: model,
            input_tokens: input,
            output_tokens: output,
            total_tokens: total,
            cache_read_tokens: token_field(&usage, "cache_read_tokens"),
            cache_creation_tokens: token_field(&usage, "cache_creation_tokens"),
            reasoning_tokens: token_field(&usage, "reasoning_tokens"),
            measurement_source: MeasurementSource::LegacyMigrated,
            estimator_id: None,
            estimator_version: None,
            outcome: UsageOutcome::Completed,
            counts_toward_totals: true,
            counts_toward_activity: true,
            counts_toward_trend: true,
            occurred_at_utc: occurred_at,
            local_date,
            timezone_id: None,
            utc_offset_minutes: 0,
            metadata_json: json!({
                "legacy_source": "messages.token_usage",
                "historical_timezone_unknown": true
            })
            .to_string(),
            source_installation_id: None,
            source_event_id: None,
        };
        diagnostics.message_events_inserted +=
            UsageRepo::insert_batch_idempotent(&tx, &[event])?.len() as u64;
    }
    drop(statement);

    let mut sessions =
        tx.prepare("SELECT id, total_input_tokens, total_output_tokens, created_at FROM sessions")?;
    let rows = sessions.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    for row in rows {
        let (session_id, projected_input, projected_output, created_at) = row?;
        let (ledger_input, ledger_output): (i64, i64) = tx.query_row(
            "SELECT COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0)
             FROM llm_usage_events
             WHERE session_id = ?1 AND counts_toward_totals = 1",
            [&session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        if projected_input < ledger_input || projected_output < ledger_output {
            diagnostics.projection_below_ledger_count += 1;
            continue;
        }
        let residual_input = u64::try_from(projected_input - ledger_input).unwrap_or(0);
        let residual_output = u64::try_from(projected_output - ledger_output).unwrap_or(0);
        if residual_input == 0 && residual_output == 0 {
            continue;
        }
        let Some((occurred_at, local_date)) = normalize_legacy_timestamp(&created_at) else {
            diagnostics.invalid_date_count += 1;
            continue;
        };
        let total = residual_input.checked_add(residual_output);
        let event = NewUsageEvent {
            event_id: format!("legacy-session-{session_id}-residual"),
            profile_id: profile.profile_id.clone(),
            operation_key: format!("legacy:session:{session_id}:residual"),
            measurement_key: format!("legacy:session:{session_id}:residual"),
            operation_kind: UsageOperationKind::LegacyBackfill,
            session_id: Some(session_id),
            message_id: None,
            provider_config_id: None,
            provider_id: None,
            vendor_id: None,
            selected_model_id: None,
            effective_model_id: None,
            model_display_name: None,
            input_tokens: Some(residual_input),
            output_tokens: Some(residual_output),
            total_tokens: total,
            cache_read_tokens: None,
            cache_creation_tokens: None,
            reasoning_tokens: None,
            measurement_source: MeasurementSource::LegacyMigrated,
            estimator_id: None,
            estimator_version: None,
            outcome: UsageOutcome::Completed,
            counts_toward_totals: true,
            counts_toward_activity: false,
            counts_toward_trend: false,
            occurred_at_utc: occurred_at,
            local_date,
            timezone_id: None,
            utc_offset_minutes: 0,
            metadata_json: json!({
                "legacy_source": "sessions.total_*_tokens residual",
                "historical_timezone_unknown": true
            })
            .to_string(),
            source_installation_id: None,
            source_event_id: None,
        };
        diagnostics.residual_events_inserted +=
            UsageRepo::insert_batch_idempotent(&tx, &[event])?.len() as u64;
    }
    drop(sessions);
    tx.commit()?;
    Ok(diagnostics)
}

fn token_field(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn normalize_legacy_timestamp(value: &str) -> Option<(String, String)> {
    let normalized = value.trim().replace(' ', "T");
    let normalized = normalized.trim_end_matches('Z');
    let date = normalized.get(..10)?;
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let datetime = NaiveDateTime::parse_from_str(
        normalized.get(..19).unwrap_or(&normalized),
        "%Y-%m-%dT%H:%M:%S",
    )
    .ok()?
    .and_utc()
    .to_rfc3339_opts(SecondsFormat::Secs, true);
    Some((datetime, date.to_string()))
}
