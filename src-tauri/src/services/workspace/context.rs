use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use futures::future::{BoxFuture, Shared};
use futures::FutureExt;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::contracts::{DomainEvent, WorkspaceContext, WorkspaceContextDiagnostic, WorkspaceKind};

use super::types::{CanonicalWorkspace, DetectionCancellation, VcsContext, VcsProvider};

const CACHE_TTL: Duration = Duration::from_secs(3);
const WATCH_DEBOUNCE: Duration = Duration::from_millis(300);

type CacheKey = (String, u64);
type SharedDetection = Shared<BoxFuture<'static, DetectionOutcome>>;
type EventSink = Arc<dyn Fn(DomainEvent<WorkspaceContext>) + Send + Sync>;
type TaskSpawner = Arc<dyn Fn(BoxFuture<'static, ()>) + Send + Sync>;

#[derive(Clone)]
struct Binding {
    workspace: CanonicalWorkspace,
    generation: u64,
}

#[derive(Clone)]
struct CacheEntry {
    context: WorkspaceContext,
    inserted_at: Instant,
}

#[derive(Clone)]
struct DetectionOutcome {
    context: WorkspaceContext,
    vcs: Option<VcsContext>,
}

struct InFlight {
    future: SharedDetection,
    cancellation: DetectionCancellation,
}

struct WorkspaceWatcher {
    generation: u64,
    _watcher: RecommendedWatcher,
}

pub struct WorkspaceContextService {
    provider: Arc<dyn VcsProvider>,
    next_generation: AtomicU64,
    bindings: Mutex<HashMap<String, Binding>>,
    cache: Mutex<HashMap<CacheKey, CacheEntry>>,
    in_flight: Mutex<HashMap<CacheKey, InFlight>>,
    watchers: Mutex<HashMap<String, WorkspaceWatcher>>,
    refresh_tickets: Mutex<HashMap<String, u64>>,
    event_sink: Mutex<Option<EventSink>>,
    task_spawner: Mutex<Option<TaskSpawner>>,
}

impl WorkspaceContextService {
    pub fn new(provider: Arc<dyn VcsProvider>) -> Self {
        Self {
            provider,
            next_generation: AtomicU64::new(1),
            bindings: Mutex::new(HashMap::new()),
            cache: Mutex::new(HashMap::new()),
            in_flight: Mutex::new(HashMap::new()),
            watchers: Mutex::new(HashMap::new()),
            refresh_tickets: Mutex::new(HashMap::new()),
            event_sink: Mutex::new(None),
            task_spawner: Mutex::new(None),
        }
    }

    pub fn start(
        &self,
        event_sink: impl Fn(DomainEvent<WorkspaceContext>) + Send + Sync + 'static,
        task_spawner: impl Fn(BoxFuture<'static, ()>) + Send + Sync + 'static,
    ) {
        *self
            .event_sink
            .lock()
            .expect("workspace event sink mutex poisoned") = Some(Arc::new(event_sink));
        *self
            .task_spawner
            .lock()
            .expect("workspace task spawner mutex poisoned") = Some(Arc::new(task_spawner));
    }

    pub async fn get_context(
        self: &Arc<Self>,
        session_id: &str,
        workspace: CanonicalWorkspace,
        force_refresh: bool,
    ) -> WorkspaceContext {
        let binding = self.bind(session_id, workspace);
        let key = cache_key(&binding);
        if force_refresh {
            self.cache
                .lock()
                .expect("workspace cache mutex poisoned")
                .remove(&key);
        } else if let Some(context) = self.cached_context(&key) {
            return context;
        }

        let future = {
            let mut flights = self
                .in_flight
                .lock()
                .expect("workspace flight mutex poisoned");
            if let Some(flight) = flights.get(&key) {
                flight.future.clone()
            } else {
                let provider = Arc::clone(&self.provider);
                let workspace = binding.workspace.clone();
                let generation = binding.generation;
                let cancellation = DetectionCancellation::default();
                let future_cancellation = cancellation.clone();
                let future = async move {
                    detect_context(provider, workspace, generation, future_cancellation).await
                }
                .boxed()
                .shared();
                flights.insert(
                    key.clone(),
                    InFlight {
                        future: future.clone(),
                        cancellation,
                    },
                );
                future
            }
        };

        let outcome = future.await;
        self.in_flight
            .lock()
            .expect("workspace flight mutex poisoned")
            .remove(&key);

        if self.binding_matches(session_id, &binding) {
            self.cache
                .lock()
                .expect("workspace cache mutex poisoned")
                .insert(
                    key,
                    CacheEntry {
                        context: outcome.context.clone(),
                        inserted_at: Instant::now(),
                    },
                );
            self.install_watcher(session_id, &binding, outcome.vcs.as_ref());
        }

        outcome.context
    }

    pub fn unbind_session(&self, session_id: &str) {
        let old = self
            .bindings
            .lock()
            .expect("workspace binding mutex poisoned")
            .remove(session_id);
        if let Some(old) = old {
            let key = cache_key(&old);
            self.cache
                .lock()
                .expect("workspace cache mutex poisoned")
                .remove(&key);
            if let Some(flight) = self
                .in_flight
                .lock()
                .expect("workspace flight mutex poisoned")
                .remove(&key)
            {
                flight.cancellation.cancel();
            }
        }
        self.watchers
            .lock()
            .expect("workspace watcher mutex poisoned")
            .remove(session_id);
    }

    pub fn request_refresh_all(self: &Arc<Self>) {
        let sessions = self
            .bindings
            .lock()
            .expect("workspace binding mutex poisoned")
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for session_id in sessions {
            self.request_refresh(&session_id);
        }
    }

    pub fn request_refresh(self: &Arc<Self>, session_id: &str) {
        let ticket = {
            let mut tickets = self
                .refresh_tickets
                .lock()
                .expect("workspace refresh mutex poisoned");
            let ticket = tickets.get(session_id).copied().unwrap_or_default() + 1;
            tickets.insert(session_id.to_string(), ticket);
            ticket
        };
        let service = Arc::clone(self);
        let session_id = session_id.to_string();
        let task = async move {
            tokio::time::sleep(WATCH_DEBOUNCE).await;
            let current_ticket = service
                .refresh_tickets
                .lock()
                .expect("workspace refresh mutex poisoned")
                .get(&session_id)
                .copied();
            if current_ticket != Some(ticket) {
                return;
            }
            service.refresh_and_emit(&session_id).await;
        }
        .boxed();
        if let Some(spawner) = self
            .task_spawner
            .lock()
            .expect("workspace task spawner mutex poisoned")
            .as_ref()
            .cloned()
        {
            spawner(task);
        }
    }

    async fn refresh_and_emit(self: &Arc<Self>, session_id: &str) {
        let binding = self
            .bindings
            .lock()
            .expect("workspace binding mutex poisoned")
            .get(session_id)
            .cloned();
        let Some(binding) = binding else {
            return;
        };
        let key = cache_key(&binding);
        let before = self
            .cache
            .lock()
            .expect("workspace cache mutex poisoned")
            .remove(&key)
            .map(|entry| entry.context);
        let after = self
            .get_context(session_id, binding.workspace.clone(), true)
            .await;
        if before
            .as_ref()
            .is_some_and(|before| semantically_equal(before, &after))
        {
            return;
        }
        let event = DomainEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            aggregate_id: session_id.to_string(),
            generation: after.generation,
            occurred_at: chrono::Utc::now().to_rfc3339(),
            payload: after,
        };
        if let Some(sink) = self
            .event_sink
            .lock()
            .expect("workspace event sink mutex poisoned")
            .as_ref()
            .cloned()
        {
            sink(event);
        }
    }

    fn bind(&self, session_id: &str, workspace: CanonicalWorkspace) -> Binding {
        let mut bindings = self
            .bindings
            .lock()
            .expect("workspace binding mutex poisoned");
        if let Some(existing) = bindings.get(session_id) {
            if existing.workspace == workspace {
                return existing.clone();
            }
            let old_key = cache_key(existing);
            self.cache
                .lock()
                .expect("workspace cache mutex poisoned")
                .remove(&old_key);
            if let Some(flight) = self
                .in_flight
                .lock()
                .expect("workspace flight mutex poisoned")
                .remove(&old_key)
            {
                flight.cancellation.cancel();
            }
            self.watchers
                .lock()
                .expect("workspace watcher mutex poisoned")
                .remove(session_id);
        }
        let binding = Binding {
            workspace,
            generation: self.next_generation.fetch_add(1, Ordering::Relaxed),
        };
        bindings.insert(session_id.to_string(), binding.clone());
        binding
    }

    fn cached_context(&self, key: &CacheKey) -> Option<WorkspaceContext> {
        let mut cache = self.cache.lock().expect("workspace cache mutex poisoned");
        let entry = cache.get(key)?;
        if entry.inserted_at.elapsed() <= CACHE_TTL {
            return Some(entry.context.clone());
        }
        cache.remove(key);
        None
    }

    fn binding_matches(&self, session_id: &str, expected: &Binding) -> bool {
        self.bindings
            .lock()
            .expect("workspace binding mutex poisoned")
            .get(session_id)
            .is_some_and(|current| {
                current.generation == expected.generation && current.workspace == expected.workspace
            })
    }

    fn install_watcher(
        self: &Arc<Self>,
        session_id: &str,
        binding: &Binding,
        vcs: Option<&VcsContext>,
    ) {
        let Some(vcs) = vcs else {
            self.watchers
                .lock()
                .expect("workspace watcher mutex poisoned")
                .remove(session_id);
            return;
        };
        let mut watchers = self
            .watchers
            .lock()
            .expect("workspace watcher mutex poisoned");
        if watchers
            .get(session_id)
            .is_some_and(|watcher| watcher.generation == binding.generation)
        {
            return;
        }

        let weak: Weak<Self> = Arc::downgrade(self);
        let watched_session = session_id.to_string();
        let watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            if event.is_ok() {
                if let Some(service) = weak.upgrade() {
                    service.request_refresh(&watched_session);
                }
            }
        });
        let Ok(mut watcher) = watcher else {
            tracing::warn!("Unable to create workspace Git watcher");
            return;
        };
        let mut watched = false;
        for (path, mode) in watch_paths(vcs) {
            if path.exists() && watcher.watch(&path, mode).is_ok() {
                watched = true;
            }
        }
        if watched {
            watchers.insert(
                session_id.to_string(),
                WorkspaceWatcher {
                    generation: binding.generation,
                    _watcher: watcher,
                },
            );
        }
    }
}

async fn detect_context(
    provider: Arc<dyn VcsProvider>,
    workspace: CanonicalWorkspace,
    generation: u64,
    cancellation: DetectionCancellation,
) -> DetectionOutcome {
    match provider.detect(&workspace, cancellation).await {
        Ok(Some(vcs)) => DetectionOutcome {
            context: WorkspaceContext {
                workspace_path: workspace.display_path(),
                kind: WorkspaceKind::Git,
                repository_root: Some(vcs.repository_root.to_string_lossy().into_owned()),
                branch: vcs.branch.clone(),
                detached_head: vcs.detached_head.clone(),
                generation,
                diagnostic: None,
            },
            vcs: Some(vcs),
        },
        Ok(None) => DetectionOutcome {
            context: local_context(workspace, generation, None),
            vcs: None,
        },
        Err(diagnostic) => {
            let correlation_id = uuid::Uuid::new_v4().to_string();
            tracing::warn!(
                reason = diagnostic.internal_reason,
                correlation_id,
                "Workspace Git detection degraded to local context"
            );
            let public = WorkspaceContextDiagnostic {
                code: diagnostic.code,
                message_key: diagnostic.message_key.to_string(),
                retryable: diagnostic.retryable,
                correlation_id,
            };
            DetectionOutcome {
                context: local_context(workspace, generation, Some(public)),
                vcs: None,
            }
        }
    }
}

fn local_context(
    workspace: CanonicalWorkspace,
    generation: u64,
    diagnostic: Option<WorkspaceContextDiagnostic>,
) -> WorkspaceContext {
    WorkspaceContext {
        workspace_path: workspace.display_path(),
        kind: WorkspaceKind::Local,
        repository_root: None,
        branch: None,
        detached_head: None,
        generation,
        diagnostic,
    }
}

fn cache_key(binding: &Binding) -> CacheKey {
    (binding.workspace.display_path(), binding.generation)
}

fn watch_paths(vcs: &VcsContext) -> Vec<(PathBuf, RecursiveMode)> {
    vec![
        (vcs.git_dir.join("HEAD"), RecursiveMode::NonRecursive),
        (vcs.git_common_dir.join("refs"), RecursiveMode::Recursive),
        (
            vcs.git_common_dir.join("packed-refs"),
            RecursiveMode::NonRecursive,
        ),
    ]
}

fn semantically_equal(left: &WorkspaceContext, right: &WorkspaceContext) -> bool {
    left.workspace_path == right.workspace_path
        && left.kind == right.kind
        && left.repository_root == right.repository_root
        && left.branch == right.branch
        && left.detached_head == right.detached_head
        && left.diagnostic.as_ref().map(|item| item.code)
            == right.diagnostic.as_ref().map(|item| item.code)
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use tempfile::TempDir;

    use crate::contracts::AppErrorCode;

    use super::*;
    use crate::services::workspace::types::{DetectionCancellation, VcsDiagnostic};

    struct StaticProvider {
        result: Mutex<Result<Option<VcsContext>, VcsDiagnostic>>,
        calls: AtomicU64,
        delay: Duration,
    }

    #[async_trait]
    impl VcsProvider for StaticProvider {
        async fn detect(
            &self,
            _workspace: &CanonicalWorkspace,
            _cancellation: DetectionCancellation,
        ) -> Result<Option<VcsContext>, VcsDiagnostic> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            tokio::time::sleep(self.delay).await;
            self.result.lock().unwrap().clone()
        }
    }

    fn workspace() -> (TempDir, CanonicalWorkspace) {
        let temp = tempfile::tempdir().unwrap();
        let workspace = CanonicalWorkspace::new(temp.path()).unwrap();
        (temp, workspace)
    }

    #[tokio::test]
    async fn caches_single_workspace_generation_and_rebinds_monotonically() {
        let provider = Arc::new(StaticProvider {
            result: Mutex::new(Ok(None)),
            calls: AtomicU64::new(0),
            delay: Duration::ZERO,
        });
        let service = Arc::new(WorkspaceContextService::new(provider.clone()));
        let (_first_dir, first) = workspace();
        let (_second_dir, second) = workspace();

        let first_result = service.get_context("session", first.clone(), false).await;
        let cached = service.get_context("session", first, false).await;
        let rebound = service.get_context("session", second, false).await;

        assert_eq!(first_result.generation, cached.generation);
        assert!(rebound.generation > cached.generation);
        assert_eq!(provider.calls.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn provider_failure_is_redacted_to_local_context() {
        let provider = Arc::new(StaticProvider {
            result: Mutex::new(Err(VcsDiagnostic::git_unavailable("secret/internal/path"))),
            calls: AtomicU64::new(0),
            delay: Duration::ZERO,
        });
        let service = Arc::new(WorkspaceContextService::new(provider));
        let (_dir, workspace) = workspace();

        let result = service.get_context("session", workspace, false).await;

        assert_eq!(result.kind, WorkspaceKind::Local);
        assert_eq!(
            result.diagnostic.as_ref().map(|item| item.code),
            Some(AppErrorCode::GitNotAvailable)
        );
        assert!(!serde_json::to_string(&result)
            .unwrap()
            .contains("secret/internal/path"));
    }

    #[tokio::test]
    async fn concurrent_requests_share_one_provider_flight() {
        let provider = Arc::new(StaticProvider {
            result: Mutex::new(Ok(None)),
            calls: AtomicU64::new(0),
            delay: Duration::from_millis(25),
        });
        let service = Arc::new(WorkspaceContextService::new(provider.clone()));
        let (_dir, workspace) = workspace();

        let (left, right) = tokio::join!(
            service.get_context("session", workspace.clone(), false),
            service.get_context("session", workspace, false),
        );

        assert_eq!(left.generation, right.generation);
        assert_eq!(provider.calls.load(Ordering::Relaxed), 1);
    }
}
