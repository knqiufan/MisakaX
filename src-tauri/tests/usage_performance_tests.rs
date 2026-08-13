use std::time::{Duration, Instant};

use chrono::NaiveDate;
use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::repository::ProfileRepo;
use misaka_x_lib::services::usage::query::{get_dashboard_at, DashboardQuery};
use rusqlite::Connection;

const TARGET_P95: Duration = Duration::from_millis(100);

fn fixture(event_count: usize) -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=MEMORY;")
        .unwrap();
    run_migrations(&conn).unwrap();
    let profile = ProfileRepo::ensure_default(&conn).unwrap();
    conn.execute(
        "WITH digits(value) AS (
             VALUES (0),(1),(2),(3),(4),(5),(6),(7),(8),(9)
         ), numbers(value) AS (
             SELECT a.value + 10*b.value + 100*c.value + 1000*d.value + 10000*e.value
             FROM digits a CROSS JOIN digits b CROSS JOIN digits c
             CROSS JOIN digits d CROSS JOIN digits e
         )
         INSERT INTO llm_usage_events (
             event_id, profile_id, operation_key, measurement_key, operation_kind,
             provider_config_id, provider_id, effective_model_id, model_display_name,
             input_tokens, output_tokens, total_tokens, measurement_source, outcome,
             counts_toward_totals, counts_toward_activity, counts_toward_trend,
             occurred_at_utc, local_date, utc_offset_minutes, metadata_json
         )
         SELECT
             printf('event-%d', value), ?1, printf('operation-%d', value),
             printf('measurement-%d', value), 'chat',
             printf('config-%d', value % 8), 'provider',
             printf('model-%d', value % 8), printf('Model %d', value % 8),
             CASE WHEN value % 11 = 0 THEN NULL ELSE (value % 400) + 1 END,
             CASE WHEN value % 11 = 0 THEN NULL ELSE (value % 200) + 1 END,
             CASE WHEN value % 11 = 0 THEN NULL ELSE (value % 600) + 2 END,
             CASE value % 5
                 WHEN 0 THEN 'provider_reported'
                 WHEN 1 THEN 'tokenizer_estimated'
                 WHEN 2 THEN 'heuristic_estimated'
                 WHEN 3 THEN 'legacy_migrated'
                 ELSE 'unavailable'
             END,
             'completed', 1, 1, 1,
             date('2026-08-13', printf('-%d day', value % 730)) || 'T08:00:00Z',
             date('2026-08-13', printf('-%d day', value % 730)),
             480, '{}'
         FROM numbers WHERE value < ?2",
        rusqlite::params![profile.profile_id, event_count as i64],
    )
    .unwrap();
    conn
}

fn p95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    let index = ((samples.len() as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(samples.len() - 1);
    samples[index]
}

#[test]
fn dashboard_query_p95_stays_under_100ms_for_1k_10k_and_100k_events() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    let today = NaiveDate::from_ymd_opt(2026, 8, 13).unwrap();
    for event_count in [1_000, 10_000, 100_000] {
        let conn = fixture(event_count);
        get_dashboard_at(&conn, DashboardQuery::default(), today).unwrap();
        let mut samples = Vec::with_capacity(7);
        for _ in 0..7 {
            let started = Instant::now();
            let dashboard = get_dashboard_at(&conn, DashboardQuery::default(), today).unwrap();
            assert_eq!(dashboard.daily_activity.len(), 365);
            assert_eq!(dashboard.model_series.len(), 5);
            samples.push(started.elapsed());
        }
        let measured_p95 = p95(&mut samples);
        eprintln!("usage dashboard fixture={event_count} p95={measured_p95:?}");
        assert!(
            measured_p95 < TARGET_P95,
            "{event_count} event dashboard p95 {measured_p95:?} exceeded {TARGET_P95:?}"
        );
    }
}

#[test]
fn dashboard_queries_use_profile_date_and_model_indexes() {
    let conn = fixture(1_000);
    let activity_plan: Vec<String> = conn
        .prepare(
            "EXPLAIN QUERY PLAN SELECT local_date, SUM(total_tokens)
             FROM llm_usage_events
             WHERE profile_id = ?1 AND counts_toward_activity = 1 AND local_date >= ?2
             GROUP BY local_date",
        )
        .unwrap()
        .query_map(["unused-profile", "2025-08-14"], |row| row.get(3))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert!(activity_plan
        .iter()
        .any(|detail| detail.contains("idx_usage_profile_date")));

    let model_plan: Vec<String> = conn
        .prepare(
            "EXPLAIN QUERY PLAN SELECT local_date, SUM(total_tokens)
             FROM llm_usage_events
             WHERE profile_id = ?1 AND counts_toward_trend = 1
               AND provider_config_id = ?2 AND effective_model_id = ?3
               AND local_date >= ?4
             GROUP BY local_date",
        )
        .unwrap()
        .query_map(
            ["unused-profile", "config-1", "model-1", "2026-07-15"],
            |row| row.get(3),
        )
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert!(model_plan
        .iter()
        .any(|detail| detail.contains("idx_usage_profile_model_date")));
}
