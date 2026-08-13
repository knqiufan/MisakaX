use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::provider::ProviderExecutionResult;
use super::snapshot::{validate_collected_result, ResultValidationPolicy};
use super::{
    ExecutionIntent, ExecutionLease, ExecutionLocation, ExecutionPlan, ExecutionPolicy,
    SandboxError, SandboxProvider, ValidatedCollectedResult, WorkspaceSnapshot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionAuditState {
    Preparing,
    Running,
    Collecting,
    Destroying,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionAuditEvent {
    pub execution_id: String,
    pub session_id: String,
    pub provider_id: Option<String>,
    pub location: ExecutionLocation,
    pub state: ExecutionAuditState,
    pub policy_hash: String,
    pub error_code: Option<super::SandboxErrorCode>,
    pub occurred_at: String,
}

pub trait AuditSink: Send + Sync {
    fn record(&self, event: ExecutionAuditEvent);
}

#[derive(Default)]
struct NoopAuditSink;

impl AuditSink for NoopAuditSink {
    fn record(&self, _event: ExecutionAuditEvent) {}
}

#[derive(Debug)]
pub struct BrokerExecutionOutcome {
    plan: ExecutionPlan,
    lease: ExecutionLease,
    result: ProviderExecutionResult,
    validated: ValidatedCollectedResult,
}

impl BrokerExecutionOutcome {
    pub fn plan(&self) -> &ExecutionPlan {
        &self.plan
    }

    pub fn lease(&self) -> &ExecutionLease {
        &self.lease
    }

    pub fn result_manifest(&self) -> &super::ResultManifest {
        self.result.manifest()
    }

    pub fn validated(&self) -> &ValidatedCollectedResult {
        &self.validated
    }
}

/// Execution Isolation Broker control plane.
///
/// Provider selection is exact. A missing, unhealthy or experimental provider
/// never causes a retry through `host_direct` or any other location.
pub struct SandboxBroker {
    providers: BTreeMap<ExecutionLocation, Arc<dyn SandboxProvider>>,
    audit: Arc<dyn AuditSink>,
    result_policy: ResultValidationPolicy,
}

impl SandboxBroker {
    pub fn unavailable() -> Self {
        Self {
            providers: BTreeMap::new(),
            audit: Arc::new(NoopAuditSink),
            result_policy: ResultValidationPolicy::default(),
        }
    }

    pub fn new(
        providers: Vec<Arc<dyn SandboxProvider>>,
        audit: Arc<dyn AuditSink>,
        result_policy: ResultValidationPolicy,
    ) -> Result<Self, SandboxError> {
        let mut registry = BTreeMap::new();
        for provider in providers {
            let descriptor = provider.descriptor();
            descriptor.validate()?;
            if registry.insert(descriptor.location, provider).is_some() {
                return Err(SandboxError::PolicyInvalid("provider_registry"));
            }
        }
        Ok(Self {
            providers: registry,
            audit,
            result_policy,
        })
    }

    pub async fn execute(
        &self,
        intent: ExecutionIntent,
        policy: ExecutionPolicy,
        snapshot: &WorkspaceSnapshot,
    ) -> Result<BrokerExecutionOutcome, SandboxError> {
        let plan = ExecutionPlan::compile(intent, policy, snapshot.manifest())?;
        self.audit(&plan, None, ExecutionAuditState::Preparing, None);

        let provider = self.providers.get(&plan.policy.location).ok_or_else(|| {
            self.audit(
                &plan,
                None,
                ExecutionAuditState::Failed,
                Some(&SandboxError::Unavailable),
            );
            SandboxError::Unavailable
        })?;
        let descriptor = provider.descriptor();
        if descriptor.location != plan.policy.location
            || !descriptor
                .availability
                .can_execute(plan.policy.allow_experimental_provider)
        {
            self.audit(
                &plan,
                Some(&descriptor.provider_id),
                ExecutionAuditState::Failed,
                Some(&SandboxError::Unavailable),
            );
            return Err(SandboxError::Unavailable);
        }
        if plan.policy.guarantee == super::ExecutionGuarantee::Strict
            && !descriptor.capabilities.strict_gaps().is_empty()
        {
            self.audit(
                &plan,
                Some(&descriptor.provider_id),
                ExecutionAuditState::Failed,
                Some(&SandboxError::Unavailable),
            );
            return Err(SandboxError::Unavailable);
        }

        let lease = match provider.create_execution(&plan, snapshot).await {
            Ok(lease) => match lease.validate(&plan.execution_id, &descriptor.provider_id) {
                Ok(()) => lease,
                Err(error) => {
                    let _ = provider.destroy_execution(&lease).await;
                    self.audit(
                        &plan,
                        Some(&descriptor.provider_id),
                        ExecutionAuditState::Failed,
                        Some(&error),
                    );
                    return Err(error);
                }
            },
            Err(error) => {
                self.audit(
                    &plan,
                    Some(&descriptor.provider_id),
                    ExecutionAuditState::Failed,
                    Some(&error),
                );
                return Err(error);
            }
        };
        self.audit(
            &plan,
            Some(&descriptor.provider_id),
            ExecutionAuditState::Running,
            None,
        );
        self.audit(
            &plan,
            Some(&descriptor.provider_id),
            ExecutionAuditState::Collecting,
            None,
        );
        let collected = provider.collect_result(&lease).await;
        self.audit(
            &plan,
            Some(&descriptor.provider_id),
            ExecutionAuditState::Destroying,
            None,
        );
        let destroyed = provider.destroy_execution(&lease).await;

        let result = match (collected, destroyed) {
            (Ok(result), Ok(())) => result,
            (Err(error), _) => {
                self.audit(
                    &plan,
                    Some(&descriptor.provider_id),
                    ExecutionAuditState::Failed,
                    Some(&error),
                );
                return Err(error);
            }
            (Ok(_), Err(error)) => {
                self.audit(
                    &plan,
                    Some(&descriptor.provider_id),
                    ExecutionAuditState::Failed,
                    Some(&error),
                );
                return Err(error);
            }
        };
        if result.manifest().execution_id != plan.execution_id
            || result.manifest().provider_id != descriptor.provider_id
            || result.manifest().base_generation != plan.intent.base_generation
        {
            let error = SandboxError::ResultInvalid("result_identity");
            self.audit(
                &plan,
                Some(&descriptor.provider_id),
                ExecutionAuditState::Failed,
                Some(&error),
            );
            return Err(error);
        }
        if plan.policy.guarantee == super::ExecutionGuarantee::Strict
            && !result.manifest().evidence.strict_gaps().is_empty()
        {
            let error = SandboxError::ResultInvalid("isolation_evidence");
            self.audit(
                &plan,
                Some(&descriptor.provider_id),
                ExecutionAuditState::Failed,
                Some(&error),
            );
            return Err(error);
        }
        let validated = match validate_collected_result(
            snapshot,
            result.result_root(),
            result.manifest(),
            self.result_policy,
        ) {
            Ok(validated) => validated,
            Err(error) => {
                self.audit(
                    &plan,
                    Some(&descriptor.provider_id),
                    ExecutionAuditState::Failed,
                    Some(&error),
                );
                return Err(error);
            }
        };
        self.audit(
            &plan,
            Some(&descriptor.provider_id),
            ExecutionAuditState::Succeeded,
            None,
        );
        Ok(BrokerExecutionOutcome {
            plan,
            lease,
            result,
            validated,
        })
    }

    fn audit(
        &self,
        plan: &ExecutionPlan,
        provider_id: Option<&str>,
        state: ExecutionAuditState,
        error: Option<&SandboxError>,
    ) {
        self.audit.record(ExecutionAuditEvent {
            execution_id: plan.execution_id.clone(),
            session_id: plan.intent.session_id.clone(),
            provider_id: provider_id.map(str::to_string),
            location: plan.policy.location,
            state,
            policy_hash: plan.plan_digest.clone(),
            error_code: error.map(SandboxError::code),
            occurred_at: chrono::Utc::now().to_rfc3339(),
        });
    }
}
