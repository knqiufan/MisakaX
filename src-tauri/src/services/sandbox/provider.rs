use std::path::{Path, PathBuf};

use async_trait::async_trait;

use super::{
    ExecutionLease, ExecutionPlan, ProviderDescriptor, ResultManifest, SandboxError,
    WorkspaceSnapshot,
};

/// Result bytes stay behind the Rust provider boundary. The public manifest
/// contains only relative paths, hashes and evidence; `result_root` is never
/// serialized to React, Python, Skills or MCP.
#[derive(Debug)]
pub struct ProviderExecutionResult {
    manifest: ResultManifest,
    result_root: PathBuf,
}

impl ProviderExecutionResult {
    pub fn new(manifest: ResultManifest, result_root: PathBuf) -> Self {
        Self {
            manifest,
            result_root,
        }
    }

    pub(crate) fn result_root(&self) -> &Path {
        &self.result_root
    }

    pub fn manifest(&self) -> &ResultManifest {
        &self.manifest
    }
}

#[async_trait]
pub trait SandboxProvider: Send + Sync {
    fn descriptor(&self) -> ProviderDescriptor;

    async fn create_execution(
        &self,
        plan: &ExecutionPlan,
        snapshot: &WorkspaceSnapshot,
    ) -> Result<ExecutionLease, SandboxError>;

    async fn collect_result(
        &self,
        lease: &ExecutionLease,
    ) -> Result<ProviderExecutionResult, SandboxError>;

    async fn cancel_execution(&self, lease: &ExecutionLease) -> Result<(), SandboxError>;

    async fn destroy_execution(&self, lease: &ExecutionLease) -> Result<(), SandboxError>;
}
