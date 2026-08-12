use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::types::{is_sha256, validate_identifier, EXECUTION_CONTRACT_VERSION};
use super::{
    ResultArtifact, ResultChangeAction, ResultManifest, SandboxError, SnapshotEntry,
    SnapshotExclusion, SnapshotManifest, WritebackApproval,
};

#[derive(Debug, Clone, Copy)]
pub struct SnapshotPolicy {
    pub max_files: usize,
    pub max_total_bytes: u64,
    pub max_file_bytes: u64,
}

impl Default for SnapshotPolicy {
    fn default() -> Self {
        Self {
            max_files: 20_000,
            max_total_bytes: 512 * 1024 * 1024,
            max_file_bytes: 25 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResultValidationPolicy {
    pub max_changes: usize,
    pub max_artifacts: usize,
    pub max_total_bytes: u64,
    pub max_file_bytes: u64,
}

impl Default for ResultValidationPolicy {
    fn default() -> Self {
        Self {
            max_changes: 2_000,
            max_artifacts: 256,
            max_total_bytes: 256 * 1024 * 1024,
            max_file_bytes: 25 * 1024 * 1024,
        }
    }
}

/// Internal snapshot handle. Host paths are deliberately omitted from the
/// serializable `SnapshotManifest`.
#[derive(Debug)]
pub struct WorkspaceSnapshot {
    workspace_root: PathBuf,
    snapshot_root: PathBuf,
    manifest: SnapshotManifest,
}

impl WorkspaceSnapshot {
    pub fn manifest(&self) -> &SnapshotManifest {
        &self.manifest
    }

    pub(crate) fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Internal provider input root. This path is never part of an IPC DTO.
    pub fn snapshot_root(&self) -> &Path {
        &self.snapshot_root
    }

    pub fn validate_collected_result(
        &self,
        result_root: &Path,
        manifest: &ResultManifest,
        policy: ResultValidationPolicy,
    ) -> Result<ValidatedCollectedResult, SandboxError> {
        validate_collected_result(self, result_root, manifest, policy)
    }

    /// Performs all path, hash, generation and approval checks and returns an
    /// opaque writeback plan. It deliberately does not mutate the real
    /// workspace; applying a reviewed plan is a later application-service
    /// operation and must be transactional.
    pub fn prepare_writeback(
        &self,
        result_root: &Path,
        manifest: &ResultManifest,
        approval: &WritebackApproval,
        current_generation: u64,
        policy: ResultValidationPolicy,
    ) -> Result<ValidatedWriteback, SandboxError> {
        prepare_writeback(
            self,
            result_root,
            manifest,
            approval,
            current_generation,
            policy,
        )
    }
}

pub struct WorkspaceSnapshotService {
    policy: SnapshotPolicy,
}

impl WorkspaceSnapshotService {
    pub fn new(policy: SnapshotPolicy) -> Self {
        Self { policy }
    }

    pub fn build(
        &self,
        workspace_root: &Path,
        snapshot_parent: &Path,
        base_generation: u64,
    ) -> Result<WorkspaceSnapshot, SandboxError> {
        if self.policy.max_files == 0
            || self.policy.max_total_bytes == 0
            || self.policy.max_file_bytes == 0
            || self.policy.max_file_bytes > self.policy.max_total_bytes
        {
            return Err(SandboxError::SnapshotInvalid("snapshot_policy"));
        }
        let workspace_root = fs::canonicalize(workspace_root)
            .map_err(|_| SandboxError::SnapshotInvalid("workspace_root"))?;
        if !workspace_root.is_dir() {
            return Err(SandboxError::SnapshotInvalid("workspace_root"));
        }
        fs::create_dir_all(snapshot_parent)
            .map_err(|_| SandboxError::SnapshotInvalid("snapshot_parent"))?;
        let snapshot_parent = fs::canonicalize(snapshot_parent)
            .map_err(|_| SandboxError::SnapshotInvalid("snapshot_parent"))?;
        if snapshot_parent.starts_with(&workspace_root)
            || workspace_root.starts_with(&snapshot_parent)
        {
            return Err(SandboxError::SnapshotInvalid("snapshot_location"));
        }

        let snapshot_id = uuid::Uuid::new_v4().to_string();
        let snapshot_root = snapshot_parent.join(format!("snapshot-{snapshot_id}"));
        fs::create_dir(&snapshot_root)
            .map_err(|_| SandboxError::SnapshotInvalid("snapshot_create"))?;

        let build_result = self.copy_workspace(
            &workspace_root,
            &snapshot_root,
            snapshot_id,
            base_generation,
        );
        if build_result.is_err() {
            let _ = fs::remove_dir_all(&snapshot_root);
        }
        build_result
    }

    fn copy_workspace(
        &self,
        workspace_root: &Path,
        snapshot_root: &Path,
        snapshot_id: String,
        base_generation: u64,
    ) -> Result<WorkspaceSnapshot, SandboxError> {
        let mut entries = Vec::new();
        let mut exclusions = Vec::new();
        let mut total_bytes = 0u64;
        let mut walker = WalkDir::new(workspace_root).follow_links(false).into_iter();

        while let Some(next) = walker.next() {
            let entry = next.map_err(|_| SandboxError::SnapshotInvalid("workspace_walk"))?;
            let source = entry.path();
            let relative = source
                .strip_prefix(workspace_root)
                .map_err(|_| SandboxError::SnapshotInvalid("workspace_boundary"))?;
            if relative.as_os_str().is_empty() {
                continue;
            }
            let relative_contract = relative_to_contract(relative)?;
            let metadata = fs::symlink_metadata(source)
                .map_err(|_| SandboxError::SnapshotInvalid("entry_metadata"))?;

            if let Some(reason) = exclusion_reason(relative) {
                if metadata.is_dir() {
                    walker.skip_current_dir();
                }
                exclusions.push(SnapshotExclusion {
                    relative_path: relative_contract,
                    reason: reason.to_string(),
                });
                continue;
            }
            if is_link_like(&metadata) {
                if metadata.is_dir() {
                    walker.skip_current_dir();
                }
                exclusions.push(SnapshotExclusion {
                    relative_path: relative_contract,
                    reason: "link_or_reparse_point".to_string(),
                });
                continue;
            }
            if metadata.is_dir() {
                continue;
            }
            if !metadata.is_file() {
                exclusions.push(SnapshotExclusion {
                    relative_path: relative_contract,
                    reason: "non_regular_file".to_string(),
                });
                continue;
            }
            validate_relative_path(&relative_contract, false)
                .map_err(|_| SandboxError::SnapshotInvalid("relative_path"))?;
            if entries.len() >= self.policy.max_files {
                return Err(SandboxError::SnapshotInvalid("file_count_limit"));
            }
            if metadata.len() > self.policy.max_file_bytes {
                return Err(SandboxError::SnapshotInvalid("file_size_limit"));
            }
            total_bytes = total_bytes
                .checked_add(metadata.len())
                .ok_or(SandboxError::SnapshotInvalid("snapshot_size_limit"))?;
            if total_bytes > self.policy.max_total_bytes {
                return Err(SandboxError::SnapshotInvalid("snapshot_size_limit"));
            }

            let canonical_source = fs::canonicalize(source)
                .map_err(|_| SandboxError::SnapshotInvalid("entry_canonicalize"))?;
            if !canonical_source.starts_with(workspace_root) {
                return Err(SandboxError::SnapshotInvalid("link_escape"));
            }
            let destination = snapshot_root.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .map_err(|_| SandboxError::SnapshotInvalid("snapshot_create"))?;
            }
            let (byte_size, sha256) =
                copy_and_hash(&canonical_source, &destination, self.policy.max_file_bytes)?;
            if byte_size != metadata.len() {
                return Err(SandboxError::SnapshotInvalid("source_changed"));
            }
            entries.push(SnapshotEntry {
                relative_path: relative_contract,
                byte_size,
                sha256,
            });
        }

        entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        exclusions.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        let digest_bytes = serde_json::to_vec(&(
            EXECUTION_CONTRACT_VERSION,
            base_generation,
            total_bytes,
            &entries,
            &exclusions,
        ))
        .map_err(|_| SandboxError::Internal("snapshot_serialization"))?;
        let manifest = SnapshotManifest {
            schema_version: EXECUTION_CONTRACT_VERSION,
            snapshot_id,
            base_generation,
            digest: super::types::hex_hash(&digest_bytes),
            total_bytes,
            entries,
            exclusions,
        };
        Ok(WorkspaceSnapshot {
            workspace_root: workspace_root.to_path_buf(),
            snapshot_root: snapshot_root.to_path_buf(),
            manifest,
        })
    }
}

#[derive(Debug)]
pub struct ValidatedArtifact {
    metadata: ResultArtifact,
    source_path: PathBuf,
}

impl ValidatedArtifact {
    pub fn metadata(&self) -> &ResultArtifact {
        &self.metadata
    }

    /// Internal Rust path for the subsequent `ArtifactService` ingress.
    /// This value must never be serialized or returned to an untrusted caller.
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }
}

#[derive(Debug)]
pub struct ValidatedCollectedResult {
    manifest_digest: String,
    artifacts: Vec<ValidatedArtifact>,
}

impl ValidatedCollectedResult {
    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    pub fn artifacts(&self) -> &[ValidatedArtifact] {
        &self.artifacts
    }
}

#[derive(Debug)]
pub struct ValidatedWritebackChange {
    relative_path: String,
    action: ResultChangeAction,
    source_path: Option<PathBuf>,
    target_path: PathBuf,
}

impl ValidatedWritebackChange {
    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    pub fn action(&self) -> ResultChangeAction {
        self.action
    }

    pub fn source_path(&self) -> Option<&Path> {
        self.source_path.as_deref()
    }

    pub fn target_path(&self) -> &Path {
        &self.target_path
    }
}

#[derive(Debug)]
pub struct ValidatedWriteback {
    execution_id: String,
    manifest_digest: String,
    changes: Vec<ValidatedWritebackChange>,
}

impl ValidatedWriteback {
    pub fn execution_id(&self) -> &str {
        &self.execution_id
    }

    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    pub fn changes(&self) -> &[ValidatedWritebackChange] {
        &self.changes
    }
}

pub(crate) fn validate_collected_result(
    snapshot: &WorkspaceSnapshot,
    result_root: &Path,
    manifest: &ResultManifest,
    policy: ResultValidationPolicy,
) -> Result<ValidatedCollectedResult, SandboxError> {
    validate_manifest_structure(snapshot, manifest, policy)?;
    let result_root = canonical_disjoint_root(result_root, snapshot)?;
    let mut artifacts = Vec::with_capacity(manifest.artifacts.len());
    let mut total_bytes = 0u64;

    for artifact in &manifest.artifacts {
        validate_relative_path(&artifact.relative_path, false)?;
        if artifact.display_name.trim().is_empty()
            || artifact.display_name.len() > 180
            || artifact
                .display_name
                .chars()
                .any(|character| character.is_control() || "/\\:*?\"<>|".contains(character))
            || artifact.media_type.trim().is_empty()
            || artifact.media_type.len() > 128
            || !artifact.media_type.bytes().all(|character| {
                character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || b"/+-._".contains(&character)
            })
            || !is_sha256(&artifact.sha256)
        {
            return Err(SandboxError::ResultInvalid("artifact_metadata"));
        }
        total_bytes = checked_result_bytes(total_bytes, artifact.byte_size, policy)?;
        let source_path = validate_result_file(
            &result_root,
            &artifact.relative_path,
            artifact.byte_size,
            &artifact.sha256,
        )?;
        artifacts.push(ValidatedArtifact {
            metadata: artifact.clone(),
            source_path,
        });
    }
    for change in &manifest.changes {
        if matches!(
            change.action,
            ResultChangeAction::Create | ResultChangeAction::Modify
        ) {
            let byte_size = change
                .byte_size
                .ok_or(SandboxError::ResultInvalid("change_metadata"))?;
            let sha256 = change
                .sha256
                .as_deref()
                .ok_or(SandboxError::ResultInvalid("change_metadata"))?;
            total_bytes = checked_result_bytes(total_bytes, byte_size, policy)?;
            validate_result_file(&result_root, &change.relative_path, byte_size, sha256)?;
        }
    }
    Ok(ValidatedCollectedResult {
        manifest_digest: manifest.digest()?,
        artifacts,
    })
}

pub(crate) fn prepare_writeback(
    snapshot: &WorkspaceSnapshot,
    result_root: &Path,
    manifest: &ResultManifest,
    approval: &WritebackApproval,
    current_generation: u64,
    policy: ResultValidationPolicy,
) -> Result<ValidatedWriteback, SandboxError> {
    let digest = manifest.digest()?;
    if approval.approval_id.trim().is_empty()
        || approval.execution_id != manifest.execution_id
        || approval.base_generation != manifest.base_generation
        || approval.manifest_digest != digest
    {
        return Err(SandboxError::ApprovalRequired);
    }
    validate_collected_result(snapshot, result_root, manifest, policy)?;
    if current_generation != snapshot.manifest.base_generation
        || current_generation != manifest.base_generation
    {
        return Err(SandboxError::Conflict("base_generation"));
    }
    let result_root = canonical_disjoint_root(result_root, snapshot)?;
    let snapshot_entries = snapshot.manifest.entry_map();
    let mut changes = Vec::with_capacity(manifest.changes.len());

    for change in &manifest.changes {
        let relative = validate_relative_path(&change.relative_path, false)?;
        ensure_workspace_ancestors_safe(snapshot.workspace_root(), &relative)?;
        let target_path = snapshot.workspace_root().join(&relative);
        let original = snapshot_entries.get(change.relative_path.as_str()).copied();
        let source_path = match change.action {
            ResultChangeAction::Create => {
                if original.is_some() || target_path.exists() {
                    return Err(SandboxError::Conflict("create_target_exists"));
                }
                Some(validate_change_source(&result_root, change)?)
            }
            ResultChangeAction::Modify => {
                let original = original.ok_or(SandboxError::Conflict("modify_target_missing"))?;
                validate_workspace_file(&target_path, original)?;
                Some(validate_change_source(&result_root, change)?)
            }
            ResultChangeAction::Delete => {
                let original = original.ok_or(SandboxError::Conflict("delete_target_missing"))?;
                validate_workspace_file(&target_path, original)?;
                None
            }
        };
        changes.push(ValidatedWritebackChange {
            relative_path: change.relative_path.clone(),
            action: change.action,
            source_path,
            target_path,
        });
    }
    Ok(ValidatedWriteback {
        execution_id: manifest.execution_id.clone(),
        manifest_digest: digest,
        changes,
    })
}

fn validate_manifest_structure(
    snapshot: &WorkspaceSnapshot,
    manifest: &ResultManifest,
    policy: ResultValidationPolicy,
) -> Result<(), SandboxError> {
    if policy.max_changes == 0
        || policy.max_artifacts == 0
        || policy.max_total_bytes == 0
        || policy.max_file_bytes == 0
        || policy.max_file_bytes > policy.max_total_bytes
    {
        return Err(SandboxError::ResultInvalid("result_policy"));
    }
    if manifest.schema_version != EXECUTION_CONTRACT_VERSION
        || manifest.execution_id.trim().is_empty()
        || manifest.execution_id.len() > 128
        || manifest.provider_id.trim().is_empty()
        || manifest.provider_id.len() > 128
        || manifest.base_generation != snapshot.manifest.base_generation
        || manifest.changes.len() > policy.max_changes
        || manifest.artifacts.len() > policy.max_artifacts
    {
        return Err(SandboxError::ResultInvalid("manifest_identity"));
    }
    validate_identifier(&manifest.provider_id)
        .map_err(|_| SandboxError::ResultInvalid("manifest_identity"))?;
    manifest.evidence.validate_metadata()?;
    let completed_at = chrono::DateTime::parse_from_rfc3339(&manifest.completed_at)
        .map_err(|_| SandboxError::ResultInvalid("completed_at"))?
        .with_timezone(&chrono::Utc);
    let now = chrono::Utc::now();
    if completed_at < now - chrono::Duration::hours(24)
        || completed_at > now + chrono::Duration::minutes(10)
    {
        return Err(SandboxError::ResultInvalid("completed_at"));
    }
    let snapshot_entries = snapshot.manifest.entry_map();
    let mut paths = BTreeSet::new();
    for change in &manifest.changes {
        validate_relative_path(&change.relative_path, false)?;
        if !paths.insert(change.relative_path.as_str()) {
            return Err(SandboxError::ResultInvalid("duplicate_change"));
        }
        let existed = snapshot_entries.contains_key(change.relative_path.as_str());
        match change.action {
            ResultChangeAction::Create if existed => {
                return Err(SandboxError::ResultInvalid("create_existing"));
            }
            ResultChangeAction::Modify | ResultChangeAction::Delete if !existed => {
                return Err(SandboxError::ResultInvalid("change_missing"));
            }
            _ => {}
        }
        match change.action {
            ResultChangeAction::Delete => {
                if change.byte_size.is_some() || change.sha256.is_some() {
                    return Err(SandboxError::ResultInvalid("delete_metadata"));
                }
            }
            ResultChangeAction::Create | ResultChangeAction::Modify => {
                if change.byte_size.is_none()
                    || change.byte_size > Some(policy.max_file_bytes)
                    || change.sha256.as_deref().is_none_or(|hash| !is_sha256(hash))
                {
                    return Err(SandboxError::ResultInvalid("change_metadata"));
                }
            }
        }
    }
    let mut artifact_paths = BTreeSet::new();
    for artifact in &manifest.artifacts {
        validate_relative_path(&artifact.relative_path, false)?;
        if !artifact_paths.insert(artifact.relative_path.as_str())
            || paths.contains(artifact.relative_path.as_str())
            || artifact.byte_size > policy.max_file_bytes
        {
            return Err(SandboxError::ResultInvalid("artifact_metadata"));
        }
    }
    Ok(())
}

fn validate_change_source(
    result_root: &Path,
    change: &super::ResultChange,
) -> Result<PathBuf, SandboxError> {
    let byte_size = change
        .byte_size
        .ok_or(SandboxError::ResultInvalid("change_metadata"))?;
    let sha256 = change
        .sha256
        .as_deref()
        .ok_or(SandboxError::ResultInvalid("change_metadata"))?;
    validate_result_file(result_root, &change.relative_path, byte_size, sha256)
}

fn validate_workspace_file(path: &Path, original: &SnapshotEntry) -> Result<(), SandboxError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| SandboxError::Conflict("workspace_changed"))?;
    if !metadata.is_file() || is_link_like(&metadata) || metadata.len() != original.byte_size {
        return Err(SandboxError::Conflict("workspace_changed"));
    }
    let actual = hash_file(path, original.byte_size)
        .map_err(|_| SandboxError::Conflict("workspace_changed"))?;
    if actual != original.sha256 {
        return Err(SandboxError::Conflict("workspace_changed"));
    }
    Ok(())
}

fn validate_result_file(
    result_root: &Path,
    relative_path: &str,
    expected_size: u64,
    expected_hash: &str,
) -> Result<PathBuf, SandboxError> {
    if !is_sha256(expected_hash) {
        return Err(SandboxError::ResultInvalid("file_hash"));
    }
    let relative = validate_relative_path(relative_path, false)?;
    let source = result_root.join(relative);
    let metadata = fs::symlink_metadata(&source)
        .map_err(|_| SandboxError::ResultInvalid("result_file_missing"))?;
    if !metadata.is_file() || is_link_like(&metadata) || metadata.len() != expected_size {
        return Err(SandboxError::ResultInvalid("result_file_metadata"));
    }
    let canonical = fs::canonicalize(&source)
        .map_err(|_| SandboxError::ResultInvalid("result_file_missing"))?;
    if !canonical.starts_with(result_root) {
        return Err(SandboxError::ResultInvalid("result_path_escape"));
    }
    let actual = hash_file(&canonical, expected_size)?;
    if actual != expected_hash {
        return Err(SandboxError::ResultInvalid("file_hash"));
    }
    Ok(canonical)
}

fn canonical_disjoint_root(
    root: &Path,
    snapshot: &WorkspaceSnapshot,
) -> Result<PathBuf, SandboxError> {
    let root = fs::canonicalize(root).map_err(|_| SandboxError::ResultInvalid("result_root"))?;
    if !root.is_dir()
        || root.starts_with(snapshot.workspace_root())
        || snapshot.workspace_root().starts_with(&root)
        || root.starts_with(snapshot.snapshot_root())
        || snapshot.snapshot_root().starts_with(&root)
    {
        return Err(SandboxError::ResultInvalid("result_root"));
    }
    Ok(root)
}

fn checked_result_bytes(
    current: u64,
    next: u64,
    policy: ResultValidationPolicy,
) -> Result<u64, SandboxError> {
    if next > policy.max_file_bytes {
        return Err(SandboxError::ResultInvalid("file_size_limit"));
    }
    let total = current
        .checked_add(next)
        .ok_or(SandboxError::ResultInvalid("result_size_limit"))?;
    if total > policy.max_total_bytes {
        return Err(SandboxError::ResultInvalid("result_size_limit"));
    }
    Ok(total)
}

fn copy_and_hash(
    source: &Path,
    destination: &Path,
    max_bytes: u64,
) -> Result<(u64, String), SandboxError> {
    let mut input = File::open(source).map_err(|_| SandboxError::SnapshotInvalid("source_open"))?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|_| SandboxError::SnapshotInvalid("snapshot_write"))?;
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = input
            .read(&mut buffer)
            .map_err(|_| SandboxError::SnapshotInvalid("source_read"))?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(count as u64)
            .ok_or(SandboxError::SnapshotInvalid("file_size_limit"))?;
        if total > max_bytes {
            return Err(SandboxError::SnapshotInvalid("file_size_limit"));
        }
        output
            .write_all(&buffer[..count])
            .map_err(|_| SandboxError::SnapshotInvalid("snapshot_write"))?;
        hasher.update(&buffer[..count]);
    }
    output
        .sync_all()
        .map_err(|_| SandboxError::SnapshotInvalid("snapshot_write"))?;
    Ok((total, format!("{:x}", hasher.finalize())))
}

fn hash_file(path: &Path, max_bytes: u64) -> Result<String, SandboxError> {
    let mut input = File::open(path).map_err(|_| SandboxError::ResultInvalid("file_read"))?;
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = input
            .read(&mut buffer)
            .map_err(|_| SandboxError::ResultInvalid("file_read"))?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(count as u64)
            .ok_or(SandboxError::ResultInvalid("file_size_limit"))?;
        if total > max_bytes {
            return Err(SandboxError::ResultInvalid("file_size_limit"));
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn ensure_workspace_ancestors_safe(
    workspace_root: &Path,
    relative: &Path,
) -> Result<(), SandboxError> {
    let mut current = workspace_root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        if !current.exists() {
            continue;
        }
        let metadata = fs::symlink_metadata(&current)
            .map_err(|_| SandboxError::Conflict("workspace_changed"))?;
        if is_link_like(&metadata) {
            return Err(SandboxError::Conflict("link_or_reparse_point"));
        }
    }
    Ok(())
}

pub(crate) fn validate_relative_path(
    value: &str,
    allow_root: bool,
) -> Result<PathBuf, SandboxError> {
    if (value.is_empty() && !allow_root)
        || value.len() > 1_024
        || value.contains(['\0', '\\', ':'])
        || value.starts_with('/')
        || value.ends_with('/')
    {
        return Err(SandboxError::ResultInvalid("relative_path"));
    }
    if value.is_empty() && allow_root {
        return Ok(PathBuf::new());
    }
    let path = Path::new(value);
    let mut count = 0usize;
    for component in path.components() {
        let Component::Normal(component) = component else {
            return Err(SandboxError::ResultInvalid("relative_path"));
        };
        let name = component
            .to_str()
            .ok_or(SandboxError::ResultInvalid("relative_path"))?;
        if name.is_empty()
            || name.len() > 255
            || name.ends_with(['.', ' '])
            || is_windows_reserved_name(name)
            || is_protected_component(name)
            || is_sensitive_file_name(name)
        {
            return Err(SandboxError::ResultInvalid("protected_path"));
        }
        count += 1;
        if count > 64 {
            return Err(SandboxError::ResultInvalid("relative_path"));
        }
    }
    Ok(path.to_path_buf())
}

fn relative_to_contract(path: &Path) -> Result<String, SandboxError> {
    let mut parts = Vec::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            return Err(SandboxError::SnapshotInvalid("relative_path"));
        };
        parts.push(
            component
                .to_str()
                .ok_or(SandboxError::SnapshotInvalid("relative_path"))?,
        );
    }
    Ok(parts.join("/"))
}

fn exclusion_reason(path: &Path) -> Option<&'static str> {
    for component in path.components() {
        let name = component.as_os_str().to_string_lossy();
        if is_protected_component(&name) {
            return Some("protected_directory");
        }
        if is_sensitive_file_name(&name) {
            return Some("credential_pattern");
        }
    }
    None
}

fn is_protected_component(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        ".git" | ".misakax" | ".codex" | ".agents" | ".ssh" | ".aws" | ".azure" | ".gnupg"
    )
}

fn is_sensitive_file_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let env_secret = lower == ".env"
        || (lower.starts_with(".env.")
            && !lower.ends_with(".example")
            && !lower.ends_with(".sample")
            && !lower.ends_with(".template"));
    env_secret
        || matches!(
            lower.as_str(),
            "id_rsa"
                | "id_ed25519"
                | "credentials"
                | "credentials.json"
                | ".npmrc"
                | ".pypirc"
                | ".netrc"
        )
        || lower.ends_with(".pem")
        || lower.ends_with(".key")
}

fn is_windows_reserved_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem
            .strip_prefix("COM")
            .or_else(|| stem.strip_prefix("LPT"))
            .is_some_and(|suffix| suffix.len() == 1 && matches!(suffix.as_bytes()[0], b'1'..=b'9'))
}

fn is_link_like(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        false
    }
}
