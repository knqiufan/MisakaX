use std::collections::BTreeMap;

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat, Utc};
use rusqlite::Connection;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};

use crate::db::models::NewUsageEvent;
use crate::db::repository::{MessageRepo, ProfileRepo, RouterConfigRepo, SessionRepo, UsageRepo};

use super::collector::aggregate_captures;
use super::{MeasurementSource, UsageCapture, UsageOperationKind, UsageOutcome};

#[derive(Debug, Clone)]
pub struct FinalizeTurnRequest {
    pub operation_key: String,
    pub operation_kind: UsageOperationKind,
    pub session_id: Option<String>,
    pub message_id: Option<String>,
    pub selected_model_id: Option<String>,
    pub effective_provider_config_id: Option<String>,
    pub effective_model_id: Option<String>,
    pub vendor_id: Option<String>,
    pub content: String,
    pub thinking: String,
    pub tool_calls_json: Option<String>,
    pub captures: Vec<UsageCapture>,
    pub was_aborted: bool,
    pub stream_error: Option<String>,
    pub session_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UsageRecordedPayload {
    pub profile_id: String,
    pub operation_key: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone)]
pub struct FinalizeTurnOutcome {
    pub inserted_count: usize,
    pub outcome: UsageOutcome,
    pub recorded: UsageRecordedPayload,
}

#[derive(Debug, Serialize)]
struct MessageUsageEnvelope<'a> {
    schema_version: u16,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    total_tokens: Option<u64>,
    cache_read_tokens: Option<u64>,
    cache_creation_tokens: Option<u64>,
    reasoning_tokens: Option<u64>,
    measurement_source: MeasurementSource,
    estimator_id: Option<&'a str>,
    estimator_version: Option<&'a str>,
    measurements: &'a [UsageCapture],
}

#[derive(Debug)]
struct CaptureGroup {
    model: Option<String>,
    source: MeasurementSource,
    estimator_id: Option<String>,
    estimator_version: Option<String>,
    captures: Vec<UsageCapture>,
}

pub fn finalize_turn(
    conn: &mut Connection,
    request: &FinalizeTurnRequest,
) -> Result<FinalizeTurnOutcome> {
    let local_now = Local::now();
    let occurred_at = local_now
        .with_timezone(&Utc)
        .to_rfc3339_opts(SecondsFormat::Millis, true);
    let local_date = local_now.date_naive().format("%Y-%m-%d").to_string();
    let utc_offset_minutes = local_now.offset().local_minus_utc() / 60;
    let outcome = classify_outcome(request);
    let aggregate = aggregate_captures(&request.captures)?;
    let message_usage_json = serde_json::to_string(&MessageUsageEnvelope {
        schema_version: 1,
        input_tokens: aggregate.input_tokens,
        output_tokens: aggregate.output_tokens,
        total_tokens: aggregate.resolved_total().map_err(anyhow::Error::msg)?,
        cache_read_tokens: aggregate.cache_read_tokens,
        cache_creation_tokens: aggregate.cache_creation_tokens,
        reasoning_tokens: aggregate.reasoning_tokens,
        measurement_source: aggregate.source,
        estimator_id: aggregate.estimator.as_ref().map(|value| value.id.as_str()),
        estimator_version: aggregate
            .estimator
            .as_ref()
            .map(|value| value.version.as_str()),
        measurements: &request.captures,
    })?;

    let transaction = conn.transaction()?;
    let profile = ProfileRepo::get_current(&transaction)?;
    let router = request
        .effective_provider_config_id
        .as_deref()
        .map(|id| RouterConfigRepo::find_by_id(&transaction, id))
        .transpose()?;
    let provider_id = router.as_ref().map(|value| value.provider.clone());
    let timezone_id = profile.timezone_id.clone();
    let groups = group_captures(&request.captures, request.effective_model_id.as_deref());
    let mut events = Vec::with_capacity(groups.len());

    for group in groups {
        let measurement = aggregate_captures(&group.captures)?;
        let model = group
            .model
            .clone()
            .or_else(|| request.effective_model_id.clone());
        let series_hash = series_hash(
            request.effective_provider_config_id.as_deref(),
            provider_id.as_deref(),
            model.as_deref(),
        );
        let measurement_key = format!(
            "{}:model:{}:source:{}",
            request.operation_key,
            series_hash,
            group.source.as_str()
        );
        let metadata_json = serde_json::to_string(&json!({
            "capture_ids": group.captures.iter().map(|capture| &capture.capture_id).collect::<Vec<_>>(),
            "provider_metadata": group.captures.iter().map(|capture| &capture.measurement.provider_metadata).collect::<Vec<_>>(),
            "stream_error": request.stream_error.as_deref(),
        }))?;
        events.push(NewUsageEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            profile_id: profile.profile_id.clone(),
            operation_key: request.operation_key.clone(),
            measurement_key,
            operation_kind: request.operation_kind,
            session_id: request.session_id.clone(),
            message_id: request.message_id.clone(),
            provider_config_id: request.effective_provider_config_id.clone(),
            provider_id: provider_id.clone(),
            vendor_id: request.vendor_id.clone(),
            selected_model_id: request.selected_model_id.clone(),
            effective_model_id: model.clone(),
            model_display_name: model,
            input_tokens: measurement.input_tokens,
            output_tokens: measurement.output_tokens,
            total_tokens: measurement.resolved_total().map_err(anyhow::Error::msg)?,
            cache_read_tokens: measurement.cache_read_tokens,
            cache_creation_tokens: measurement.cache_creation_tokens,
            reasoning_tokens: measurement.reasoning_tokens,
            measurement_source: group.source,
            estimator_id: group.estimator_id,
            estimator_version: group.estimator_version,
            outcome,
            counts_toward_totals: !matches!(request.operation_kind, UsageOperationKind::ModelProbe),
            counts_toward_activity: matches!(
                request.operation_kind,
                UsageOperationKind::Chat
                    | UsageOperationKind::Research
                    | UsageOperationKind::ToolRound
            ),
            counts_toward_trend: !matches!(request.operation_kind, UsageOperationKind::ModelProbe),
            occurred_at_utc: occurred_at.clone(),
            local_date: local_date.clone(),
            timezone_id: timezone_id.clone(),
            utc_offset_minutes,
            metadata_json,
            source_installation_id: None,
            source_event_id: None,
        });
    }

    if let Some(message_id) = request.message_id.as_deref() {
        let thinking = (!request.thinking.is_empty()).then_some(request.thinking.as_str());
        MessageRepo::update_assistant_content(
            &transaction,
            message_id,
            &request.content,
            thinking,
            Some(&message_usage_json),
            request.was_aborted,
            request.tool_calls_json.as_deref(),
        )?;
        if request.stream_error.is_some() {
            MessageRepo::update_status(&transaction, message_id, "error")?;
        }
    }

    if let (Some(session_id), Some(title)) = (
        request.session_id.as_deref(),
        request.session_title.as_deref(),
    ) {
        SessionRepo::update(&transaction, session_id, Some(title), None, None, None)?;
    }

    let inserted = UsageRepo::insert_batch_idempotent(&transaction, &events)?;
    let projected: Vec<NewUsageEvent> = inserted
        .iter()
        .filter(|event| event.counts_toward_totals)
        .cloned()
        .collect();
    if !projected.is_empty() {
        if let Some(session_id) = request.session_id.as_deref() {
            let input = sum_event_tokens(&projected, |event| event.input_tokens)?;
            let output = sum_event_tokens(&projected, |event| event.output_tokens)?;
            SessionRepo::update_stats(&transaction, session_id, Some(input), Some(output))?;
        }
    }
    transaction.commit()?;

    let source_counts = |source: MeasurementSource| {
        inserted
            .iter()
            .filter(|event| event.measurement_source == source)
            .count()
    };
    tracing::info!(
        inserted_count = inserted.len(),
        capture_count = request.captures.len(),
        provider_reported_count = source_counts(MeasurementSource::ProviderReported),
        tokenizer_estimated_count = source_counts(MeasurementSource::TokenizerEstimated),
        heuristic_estimated_count = source_counts(MeasurementSource::HeuristicEstimated),
        legacy_migrated_count = source_counts(MeasurementSource::LegacyMigrated),
        unavailable_count = source_counts(MeasurementSource::Unavailable),
        "Finalized assistant usage operation"
    );
    Ok(FinalizeTurnOutcome {
        inserted_count: inserted.len(),
        outcome,
        recorded: UsageRecordedPayload {
            profile_id: profile.profile_id,
            operation_key: request.operation_key.clone(),
            occurred_at,
        },
    })
}

pub fn emit_usage_recorded(app: &AppHandle, payload: &UsageRecordedPayload) {
    if let Err(error) = app.emit("usage:recorded", payload) {
        tracing::warn!(error = %error, "Failed to emit usage:recorded after commit");
    }
}

fn classify_outcome(request: &FinalizeTurnRequest) -> UsageOutcome {
    if request.was_aborted {
        UsageOutcome::Aborted
    } else if request.stream_error.is_some() {
        if request.content.is_empty()
            && request.captures.iter().all(|capture| {
                capture
                    .measurement
                    .resolved_total()
                    .ok()
                    .flatten()
                    .is_none()
            })
        {
            UsageOutcome::Failed
        } else {
            UsageOutcome::Partial
        }
    } else {
        UsageOutcome::Completed
    }
}

fn group_captures(captures: &[UsageCapture], fallback_model: Option<&str>) -> Vec<CaptureGroup> {
    let mut groups: BTreeMap<String, CaptureGroup> = BTreeMap::new();
    for capture in captures {
        let model = capture
            .model
            .as_deref()
            .or(fallback_model)
            .map(ToString::to_string);
        let estimator_id = capture.measurement.estimator.as_ref().map(|e| e.id.clone());
        let estimator_version = capture
            .measurement
            .estimator
            .as_ref()
            .map(|e| e.version.clone());
        let key = format!(
            "{}|{}",
            model.as_deref().unwrap_or("unknown"),
            capture.measurement.source.as_str()
        );
        groups
            .entry(key)
            .and_modify(|group| {
                if group.estimator_id != estimator_id {
                    group.estimator_id = None;
                }
                if group.estimator_version != estimator_version {
                    group.estimator_version = None;
                }
                group.captures.push(capture.clone());
            })
            .or_insert_with(|| CaptureGroup {
                model,
                source: capture.measurement.source,
                estimator_id,
                estimator_version,
                captures: vec![capture.clone()],
            });
    }
    groups.into_values().collect()
}

fn series_hash(config: Option<&str>, provider: Option<&str>, model: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    for part in [config, provider, model] {
        hasher.update(part.unwrap_or("unknown").as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())[..16].to_string()
}

fn sum_event_tokens(
    events: &[NewUsageEvent],
    field: impl Fn(&NewUsageEvent) -> Option<u64>,
) -> Result<u64> {
    events.iter().try_fold(0_u64, |acc, event| {
        acc.checked_add(field(event).unwrap_or(0))
            .context("session usage projection overflowed u64")
    })
}
