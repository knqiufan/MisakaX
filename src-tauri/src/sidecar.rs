use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::{watch, Mutex as AsyncMutex};

const STARTUP_MAX_RETRIES: u32 = 3;
const RUNTIME_MAX_RESTARTS: u32 = 3;
const STARTUP_RETRY_DELAY: Duration = Duration::from_secs(2);
const RESTART_DELAY: Duration = Duration::from_millis(500);
const WATCHDOG_INTERVAL: Duration = Duration::from_secs(5);
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(2);
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

// --------------------------------------------------------------------------- //
//  Agent directory resolution
// --------------------------------------------------------------------------- //

/// Resolve the working directory for the Python sidecar (`uvicorn app.main:app`).
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
    agent_dir: PathBuf,
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
}

impl SidecarManager {
    pub fn new(agent_dir: PathBuf, port: u16) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        Self {
            port,
            agent_dir,
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

        if Self::is_healthy(self.port).await {
            tracing::info!("Python Sidecar already running on port {}", self.port);
            self.set_status(SidecarStatus::Ready, None, app);
            self.spawn_watchdog_once(app.clone());
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
                    {
                        let mut guard = self.child.lock().unwrap();
                        *guard = Some(child);
                    }

                    if self.wait_for_healthy(app).await {
                        tracing::info!("Python Sidecar ready on port {}", self.port);
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

    async fn wait_for_healthy(&self, _app: &AppHandle) -> bool {
        let mut rx = self.shutdown_rx.clone();
        for _ in 0..STARTUP_HEALTH_CHECK_ATTEMPTS {
            tokio::select! {
                _ = tokio::time::sleep(STARTUP_HEALTH_CHECK_INTERVAL) => {}
                _ = rx.changed() => {
                    return false;
                }
            }
            if Self::is_healthy(self.port).await {
                return true;
            }
        }
        false
    }

    async fn is_healthy(port: u16) -> bool {
        let url = health_check_url(port);
        let client = reqwest::Client::builder()
            .timeout(HEALTH_CHECK_TIMEOUT)
            .build();
        match client {
            Ok(c) => c
                .get(&url)
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    /// Spawn the sidecar process.
    ///
    /// Prefers a packaged Nuitka binary (`misaka-agent`) when present, passing
    /// the port via `MISAKA_PORT` since `run.py` has no CLI flags. Falls back to
    /// `python -m uvicorn` in dev when only Python sources exist.
    fn spawn_child(&self) -> std::io::Result<Child> {
        match resolve_sidecar_executable(&self.agent_dir) {
            Some(binary) => self.spawn_packaged_binary(&binary),
            None => self.spawn_python_uvicorn(),
        }
    }

    fn spawn_packaged_binary(&self, binary: &Path) -> std::io::Result<Child> {
        tracing::info!("Starting packaged Sidecar binary: {}", binary.display());
        std::process::Command::new(binary)
            .env("MISAKA_HOST", "127.0.0.1")
            .env("MISAKA_PORT", self.port.to_string())
            .current_dir(&self.agent_dir)
            .spawn()
    }

    fn spawn_python_uvicorn(&self) -> std::io::Result<Child> {
        let port = self.port.to_string();
        std::process::Command::new("python")
            .args([
                "-m",
                "uvicorn",
                "app.main:app",
                "--host",
                "127.0.0.1",
                "--port",
                port.as_str(),
            ])
            .current_dir(&self.agent_dir)
            .spawn()
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
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(WATCHDOG_INTERVAL) => {}
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

                if let Some(reason) = manager.detect_runtime_failure().await {
                    manager.finish_watchdog();
                    manager
                        .handle_runtime_failure(&app, reason, generation)
                        .await;
                    return;
                }
            }

            if let Some(manager) = mgr.upgrade() {
                manager.finish_watchdog();
            }
        });
    }

    async fn detect_runtime_failure(&self) -> Option<String> {
        if let Some(reason) = self.managed_child_exit_reason() {
            return Some(reason);
        }

        if !Self::is_healthy(self.port).await {
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
