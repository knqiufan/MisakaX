use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde_json::json;

use super::{EstimatorDescriptor, MeasurementSource, UsageMeasurement};

pub const HEURISTIC_ESTIMATOR_ID: &str = "misakax-unicode-heuristic";
pub const HEURISTIC_ESTIMATOR_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFamily {
    OpenAi,
    Anthropic,
    Gemini,
    Generic,
}

pub fn model_family(provider: Option<&str>, model: Option<&str>) -> ModelFamily {
    let provider = provider.unwrap_or_default().to_ascii_lowercase();
    let model = model.unwrap_or_default().to_ascii_lowercase();
    if provider.contains("openai") || model.starts_with("gpt-") || model.starts_with("o1") {
        ModelFamily::OpenAi
    } else if provider.contains("anthropic") || model.starts_with("claude") {
        ModelFamily::Anthropic
    } else if provider.contains("google") || model.starts_with("gemini") {
        ModelFamily::Gemini
    } else {
        ModelFamily::Generic
    }
}

pub struct EstimatorInput<'a> {
    pub provider: Option<&'a str>,
    pub model: Option<&'a str>,
    pub input_segments: &'a [&'a str],
    pub output_text: &'a str,
    pub has_image_attachments: bool,
    pub has_tool_messages: bool,
}

pub trait TokenEstimator {
    fn descriptor(&self) -> EstimatorDescriptor;
    fn estimate(&self, input: &EstimatorInput<'_>) -> Result<UsageMeasurement>;
}

/// Versioned provider-agnostic fallback.
///
/// This intentionally estimates text only. Image bytes are never converted
/// into fake exact tokens; the limitation is carried in provider metadata.
#[derive(Debug, Default)]
pub struct UnicodeHeuristicEstimatorV1;

impl TokenEstimator for UnicodeHeuristicEstimatorV1 {
    fn descriptor(&self) -> EstimatorDescriptor {
        EstimatorDescriptor {
            id: HEURISTIC_ESTIMATOR_ID.to_string(),
            version: HEURISTIC_ESTIMATOR_VERSION.to_string(),
        }
    }

    fn estimate(&self, input: &EstimatorInput<'_>) -> Result<UsageMeasurement> {
        let family = model_family(input.provider, input.model);
        let input_tokens = input.input_segments.iter().try_fold(0_u64, |acc, text| {
            acc.checked_add(estimate_text(text, family))
                .context("estimated input token count overflowed u64")
        })?;
        let output_tokens = estimate_text(input.output_text, family);
        let total_tokens = input_tokens
            .checked_add(output_tokens)
            .context("estimated total token count overflowed u64")?;

        let mut metadata = BTreeMap::new();
        metadata.insert(
            "model_family".to_string(),
            json!(format!("{family:?}").to_lowercase()),
        );
        metadata.insert(
            "image_tokens_unknown".to_string(),
            json!(input.has_image_attachments),
        );
        metadata.insert(
            "includes_tool_text".to_string(),
            json!(input.has_tool_messages),
        );

        Ok(UsageMeasurement {
            input_tokens: Some(input_tokens),
            output_tokens: Some(output_tokens),
            total_tokens: Some(total_tokens),
            cache_read_tokens: None,
            cache_creation_tokens: None,
            reasoning_tokens: None,
            source: MeasurementSource::HeuristicEstimated,
            estimator: Some(self.descriptor()),
            provider_metadata: metadata,
        })
    }
}

pub fn unavailable_measurement(reason: &str) -> UsageMeasurement {
    UsageMeasurement {
        input_tokens: None,
        output_tokens: None,
        total_tokens: None,
        cache_read_tokens: None,
        cache_creation_tokens: None,
        reasoning_tokens: None,
        source: MeasurementSource::Unavailable,
        estimator: None,
        provider_metadata: BTreeMap::from([("reason".to_string(), json!(reason))]),
    }
}

fn estimate_text(text: &str, family: ModelFamily) -> u64 {
    let ascii_chars_per_token = match family {
        ModelFamily::Anthropic => 3,
        ModelFamily::OpenAi | ModelFamily::Gemini | ModelFamily::Generic => 4,
    };
    let mut tokens = 0_u64;
    let mut ascii_run = 0_u64;
    let flush_ascii = |tokens: &mut u64, run: &mut u64| {
        if *run > 0 {
            *tokens = tokens.saturating_add(run.div_ceil(ascii_chars_per_token));
            *run = 0;
        }
    };

    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            ascii_run = ascii_run.saturating_add(1);
        } else if ch.is_whitespace() {
            flush_ascii(&mut tokens, &mut ascii_run);
        } else {
            flush_ascii(&mut tokens, &mut ascii_run);
            // CJK characters and punctuation are both conservatively counted
            // as one token. This is deliberately approximate and versioned.
            tokens = tokens.saturating_add(1);
        }
    }
    flush_ascii(&mut tokens, &mut ascii_run);
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_text_fixtures_are_stable() {
        // Whitespace closes an ASCII word run, so two five-character words
        // are estimated independently as 2 + 2 tokens.
        assert_eq!(estimate_text("hello world", ModelFamily::OpenAi), 4);
        assert_eq!(estimate_text("你好，世界", ModelFamily::Generic), 5);
        assert_eq!(estimate_text("const value = 42;", ModelFamily::OpenAi), 7);
        assert_eq!(
            estimate_text(&"a".repeat(10_000), ModelFamily::OpenAi),
            2_500
        );
    }

    #[test]
    fn image_bytes_are_never_estimated_as_text() {
        let estimator = UnicodeHeuristicEstimatorV1;
        let measurement = estimator
            .estimate(&EstimatorInput {
                provider: Some("openai"),
                model: Some("gpt-4o"),
                input_segments: &["describe image"],
                output_text: "ok",
                has_image_attachments: true,
                has_tool_messages: false,
            })
            .unwrap();
        assert_eq!(measurement.input_tokens, Some(4));
        assert_eq!(measurement.provider_metadata["image_tokens_unknown"], true);
    }
}
