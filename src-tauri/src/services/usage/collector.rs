use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use serde_json::json;

use super::estimator::{
    unavailable_measurement, EstimatorInput, TokenEstimator, UnicodeHeuristicEstimatorV1,
};
use super::{MeasurementSource, UsageCapture, UsageMeasurement};

pub fn provider_capture(
    capture_id: impl Into<String>,
    model: Option<String>,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    total_tokens: Option<u64>,
    cache_read_tokens: Option<u64>,
    cache_creation_tokens: Option<u64>,
    reasoning_tokens: Option<u64>,
) -> UsageCapture {
    let mut metadata = BTreeMap::new();
    if let (Some(input), Some(output), Some(total)) = (input_tokens, output_tokens, total_tokens) {
        if input.checked_add(output) != Some(total) {
            metadata.insert("provider_total_mismatch".to_string(), json!(true));
        }
    }
    UsageCapture {
        capture_id: capture_id.into(),
        model,
        measurement: UsageMeasurement {
            input_tokens,
            output_tokens,
            total_tokens,
            cache_read_tokens,
            cache_creation_tokens,
            reasoning_tokens,
            source: MeasurementSource::ProviderReported,
            estimator: None,
            provider_metadata: metadata,
        },
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ensure_fallback_capture(
    captures: &mut Vec<UsageCapture>,
    provider: Option<&str>,
    model: Option<&str>,
    input_segments: &[&str],
    output_text: &str,
    has_image_attachments: bool,
    has_tool_messages: bool,
    call_started: bool,
) {
    if captures.iter().any(|capture| {
        capture
            .measurement
            .resolved_total()
            .ok()
            .flatten()
            .is_some()
    }) {
        return;
    }

    let measurement = if call_started {
        UnicodeHeuristicEstimatorV1
            .estimate(&EstimatorInput {
                provider,
                model,
                input_segments,
                output_text,
                has_image_attachments,
                has_tool_messages,
            })
            .unwrap_or_else(|error| unavailable_measurement(&error.to_string()))
    } else {
        unavailable_measurement("provider call did not start")
    };
    captures.push(UsageCapture {
        capture_id: format!("fallback:{}", measurement.source.as_str()),
        model: model.map(ToString::to_string),
        measurement,
    });
}

pub fn aggregate_captures(captures: &[UsageCapture]) -> Result<UsageMeasurement> {
    if captures.is_empty() {
        return Ok(unavailable_measurement("no usage capture"));
    }

    let input_tokens = sum_optional(captures, |m| m.input_tokens)?;
    let output_tokens = sum_optional(captures, |m| m.output_tokens)?;
    let cache_read_tokens = sum_optional(captures, |m| m.cache_read_tokens)?;
    let cache_creation_tokens = sum_optional(captures, |m| m.cache_creation_tokens)?;
    let reasoning_tokens = sum_optional(captures, |m| m.reasoning_tokens)?;
    let total_tokens = captures.iter().try_fold(None, |acc, capture| {
        let total = capture
            .measurement
            .resolved_total()
            .map_err(anyhow::Error::msg)?;
        add_optional(acc, total)
    })?;

    let sources: BTreeSet<&str> = captures
        .iter()
        .map(|capture| capture.measurement.source.as_str())
        .collect();
    let source = aggregate_source(captures);
    let estimator = if captures.len() == 1 {
        captures[0].measurement.estimator.clone()
    } else {
        None
    };
    let mut metadata = BTreeMap::new();
    metadata.insert("capture_count".to_string(), json!(captures.len()));
    metadata.insert("sources".to_string(), json!(sources));

    Ok(UsageMeasurement {
        input_tokens,
        output_tokens,
        total_tokens,
        cache_read_tokens,
        cache_creation_tokens,
        reasoning_tokens,
        source,
        estimator,
        provider_metadata: metadata,
    })
}

fn aggregate_source(captures: &[UsageCapture]) -> MeasurementSource {
    if captures
        .iter()
        .any(|capture| capture.measurement.source == MeasurementSource::Unavailable)
    {
        MeasurementSource::Unavailable
    } else if captures
        .iter()
        .any(|capture| capture.measurement.source == MeasurementSource::HeuristicEstimated)
    {
        MeasurementSource::HeuristicEstimated
    } else if captures
        .iter()
        .any(|capture| capture.measurement.source == MeasurementSource::TokenizerEstimated)
    {
        MeasurementSource::TokenizerEstimated
    } else if captures
        .iter()
        .any(|capture| capture.measurement.source == MeasurementSource::LegacyMigrated)
    {
        MeasurementSource::LegacyMigrated
    } else {
        MeasurementSource::ProviderReported
    }
}

fn sum_optional(
    captures: &[UsageCapture],
    field: impl Fn(&UsageMeasurement) -> Option<u64>,
) -> Result<Option<u64>> {
    captures.iter().try_fold(None, |acc, capture| {
        add_optional(acc, field(&capture.measurement))
    })
}

fn add_optional(acc: Option<u64>, next: Option<u64>) -> Result<Option<u64>> {
    match (acc, next) {
        (None, None) => Ok(None),
        (Some(value), None) | (None, Some(value)) => Ok(Some(value)),
        (Some(left), Some(right)) => left
            .checked_add(right)
            .map(Some)
            .context("usage aggregation overflowed u64"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_total_is_authoritative_and_cache_is_detail_only() {
        let capture = provider_capture(
            "rig",
            None,
            Some(10),
            Some(5),
            Some(99),
            Some(4),
            Some(3),
            Some(2),
        );
        let aggregate = aggregate_captures(&[capture]).unwrap();
        assert_eq!(aggregate.total_tokens, Some(99));
        assert_eq!(aggregate.cache_read_tokens, Some(4));
    }

    #[test]
    fn fallback_is_versioned_and_does_not_replace_exact_usage() {
        let mut captures = vec![provider_capture(
            "rig",
            Some("m".into()),
            Some(4),
            Some(2),
            Some(6),
            None,
            None,
            None,
        )];
        ensure_fallback_capture(
            &mut captures,
            None,
            Some("m"),
            &["hello"],
            "world",
            false,
            false,
            true,
        );
        assert_eq!(captures.len(), 1);
    }
}
