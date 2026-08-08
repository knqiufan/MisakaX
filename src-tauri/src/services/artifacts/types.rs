use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactOrigin {
    Model,
    Agent,
    Mcp,
    Skill,
    User,
    Preview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewState {
    None,
    Queued,
    Ready,
    Failed,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetentionState {
    Active,
    Expired,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub artifact_id: String,
    pub owner_session_id: String,
    pub origin_message_id: Option<String>,
    pub origin_kind: ArtifactOrigin,
    pub display_name: String,
    pub media_type: String,
    pub byte_size: u64,
    pub sha256: String,
    #[serde(skip_serializing)]
    pub storage_key: String,
    pub preview_state: PreviewState,
    pub preview_artifact_id: Option<String>,
    pub retention_state: RetentionState,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub artifact_id: String,
    pub display_name: String,
    pub media_type: String,
    pub byte_size: u64,
    pub sha256: String,
    pub origin_kind: ArtifactOrigin,
    pub preview_state: PreviewState,
    pub retention_state: RetentionState,
    pub created_at: String,
    pub expires_at: Option<String>,
}

impl From<&ArtifactRecord> for ArtifactMetadata {
    fn from(value: &ArtifactRecord) -> Self {
        Self {
            artifact_id: value.artifact_id.clone(),
            display_name: value.display_name.clone(),
            media_type: value.media_type.clone(),
            byte_size: value.byte_size,
            sha256: value.sha256.clone(),
            origin_kind: value.origin_kind.clone(),
            preview_state: value.preview_state.clone(),
            retention_state: value.retention_state.clone(),
            created_at: value.created_at.clone(),
            expires_at: value.expires_at.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContentSafetyPolicy {
    pub max_artifact_bytes: u64,
    pub max_total_artifact_bytes: u64,
    pub max_preview_bytes: u64,
    pub max_block_payload_bytes: usize,
    pub max_markdown_chars: usize,
    pub max_image_pixels: u64,
    pub max_chart_series: usize,
    pub max_chart_points: usize,
    pub max_map_features: usize,
    pub max_map_properties: usize,
    pub max_label_chars: usize,
    pub max_text_preview_lines: usize,
    pub max_text_preview_chars: usize,
}

impl Default for ContentSafetyPolicy {
    fn default() -> Self {
        Self {
            max_artifact_bytes: 25 * 1024 * 1024,
            max_total_artifact_bytes: 500 * 1024 * 1024,
            max_preview_bytes: 8 * 1024 * 1024,
            max_block_payload_bytes: 512 * 1024,
            max_markdown_chars: 256 * 1024,
            max_image_pixels: 32_000_000,
            max_chart_series: 24,
            max_chart_points: 5_000,
            max_map_features: 10_000,
            max_map_properties: 24,
            max_label_chars: 512,
            max_text_preview_lines: 5_000,
            max_text_preview_chars: 512 * 1024,
        }
    }
}
