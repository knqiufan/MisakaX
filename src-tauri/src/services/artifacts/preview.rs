use serde::{Deserialize, Serialize};

use super::types::{ArtifactRecord, ContentSafetyPolicy, PreviewState};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PreviewKind {
    Text,
    Csv,
    Pdf,
    Spreadsheet,
    Document,
    Image,
    DownloadOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPreview {
    pub kind: PreviewKind,
    pub state: PreviewState,
    pub message_key: Option<String>,
    pub text: Option<String>,
    pub truncated: bool,
}

pub struct PreviewerRegistry;

impl PreviewerRegistry {
    pub fn preview_for(
        artifact: &ArtifactRecord,
        bytes: Option<&[u8]>,
        policy: &ContentSafetyPolicy,
    ) -> ArtifactPreview {
        let kind = match artifact.media_type.as_str() {
            "text/plain" | "text/markdown" | "application/json" => PreviewKind::Text,
            "text/csv" => PreviewKind::Csv,
            "application/pdf" => PreviewKind::Pdf,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
                PreviewKind::Spreadsheet
            }
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                PreviewKind::Document
            }
            "image/png" | "image/jpeg" | "image/webp" | "image/gif" => PreviewKind::Image,
            _ => PreviewKind::DownloadOnly,
        };
        if artifact.byte_size > policy.max_preview_bytes {
            return ArtifactPreview {
                kind: PreviewKind::DownloadOnly,
                state: PreviewState::Unsupported,
                message_key: Some("richContent.preview.resourceLimit".to_string()),
                text: None,
                truncated: false,
            };
        }
        match kind {
            PreviewKind::Text | PreviewKind::Csv => {
                let text = bytes.and_then(|value| std::str::from_utf8(value).ok());
                match text {
                    Some(value) => {
                        let (text, truncated) = limit_text(value, policy);
                        ArtifactPreview {
                            kind,
                            state: PreviewState::Ready,
                            message_key: None,
                            text: Some(text),
                            truncated,
                        }
                    }
                    None => ArtifactPreview {
                        kind: PreviewKind::DownloadOnly,
                        state: PreviewState::Failed,
                        message_key: Some("richContent.preview.parseFailed".to_string()),
                        text: None,
                        truncated: false,
                    },
                }
            }
            PreviewKind::DownloadOnly => ArtifactPreview {
                kind,
                state: PreviewState::Unsupported,
                message_key: Some("richContent.preview.unsupported".to_string()),
                text: None,
                truncated: false,
            },
            _ => ArtifactPreview {
                kind,
                state: PreviewState::Ready,
                message_key: None,
                text: None,
                truncated: false,
            },
        }
    }
}

fn limit_text(value: &str, policy: &ContentSafetyPolicy) -> (String, bool) {
    let mut output = String::new();
    for (lines, line) in value.lines().enumerate() {
        if lines == policy.max_text_preview_lines
            || output.len() + line.len() + 1 > policy.max_text_preview_chars
        {
            return (output, true);
        }
        output.push_str(line);
        output.push('\n');
    }
    (output, false)
}
