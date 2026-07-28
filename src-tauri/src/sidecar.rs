use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::{watch, Mutex as AsyncMutex};

use crate::db::models::RouterConfig;

const STARTUP_MAX_RETRIES: u32 = 3;
const RUNTIME_MAX_RESTARTS: u32 = 3;
const STARTUP_RETRY_DELAY: Duration = Duration::from_secs(2);
const RESTART_DELAY: Duration = Duration::from_millis(500);
/// Poll the managed child frequently so a process exit is handled promptly.
const WATCHDOG_CHILD_POLL_INTERVAL: Duration = Duration::from_secs(5);
/// Probe the HTTP service less often; a healthy child process is the common case.
const WATCHDOG_HTTP_LIVENESS_INTERVAL: Duration = Duration::from_secs(30);
const WATCHDOG_HTTP_LIVENESS_POLL_COUNT: u64 =
    WATCHDOG_HTTP_LIVENESS_INTERVAL.as_secs() / WATCHDOG_CHILD_POLL_INTERVAL.as_secs();
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(2);
/// Keep the local HTTP connection alive across the 30-second liveness interval.
const HEALTH_CLIENT_POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(40);
const STARTUP_HEALTH_CHECK_ATTEMPTS: u32 = 20;
const STARTUP_HEALTH_CHECK_INTERVAL: Duration = Duration::from_millis(500);

// --------------------------------------------------------------------------- //
//  Status types
// --------------------------------------------------------------------------- //

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SidecarStatus {
    Stopped,
    Starting,
    Ready,
    Error,
    Restarting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarStatusEvent {
    pub status: SidecarStatus,
    pub message: Option<String>,
    pub port: u16,
}

pub fn health_check_url(port: u16) -> String {
    format!("http://127.0.0.1:{}/health", port)
}

pub fn should_attempt_runtime_restart(next_count: u32, max_restarts: u32) -> bool {
    next_count <= max_restarts
}

pub fn is_current_watchdog_generation(observed: u64, current: u64) -> bool {
    observed == current
}

/// Return whether the current child-process poll should also perform an HTTP
/// liveness probe. Poll counts start at one after the Sidecar becomes ready.
pub fn is_runtime_health_check_due(child_poll_count: u64) -> bool {
    child_poll_count > 0 && child_poll_count % WATCHDOG_HTTP_LIVENESS_POLL_COUNT == 0
}

/// Use the packaged binary only for release builds.
///
/// A stale `agent/dist/misaka-agent` must not shadow Python source changes
/// while `tauri dev` is running.
pub fn should_use_packaged_sidecar() -> bool {
    !cfg!(debug_assertions)
}

// --------------------------------------------------------------------------- //
//  Agent directory resolution
// --------------------------------------------------------------------------- //

/// Resolve the working directory for the Python sidecar (`python run.py`).
///
/// Resolution order:
/// 1. Repo layout: `agent/` next to `src-tauri/` (from compile-time `CARGO_MANIFEST_DIR`) — fixes
///    `tauri dev` where cwd is `src-tauri/` (naive `cwd.join("agent")` does not exist → Windows
///    **ERROR_DIRECTORY 267** on spawn).
/// 2. `agent/` next to the executable (packaged / portable installs).
/// 3. `./agent` from process cwd (legacy).
///
/// If none exist, returns the repo-layout path for error messages / logs.
pub fn resolve_agent_working_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let from_repo_root = manifest_dir
        .parent()
        .map(|p| p.join("agent"))
        .unwrap_or_else(|| manifest_dir.join("agent"));

    if from_repo_root.is_dir() {
        return from_repo_root;
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let beside_exe = dir.join("agent");
            if beside_exe.is_dir() {
                return beside_exe;
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let cwd_agent = cwd.join("agent");
        if cwd_agent.is_dir() {
            return cwd_agent;
        }
    }

    from_repo_root
}

// --------------------------------------------------------------------------- //
//  Packaged sidecar (Nuitka) executable resolution
// --------------------------------------------------------------------------- //

/// Base name of the packaged sidecar produced by `agent/build_nuitka.py`.
pub const SIDECAR_BINARY_STEM: &str = "misaka-agent";

/// Platform-specific file name of the packaged sidecar binary.
pub fn sidecar_executable_name() -> String {
    if cfg!(windows) {
        format!("{}.exe", SIDECAR_BINARY_STEM)
    } else {
        SIDECAR_BINARY_STEM.to_string()
    }
}

/// Pure lookup: return the first `exe_name` that exists as a file in `dirs`.
///
/// Extracted so it can be unit-tested without touching `current_exe()` / the
/// real filesystem layout.
pub fn find_sidecar_executable_in(dirs: &[PathBuf], exe_name: &str) -> Option<PathBuf> {
    dirs.iter()
        .map(|dir| dir.join(exe_name))
        .find(|candidate| candidate.is_file())
}

/// Resolve a packaged sidecar binary if one is available.
///
/// Probe order:
/// 1. `agent_dir/dist/misaka-agent(.exe)` — output of `build_nuitka.py`.
/// 2. `misaka-agent(.exe)` next to the current executable (packaged installs).
///
/// Returns `None` in dev when only Python sources exist, so the caller falls
/// back to `python -m uvicorn`.
pub fn resolve_sidecar_executable(agent_dir: &Path) -> Option<PathBuf> {
    let exe_name = sidecar_executable_name();

    let mut dirs: Vec<PathBuf> = vec![agent_dir.join("dist")];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            dirs.push(dir.to_path_buf());
        }
    }

    find_sidecar_executable_in(&dirs, &exe_name)
}

// --------------------------------------------------------------------------- //
//  SidecarManager
// --------------------------------------------------------------------------- //

pub struct SidecarManager {
    port: u16,
    /// Shared localhost client for startup and runtime health probes.
    health_client: reqwest::Client,
    /// Local MCP HTTP bridge port exposed by Rust for Python tools.
    mcp_bridge_port: u16,
    agent_dir: PathBuf,
    /// Path to `~/.misakax/data/sidecar-runtime.json`.
    runtime_record_path: PathBuf,
    /// Decrypted provider keys for child env (never logged).
    api_key_env: HashMap<String, String>,
    max_retries: u32,
    max_runtime_restarts: u32,
    status: Arc<Mutex<SidecarStatus>>,
    child: Arc<Mutex<Option<Child>>>,
    shutdown_tx: watch::Sender<bool>,
    shutdown_rx: watch::Receiver<bool>,
    runtime_restart_count: Arc<Mutex<u32>>,
    lifecycle_lock: Arc<AsyncMutex<()>>,
    watchdog_active: Arc<Mutex<bool>>,
    lifecycle_generation: Arc<Mutex<u64>>,
    /// MCP HTTP bridge must be listening before Sidecar can become Ready.
    mcp_bridge_ready: Arc<Mutex<bool>>,
}

/// Input for selecting which decrypted keys to inject into the sidecar process.
#[derive(Debug, Clone)]
pub struct SidecarApiKeySource {
    pub provider: String,
    pub api_compat: Option<String>,
    pub is_active: bool,
    pub decrypted_key: String,
}

/// Build MISAKA_* API key env vars from active router configs.
///
/// Preference: exact provider match (`anthropic` / `openai`), then custom
/// providers with matching `api_compat`. First match wins (caller should pass
/// configs in `created_at DESC` order). Never logs key values.
pub fn build_sidecar_api_key_env(sources: &[SidecarApiKeySource]) -> HashMap<String, String> {
    let mut env = HashMap::new();
    for source in sources {
        if !source.is_active || source.decrypted_key.trim().is_empty() {
            continue;
        }
        let provider = source.provider.to_ascii_lowercase();
        let compat = source
            .api_compat
            .as_deref()
            .unwrap_or("")
            .to_ascii_lowercase();

        if !env.contains_key("MISAKA_ANTHROPIC_API_KEY")
            && (provider == "anthropic" || compat == "anthropic")
        {
            env.insert(
                "MISAKA_ANTHROPIC_API_KEY".to_string(),
                source.decrypted_key.clone(),
            );
        }
        if !env.contains_key("MISAKA_OPENAI_API_KEY")
            && (provider == "openai" || compat == "openai")
        {
            env.insert(
                "MISAKA_OPENAI_API_KEY".to_string(),
                source.decrypted_key.clone(),
            );
        }
        if env.len() == 2 {
            break;
        }
    }
    env
}

/// Decrypt active router configs into SidecarApiKeySource list (created_at DESC).
pub fn collect_sidecar_api_key_sources(configs: &[RouterConfig]) -> Vec<SidecarApiKeySource> {
    let mut sources = Vec::new();
    for config in configs {
        if !config.is_active {
            continue;
        }
        let Some(encrypted) = config.api_key_encrypted.as_deref() else {
            continue;
        };
        match crate::crypto::decrypt(encrypted) {
            Ok(decrypted_key) => sources.push(SidecarApiKeySource {
                provider: config.provider.clone(),
                api_compat: config.api_compat.clone(),
                is_active: config.is_active,
                decrypted_key,
            }),
            Err(e) => {
                tracing::warn!(
                    provider = %config.provider,
                    error = %e,
                    "Failed to decrypt router API key for sidecar env"
                );
            }
        }
    }
    sources
}

impl SidecarManager {
    pub fn new(agent_dir: PathBuf, port: u16) -> Self {
        Self::with_mcp_bridge_port(agent_dir, port, 9528)
    }

    pub fn with_mcp_bridge_port(agent_dir: PathBuf, port: u16, mcp_bridge_port: u16) -> Self {
        Self::with_options(agent_dir, port, mcp_bridge_port, HashMap::new())
    }

    pub fn with_options(
        agent_dir: PathBuf,
        port: u16,
        mcp_bridge_port: u16,
        api_key_env: HashMap<String, String>,
    ) -> Self {
        let runtime_record_path = crate::config::config_dir()
            .map(|root| crate::sidecar_ownership::runtime_record_path(&root.join("data")))
            .unwrap_or_else(|_| {
                PathBuf::from(".").join("sidecar-runtime.json")
            });
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let health_client = reqwest::Client::builder()
            .connect_timeout(HEALTH_CHECK_TIMEOUT)
            .timeout(HEALTH_CHECK_TIMEOUT)
            .pool_idle_timeout(HEALTH_CLIENT_POOL_IDLE_TIMEOUT)
            .pool_max_idle_per_host(1)
            .build()
            .expect("Failed to build Sidecar health-check HTTP client");
        Self {
            port,
            health_client,
            mcp_bridge_port,
            agent_dir,
            runtime_record_path,
            api_key_env,
            max_retries: STARTUP_MAX_RETRIES,
            max_runtime_restarts: RUNTIME_MAX_RESTARTS,
            status: Arc::new(Mutex::new(SidecarStatus::Stopped)),
            child: Arc::new(Mutex::new(None)),
            shutdown_tx,
            shutdown_rx,
            runtime_restart_count: Arc::new(Mutex::new(0)),
            lifecycle_lock: Arc::new(AsyncMutex::new(())),
            watchdog_active: Arc::new(Mutex::new(false)),
            lifecycle_generation: Arc::new(Mutex::new(0)),
            mcp_bridge_ready: Arc::new(Mutex::new(false)),
        }
    }

    pub fn set_mcp_bridge_ready(&self, ready: bool) {
        *self.mcp_bridge_ready.lock().unwrap() = ready;
    }

    pub fn mcp_bridge_is_ready(&self) -> bool {
        *self.mcp_bridge_ready.lock().unwrap()
    }

    /// Chat/stream may proceed only when this process owns a Ready Sidecar.
    pub fn is_ready_for_chat(&self) -> bool {
        self.status() == SidecarStatus::Ready && self.mcp_bridge_is_ready()
    }

    pub fn not_ready_message(&self) -> String {
        match self.status() {
            SidecarStatus::Ready if !self.mcp_bridge_is_ready() => {
                format!(
                    "MCP HTTP bridge is not ready on port {}. Sidecar chat is blocked.",
                    self.mcp_bridge_port
                )
            }
            SidecarStatus::Ready => "Sidecar is ready.".to_string(),
            SidecarStatus::Starting | SidecarStatus::Restarting => {
                "Sidecar is still starting. Wait until it is ready.".to_string()
            }
            SidecarStatus::Stopped => "Sidecar is stopped.".to_string(),
            SidecarStatus::Error => {
                "Sidecar is not ready. Check Sidecar status and free ports 9527/9528 if needed."
                    .to_string()
            }
        }
    }

    fn mcp_bridge_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.mcp_bridge_port)
    }

    fn apply_common_env(&self, cmd: &mut Command) {
        cmd.env("MISAKA_HOST", "127.0.0.1")
            .env("MISAKA_PORT", self.port.to_string())
            .env("MISAKA_MCP_BRIDGE_URL", self.mcp_bridge_url());
        for (key, value) in &self.api_key_env {
            cmd.env(key, value);
        }
        if !self.api_key_env.is_empty() {
            tracing::info!(
                keys = ?self.api_key_env.keys().collect::<Vec<_>>(),
                "Injected sidecar API key env vars"
            );
        }
    }

    pub fn status(&self) -> SidecarStatus {
        *self.status.lock().unwrap()
    }

    /// Start the sidecar process asynchronously in a background task.
    pub fn preheat(self: &Arc<Self>, app: AppHandle) {
        let mgr = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            mgr.start_process(&app).await;
        });
    }

    /// Restart the sidecar (stop then start).
    pub async fn restart(self: &Arc<Self>, app: AppHandle) {
        let _guard = self.lifecycle_lock.lock().await;
        self.advance_lifecycle_generation();
        self.reset_runtime_restart_count();
        self.set_status(SidecarStatus::Restarting, None, &app);
        self.stop_process();
        tokio::time::sleep(RESTART_DELAY).await;
        self.start_process_inner(&app).await;
    }

    /// Graceful shutdown: send kill, wait for exit.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
        self.set_status_only(SidecarStatus::Stopped);
        self.stop_process();
    }

    // ---------------------------------------------------------------------- //
    //  Internal helpers
    // ---------------------------------------------------------------------- //

    async fn start_process(self: &Arc<Self>, app: &AppHandle) {
        let _guard = self.lifecycle_lock.lock().await;
        self.start_process_inner(app).await;
    }

    async fn start_process_inner(self: &Arc<Self>, app: &AppHandle) {
        self.set_status(SidecarStatus::Starting, None, app);

        if !self.wait_for_mcp_bridge_ready().await {
            let msg = format!(
                "MCP HTTP bridge is not listening on port {}. Sidecar will not start.",
                self.mcp_bridge_port
            );
            tracing::error!("{}", msg);
            self.set_status(SidecarStatus::Error, Some(msg), app);
            return;
        }

        if let Err(msg) = self.reconcile_port_before_spawn().await {
            tracing::error!("{}", msg);
            self.set_status(SidecarStatus::Error, Some(msg), app);
            return;
        }

        for attempt in 1..=self.max_retries {
            tracing::info!(
                "Starting Python Sidecar (attempt {}/{}) on port {}...",
                attempt,
                self.max_retries,
                self.port,
            );

            let spawn_result = self.spawn_child();

            match spawn_result {
                Ok(child) => {
                    let pid = child.id();
                    let kind = if should_use_packaged_sidecar()
                        && resolve_sidecar_executable(&self.agent_dir).is_some()
                    {
                        crate::sidecar_ownership::ManagedKind::PackagedBinary
                    } else {
                        crate::sidecar_ownership::ManagedKind::PythonUvicorn
                    };
                    {
                        let mut guard = self.child.lock().unwrap();
                        *guard = Some(child);
                    }
                    self.persist_runtime_record(pid, kind);

                    if self.wait_for_healthy(app).await {
                        tracing::info!(
                            pid,
                            port = self.port,
                            "Python Sidecar ready"
                        );
                        self.set_status(SidecarStatus::Ready, None, app);
                        self.spawn_watchdog_once(app.clone());
                        return;
                    }

                    self.stop_process();
                    let msg = format!(
                        "Health check timed out (attempt {}/{})",
                        attempt, self.max_retries
                    );
                    tracing::warn!("{}", msg);
                }
                Err(e) => {
                    let msg = format!(
                        "Spawn failed: {} (attempt {}/{})",
                        e, attempt, self.max_retries
                    );
                    tracing::error!("{}", msg);
                }
            }

            if attempt < self.max_retries {
                tokio::time::sleep(STARTUP_RETRY_DELAY).await;
            }
        }

        let msg = format!(
            "Failed to start sidecar after {} attempts on port {}",
            self.max_retries, self.port
        );
        tracing::error!("{}", msg);
        self.set_status(SidecarStatus::Error, Some(msg), app);
    }

    async fn wait_for_mcp_bridge_ready(&self) -> bool {
        for _ in 0..40 {
            if self.mcp_bridge_is_ready() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        self.mcp_bridge_is_ready()
    }

    async fn reconcile_port_before_spawn(&self) -> Result<(), String> {
        use crate::sidecar_ownership::{
            classify_port_occupancy, clear_runtime_record, listening_pids_on_port,
            process_command_line, process_is_alive, read_runtime_record, terminate_pid,
            unknown_occupant_message, PortOccupancy,
        };

        let healthy = self.is_healthy().await;
        let pids = listening_pids_on_port(self.port);
        let port_in_use = healthy || !pids.is_empty();
        let record = read_runtime_record(&self.runtime_record_path);
        let (pid_alive, cmdline) = match record.as_ref() {
            Some(rec) => (
                process_is_alive(rec.pid),
                process_command_line(rec.pid),
            ),
            None => (false, None),
        };

        match classify_port_occupancy(
            port_in_use,
            &pids,
            record.as_ref(),
            pid_alive,
            cmdline.as_deref(),
        ) {
            PortOccupancy::Free => {
                clear_runtime_record(&self.runtime_record_path);
                Ok(())
            }
            PortOccupancy::Managed(rec) => {
                tracing::warn!(
                    pid = rec.pid,
                    port = rec.port,
                    "Terminating previous managed Sidecar before spawn"
                );
                let _ = terminate_pid(rec.pid);
                clear_runtime_record(&self.runtime_record_path);
                for _ in 0..20 {
                    if !self.is_healthy().await
                        && listening_pids_on_port(self.port).is_empty()
                    {
                        return Ok(());
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(format!(
                    "Managed Sidecar PID {} did not release port {}",
                    rec.pid, self.port
                ))
            }
            PortOccupancy::Unknown { pid } => Err(unknown_occupant_message(self.port, pid)),
        }
    }

    fn persist_runtime_record(&self, pid: u32, kind: crate::sidecar_ownership::ManagedKind) {
        use crate::sidecar_ownership::{now_millis, write_runtime_record, SidecarRuntimeRecord};
        let record = SidecarRuntimeRecord {
            nonce: uuid::Uuid::new_v4().to_string(),
            pid,
            port: self.port,
            agent_dir: self.agent_dir.to_string_lossy().to_string(),
            kind,
            started_at_ms: now_millis(),
        };
        if let Err(e) = write_runtime_record(&self.runtime_record_path, &record) {
            tracing::warn!(error = %e, "Failed to persist Sidecar runtime record");
        } else {
            tracing::info!(
                pid = record.pid,
                nonce = %record.nonce,
                path = %self.runtime_record_path.display(),
                "Persisted Sidecar runtime ownership record"
            );
        }
    }

    async fn wait_for_healthy(&self, _app: &AppHandle) -> bool {
        let mut rx = self.shutdown_rx.clone();
        for _ in 0..STARTUP_HEALTH_CHECK_ATTEMPTS {
            tokio::select! {
                _ = tokio::time::sleep(STARTUP_HEALTH_CHECK_INTERVAL) => {}
                _ = rx.changed() => {
                    return false;
                }
            }
            if self.is_healthy().await {
                return true;
            }
        }
        false
    }

    async fn is_healthy(&self) -> bool {
        let response = match self
            .health_client
            .get(health_check_url(self.port))
            .send()
            .await
        {
            Ok(response) => response,
            Err(_) => return false,
        };

        if !response.status().is_success() {
            return false;
        }

        // Consume the small JSON response so reqwest can return this
        // keep-alive connection to its pool for the next liveness probe.
        response.bytes().await.is_ok()
    }

    /// Spawn the sidecar process.
    ///
    /// Prefers a packaged Nuitka binary (`misaka-agent`) when present, passing
    /// the port via `MISAKA_PORT` since `run.py` has no CLI flags. Falls back to
    /// `python run.py` in dev when only Python sources exist.
    fn spawn_child(&self) -> std::io::Result<Child> {
        if should_use_packaged_sidecar() {
            return match resolve_sidecar_executable(&self.agent_dir) {
                Some(binary) => self.spawn_packaged_binary(&binary),
                None => self.spawn_python_uvicorn(),
            };
        }
        self.spawn_python_uvicorn()
    }

    fn spawn_packaged_binary(&self, binary: &Path) -> std::io::Result<Child> {
        tracing::info!("Starting packaged Sidecar binary: {}", binary.display());
        let mut cmd = Command::new(binary);
        self.apply_common_env(&mut cmd);
        cmd.current_dir(&self.agent_dir).spawn()
    }

    fn spawn_python_uvicorn(&self) -> std::io::Result<Child> {
        let mut cmd = Command::new("python");
        // Keep the absolute script path in the process command line so the
        // ownership guard can distinguish our Sidecar from another `run.py`.
        cmd.arg(self.agent_dir.join("run.py"));
        self.apply_common_env(&mut cmd);
        cmd.current_dir(&self.agent_dir).spawn()
    }

    fn spawn_watchdog_once(self: &Arc<Self>, app: AppHandle) {
        {
            let mut active = self.watchdog_active.lock().unwrap();
            if *active {
                return;
            }
            *active = true;
        }

        let mut shutdown_rx = self.shutdown_rx.clone();
        let mgr = Arc::downgrade(self);
        let generation = self.current_lifecycle_generation();
        tauri::async_runtime::spawn(async move {
            let mut child_poll_count = 0_u64;
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(WATCHDOG_CHILD_POLL_INTERVAL) => {}
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                }

                let Some(manager) = mgr.upgrade() else {
                    break;
                };

                if *shutdown_rx.borrow() || manager.status() != SidecarStatus::Ready {
                    continue;
                }

                child_poll_count = child_poll_count.saturating_add(1);
                if let Some(reason) = manager.managed_child_exit_reason() {
                    manager.finish_watchdog();
                    manager
                        .handle_runtime_failure(&app, reason, generation)
                        .await;
                    return;
                }

                if is_runtime_health_check_due(child_poll_count) {
                    if let Some(reason) = manager.detect_runtime_health_failure().await {
                        manager.finish_watchdog();
                        manager
                            .handle_runtime_failure(&app, reason, generation)
                            .await;
                        return;
                    }
                }
            }

            if let Some(manager) = mgr.upgrade() {
                manager.finish_watchdog();
            }
        });
    }

    async fn detect_runtime_health_failure(&self) -> Option<String> {
        if !self.is_healthy().await {
            return Some(format!(
                "Health check failed for {}",
                health_check_url(self.port)
            ));
        }

        None
    }

    async fn handle_runtime_failure(
        self: &Arc<Self>,
        app: &AppHandle,
        reason: String,
        generation: u64,
    ) {
        let _guard = self.lifecycle_lock.lock().await;

        if *self.shutdown_rx.borrow()
            || self.status() != SidecarStatus::Ready
            || !is_current_watchdog_generation(generation, self.current_lifecycle_generation())
        {
            return;
        }

        let restart_count = self.increment_runtime_restart_count();
        if !should_attempt_runtime_restart(restart_count, self.max_runtime_restarts) {
            let msg = format!(
                "Sidecar runtime restart limit exceeded after {} attempts: {}",
                self.max_runtime_restarts, reason
            );
            tracing::error!("{}", msg);
            self.stop_process();
            self.set_status(SidecarStatus::Error, Some(msg), app);
            return;
        }

        self.advance_lifecycle_generation();
        tracing::warn!(
            attempt = restart_count,
            max = self.max_runtime_restarts,
            reason = %reason,
            "Sidecar watchdog detected runtime failure; restarting"
        );
        self.set_status(SidecarStatus::Restarting, Some(reason), app);
        self.stop_process();
        tokio::time::sleep(RESTART_DELAY).await;
        self.start_process_inner(app).await;
    }

    fn managed_child_exit_reason(&self) -> Option<String> {
        let mut guard = self.child.lock().unwrap();
        let reason = match guard.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(Some(status)) => Some(format!("Sidecar process exited with status {}", status)),
                Ok(None) => None,
                Err(e) => Some(format!("Failed to inspect sidecar process: {}", e)),
            },
            None => None,
        };

        if reason.is_some() {
            *guard = None;
        }

        reason
    }

    fn increment_runtime_restart_count(&self) -> u32 {
        let mut count = self.runtime_restart_count.lock().unwrap();
        *count += 1;
        *count
    }

    fn reset_runtime_restart_count(&self) {
        let mut count = self.runtime_restart_count.lock().unwrap();
        *count = 0;
    }

    fn finish_watchdog(&self) {
        let mut active = self.watchdog_active.lock().unwrap();
        *active = false;
    }

    fn current_lifecycle_generation(&self) -> u64 {
        *self.lifecycle_generation.lock().unwrap()
    }

    fn advance_lifecycle_generation(&self) {
        let mut generation = self.lifecycle_generation.lock().unwrap();
        *generation = generation.saturating_add(1);
    }

    fn stop_process(&self) {
        let mut guard = self.child.lock().unwrap();
        if let Some(ref mut child) = *guard {
            tracing::info!("Shutting down Python Sidecar (pid={})...", child.id());
            let _ = child.kill();
            let _ = child.wait();
        }
        *guard = None;
        crate::sidecar_ownership::clear_runtime_record(&self.runtime_record_path);
    }

    fn set_status(&self, new_status: SidecarStatus, message: Option<String>, app: &AppHandle) {
        self.set_status_only(new_status);
        let event = SidecarStatusEvent {
            status: new_status,
            message,
            port: self.port,
        };
        tracing::debug!("Sidecar status → {:?}", new_status);
        let _ = app.emit("sidecar:status", &event);
    }

    fn set_status_only(&self, new_status: SidecarStatus) {
        let mut guard = self.status.lock().unwrap();
        *guard = new_status;
    }
}

impl Drop for SidecarManager {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(true);
        self.stop_process();
    }
}
