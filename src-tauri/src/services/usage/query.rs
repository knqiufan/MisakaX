use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::time::Instant;

use anyhow::{Context, Result};
use chrono::{Local, NaiveDate, SecondsFormat, Utc};
use rusqlite::Connection;

use crate::db::models::UsageEvent;
use crate::db::repository::{ProfileRepo, UsageRepo};

use super::{
    calculate_streaks, local_date_sequence, DailyUsageV1, MeasurementSource, ModelUsagePointV1,
    ModelUsageSeriesV1, UsageDashboardV1, UsageOverviewV1, UsageQualityV1, UsageRangeV1,
    USAGE_DASHBOARD_SCHEMA_VERSION,
};

pub const DEFAULT_ACTIVITY_DAYS: u32 = 365;
pub const DEFAULT_TREND_DAYS: u32 = 30;
pub const DEFAULT_MAX_SERIES: u32 = 5;
pub const MAX_ACTIVITY_DAYS: u32 = 366;
pub const MAX_TREND_DAYS: u32 = 90;
pub const MAX_MODEL_SERIES: u32 = 10;

#[derive(Debug, Clone, Copy)]
pub struct DashboardQuery {
    pub activity_days: u32,
    pub trend_days: u32,
    pub max_series: u32,
}

impl Default for DashboardQuery {
    fn default() -> Self {
        Self {
            activity_days: DEFAULT_ACTIVITY_DAYS,
            trend_days: DEFAULT_TREND_DAYS,
            max_series: DEFAULT_MAX_SERIES,
        }
    }
}

impl DashboardQuery {
    pub fn validate(self) -> Result<Self> {
        if !(1..=MAX_ACTIVITY_DAYS).contains(&self.activity_days) {
            anyhow::bail!("activity_days must be between 1 and {MAX_ACTIVITY_DAYS}");
        }
        if !(1..=MAX_TREND_DAYS).contains(&self.trend_days) {
            anyhow::bail!("trend_days must be between 1 and {MAX_TREND_DAYS}");
        }
        if !(1..=MAX_MODEL_SERIES).contains(&self.max_series) {
            anyhow::bail!("max_series must be between 1 and {MAX_MODEL_SERIES}");
        }
        Ok(self)
    }
}

pub fn get_dashboard(conn: &Connection, query: DashboardQuery) -> Result<UsageDashboardV1> {
    get_dashboard_at(conn, query, Local::now().date_naive())
}

pub fn get_dashboard_at(
    conn: &Connection,
    query: DashboardQuery,
    today: NaiveDate,
) -> Result<UsageDashboardV1> {
    let started = Instant::now();
    let query = query.validate()?;
    let profile = ProfileRepo::get_current(conn)?;
    let events = UsageRepo::list_for_profile(conn, &profile.profile_id)?;
    let activity_dates = local_date_sequence(today, query.activity_days);
    let trend_dates = local_date_sequence(today, query.trend_days);
    let (model_series, other_series) = build_trend(&events, &trend_dates, query.max_series)?;

    let dashboard = UsageDashboardV1 {
        schema_version: USAGE_DASHBOARD_SCHEMA_VERSION,
        profile_id: profile.profile_id.clone(),
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        timezone_mode: profile.timezone_mode.clone(),
        timezone_id: profile.timezone_id.clone(),
        utc_offset_minutes: Local::now().offset().local_minus_utc() / 60,
        range: UsageRangeV1 {
            activity_start: activity_dates.first().unwrap().to_string(),
            activity_end: activity_dates.last().unwrap().to_string(),
            trend_start: trend_dates.first().unwrap().to_string(),
            trend_end: trend_dates.last().unwrap().to_string(),
        },
        overview: build_overview(&events, today)?,
        daily_activity: build_activity(&events, &activity_dates)?,
        model_series,
        other_series,
    };

    tracing::info!(
        duration_ms = started.elapsed().as_millis() as u64,
        event_count = events.len(),
        "Built usage dashboard snapshot"
    );
    Ok(dashboard)
}

fn build_overview(events: &[UsageEvent], today: NaiveDate) -> Result<UsageOverviewV1> {
    let totals_events: Vec<&UsageEvent> = events
        .iter()
        .filter(|event| event.counts_toward_totals)
        .collect();
    let exact = sum_by_source(&totals_events, |source| {
        source == MeasurementSource::ProviderReported
    })?;
    let estimated = sum_by_source(&totals_events, MeasurementSource::is_estimated)?;
    let legacy = sum_by_source(&totals_events, |source| {
        source == MeasurementSource::LegacyMigrated
    })?;
    let unknown_operation_count = distinct_unknown_operations(&totals_events) as u64;
    let activity_dates: BTreeSet<NaiveDate> = events
        .iter()
        .filter(|event| event.counts_toward_activity)
        .filter_map(|event| parse_date(&event.local_date))
        .collect();
    let streak = calculate_streaks(activity_dates.iter().copied(), today);
    let total = exact
        .checked_add(estimated)
        .and_then(|value| value.checked_add(legacy))
        .context("overview total overflowed u64")?;

    Ok(UsageOverviewV1 {
        total_tokens: total.to_string(),
        exact_tokens: exact.to_string(),
        estimated_tokens: estimated.to_string(),
        legacy_tokens: legacy.to_string(),
        unknown_operation_count,
        total_days: u32::try_from(activity_dates.len()).unwrap_or(u32::MAX),
        current_streak: streak.current,
        longest_streak: streak.longest,
    })
}

fn build_activity(events: &[UsageEvent], dates: &[NaiveDate]) -> Result<Vec<DailyUsageV1>> {
    let allowed: HashSet<String> = dates.iter().map(ToString::to_string).collect();
    let mut by_date: HashMap<&str, Vec<&UsageEvent>> = HashMap::new();
    for event in events
        .iter()
        .filter(|event| event.counts_toward_activity && allowed.contains(event.local_date.as_str()))
    {
        by_date.entry(&event.local_date).or_default().push(event);
    }

    dates
        .iter()
        .map(|date| {
            let key = date.to_string();
            let day = by_date.get(key.as_str()).cloned().unwrap_or_default();
            let operations: HashSet<&str> = day
                .iter()
                .map(|event| event.operation_key.as_str())
                .collect();
            let total_tokens = sum_known(&day, |event| event.total_tokens)?;
            let input_tokens = sum_known(&day, |event| event.input_tokens)?;
            let output_tokens = sum_known(&day, |event| event.output_tokens)?;
            let exact =
                sum_by_source(&day, |source| source == MeasurementSource::ProviderReported)?;
            let estimated = sum_by_source(&day, MeasurementSource::is_estimated)?;
            let legacy = sum_by_source(&day, |source| source == MeasurementSource::LegacyMigrated)?;
            Ok(DailyUsageV1 {
                local_date: key,
                total_tokens: total_tokens.map(|value| value.to_string()),
                input_tokens: input_tokens.map(|value| value.to_string()),
                output_tokens: output_tokens.map(|value| value.to_string()),
                operation_count: operations.len() as u64,
                primary_model: primary_model(&day),
                quality: UsageQualityV1 {
                    exact_tokens: exact.to_string(),
                    estimated_tokens: estimated.to_string(),
                    legacy_tokens: legacy.to_string(),
                    unknown_operation_count: distinct_unknown_operations(&day) as u64,
                },
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SeriesKey {
    provider_config_id: Option<String>,
    provider_id: Option<String>,
    model: String,
}

fn build_trend(
    events: &[UsageEvent],
    dates: &[NaiveDate],
    max_series: u32,
) -> Result<(Vec<ModelUsageSeriesV1>, Option<ModelUsageSeriesV1>)> {
    let allowed: HashSet<String> = dates.iter().map(ToString::to_string).collect();
    let mut grouped: BTreeMap<SeriesKey, Vec<&UsageEvent>> = BTreeMap::new();
    for event in events
        .iter()
        .filter(|event| event.counts_toward_trend && allowed.contains(event.local_date.as_str()))
    {
        let Some(model) = event.effective_model_id.as_ref() else {
            continue;
        };
        grouped
            .entry(SeriesKey {
                provider_config_id: event.provider_config_id.clone(),
                provider_id: event.provider_id.clone(),
                model: model.clone(),
            })
            .or_default()
            .push(event);
    }

    let mut ranked: Vec<(SeriesKey, Vec<&UsageEvent>)> = grouped.into_iter().collect();
    ranked.sort_by_key(|(key, events)| {
        let known = events
            .iter()
            .filter_map(|event| event.total_tokens)
            .fold(0_u64, u64::saturating_add);
        let operations = events
            .iter()
            .map(|event| event.operation_key.as_str())
            .collect::<HashSet<_>>()
            .len();
        (Reverse(known), Reverse(operations), key.clone())
    });

    let split = ranked.len().min(max_series as usize);
    let other_events: Vec<&UsageEvent> = ranked[split..]
        .iter()
        .flat_map(|(_, events)| events.iter().copied())
        .collect();
    let series = ranked[..split]
        .iter()
        .map(|(key, events)| series_from_events(key, events, dates))
        .collect::<Result<Vec<_>>>()?;
    let other_series = (!other_events.is_empty())
        .then(|| {
            series_from_events(
                &SeriesKey {
                    provider_config_id: None,
                    provider_id: None,
                    model: "__other__".to_string(),
                },
                &other_events,
                dates,
            )
        })
        .transpose()?;
    Ok((series, other_series))
}

fn series_from_events(
    key: &SeriesKey,
    events: &[&UsageEvent],
    dates: &[NaiveDate],
) -> Result<ModelUsageSeriesV1> {
    let mut by_date: HashMap<&str, Vec<&UsageEvent>> = HashMap::new();
    for event in events {
        by_date.entry(&event.local_date).or_default().push(event);
    }
    let points = dates
        .iter()
        .map(|date| {
            let date = date.to_string();
            let day = by_date.get(date.as_str()).cloned().unwrap_or_default();
            let has_calls = !day.is_empty();
            let known = sum_known(&day, |event| event.total_tokens)?;
            Ok(ModelUsagePointV1 {
                local_date: date,
                total_tokens: match (has_calls, known) {
                    (false, _) => Some("0".to_string()),
                    (true, Some(value)) => Some(value.to_string()),
                    (true, None) => None,
                },
                unknown_operation_count: distinct_unknown_operations(&day) as u64,
                estimated_tokens: sum_by_source(&day, MeasurementSource::is_estimated)?.to_string(),
                legacy_tokens: sum_by_source(&day, |source| {
                    source == MeasurementSource::LegacyMigrated
                })?
                .to_string(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let series_key = format!(
        "{}:{}:{}",
        key.provider_config_id.as_deref().unwrap_or("unknown"),
        key.provider_id.as_deref().unwrap_or("unknown"),
        key.model
    );
    let display_name = if key.model == "__other__" {
        "Other models".to_string()
    } else if let Some(provider) = key.provider_id.as_deref() {
        format!("{} · {provider}", key.model)
    } else {
        key.model.clone()
    };
    Ok(ModelUsageSeriesV1 {
        series_key,
        display_name,
        provider_config_id: key.provider_config_id.clone(),
        provider_id: key.provider_id.clone(),
        effective_model_id: key.model.clone(),
        points,
    })
}

fn primary_model(events: &[&UsageEvent]) -> Option<String> {
    let mut totals: BTreeMap<&str, u64> = BTreeMap::new();
    for event in events {
        if let Some(model) = event.effective_model_id.as_deref() {
            let total = totals.entry(model).or_default();
            *total = total.saturating_add(event.total_tokens.unwrap_or(0));
        }
    }
    totals
        .into_iter()
        .max_by_key(|(model, total)| (*total, Reverse(*model)))
        .map(|(model, _)| model.to_string())
}

fn sum_by_source<T>(events: &[T], predicate: impl Fn(MeasurementSource) -> bool) -> Result<u64>
where
    T: std::borrow::Borrow<UsageEvent>,
{
    events.iter().try_fold(0_u64, |acc, event| {
        let event = event.borrow();
        if predicate(event.measurement_source) {
            acc.checked_add(event.total_tokens.unwrap_or(0))
                .context("usage total overflowed u64")
        } else {
            Ok(acc)
        }
    })
}

fn sum_known<T>(events: &[T], field: impl Fn(&UsageEvent) -> Option<u64>) -> Result<Option<u64>>
where
    T: std::borrow::Borrow<UsageEvent>,
{
    let mut total = None;
    for event in events {
        if let Some(value) = field(event.borrow()) {
            total = Some(
                total
                    .unwrap_or(0_u64)
                    .checked_add(value)
                    .context("usage field overflowed u64")?,
            );
        }
    }
    Ok(total)
}

fn distinct_unknown_operations<T>(events: &[T]) -> usize
where
    T: std::borrow::Borrow<UsageEvent>,
{
    events
        .iter()
        .filter_map(|event| {
            let event = event.borrow();
            event
                .total_tokens
                .is_none()
                .then_some(event.operation_key.as_str())
        })
        .collect::<HashSet<_>>()
        .len()
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}
