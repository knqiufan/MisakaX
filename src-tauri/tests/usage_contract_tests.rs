use chrono::NaiveDate;
use misaka_x_lib::services::llm::TokenUsageInfo;
use misaka_x_lib::services::mcp::tool_loop::merge_token_usage;
use misaka_x_lib::services::usage::{
    calculate_streaks, DailyUsageV1, MeasurementSource, SidecarUsageEventV1, UsageDashboardV1,
    UsageMeasurement, UsageOperationKind, UsageOutcome, UsageOverviewV1, UsageQualityV1,
    UsageRangeV1, USAGE_DASHBOARD_SCHEMA_VERSION,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
}

fn usage(input: u64, output: u64, total: u64) -> TokenUsageInfo {
    TokenUsageInfo {
        input_tokens: input,
        output_tokens: output,
        total_tokens: total,
    }
}

#[test]
fn enum_wire_names_are_frozen() {
    assert_eq!(
        serde_json::to_value(MeasurementSource::ProviderReported).unwrap(),
        json!("provider_reported")
    );
    assert_eq!(
        serde_json::to_value(MeasurementSource::HeuristicEstimated).unwrap(),
        json!("heuristic_estimated")
    );
    assert_eq!(
        serde_json::to_value(UsageOperationKind::SessionTitle).unwrap(),
        json!("session_title")
    );
    assert_eq!(
        serde_json::to_value(UsageOutcome::Partial).unwrap(),
        json!("partial")
    );
}

#[test]
fn provider_total_is_authoritative_and_details_are_not_double_counted() {
    let measurement = UsageMeasurement {
        input_tokens: Some(100),
        output_tokens: Some(50),
        total_tokens: Some(140),
        cache_read_tokens: Some(70),
        cache_creation_tokens: Some(30),
        reasoning_tokens: Some(20),
        source: MeasurementSource::ProviderReported,
        estimator: None,
        provider_metadata: BTreeMap::new(),
    };
    assert_eq!(measurement.resolved_total().unwrap(), Some(140));
}

#[test]
fn missing_total_sums_input_and_output_with_overflow_check() {
    let measurement = UsageMeasurement {
        input_tokens: Some(42),
        output_tokens: Some(8),
        total_tokens: None,
        cache_read_tokens: None,
        cache_creation_tokens: None,
        reasoning_tokens: None,
        source: MeasurementSource::ProviderReported,
        estimator: None,
        provider_metadata: BTreeMap::new(),
    };
    assert_eq!(measurement.resolved_total().unwrap(), Some(50));

    let overflowing = UsageMeasurement {
        input_tokens: Some(u64::MAX),
        output_tokens: Some(1),
        ..measurement
    };
    assert!(overflowing.resolved_total().is_err());
}

#[test]
fn dashboard_contract_keeps_token_values_as_decimal_strings() {
    let unsafe_integer = "9007199254740993".to_string();
    let dashboard = UsageDashboardV1 {
        schema_version: USAGE_DASHBOARD_SCHEMA_VERSION,
        profile_id: "local-profile".to_string(),
        generated_at: "2026-08-13T00:00:00Z".to_string(),
        timezone_mode: "system".to_string(),
        timezone_id: Some("Asia/Shanghai".to_string()),
        utc_offset_minutes: 480,
        range: UsageRangeV1 {
            activity_start: "2025-08-14".to_string(),
            activity_end: "2026-08-13".to_string(),
            trend_start: "2026-07-15".to_string(),
            trend_end: "2026-08-13".to_string(),
        },
        overview: UsageOverviewV1 {
            total_tokens: unsafe_integer.clone(),
            exact_tokens: unsafe_integer.clone(),
            estimated_tokens: "0".to_string(),
            legacy_tokens: "0".to_string(),
            unknown_operation_count: 0,
            total_days: 1,
            current_streak: 1,
            longest_streak: 1,
        },
        daily_activity: vec![DailyUsageV1 {
            local_date: "2026-08-13".to_string(),
            total_tokens: Some(unsafe_integer.clone()),
            input_tokens: None,
            output_tokens: None,
            operation_count: 1,
            primary_model: Some("gpt-test".to_string()),
            quality: UsageQualityV1 {
                exact_tokens: unsafe_integer.clone(),
                estimated_tokens: "0".to_string(),
                legacy_tokens: "0".to_string(),
                unknown_operation_count: 0,
            },
        }],
        model_series: Vec::new(),
        other_series: None,
    };

    let value = serde_json::to_value(dashboard).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["overview"]["total_tokens"], unsafe_integer);
    assert!(value["overview"]["total_tokens"].is_string());
}

#[test]
fn sidecar_usage_v1_accepts_single_and_multi_run_fixtures() {
    for fixture in [
        json!({
            "schema_version": 1,
            "measurements": [{
                "run_id": "run-1",
                "model": "model-a",
                "input_tokens": 10,
                "output_tokens": 4,
                "total_tokens": 14,
                "cache_read_tokens": null,
                "cache_creation_tokens": null,
                "reasoning_tokens": null,
                "source": "provider_reported"
            }]
        }),
        json!({
            "schema_version": 1,
            "measurements": [
                {
                    "run_id": "run-tool",
                    "model": "model-a",
                    "input_tokens": 10,
                    "output_tokens": 4,
                    "total_tokens": 14,
                    "cache_read_tokens": 2,
                    "cache_creation_tokens": null,
                    "reasoning_tokens": null,
                    "source": "provider_reported"
                },
                {
                    "run_id": "run-research",
                    "model": "model-b",
                    "input_tokens": null,
                    "output_tokens": null,
                    "total_tokens": null,
                    "cache_read_tokens": null,
                    "cache_creation_tokens": null,
                    "reasoning_tokens": null,
                    "source": "unavailable"
                }
            ]
        }),
    ] {
        let parsed: SidecarUsageEventV1 = serde_json::from_value(fixture).unwrap();
        parsed.validate().unwrap();
    }
}

#[test]
fn sidecar_usage_v1_keeps_duplicate_run_fixtures_for_p2_deduplication() {
    let fixture = json!({
        "schema_version": 1,
        "measurements": [
            {
                "run_id": "duplicate-run",
                "model": "model-a",
                "input_tokens": 10,
                "output_tokens": 4,
                "total_tokens": 14,
                "cache_read_tokens": null,
                "cache_creation_tokens": null,
                "reasoning_tokens": null,
                "source": "provider_reported"
            },
            {
                "run_id": "duplicate-run",
                "model": "model-a",
                "input_tokens": 10,
                "output_tokens": 4,
                "total_tokens": 14,
                "cache_read_tokens": 2,
                "cache_creation_tokens": null,
                "reasoning_tokens": null,
                "source": "provider_reported"
            }
        ]
    });
    let parsed: SidecarUsageEventV1 = serde_json::from_value(fixture).unwrap();
    parsed.validate().unwrap();
    assert_eq!(parsed.measurements.len(), 2);
    assert_eq!(parsed.measurements[0].run_id, parsed.measurements[1].run_id);
}

#[test]
fn sidecar_usage_v1_rejects_bad_schema_missing_run_id_and_negative_values() {
    let bad_schema: SidecarUsageEventV1 = serde_json::from_value(json!({
        "schema_version": 2,
        "measurements": [{
            "run_id": "run-1",
            "model": null,
            "input_tokens": null,
            "output_tokens": null,
            "total_tokens": null,
            "cache_read_tokens": null,
            "cache_creation_tokens": null,
            "reasoning_tokens": null,
            "source": "unavailable"
        }]
    }))
    .unwrap();
    assert!(bad_schema.validate().is_err());

    let missing_run_id = serde_json::from_value::<SidecarUsageEventV1>(json!({
        "schema_version": 1,
        "measurements": [{
            "model": null,
            "input_tokens": null,
            "output_tokens": null,
            "total_tokens": null,
            "cache_read_tokens": null,
            "cache_creation_tokens": null,
            "reasoning_tokens": null,
            "source": "unavailable"
        }]
    }));
    assert!(missing_run_id.is_err());

    let negative = serde_json::from_value::<SidecarUsageEventV1>(json!({
        "schema_version": 1,
        "measurements": [{
            "run_id": "run-1",
            "model": null,
            "input_tokens": -1,
            "output_tokens": null,
            "total_tokens": null,
            "cache_read_tokens": null,
            "cache_creation_tokens": null,
            "reasoning_tokens": null,
            "source": "provider_reported"
        }]
    }));
    assert!(negative.is_err());
}

#[test]
fn current_rig_usage_merge_characterizes_single_multi_and_abort_paths() {
    let single = merge_token_usage(None, Some(usage(10, 5, 15))).unwrap();
    assert_eq!(single.total_tokens, 15);

    let multiple = merge_token_usage(Some(usage(10, 5, 15)), Some(usage(7, 3, 10))).unwrap();
    assert_eq!(multiple.input_tokens, 17);
    assert_eq!(multiple.output_tokens, 8);
    assert_eq!(multiple.total_tokens, 25);

    let partial_abort = merge_token_usage(Some(usage(10, 5, 15)), None).unwrap();
    assert_eq!(partial_abort.total_tokens, 15);
    assert!(merge_token_usage(None, None).is_none());
}

#[test]
fn streak_contract_covers_today_yesterday_gaps_years_and_leap_day() {
    let cases: Vec<(&str, Vec<&str>, (u32, u32))> = vec![
        ("2026-08-13", vec![], (0, 0)),
        ("2026-08-13", vec!["2026-08-13"], (1, 1)),
        (
            "2026-08-13",
            vec!["2026-08-10", "2026-08-11", "2026-08-12"],
            (3, 3),
        ),
        (
            "2026-08-13",
            vec!["2026-08-09", "2026-08-10", "2026-08-11"],
            (0, 3),
        ),
        (
            "2026-01-01",
            vec!["2025-12-30", "2025-12-31", "2026-01-01"],
            (3, 3),
        ),
        (
            "2024-03-01",
            vec!["2024-02-28", "2024-02-29", "2024-03-01"],
            (3, 3),
        ),
    ];

    for (today, dates, expected) in cases {
        let summary = calculate_streaks(dates.into_iter().map(date), date(today));
        assert_eq!((summary.current, summary.longest), expected);
    }
}

#[test]
fn sidecar_baseline_fixture_documents_missing_usage_before_p2() {
    let current_done_event: Value = json!({"finished": true});
    assert!(current_done_event.get("usage").is_none());
}
