use crate::services::artifacts::ArtifactRecord;
use crate::services::content::ContentBlock;
use crate::services::usage::{MeasurementSource, UsageOperationKind, UsageOutcome};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub working_directory: Option<String>,
    pub project_name: Option<String>,
    /// `default` = app-managed ~/.misakax/workspace; `custom` = user-picked path.
    #[serde(default = "default_workspace_kind")]
    pub workspace_kind: String,
    pub status: String,
    pub mode: String,
    #[serde(default)]
    pub total_input_tokens: i64,
    #[serde(default)]
    pub total_output_tokens: i64,
    pub last_message_at: Option<String>,
    #[serde(default)]
    pub pinned: bool,
    pub group_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn default_workspace_kind() -> String {
    "custom".to_string()
}

/// Backward-compatible DB-facing name for the canonical stream/message usage DTO.
pub type TokenUsage = crate::services::llm::TokenUsageInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub token_usage: Option<String>,
    pub model: Option<String>,
    pub thinking_content: Option<String>,
    pub attachments: Option<String>,
    #[serde(default = "default_message_status")]
    pub status: String,
    pub tool_calls: Option<String>,
    #[serde(default)]
    pub blocks: Vec<ContentBlock>,
    pub created_at: String,
}

fn default_message_status() -> String {
    "complete".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserProfile {
    pub profile_id: String,
    pub profile_kind: String,
    pub display_name: String,
    pub avatar_storage_key: Option<String>,
    pub avatar_sha256: Option<String>,
    pub timezone_mode: String,
    pub timezone_id: Option<String>,
    pub week_start: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewUsageEvent {
    pub event_id: String,
    pub profile_id: String,
    pub operation_key: String,
    pub measurement_key: String,
    pub operation_kind: UsageOperationKind,
    pub session_id: Option<String>,
    pub message_id: Option<String>,
    pub provider_config_id: Option<String>,
    pub provider_id: Option<String>,
    pub vendor_id: Option<String>,
    pub selected_model_id: Option<String>,
    pub effective_model_id: Option<String>,
    pub model_display_name: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub cache_read_tokens: Option<u64>,
    pub cache_creation_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub measurement_source: MeasurementSource,
    pub estimator_id: Option<String>,
    pub estimator_version: Option<String>,
    pub outcome: UsageOutcome,
    pub counts_toward_totals: bool,
    pub counts_toward_activity: bool,
    pub counts_toward_trend: bool,
    pub occurred_at_utc: String,
    pub local_date: String,
    pub timezone_id: Option<String>,
    pub utc_offset_minutes: i32,
    pub metadata_json: String,
    pub source_installation_id: Option<String>,
    pub source_event_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageEvent {
    pub event_id: String,
    pub profile_id: String,
    pub operation_key: String,
    pub measurement_key: String,
    pub operation_kind: UsageOperationKind,
    pub session_id: Option<String>,
    pub message_id: Option<String>,
    pub provider_config_id: Option<String>,
    pub provider_id: Option<String>,
    pub vendor_id: Option<String>,
    pub selected_model_id: Option<String>,
    pub effective_model_id: Option<String>,
    pub model_display_name: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub cache_read_tokens: Option<u64>,
    pub cache_creation_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub measurement_source: MeasurementSource,
    pub estimator_id: Option<String>,
    pub estimator_version: Option<String>,
    pub outcome: UsageOutcome,
    pub counts_toward_totals: bool,
    pub counts_toward_activity: bool,
    pub counts_toward_trend: bool,
    pub occurred_at_utc: String,
    pub local_date: String,
    pub timezone_id: Option<String>,
    pub utc_offset_minutes: i32,
    pub metadata_json: String,
    pub source_installation_id: Option<String>,
    pub source_event_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub vendor: Option<String>,
    pub api_key_encrypted: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub config_json: Option<String>,
    pub advanced_json: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    /// 接口兼容模式（仅自定义 Provider 使用）：
    /// `None` = 根据 provider 自动判断
    /// `"openai"` = OpenAI 兼容接口
    /// `"anthropic"` = Anthropic 兼容接口
    #[serde(default)]
    pub api_compat: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default)]
    pub max_tokens: Option<i32>,
    #[serde(default)]
    pub proxy: Option<String>,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: None,
            proxy: None,
        }
    }
}

fn default_temperature() -> f32 {
    0.7
}

/// 用户自定义模型（绑定到 router_configs）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomModel {
    pub id: String,
    pub router_config_id: String,
    pub model_id: String,
    pub display_name: String,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    /// Capability categories selected by the user or inferred while fetching.
    #[serde(default)]
    pub model_types: Vec<String>,
    pub max_tokens: Option<i32>,
    pub context_window: Option<i32>,
    #[serde(default = "default_custom_model_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub sort_order: i32,
    /// Same-provider non-thinking model_id used when thinking is toggled off.
    #[serde(default)]
    pub thinking_off_model_id: Option<String>,
    pub created_at: String,
}

/// 创建自定义模型的请求参数
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCustomModel {
    pub model_id: String,
    pub display_name: String,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default)]
    pub supports_thinking: bool,
    #[serde(default)]
    pub model_types: Vec<String>,
    pub max_tokens: Option<i32>,
    pub context_window: Option<i32>,
    #[serde(default = "default_custom_model_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    pub thinking_off_model_id: Option<String>,
}

fn default_custom_model_enabled() -> bool {
    true
}

impl Default for CreateCustomModel {
    fn default() -> Self {
        Self {
            model_id: String::new(),
            display_name: String::new(),
            supports_vision: false,
            supports_thinking: false,
            model_types: vec!["text".to_string()],
            max_tokens: None,
            context_window: None,
            enabled: true,
            sort_order: 0,
            thinking_off_model_id: None,
        }
    }
}

/// FTS5 全文搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSearchResult {
    pub id: String,
    pub session_id: String,
    pub session_title: Option<String>,
    pub role: String,
    pub snippet: String,
    pub created_at: String,
}

/// 会话导出数据（顶层容器）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub version: u32,
    pub exported_at: String,
    pub app: String,
    pub sessions: Vec<ExportSession>,
    #[serde(default)]
    pub artifact_manifest: Vec<ArtifactRecord>,
    #[serde(default)]
    pub profile: Option<ExportProfileMetadata>,
    #[serde(default)]
    pub usage_events: Vec<ExportUsageEvent>,
}

/// Safe, non-secret profile metadata included in ordinary JSON exports.
/// Avatar keys, hashes and bytes intentionally have no field here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProfileMetadata {
    pub display_name: String,
    pub timezone_mode: String,
    pub timezone_id: Option<String>,
    pub week_start: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportUsageEvent {
    pub source_installation_id: String,
    pub source_event_id: String,
    pub operation_key: String,
    pub operation_kind: UsageOperationKind,
    pub session_id: Option<String>,
    pub message_id: Option<String>,
    pub provider_config_id: Option<String>,
    pub provider_id: Option<String>,
    pub vendor_id: Option<String>,
    pub selected_model_id: Option<String>,
    pub effective_model_id: Option<String>,
    pub model_display_name: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub cache_read_tokens: Option<u64>,
    pub cache_creation_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub measurement_source: MeasurementSource,
    pub estimator_id: Option<String>,
    pub estimator_version: Option<String>,
    pub outcome: UsageOutcome,
    pub counts_toward_totals: bool,
    pub counts_toward_activity: bool,
    pub counts_toward_trend: bool,
    pub occurred_at_utc: String,
    pub local_date: String,
    pub timezone_id: Option<String>,
    pub utc_offset_minutes: i32,
}

/// 单个会话的导出数据（含消息列表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSession {
    pub session: Session,
    pub messages: Vec<Message>,
}

/// 导入结果统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported_count: u32,
    pub skipped_count: u32,
    pub errors: Vec<String>,
    #[serde(default)]
    pub usage_imported_count: u32,
    #[serde(default)]
    pub usage_skipped_count: u32,
}
