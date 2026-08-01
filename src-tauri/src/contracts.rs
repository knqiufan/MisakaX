use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Stable application error identifiers shared by Rust, React, and the Sidecar.
/// User-facing copy is selected by `message_key`; callers must not parse prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppErrorCode {
    SkillDisabled,
    SkillScanRequired,
    SkillScanStale,
    SkillReviewRequired,
    SkillPolicyBlocked,
    SkillApprovalExpired,
    SkillScanCancelled,
    SkillScanTimeout,
    SkillQuarantineQuota,
    SkillPathInvalid,
    FilePreviewTooLarge,
    WorkspaceNotFound,
    GitNotAvailable,
    TerminalSessionNotFound,
    TerminalSessionOwnershipMismatch,
    InternalError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppErrorPayload {
    pub code: AppErrorCode,
    pub message_key: String,
    #[serde(default)]
    pub params: BTreeMap<String, String>,
    pub retryable: bool,
    pub correlation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SkillId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TerminalSessionId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanState {
    Unscanned,
    Queued,
    Scanning,
    Passed,
    Warnings,
    ReviewRequired,
    Blocked,
    Error,
    Stale,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanDecision {
    Unknown,
    Allow,
    AllowWithAck,
    Review,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceKind {
    Git,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceContextDiagnostic {
    pub code: AppErrorCode,
    pub message_key: String,
    pub retryable: bool,
    pub correlation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceContext {
    pub workspace_path: String,
    pub kind: WorkspaceKind,
    pub repository_root: Option<String>,
    pub branch: Option<String>,
    pub detached_head: Option<String>,
    pub generation: u64,
    pub diagnostic: Option<WorkspaceContextDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspacePanelMode {
    Explorer,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainEvent<T> {
    pub event_id: String,
    pub aggregate_id: String,
    pub generation: u64,
    pub occurred_at: String,
    pub payload: T,
}

/// Feature switches start disabled so S0/W0 preserve the established behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FeatureFlags {
    pub workspace_terminal: bool,
    pub narrow_webview_capabilities: bool,
}

impl FeatureFlags {
    pub fn from_env() -> Self {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    fn from_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let mut enabled = |name| {
            lookup(name).is_some_and(|value| {
                matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true")
            })
        };
        Self {
            workspace_terminal: enabled("MISAKAX_WORKSPACE_TERMINAL"),
            narrow_webview_capabilities: enabled("MISAKAX_NARROW_WEBVIEW_CAPABILITIES"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_enums_use_stable_wire_values() {
        assert_eq!(
            serde_json::to_string(&AppErrorCode::SkillScanRequired).unwrap(),
            "\"SKILL_SCAN_REQUIRED\""
        );
        assert_eq!(
            serde_json::to_string(&ScanState::ReviewRequired).unwrap(),
            "\"review_required\""
        );
        assert_eq!(
            serde_json::to_string(&WorkspacePanelMode::Terminal).unwrap(),
            "\"terminal\""
        );
    }

    #[test]
    fn feature_flags_default_to_legacy_behavior_and_require_explicit_true() {
        assert_eq!(FeatureFlags::default(), FeatureFlags::from_lookup(|_| None));
        let flags = FeatureFlags::from_lookup(|name| {
            (name == "MISAKAX_WORKSPACE_TERMINAL").then(|| "TRUE".to_string())
        });
        assert!(flags.workspace_terminal);
        assert!(!flags.narrow_webview_capabilities);
    }

    #[test]
    fn domain_event_serializes_generation_in_camel_case_envelope() {
        let event = DomainEvent {
            event_id: "event-1".to_string(),
            aggregate_id: "workspace-1".to_string(),
            generation: 7,
            occurred_at: "2026-08-01T00:00:00Z".to_string(),
            payload: WorkspacePanelMode::Explorer,
        };
        let value = serde_json::to_value(event).unwrap();
        assert_eq!(value["eventId"], "event-1");
        assert_eq!(value["generation"], 7);
    }
}
