use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tempfile::TempDir;

use super::*;

fn build_snapshot(temp: &TempDir) -> WorkspaceSnapshot {
    let workspace = temp.path().join("workspace 项目");
    let snapshots = temp.path().join("isolated snapshots");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(workspace.join("src/main.rs"), "fn main() {}\n").unwrap();
    WorkspaceSnapshotService::new(SnapshotPolicy::default())
        .build(&workspace, &snapshots, 7)
        .unwrap()
}

fn base_intent() -> ExecutionIntent {
    ExecutionIntent {
        schema_version: 1,
        session_id: "session-1".to_string(),
        message_id: Some("message-1".to_string()),
        source: ExecutionSource::Agent,
        source_id: None,
        base_generation: 7,
        command: ExecutionCommand::RegisteredTool {
            tool_id: "python-3".to_string(),
            argv: vec!["script.py".to_string()],
            relative_cwd: "src".to_string(),
        },
        input_artifact_ids: Vec::new(),
        requested_outputs: vec![OutputKind::WorkspaceChanges],
        requires_network: false,
    }
}

fn strict_policy(location: ExecutionLocation) -> ExecutionPolicy {
    ExecutionPolicy {
        guarantee: ExecutionGuarantee::Strict,
        location,
        workspace_delivery: WorkspaceDelivery::Snapshot,
        network: NetworkMode::Deny,
        resource_budget: ResourceBudget::default(),
        allowed_environment: BTreeSet::from(["LANG".to_string()]),
        approval_id: None,
        allow_experimental_provider: false,
    }
}

fn empty_manifest(execution_id: &str, provider_id: &str) -> ResultManifest {
    ResultManifest {
        schema_version: 1,
        execution_id: execution_id.to_string(),
        base_generation: 7,
        provider_id: provider_id.to_string(),
        changes: Vec::new(),
        artifacts: Vec::new(),
        evidence: IsolationEvidence::verified_strict(),
        exit_code: Some(0),
        completed_at: chrono::Utc::now().to_rfc3339(),
    }
}

#[test]
fn snapshot_excludes_credentials_control_directories_and_links() {
    let temp = TempDir::new().unwrap();
    let workspace = temp.path().join("workspace");
    let outside = temp.path().join("outside.txt");
    fs::create_dir_all(workspace.join(".git")).unwrap();
    fs::create_dir_all(workspace.join("nested")).unwrap();
    fs::write(workspace.join("safe.txt"), "safe").unwrap();
    fs::write(workspace.join(".git/config"), "secret").unwrap();
    fs::write(workspace.join(".env"), "TOKEN=secret").unwrap();
    fs::write(workspace.join("nested/id_rsa"), "secret").unwrap();
    fs::write(workspace.join(".env.example"), "TOKEN=").unwrap();
    fs::write(&outside, "outside").unwrap();

    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, workspace.join("outside-link")).unwrap();
    #[cfg(windows)]
    let linked =
        std::os::windows::fs::symlink_file(&outside, workspace.join("outside-link")).is_ok();

    let snapshot = WorkspaceSnapshotService::new(SnapshotPolicy::default())
        .build(&workspace, &temp.path().join("snapshots"), 7)
        .unwrap();
    let paths: BTreeSet<_> = snapshot
        .manifest()
        .entries
        .iter()
        .map(|entry| entry.relative_path.as_str())
        .collect();
    assert_eq!(paths, BTreeSet::from([".env.example", "safe.txt"]));
    assert!(snapshot
        .manifest()
        .exclusions
        .iter()
        .any(|item| item.relative_path == ".git"));
    assert!(snapshot
        .manifest()
        .exclusions
        .iter()
        .any(|item| item.relative_path == ".env"));
    assert!(snapshot
        .manifest()
        .exclusions
        .iter()
        .any(|item| item.relative_path == "nested/id_rsa"));
    #[cfg(unix)]
    assert!(snapshot
        .manifest()
        .exclusions
        .iter()
        .any(|item| item.relative_path == "outside-link"));
    #[cfg(windows)]
    if linked {
        assert!(snapshot
            .manifest()
            .exclusions
            .iter()
            .any(|item| item.relative_path == "outside-link"));
    }
}

#[test]
fn strict_policy_rejects_host_direct_direct_mount_and_secret_environment() {
    let intent = base_intent();
    let mut policy = strict_policy(ExecutionLocation::HostDirect);
    assert_eq!(
        policy.validate(&intent),
        Err(SandboxError::PolicyInvalid("strict_boundary"))
    );

    policy.location = ExecutionLocation::RemoteMicroVm;
    policy.workspace_delivery = WorkspaceDelivery::DirectMount;
    assert_eq!(
        policy.validate(&intent),
        Err(SandboxError::PolicyInvalid("strict_boundary"))
    );

    policy.workspace_delivery = WorkspaceDelivery::Snapshot;
    policy.allowed_environment.insert("OPENAI_API_KEY".into());
    assert_eq!(
        policy.validate(&intent),
        Err(SandboxError::PolicyInvalid("environment_allowlist"))
    );
}

#[test]
fn full_access_requires_explicit_host_direct_approval() {
    let intent = base_intent();
    let mut policy = ExecutionPolicy {
        guarantee: ExecutionGuarantee::FullAccess,
        location: ExecutionLocation::HostDirect,
        workspace_delivery: WorkspaceDelivery::DirectMount,
        network: NetworkMode::ExplicitHostDirect,
        resource_budget: ResourceBudget::default(),
        allowed_environment: BTreeSet::new(),
        approval_id: None,
        allow_experimental_provider: false,
    };
    assert_eq!(
        policy.validate(&intent),
        Err(SandboxError::ApprovalRequired)
    );
    policy.approval_id = Some("approval-1".into());
    assert_eq!(policy.validate(&intent), Ok(()));
}

#[test]
fn result_manifest_rejects_traversal_protected_paths_and_hash_mismatch() {
    let temp = TempDir::new().unwrap();
    let snapshot = build_snapshot(&temp);
    let result_root = temp.path().join("result");
    fs::create_dir_all(&result_root).unwrap();
    fs::write(result_root.join("escape.txt"), "changed").unwrap();

    for path in ["../escape.txt", ".git/config", "nested\\escape.txt"] {
        let manifest = ResultManifest {
            changes: vec![ResultChange {
                relative_path: path.to_string(),
                action: ResultChangeAction::Create,
                byte_size: Some(7),
                sha256: Some(super::types::hex_hash(b"changed")),
            }],
            ..empty_manifest("execution-1", "fake")
        };
        assert!(matches!(
            snapshot.validate_collected_result(
                &result_root,
                &manifest,
                ResultValidationPolicy::default()
            ),
            Err(SandboxError::ResultInvalid(_))
        ));
    }

    fs::create_dir_all(result_root.join("src")).unwrap();
    fs::write(result_root.join("src/main.rs"), "changed").unwrap();
    let manifest = ResultManifest {
        changes: vec![ResultChange {
            relative_path: "src/main.rs".to_string(),
            action: ResultChangeAction::Modify,
            byte_size: Some(7),
            sha256: Some(super::types::hex_hash(b"different")),
        }],
        ..empty_manifest("execution-1", "fake")
    };
    assert_eq!(
        snapshot
            .validate_collected_result(&result_root, &manifest, ResultValidationPolicy::default())
            .unwrap_err(),
        SandboxError::ResultInvalid("file_hash")
    );
}

#[test]
fn writeback_preflight_requires_exact_approval_and_detects_workspace_conflicts() {
    let temp = TempDir::new().unwrap();
    let snapshot = build_snapshot(&temp);
    let result_root = temp.path().join("result");
    fs::create_dir_all(result_root.join("src")).unwrap();
    let changed = b"fn main() { println!(\"safe\"); }\n";
    fs::write(result_root.join("src/main.rs"), changed).unwrap();
    let manifest = ResultManifest {
        changes: vec![ResultChange {
            relative_path: "src/main.rs".to_string(),
            action: ResultChangeAction::Modify,
            byte_size: Some(changed.len() as u64),
            sha256: Some(super::types::hex_hash(changed)),
        }],
        ..empty_manifest("execution-1", "fake")
    };
    let approval = WritebackApproval {
        approval_id: "approval-1".into(),
        execution_id: "execution-1".into(),
        base_generation: 7,
        manifest_digest: manifest.digest().unwrap(),
    };
    let prepared = snapshot
        .prepare_writeback(
            &result_root,
            &manifest,
            &approval,
            7,
            ResultValidationPolicy::default(),
        )
        .unwrap();
    assert_eq!(prepared.changes().len(), 1);
    assert!(prepared.changes()[0].source_path().is_some());
    assert!(prepared.changes()[0].target_path().ends_with("src/main.rs"));

    let mut wrong = approval.clone();
    wrong.manifest_digest = "0".repeat(64);
    assert_eq!(
        snapshot
            .prepare_writeback(
                &result_root,
                &manifest,
                &wrong,
                7,
                ResultValidationPolicy::default(),
            )
            .unwrap_err(),
        SandboxError::ApprovalRequired
    );

    fs::write(snapshot.workspace_root().join("src/main.rs"), "user edit\n").unwrap();
    assert_eq!(
        snapshot
            .prepare_writeback(
                &result_root,
                &manifest,
                &approval,
                7,
                ResultValidationPolicy::default(),
            )
            .unwrap_err(),
        SandboxError::Conflict("workspace_changed")
    );
}

#[derive(Default)]
struct MemoryAudit {
    events: Mutex<Vec<ExecutionAuditEvent>>,
}

impl AuditSink for MemoryAudit {
    fn record(&self, event: ExecutionAuditEvent) {
        self.events.lock().unwrap().push(event);
    }
}

struct FakeProvider {
    descriptor: ProviderDescriptor,
    result_root: PathBuf,
    current_execution: Mutex<Option<String>>,
    destroyed: Mutex<bool>,
    result_evidence: IsolationEvidence,
    lease_expires_at: Option<String>,
}

impl FakeProvider {
    fn strict(result_root: PathBuf) -> Self {
        Self {
            descriptor: ProviderDescriptor {
                provider_id: "fake-remote".into(),
                location: ExecutionLocation::RemoteMicroVm,
                availability: ProviderAvailability::Available,
                capabilities: IsolationEvidence::verified_strict(),
            },
            result_root,
            current_execution: Mutex::new(None),
            destroyed: Mutex::new(false),
            result_evidence: IsolationEvidence::verified_strict(),
            lease_expires_at: None,
        }
    }
}

#[async_trait]
impl SandboxProvider for FakeProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        self.descriptor.clone()
    }

    async fn create_execution(
        &self,
        plan: &ExecutionPlan,
        snapshot: &WorkspaceSnapshot,
    ) -> Result<ExecutionLease, SandboxError> {
        assert!(snapshot.snapshot_root().is_dir());
        *self.current_execution.lock().unwrap() = Some(plan.execution_id.clone());
        Ok(ExecutionLease {
            execution_id: plan.execution_id.clone(),
            provider_id: self.descriptor.provider_id.clone(),
            expires_at: self.lease_expires_at.clone().unwrap_or_else(|| {
                (chrono::Utc::now() + chrono::Duration::minutes(5)).to_rfc3339()
            }),
        })
    }

    async fn collect_result(
        &self,
        lease: &ExecutionLease,
    ) -> Result<ProviderExecutionResult, SandboxError> {
        Ok(ProviderExecutionResult::new(
            ResultManifest {
                evidence: self.result_evidence.clone(),
                ..empty_manifest(&lease.execution_id, &self.descriptor.provider_id)
            },
            self.result_root.clone(),
        ))
    }

    async fn cancel_execution(&self, _lease: &ExecutionLease) -> Result<(), SandboxError> {
        Ok(())
    }

    async fn destroy_execution(&self, _lease: &ExecutionLease) -> Result<(), SandboxError> {
        *self.destroyed.lock().unwrap() = true;
        Ok(())
    }
}

#[tokio::test]
async fn broker_executes_exact_provider_records_lifecycle_and_destroys_lease() {
    let temp = TempDir::new().unwrap();
    let snapshot = build_snapshot(&temp);
    let result_root = temp.path().join("result");
    fs::create_dir_all(&result_root).unwrap();
    let provider = Arc::new(FakeProvider::strict(result_root));
    let audit = Arc::new(MemoryAudit::default());
    let broker = SandboxBroker::new(
        vec![provider.clone()],
        audit.clone(),
        ResultValidationPolicy::default(),
    )
    .unwrap();
    let outcome = broker
        .execute(
            base_intent(),
            strict_policy(ExecutionLocation::RemoteMicroVm),
            &snapshot,
        )
        .await
        .unwrap();
    assert_eq!(outcome.result_manifest().provider_id, "fake-remote");
    assert!(*provider.destroyed.lock().unwrap());
    let states: Vec<_> = audit
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|event| event.state)
        .collect();
    assert_eq!(
        states,
        vec![
            ExecutionAuditState::Preparing,
            ExecutionAuditState::Running,
            ExecutionAuditState::Collecting,
            ExecutionAuditState::Destroying,
            ExecutionAuditState::Succeeded,
        ]
    );
}

#[tokio::test]
async fn missing_strict_provider_fails_closed_without_host_direct_fallback() {
    let temp = TempDir::new().unwrap();
    let snapshot = build_snapshot(&temp);
    let broker = SandboxBroker::unavailable();
    let error = broker
        .execute(
            base_intent(),
            strict_policy(ExecutionLocation::RemoteMicroVm),
            &snapshot,
        )
        .await
        .unwrap_err();
    assert_eq!(error.code(), SandboxErrorCode::SandboxUnavailable);
}

#[tokio::test]
async fn strict_result_missing_evidence_is_rejected_after_provider_cleanup() {
    let temp = TempDir::new().unwrap();
    let snapshot = build_snapshot(&temp);
    let result_root = temp.path().join("result");
    fs::create_dir_all(&result_root).unwrap();
    let mut fake = FakeProvider::strict(result_root);
    fake.result_evidence.network = EvidenceState::Missing;
    let provider = Arc::new(fake);
    let broker = SandboxBroker::new(
        vec![provider.clone()],
        Arc::new(MemoryAudit::default()),
        ResultValidationPolicy::default(),
    )
    .unwrap();
    let error = broker
        .execute(
            base_intent(),
            strict_policy(ExecutionLocation::RemoteMicroVm),
            &snapshot,
        )
        .await
        .unwrap_err();
    assert_eq!(error, SandboxError::ResultInvalid("isolation_evidence"));
    assert!(*provider.destroyed.lock().unwrap());
}

#[tokio::test]
async fn invalid_provider_lease_is_rejected_and_destroyed_before_collection() {
    let temp = TempDir::new().unwrap();
    let snapshot = build_snapshot(&temp);
    let result_root = temp.path().join("result");
    fs::create_dir_all(&result_root).unwrap();
    let mut fake = FakeProvider::strict(result_root);
    fake.lease_expires_at = Some((chrono::Utc::now() - chrono::Duration::minutes(1)).to_rfc3339());
    let provider = Arc::new(fake);
    let broker = SandboxBroker::new(
        vec![provider.clone()],
        Arc::new(MemoryAudit::default()),
        ResultValidationPolicy::default(),
    )
    .unwrap();

    let error = broker
        .execute(
            base_intent(),
            strict_policy(ExecutionLocation::RemoteMicroVm),
            &snapshot,
        )
        .await
        .unwrap_err();

    assert_eq!(error, SandboxError::ProviderFailed("lease_expiry"));
    assert!(*provider.destroyed.lock().unwrap());
}
