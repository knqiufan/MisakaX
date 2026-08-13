//! Provider-neutral execution isolation contracts.
//!
//! This module intentionally does not register a production provider yet. It
//! freezes the S0 control-plane, snapshot and result-validation boundary so
//! future AppContainer, XPC/VM and remote providers cannot silently fall back
//! to a host shell.

mod broker;
mod provider;
mod snapshot;
mod types;

pub use broker::{
    AuditSink, BrokerExecutionOutcome, ExecutionAuditEvent, ExecutionAuditState, SandboxBroker,
};
pub use provider::{ProviderExecutionResult, SandboxProvider};
pub use snapshot::{
    ResultValidationPolicy, SnapshotPolicy, ValidatedArtifact, ValidatedCollectedResult,
    ValidatedWriteback, ValidatedWritebackChange, WorkspaceSnapshot, WorkspaceSnapshotService,
};
pub use types::{
    EvidenceState, ExecutionCommand, ExecutionGuarantee, ExecutionIntent, ExecutionLease,
    ExecutionLocation, ExecutionPlan, ExecutionPolicy, ExecutionSource, IsolationEvidence,
    NetworkMode, OutputKind, ProviderAvailability, ProviderDescriptor, ResourceBudget,
    ResultArtifact, ResultChange, ResultChangeAction, ResultManifest, SandboxError,
    SandboxErrorCode, SnapshotEntry, SnapshotExclusion, SnapshotManifest, WorkspaceDelivery,
    WritebackApproval,
};

#[cfg(test)]
mod tests;
