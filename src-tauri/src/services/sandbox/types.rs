use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const EXECUTION_CONTRACT_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionLocation {
    WindowsAppContainer,
    LinuxNamespace,
    RemoteMicroVm,
    MacLocalVm,
    MacXpcHelper,
    ContainerOptIn,
    HostDirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceDelivery {
    Snapshot,
    DirectMount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkMode {
    Deny,
    ProviderBrokered,
    ExplicitHostDirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionGuarantee {
    Strict,
    FullAccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAvailability {
    Available,
    NeedsSetup,
    Unsupported,
    Experimental,
    Unavailable,
}

impl ProviderAvailability {
    pub fn can_execute(self, allow_experimental: bool) -> bool {
        matches!(self, Self::Available)
            || (allow_experimental && matches!(self, Self::Experimental))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    Verified,
    Missing,
    Unsupported,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IsolationEvidence {
    pub filesystem: EvidenceState,
    pub network: EvidenceState,
    pub process_tree: EvidenceState,
    pub environment: EvidenceState,
    pub resources: EvidenceState,
    pub credentials: EvidenceState,
    pub audit: EvidenceState,
    pub provider_version: Option<String>,
    pub region: Option<String>,
}

impl IsolationEvidence {
    #[cfg(test)]
    pub fn verified_strict() -> Self {
        Self {
            filesystem: EvidenceState::Verified,
            network: EvidenceState::Verified,
            process_tree: EvidenceState::Verified,
            environment: EvidenceState::Verified,
            resources: EvidenceState::Verified,
            credentials: EvidenceState::Verified,
            audit: EvidenceState::Verified,
            provider_version: None,
            region: None,
        }
    }

    pub fn strict_gaps(&self) -> Vec<&'static str> {
        [
            ("filesystem", self.filesystem),
            ("network", self.network),
            ("process_tree", self.process_tree),
            ("environment", self.environment),
            ("resources", self.resources),
            ("credentials", self.credentials),
            ("audit", self.audit),
        ]
        .into_iter()
        .filter_map(|(name, state)| (state != EvidenceState::Verified).then_some(name))
        .collect()
    }

    pub(crate) fn validate_metadata(&self) -> Result<(), SandboxError> {
        for value in [&self.provider_version, &self.region].into_iter().flatten() {
            if value.trim().is_empty() || value.len() > 128 || value.contains(['\0', '\r', '\n']) {
                return Err(SandboxError::ResultInvalid("evidence_metadata"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub provider_id: String,
    pub location: ExecutionLocation,
    pub availability: ProviderAvailability,
    pub capabilities: IsolationEvidence,
}

impl ProviderDescriptor {
    pub(crate) fn validate(&self) -> Result<(), SandboxError> {
        validate_identifier(&self.provider_id)
            .map_err(|_| SandboxError::PolicyInvalid("provider_descriptor"))?;
        self.capabilities
            .validate_metadata()
            .map_err(|_| SandboxError::PolicyInvalid("provider_descriptor"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionSource {
    Agent,
    Skill,
    Mcp,
    Preview,
    User,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExecutionCommand {
    RegisteredTool {
        tool_id: String,
        argv: Vec<String>,
        relative_cwd: String,
    },
    TypedConverter {
        converter_id: String,
        argv: Vec<String>,
    },
}

impl ExecutionCommand {
    pub fn validate(&self) -> Result<(), SandboxError> {
        let (identifier, argv, cwd) = match self {
            Self::RegisteredTool {
                tool_id,
                argv,
                relative_cwd,
            } => (tool_id.as_str(), argv, Some(relative_cwd.as_str())),
            Self::TypedConverter { converter_id, argv } => (converter_id.as_str(), argv, None),
        };
        validate_identifier(identifier)?;
        if argv.len() > 256
            || argv
                .iter()
                .any(|argument| argument.len() > 8 * 1024 || argument.contains('\0'))
        {
            return Err(SandboxError::PolicyInvalid("argv_limits"));
        }
        if let Some(cwd) = cwd {
            super::snapshot::validate_relative_path(cwd, true)
                .map_err(|_| SandboxError::PolicyInvalid("relative_cwd"))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputKind {
    Artifact,
    Image,
    ChartData,
    MapData,
    WorkspaceChanges,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionIntent {
    pub schema_version: u16,
    pub session_id: String,
    pub message_id: Option<String>,
    pub source: ExecutionSource,
    pub source_id: Option<String>,
    pub base_generation: u64,
    pub command: ExecutionCommand,
    pub input_artifact_ids: Vec<String>,
    pub requested_outputs: Vec<OutputKind>,
    pub requires_network: bool,
}

impl ExecutionIntent {
    pub fn validate(&self) -> Result<(), SandboxError> {
        if self.schema_version != EXECUTION_CONTRACT_VERSION {
            return Err(SandboxError::PolicyInvalid("schema_version"));
        }
        if self.session_id.trim().is_empty()
            || self.session_id.len() > 128
            || self
                .message_id
                .as_ref()
                .is_some_and(|id| id.trim().is_empty() || id.len() > 128)
            || self
                .source_id
                .as_ref()
                .is_some_and(|id| id.trim().is_empty() || id.len() > 256)
            || self.input_artifact_ids.len() > 64
            || self.requested_outputs.len() > 16
        {
            return Err(SandboxError::PolicyInvalid("intent_identity"));
        }
        if self
            .input_artifact_ids
            .iter()
            .any(|id| id.trim().is_empty() || id.len() > 128)
        {
            return Err(SandboxError::PolicyInvalid("artifact_id"));
        }
        self.command.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceBudget {
    pub timeout_ms: u64,
    pub max_processes: u32,
    pub max_memory_bytes: u64,
    pub max_output_bytes: u64,
    pub max_result_files: u32,
    pub max_result_bytes: u64,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self {
            timeout_ms: 5 * 60 * 1_000,
            max_processes: 32,
            max_memory_bytes: 2 * 1024 * 1024 * 1024,
            max_output_bytes: 16 * 1024 * 1024,
            max_result_files: 2_000,
            max_result_bytes: 256 * 1024 * 1024,
        }
    }
}

impl ResourceBudget {
    fn validate(&self) -> Result<(), SandboxError> {
        if !(1_000..=60 * 60 * 1_000).contains(&self.timeout_ms)
            || !(1..=512).contains(&self.max_processes)
            || !(16 * 1024 * 1024..=64 * 1024 * 1024 * 1024).contains(&self.max_memory_bytes)
            || !(1..=1024 * 1024 * 1024).contains(&self.max_output_bytes)
            || !(1..=100_000).contains(&self.max_result_files)
            || !(1..=10 * 1024 * 1024 * 1024).contains(&self.max_result_bytes)
        {
            return Err(SandboxError::PolicyInvalid("resource_budget"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    pub guarantee: ExecutionGuarantee,
    pub location: ExecutionLocation,
    pub workspace_delivery: WorkspaceDelivery,
    pub network: NetworkMode,
    pub resource_budget: ResourceBudget,
    pub allowed_environment: BTreeSet<String>,
    pub approval_id: Option<String>,
    pub allow_experimental_provider: bool,
}

impl ExecutionPolicy {
    pub fn validate(&self, intent: &ExecutionIntent) -> Result<(), SandboxError> {
        self.resource_budget.validate()?;
        if self.allowed_environment.len() > 64 {
            return Err(SandboxError::PolicyInvalid("environment_allowlist"));
        }
        for name in &self.allowed_environment {
            validate_environment_name(name)?;
        }
        match self.guarantee {
            ExecutionGuarantee::Strict => {
                if self.location == ExecutionLocation::HostDirect
                    || self.workspace_delivery != WorkspaceDelivery::Snapshot
                    || self.network == NetworkMode::ExplicitHostDirect
                {
                    return Err(SandboxError::PolicyInvalid("strict_boundary"));
                }
                if intent.requires_network && self.network != NetworkMode::ProviderBrokered {
                    return Err(SandboxError::PolicyInvalid("network_evidence"));
                }
            }
            ExecutionGuarantee::FullAccess => {
                if self.location != ExecutionLocation::HostDirect
                    || self.workspace_delivery != WorkspaceDelivery::DirectMount
                    || self.network != NetworkMode::ExplicitHostDirect
                    || self
                        .approval_id
                        .as_ref()
                        .is_none_or(|id| id.trim().is_empty())
                {
                    return Err(SandboxError::ApprovalRequired);
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub execution_id: String,
    pub schema_version: u16,
    pub intent: ExecutionIntent,
    pub policy: ExecutionPolicy,
    pub snapshot_id: String,
    pub snapshot_digest: String,
    pub plan_digest: String,
    pub created_at: String,
}

impl ExecutionPlan {
    pub fn compile(
        intent: ExecutionIntent,
        policy: ExecutionPolicy,
        snapshot: &SnapshotManifest,
    ) -> Result<Self, SandboxError> {
        intent.validate()?;
        policy.validate(&intent)?;
        if snapshot.base_generation != intent.base_generation {
            return Err(SandboxError::Conflict("base_generation"));
        }
        let execution_id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        let digest_payload = serde_json::to_vec(&(
            EXECUTION_CONTRACT_VERSION,
            &execution_id,
            &intent,
            &policy,
            &snapshot.snapshot_id,
            &snapshot.digest,
            &created_at,
        ))
        .map_err(|_| SandboxError::Internal("plan_serialization"))?;
        let plan_digest = hex_hash(&digest_payload);
        Ok(Self {
            execution_id,
            schema_version: EXECUTION_CONTRACT_VERSION,
            intent,
            policy,
            snapshot_id: snapshot.snapshot_id.clone(),
            snapshot_digest: snapshot.digest.clone(),
            plan_digest,
            created_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionLease {
    pub execution_id: String,
    pub provider_id: String,
    pub expires_at: String,
}

impl ExecutionLease {
    pub(crate) fn validate(
        &self,
        execution_id: &str,
        provider_id: &str,
    ) -> Result<(), SandboxError> {
        if self.execution_id != execution_id || self.provider_id != provider_id {
            return Err(SandboxError::ProviderFailed("lease_identity"));
        }
        let expiry = chrono::DateTime::parse_from_rfc3339(&self.expires_at)
            .map_err(|_| SandboxError::ProviderFailed("lease_expiry"))?
            .with_timezone(&chrono::Utc);
        let now = chrono::Utc::now();
        if expiry <= now || expiry > now + chrono::Duration::hours(24) {
            return Err(SandboxError::ProviderFailed("lease_expiry"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotEntry {
    pub relative_path: String,
    pub byte_size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotExclusion {
    pub relative_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub schema_version: u16,
    pub snapshot_id: String,
    pub base_generation: u64,
    pub digest: String,
    pub total_bytes: u64,
    pub entries: Vec<SnapshotEntry>,
    pub exclusions: Vec<SnapshotExclusion>,
}

impl SnapshotManifest {
    pub fn entry_map(&self) -> BTreeMap<&str, &SnapshotEntry> {
        self.entries
            .iter()
            .map(|entry| (entry.relative_path.as_str(), entry))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultChangeAction {
    Create,
    Modify,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultChange {
    pub relative_path: String,
    pub action: ResultChangeAction,
    pub byte_size: Option<u64>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultArtifact {
    pub relative_path: String,
    pub display_name: String,
    pub media_type: String,
    pub byte_size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultManifest {
    pub schema_version: u16,
    pub execution_id: String,
    pub base_generation: u64,
    pub provider_id: String,
    pub changes: Vec<ResultChange>,
    pub artifacts: Vec<ResultArtifact>,
    pub evidence: IsolationEvidence,
    pub exit_code: Option<i32>,
    pub completed_at: String,
}

impl ResultManifest {
    pub fn digest(&self) -> Result<String, SandboxError> {
        let bytes =
            serde_json::to_vec(self).map_err(|_| SandboxError::Internal("result_serialization"))?;
        Ok(hex_hash(&bytes))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WritebackApproval {
    pub approval_id: String,
    pub execution_id: String,
    pub base_generation: u64,
    pub manifest_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SandboxErrorCode {
    SandboxUnavailable,
    SandboxPolicyInvalid,
    SandboxSnapshotInvalid,
    SandboxResultInvalid,
    SandboxApprovalRequired,
    SandboxConflict,
    SandboxProviderFailed,
    SandboxInternal,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SandboxError {
    #[error("sandbox provider unavailable")]
    Unavailable,
    #[error("sandbox policy invalid: {0}")]
    PolicyInvalid(&'static str),
    #[error("sandbox snapshot invalid: {0}")]
    SnapshotInvalid(&'static str),
    #[error("sandbox result invalid: {0}")]
    ResultInvalid(&'static str),
    #[error("sandbox approval required")]
    ApprovalRequired,
    #[error("sandbox conflict: {0}")]
    Conflict(&'static str),
    #[error("sandbox provider failed: {0}")]
    ProviderFailed(&'static str),
    #[error("sandbox internal error: {0}")]
    Internal(&'static str),
}

impl SandboxError {
    pub fn code(&self) -> SandboxErrorCode {
        match self {
            Self::Unavailable => SandboxErrorCode::SandboxUnavailable,
            Self::PolicyInvalid(_) => SandboxErrorCode::SandboxPolicyInvalid,
            Self::SnapshotInvalid(_) => SandboxErrorCode::SandboxSnapshotInvalid,
            Self::ResultInvalid(_) => SandboxErrorCode::SandboxResultInvalid,
            Self::ApprovalRequired => SandboxErrorCode::SandboxApprovalRequired,
            Self::Conflict(_) => SandboxErrorCode::SandboxConflict,
            Self::ProviderFailed(_) => SandboxErrorCode::SandboxProviderFailed,
            Self::Internal(_) => SandboxErrorCode::SandboxInternal,
        }
    }
}

pub(crate) fn hex_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|character| character.is_ascii_digit() || (b'a'..=b'f').contains(&character))
}

pub(crate) fn validate_identifier(value: &str) -> Result<(), SandboxError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|character| character.is_ascii_alphanumeric() || b"._-".contains(&character))
    {
        return Err(SandboxError::PolicyInvalid("tool_identifier"));
    }
    Ok(())
}

fn validate_environment_name(name: &str) -> Result<(), SandboxError> {
    let upper = name.to_ascii_uppercase();
    let valid = !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|character| character.is_ascii_alphanumeric() || character == b'_')
        && name.as_bytes().first().is_some_and(u8::is_ascii_alphabetic);
    let blocked = [
        "KEY",
        "TOKEN",
        "SECRET",
        "PASSWORD",
        "CREDENTIAL",
        "SSH_AUTH_SOCK",
        "AWS_",
        "AZURE_",
        "GOOGLE_",
        "GITHUB_",
        "GITLAB_",
        "PROXY",
    ]
    .iter()
    .any(|needle| upper.contains(needle));
    if !valid || blocked {
        return Err(SandboxError::PolicyInvalid("environment_allowlist"));
    }
    Ok(())
}
