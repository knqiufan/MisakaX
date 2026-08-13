use std::time::Instant;

use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, Transaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RollupRefresh {
    pub rebuilt: bool,
    pub changed_operations: usize,
    pub changed_dates: usize,
}

pub fn refresh_usage_rollups(conn: &Connection, profile_id: &str) -> Result<RollupRefresh> {
    let started = Instant::now();
    let transaction = conn.unchecked_transaction()?;
    let (event_count, last_event_rowid): (i64, i64) = transaction.query_row(
        "SELECT COUNT(*), COALESCE(MAX(rowid), 0)
         FROM llm_usage_events WHERE profile_id = ?1",
        [profile_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let state = transaction
        .query_row(
            "SELECT event_count, last_event_rowid FROM usage_rollup_state
             WHERE profile_id = ?1",
            [profile_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .optional()?;
    if state == Some((event_count, last_event_rowid)) {
        transaction.commit()?;
        return Ok(RollupRefresh {
            rebuilt: false,
            changed_operations: 0,
            changed_dates: 0,
        });
    }

    let must_rebuild = state.is_none_or(|(previous_count, previous_rowid)| {
        event_count < previous_count || last_event_rowid < previous_rowid
    });
    let (changed_operations, changed_dates) = if must_rebuild {
        rebuild_all(&transaction, profile_id)?;
        (0, 0)
    } else {
        let (_, previous_rowid) = state.expect("checked above");
        let operations = changed_values(&transaction, "operation_key", profile_id, previous_rowid)?;
        let dates = changed_values(&transaction, "local_date", profile_id, previous_rowid)?;
        for operation_key in &operations {
            rebuild_operation(&transaction, profile_id, operation_key)?;
        }
        for local_date in &dates {
            rebuild_day(&transaction, profile_id, local_date)?;
        }
        rebuild_profile(&transaction, profile_id)?;
        (operations.len(), dates.len())
    };
    transaction.execute(
        "INSERT INTO usage_rollup_state (
             profile_id, last_event_rowid, event_count, updated_at
         ) VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
         ON CONFLICT(profile_id) DO UPDATE SET
             last_event_rowid = excluded.last_event_rowid,
             event_count = excluded.event_count,
             updated_at = CURRENT_TIMESTAMP",
        rusqlite::params![profile_id, last_event_rowid, event_count],
    )?;
    transaction.commit()?;
    tracing::info!(
        duration_ms = started.elapsed().as_millis() as u64,
        rebuilt = must_rebuild,
        changed_operation_count = changed_operations,
        changed_date_count = changed_dates,
        "Refreshed usage analytics rollups"
    );
    Ok(RollupRefresh {
        rebuilt: must_rebuild,
        changed_operations,
        changed_dates,
    })
}

fn changed_values(
    transaction: &Transaction<'_>,
    column: &str,
    profile_id: &str,
    previous_rowid: i64,
) -> Result<Vec<String>> {
    debug_assert!(matches!(column, "operation_key" | "local_date"));
    let mut statement = transaction.prepare(&format!(
        "SELECT DISTINCT {column} FROM llm_usage_events
         WHERE profile_id = ?1 AND rowid > ?2 ORDER BY {column}"
    ))?;
    let rows = statement.query_map(rusqlite::params![profile_id, previous_rowid], |row| {
        row.get::<_, String>(0)
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn rebuild_all(transaction: &Transaction<'_>, profile_id: &str) -> Result<()> {
    transaction.execute(
        "DELETE FROM usage_operation_rollups WHERE profile_id = ?1",
        [profile_id],
    )?;
    transaction.execute(
        "DELETE FROM usage_daily_rollups WHERE profile_id = ?1",
        [profile_id],
    )?;
    transaction.execute(
        "DELETE FROM usage_profile_rollups WHERE profile_id = ?1",
        [profile_id],
    )?;
    transaction.execute(
        "INSERT INTO usage_operation_rollups (
             profile_id, operation_key, exact_tokens, estimated_tokens,
             legacy_tokens, has_unknown
         )
         SELECT profile_id, operation_key,
                COALESCE(SUM(CASE WHEN counts_toward_totals = 1
                    AND measurement_source = 'provider_reported'
                    THEN COALESCE(total_tokens, 0) ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN counts_toward_totals = 1
                    AND measurement_source IN ('tokenizer_estimated', 'heuristic_estimated')
                    THEN COALESCE(total_tokens, 0) ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN counts_toward_totals = 1
                    AND measurement_source = 'legacy_migrated'
                    THEN COALESCE(total_tokens, 0) ELSE 0 END), 0),
                MAX(CASE WHEN counts_toward_totals = 1 AND total_tokens IS NULL
                    THEN 1 ELSE 0 END)
         FROM llm_usage_events WHERE profile_id = ?1
         GROUP BY profile_id, operation_key",
        [profile_id],
    )?;
    transaction.execute(
        &format!(
            "INSERT INTO usage_daily_rollups (
                 profile_id, local_date, total_tokens, input_tokens, output_tokens,
                 operation_count, exact_tokens, estimated_tokens, legacy_tokens,
                 unknown_operation_count, primary_model
             ) {}",
            daily_rollup_select("profile_id = ?1")
        ),
        [profile_id],
    )?;
    rebuild_profile(transaction, profile_id)
}

fn rebuild_operation(
    transaction: &Transaction<'_>,
    profile_id: &str,
    operation_key: &str,
) -> Result<()> {
    transaction.execute(
        "DELETE FROM usage_operation_rollups
         WHERE profile_id = ?1 AND operation_key = ?2",
        rusqlite::params![profile_id, operation_key],
    )?;
    transaction.execute(
        "INSERT INTO usage_operation_rollups (
             profile_id, operation_key, exact_tokens, estimated_tokens,
             legacy_tokens, has_unknown
         )
         SELECT profile_id, operation_key,
                COALESCE(SUM(CASE WHEN counts_toward_totals = 1
                    AND measurement_source = 'provider_reported'
                    THEN COALESCE(total_tokens, 0) ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN counts_toward_totals = 1
                    AND measurement_source IN ('tokenizer_estimated', 'heuristic_estimated')
                    THEN COALESCE(total_tokens, 0) ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN counts_toward_totals = 1
                    AND measurement_source = 'legacy_migrated'
                    THEN COALESCE(total_tokens, 0) ELSE 0 END), 0),
                MAX(CASE WHEN counts_toward_totals = 1 AND total_tokens IS NULL
                    THEN 1 ELSE 0 END)
         FROM llm_usage_events
         WHERE profile_id = ?1 AND operation_key = ?2
         GROUP BY profile_id, operation_key",
        rusqlite::params![profile_id, operation_key],
    )?;
    Ok(())
}

fn rebuild_day(transaction: &Transaction<'_>, profile_id: &str, local_date: &str) -> Result<()> {
    transaction.execute(
        "DELETE FROM usage_daily_rollups
         WHERE profile_id = ?1 AND local_date = ?2",
        rusqlite::params![profile_id, local_date],
    )?;
    transaction.execute(
        &format!(
            "INSERT INTO usage_daily_rollups (
                 profile_id, local_date, total_tokens, input_tokens, output_tokens,
                 operation_count, exact_tokens, estimated_tokens, legacy_tokens,
                 unknown_operation_count, primary_model
             ) {}",
            daily_rollup_select("profile_id = ?1 AND local_date = ?2")
        ),
        rusqlite::params![profile_id, local_date],
    )?;
    Ok(())
}

fn daily_rollup_select(predicate: &str) -> String {
    format!(
        "SELECT e.profile_id, e.local_date,
                SUM(e.total_tokens), SUM(e.input_tokens), SUM(e.output_tokens),
                COUNT(DISTINCT e.operation_key),
                COALESCE(SUM(CASE WHEN e.measurement_source = 'provider_reported'
                    THEN COALESCE(e.total_tokens, 0) ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN e.measurement_source IN
                    ('tokenizer_estimated', 'heuristic_estimated')
                    THEN COALESCE(e.total_tokens, 0) ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN e.measurement_source = 'legacy_migrated'
                    THEN COALESCE(e.total_tokens, 0) ELSE 0 END), 0),
                COUNT(DISTINCT CASE WHEN e.total_tokens IS NULL
                    THEN e.operation_key END),
                (SELECT e2.effective_model_id FROM llm_usage_events e2
                 WHERE e2.profile_id = e.profile_id
                   AND e2.local_date = e.local_date
                   AND e2.counts_toward_activity = 1
                   AND e2.effective_model_id IS NOT NULL
                 GROUP BY e2.effective_model_id
                 ORDER BY SUM(COALESCE(e2.total_tokens, 0)) DESC,
                          e2.effective_model_id ASC LIMIT 1)
         FROM llm_usage_events e
         WHERE e.counts_toward_activity = 1 AND e.{predicate}
         GROUP BY e.profile_id, e.local_date"
    )
}

fn rebuild_profile(transaction: &Transaction<'_>, profile_id: &str) -> Result<()> {
    transaction.execute(
        "INSERT INTO usage_profile_rollups (
             profile_id, exact_tokens, estimated_tokens, legacy_tokens,
             unknown_operation_count
         )
         SELECT ?1, COALESCE(SUM(exact_tokens), 0),
                COALESCE(SUM(estimated_tokens), 0),
                COALESCE(SUM(legacy_tokens), 0),
                COALESCE(SUM(has_unknown), 0)
         FROM usage_operation_rollups WHERE profile_id = ?1
         ON CONFLICT(profile_id) DO UPDATE SET
             exact_tokens = excluded.exact_tokens,
             estimated_tokens = excluded.estimated_tokens,
             legacy_tokens = excluded.legacy_tokens,
             unknown_operation_count = excluded.unknown_operation_count",
        [profile_id],
    )?;
    Ok(())
}
