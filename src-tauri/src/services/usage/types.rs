use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const USAGE_DASHBOARD_SCHEMA_VERSION: u16 = 1;
pub const USAGE_SSE_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementSource {
    ProviderReported,
    TokenizerEstimated,
    HeuristicEstimated,
    LegacyMigrated,
    Unavailable,
}

impl MeasurementSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderReported => "provider_reported",
            Self::TokenizerEstimated => "tokenizer_estimated",
            Self::HeuristicEstimated => "heuristic_estimated",
            Self::LegacyMigrated => "legacy_migrated",
            Self::Unavailable => "unavailable",
        }
    }

    pub fn is_estimated(self) -> bool {
        matches!(self, Self::TokenizerEstimated | Self::HeuristicEstimated)
    }
}

impl std::str::FromStr for MeasurementSource {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "provider_reported" => Ok(Self::ProviderReported),
            "tokenizer_estimated" => Ok(Self::TokenizerEstimated),
            "heuristic_estimated" => Ok(Self::HeuristicEstimated),
            "legacy_migrated" => Ok(Self::LegacyMigrated),
            "unavailable" => Ok(Self::Unavailable),
            _ => Err("unknown measurement source"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageOperationKind {
    Chat,
    Research,
    ToolRound,
    SessionTitle,
    ModelProbe,
    LegacyBackfill,
}

impl UsageOperationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::Research => "research",
            Self::ToolRound => "tool_round",
            Self::SessionTitle => "session_title",
            Self::ModelProbe => "model_probe",
            Self::LegacyBackfill => "legacy_backfill",
        }
    }
}

impl std::str::FromStr for UsageOperationKind {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "chat" => Ok(Self::Chat),
            "research" => Ok(Self::Research),
            "tool_round" => Ok(Self::ToolRound),
            "session_title" => Ok(Self::SessionTitle),
            "model_probe" => Ok(Self::ModelProbe),
            "legacy_backfill" => Ok(Self::LegacyBackfill),
            _ => Err("unknown usage operation kind"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageOutcome {
    Completed,
    Aborted,
    Failed,
    Partial,
}

impl UsageOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Aborted => "aborted",
            Self::Failed => "failed",
            Self::Partial => "partial",
        }
    }
}

impl std::str::FromStr for UsageOutcome {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "completed" => Ok(Self::Completed),
            "aborted" => Ok(Self::Aborted),
            "failed" => Ok(Self::Failed),
            "partial" => Ok(Self::Partial),
            _ => Err("unknown usage outcome"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EstimatorDescriptor {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageMeasurement {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub cache_read_tokens: Option<u64>,
    pub cache_creation_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub source: MeasurementSource,
    pub estimator: Option<EstimatorDescriptor>,
    #[serde(default)]
    pub provider_metadata: BTreeMap<String, serde_json::Value>,
}

impl UsageMeasurement {
    /// Resolve the authoritative total without adding cache/reasoning details.
    /// Provider totals win; input + output is used only when both are known.
    pub fn resolved_total(&self) -> Result<Option<u64>, &'static str> {
        if let Some(total) = self.total_tokens {
            return Ok(Some(total));
        }
        match (self.input_tokens, self.output_tokens) {
            (Some(input), Some(output)) => input
                .checked_add(output)
                .map(Some)
                .ok_or("input_tokens + output_tokens overflowed u64"),
            _ => Ok(None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageQualityV1 {
    pub exact_tokens: String,
    pub estimated_tokens: String,
    pub legacy_tokens: String,
    pub unknown_operation_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageOverviewV1 {
    pub total_tokens: String,
    pub exact_tokens: String,
    pub estimated_tokens: String,
    pub legacy_tokens: String,
    pub unknown_operation_count: u64,
    pub total_days: u32,
    pub current_streak: u32,
    pub longest_streak: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyUsageV1 {
    pub local_date: String,
    pub total_tokens: Option<String>,
    pub input_tokens: Option<String>,
    pub output_tokens: Option<String>,
    pub operation_count: u64,
    pub primary_model: Option<String>,
    pub quality: UsageQualityV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelUsagePointV1 {
    pub local_date: String,
    pub total_tokens: Option<String>,
    pub unknown_operation_count: u64,
    pub estimated_tokens: String,
    pub legacy_tokens: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelUsageSeriesV1 {
    pub series_key: String,
    pub display_name: String,
    pub provider_config_id: Option<String>,
    pub provider_id: Option<String>,
    pub effective_model_id: String,
    pub points: Vec<ModelUsagePointV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageRangeV1 {
    pub activity_start: String,
    pub activity_end: String,
    pub trend_start: String,
    pub trend_end: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageDashboardV1 {
    pub schema_version: u16,
    pub profile_id: String,
    pub generated_at: String,
    pub timezone_mode: String,
    pub timezone_id: Option<String>,
    pub utc_offset_minutes: i32,
    pub range: UsageRangeV1,
    pub overview: UsageOverviewV1,
    pub daily_activity: Vec<DailyUsageV1>,
    pub model_series: Vec<ModelUsageSeriesV1>,
    pub other_series: Option<ModelUsageSeriesV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidecarUsageMeasurementV1 {
    pub run_id: String,
    pub model: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub cache_read_tokens: Option<u64>,
    pub cache_creation_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub source: MeasurementSource,
    #[serde(default)]
    pub provider_metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidecarUsageEventV1 {
    pub schema_version: u16,
    pub measurements: Vec<SidecarUsageMeasurementV1>,
}

impl SidecarUsageEventV1 {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != USAGE_SSE_SCHEMA_VERSION {
            return Err("unsupported usage SSE schema_version");
        }
        if self.measurements.is_empty() {
            return Err("usage SSE measurements must not be empty");
        }
        if self
            .measurements
            .iter()
            .any(|measurement| measurement.run_id.trim().is_empty())
        {
            return Err("usage SSE run_id must not be empty");
        }
        Ok(())
    }
}
