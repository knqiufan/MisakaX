use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

use anyhow::{Context, Result};
use chrono::{Local, NaiveDate, SecondsFormat, Utc};
use rusqlite::Connection;

use crate::db::models::UsageEvent;
use crate::db::repository::{ProfileRepo, UsageRepo};

use super::rollup::refresh_usage_rollups;
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
    refresh_usage_rollups(conn, &profile.profile_id)?;
    let activity_dates = local_date_sequence(today, query.activity_days);
    let trend_dates = local_date_sequence(today, query.trend_days);
    let trend_started = Instant::now();
    let trend_events = UsageRepo::list_for_profile_since(
        conn,
        &profile.profile_id,
        &trend_dates.first().unwrap().to_string(),
    )?;
    let (model_series, other_series) = build_trend(&trend_events, &trend_dates, query.max_series)?;
    let trend_ms = trend_started.elapsed().as_millis() as u64;
    let overview_started = Instant::now();
    let overview = query_overview(conn, &profile.profile_id, today)?;
    let overview_ms = overview_started.elapsed().as_millis() as u64;
    let activity_started = Instant::now();
    let daily_activity = query_activity(conn, &profile.profile_id, &activity_dates)?;
    let activity_ms = activity_started.elapsed().as_millis() as u64;

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
        overview,
        daily_activity,
        model_series,
        other_series,
    };

    tracing::info!(
        duration_ms = started.elapsed().as_millis() as u64,
        overview_ms,
        activity_ms,
        trend_ms,
        trend_event_count = trend_events.len(),
        "Built usage dashboard snapshot"
    );
    Ok(dashboard)
}

fn query_overview(
    conn: &Connection,
    profile_id: &str,
    today: NaiveDate,
) -> Result<UsageOverviewV1> {
    let (exact, estimated, legacy, unknown_operation_count): (i64, i64, i64, i64) = conn
        .query_row(
            "SELECT exact_tokens, estimated_tokens, legacy_tokens,
                    unknown_operation_count
             FROM usage_profile_rollups WHERE profile_id = ?1",
            [profile_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
    let exact = nonnegative_u64(exact, "exact overview total")?;
    let estimated = nonnegative_u64(estimated, "estimated overview total")?;
    let legacy = nonnegative_u64(legacy, "legacy overview total")?;
    let unknown_operation_count =
        nonnegative_u64(unknown_operation_count, "unknown overview operation count")?;
    let mut statement = conn.prepare(
        "SELECT local_date FROM usage_daily_rollups
         WHERE profile_id = ?1 ORDER BY local_date",
    )?;
    let activity_dates = statement
        .query_map([profile_id], |row| row.get::<_, String>(0))?
        .filter_map(|row| row.ok().and_then(|value| parse_date(&value)))
        .collect::<Vec<_>>();
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

#[derive(Debug)]
struct ActivityAggregate {
    total_tokens: Option<u64>,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    operation_count: u64,
    exact_tokens: u64,
    estimated_tokens: u64,
    legacy_tokens: u64,
    unknown_operation_count: u64,
    primary_model: Option<String>,
}

fn query_activity(
    conn: &Connection,
    profile_id: &str,
    dates: &[NaiveDate],
) -> Result<Vec<DailyUsageV1>> {
    let start = dates
        .first()
        .context("activity range is empty")?
        .to_string();
    let end = dates.last().context("activity range is empty")?.to_string();
    let mut statement = conn.prepare(
        "SELECT local_date, total_tokens, input_tokens, output_tokens,
                operation_count, exact_tokens, estimated_tokens, legacy_tokens,
                unknown_operation_count, primary_model
         FROM usage_daily_rollups
         WHERE profile_id = ?1 AND local_date BETWEEN ?2 AND ?3
         ORDER BY local_date",
    )?;
    let rows = statement.query_map(rusqlite::params![profile_id, start, end], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<i64>>(1)?,
            row.get::<_, Option<i64>>(2)?,
            row.get::<_, Option<i64>>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, i64>(7)?,
            row.get::<_, i64>(8)?,
            row.get::<_, Option<String>>(9)?,
        ))
    })?;
    let mut aggregates = HashMap::new();
    for row in rows {
        let (date, total, input, output, operations, exact, estimated, legacy, unknown, primary) =
            row?;
        aggregates.insert(
            date,
            ActivityAggregate {
                total_tokens: optional_nonnegative_u64(total, "daily total")?,
                input_tokens: optional_nonnegative_u64(input, "daily input")?,
                output_tokens: optional_nonnegative_u64(output, "daily output")?,
                operation_count: nonnegative_u64(operations, "daily operation count")?,
                exact_tokens: nonnegative_u64(exact, "daily exact total")?,
                estimated_tokens: nonnegative_u64(estimated, "daily estimated total")?,
                legacy_tokens: nonnegative_u64(legacy, "daily legacy total")?,
                unknown_operation_count: nonnegative_u64(unknown, "daily unknown count")?,
                primary_model: primary,
            },
        );
    }

    Ok(dates
        .iter()
        .map(|date| {
            let local_date = date.to_string();
            let aggregate = aggregates.remove(&local_date);
            DailyUsageV1 {
                local_date,
                total_tokens: aggregate
                    .as_ref()
                    .and_then(|value| value.total_tokens)
                    .map(|value| value.to_string()),
                input_tokens: aggregate
                    .as_ref()
                    .and_then(|value| value.input_tokens)
                    .map(|value| value.to_string()),
                output_tokens: aggregate
                    .as_ref()
                    .and_then(|value| value.output_tokens)
                    .map(|value| value.to_string()),
                operation_count: aggregate.as_ref().map_or(0, |value| value.operation_count),
                primary_model: aggregate
                    .as_ref()
                    .and_then(|value| value.primary_model.clone()),
                quality: UsageQualityV1 {
                    exact_tokens: aggregate
                        .as_ref()
                        .map_or(0, |value| value.exact_tokens)
                        .to_string(),
                    estimated_tokens: aggregate
                        .as_ref()
                        .map_or(0, |value| value.estimated_tokens)
                        .to_string(),
                    legacy_tokens: aggregate
                        .as_ref()
                        .map_or(0, |value| value.legacy_tokens)
                        .to_string(),
                    unknown_operation_count: aggregate
                        .as_ref()
                        .map_or(0, |value| value.unknown_operation_count),
                },
            }
        })
        .collect())
}

fn nonnegative_u64(value: i64, field: &str) -> Result<u64> {
    u64::try_from(value).with_context(|| format!("{field} was negative"))
}

fn optional_nonnegative_u64(value: Option<i64>, field: &str) -> Result<Option<u64>> {
    value.map(|value| nonnegative_u64(value, field)).transpose()
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
