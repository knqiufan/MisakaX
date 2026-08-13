use std::io;
use std::str::FromStr;

use anyhow::{Context, Result};
use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Transaction};

use crate::db::models::{NewUsageEvent, UsageEvent};
pub struct UsageRepo;

const USAGE_COLUMNS: &str = "event_id, profile_id, operation_key, measurement_key,
    operation_kind, session_id, message_id, provider_config_id, provider_id, vendor_id,
    selected_model_id, effective_model_id, model_display_name, input_tokens, output_tokens,
    total_tokens, cache_read_tokens, cache_creation_tokens, reasoning_tokens,
    measurement_source, estimator_id, estimator_version, outcome, counts_toward_totals,
    counts_toward_activity, counts_toward_trend, occurred_at_utc, local_date, timezone_id,
    utc_offset_minutes, metadata_json, source_installation_id, source_event_id, created_at";

impl UsageRepo {
    pub fn insert_batch_idempotent(
        conn: &Connection,
        events: &[NewUsageEvent],
    ) -> Result<Vec<NewUsageEvent>> {
        let mut inserted = Vec::with_capacity(events.len());
        for event in events {
            let input_tokens = token_to_sql("input_tokens", event.input_tokens)?;
            let output_tokens = token_to_sql("output_tokens", event.output_tokens)?;
            let total_tokens = token_to_sql("total_tokens", event.total_tokens)?;
            let cache_read_tokens = token_to_sql("cache_read_tokens", event.cache_read_tokens)?;
            let cache_creation_tokens =
                token_to_sql("cache_creation_tokens", event.cache_creation_tokens)?;
            let reasoning_tokens = token_to_sql("reasoning_tokens", event.reasoning_tokens)?;

            let affected = conn.execute(
                "INSERT INTO llm_usage_events (
                    event_id, profile_id, operation_key, measurement_key, operation_kind,
                    session_id, message_id, provider_config_id, provider_id, vendor_id,
                    selected_model_id, effective_model_id, model_display_name,
                    input_tokens, output_tokens, total_tokens, cache_read_tokens,
                    cache_creation_tokens, reasoning_tokens, measurement_source,
                    estimator_id, estimator_version, outcome, counts_toward_totals,
                    counts_toward_activity, counts_toward_trend, occurred_at_utc,
                    local_date, timezone_id, utc_offset_minutes, metadata_json,
                    source_installation_id, source_event_id
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                    ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21,
                    ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31,
                    ?32, ?33
                 ) ON CONFLICT DO NOTHING",
                rusqlite::params![
                    event.event_id,
                    event.profile_id,
                    event.operation_key,
                    event.measurement_key,
                    event.operation_kind.as_str(),
                    event.session_id,
                    event.message_id,
                    event.provider_config_id,
                    event.provider_id,
                    event.vendor_id,
                    event.selected_model_id,
                    event.effective_model_id,
                    event.model_display_name,
                    input_tokens,
                    output_tokens,
                    total_tokens,
                    cache_read_tokens,
                    cache_creation_tokens,
                    reasoning_tokens,
                    event.measurement_source.as_str(),
                    event.estimator_id,
                    event.estimator_version,
                    event.outcome.as_str(),
                    event.counts_toward_totals as i32,
                    event.counts_toward_activity as i32,
                    event.counts_toward_trend as i32,
                    event.occurred_at_utc,
                    event.local_date,
                    event.timezone_id,
                    event.utc_offset_minutes,
                    event.metadata_json,
                    event.source_installation_id,
                    event.source_event_id,
                ],
            )?;
            if affected == 1 {
                inserted.push(event.clone());
            }
        }
        let conflict_count = events.len().saturating_sub(inserted.len());
        if conflict_count > 0 {
            tracing::info!(
                attempted_count = events.len(),
                inserted_count = inserted.len(),
                conflict_count,
                "Skipped duplicate usage measurements"
            );
        }
        Ok(inserted)
    }

    pub fn find_by_operation_key(
        conn: &Connection,
        operation_key: &str,
    ) -> Result<Vec<UsageEvent>> {
        let mut statement = conn.prepare(&format!(
            "SELECT {USAGE_COLUMNS} FROM llm_usage_events
             WHERE operation_key = ?1 ORDER BY created_at, event_id"
        ))?;
        let rows = statement.query_map([operation_key], Self::map_row)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn find_by_measurement_key(
        conn: &Connection,
        measurement_key: &str,
    ) -> Result<Option<UsageEvent>> {
        conn.query_row(
            &format!("SELECT {USAGE_COLUMNS} FROM llm_usage_events WHERE measurement_key = ?1"),
            [measurement_key],
            Self::map_row,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn list_for_profile(conn: &Connection, profile_id: &str) -> Result<Vec<UsageEvent>> {
        let mut statement = conn.prepare(&format!(
            "SELECT {USAGE_COLUMNS} FROM llm_usage_events
             WHERE profile_id = ?1 ORDER BY occurred_at_utc, event_id"
        ))?;
        let rows = statement.query_map([profile_id], Self::map_row)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn list_for_session(conn: &Connection, session_id: &str) -> Result<Vec<UsageEvent>> {
        let mut statement = conn.prepare(&format!(
            "SELECT {USAGE_COLUMNS} FROM llm_usage_events
             WHERE session_id = ?1 ORDER BY occurred_at_utc, event_id"
        ))?;
        let rows = statement.query_map([session_id], Self::map_row)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn list_for_profile_since(
        conn: &Connection,
        profile_id: &str,
        local_date: &str,
    ) -> Result<Vec<UsageEvent>> {
        let mut statement = conn.prepare(&format!(
            "SELECT {USAGE_COLUMNS} FROM llm_usage_events
             WHERE profile_id = ?1 AND local_date >= ?2
             ORDER BY local_date, event_id"
        ))?;
        let rows = statement.query_map([profile_id, local_date], Self::map_row)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn clear_profile_history(transaction: &Transaction<'_>, profile_id: &str) -> Result<usize> {
        transaction
            .execute(
                "DELETE FROM llm_usage_events WHERE profile_id = ?1",
                [profile_id],
            )
            .map_err(Into::into)
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<UsageEvent> {
        Ok(UsageEvent {
            event_id: row.get(0)?,
            profile_id: row.get(1)?,
            operation_key: row.get(2)?,
            measurement_key: row.get(3)?,
            operation_kind: parse_enum(row.get::<_, String>(4)?, 4)?,
            session_id: row.get(5)?,
            message_id: row.get(6)?,
            provider_config_id: row.get(7)?,
            provider_id: row.get(8)?,
            vendor_id: row.get(9)?,
            selected_model_id: row.get(10)?,
            effective_model_id: row.get(11)?,
            model_display_name: row.get(12)?,
            input_tokens: token_from_sql(row.get(13)?, 13)?,
            output_tokens: token_from_sql(row.get(14)?, 14)?,
            total_tokens: token_from_sql(row.get(15)?, 15)?,
            cache_read_tokens: token_from_sql(row.get(16)?, 16)?,
            cache_creation_tokens: token_from_sql(row.get(17)?, 17)?,
            reasoning_tokens: token_from_sql(row.get(18)?, 18)?,
            measurement_source: parse_enum(row.get::<_, String>(19)?, 19)?,
            estimator_id: row.get(20)?,
            estimator_version: row.get(21)?,
            outcome: parse_enum(row.get::<_, String>(22)?, 22)?,
            counts_toward_totals: row.get::<_, i64>(23)? != 0,
            counts_toward_activity: row.get::<_, i64>(24)? != 0,
            counts_toward_trend: row.get::<_, i64>(25)? != 0,
            occurred_at_utc: row.get(26)?,
            local_date: row.get(27)?,
            timezone_id: row.get(28)?,
            utc_offset_minutes: row.get(29)?,
            metadata_json: row.get(30)?,
            source_installation_id: row.get(31)?,
            source_event_id: row.get(32)?,
            created_at: row.get(33)?,
        })
    }
}

fn token_to_sql(field: &'static str, value: Option<u64>) -> Result<Option<i64>> {
    value
        .map(|token| {
            i64::try_from(token).with_context(|| format!("{field} exceeds SQLite INTEGER"))
        })
        .transpose()
}

fn token_from_sql(value: Option<i64>, column: usize) -> rusqlite::Result<Option<u64>> {
    value
        .map(|token| {
            u64::try_from(token).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(column, Type::Integer, Box::new(error))
            })
        })
        .transpose()
}

fn parse_enum<T>(value: String, column: usize) -> rusqlite::Result<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    value.parse().map_err(|error: T::Err| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            Type::Text,
            Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                error.to_string(),
            )),
        )
    })
}
