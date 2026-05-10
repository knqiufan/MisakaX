use serde::{Deserialize, Serialize};
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::watch;

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

// --------------------------------------------------------------------------- //
//  SidecarManager
// --------------------------------------------------------------------------- //

pub struct SidecarManager {
    port: u16,
    agent_dir: String,
    max_retries: u32,
    status: Arc<Mutex<SidecarStatus>>,
    child: Arc<Mutex<Option<Child>>>,
    shutdown_tx: watch::Sender<bool>,
    shutdown_rx: watch::Receiver<bool>,
}

impl SidecarManager {
    pub fn new(agent_dir: String, port: u16) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        Self {
            port,
            agent_dir,
            max_retries: 3,
            status: Arc::new(Mutex::new(SidecarStatus::Stopped)),
            child: Arc::new(Mutex::new(None)),
            shutdown_tx,
            shutdown_rx,
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
        self.set_status(SidecarStatus::Restarting, None, &app);
        self.stop_process();
        tokio::time::sleep(Duration::from_millis(500)).await;
        self.start_process(&app).await;
    }

    /// Graceful shutdown: send kill, wait for exit.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
        self.stop_process();
    }

    // ---------------------------------------------------------------------- //
    //  Internal helpers
    // ---------------------------------------------------------------------- //

    async fn start_process(self: &Arc<Self>, app: &AppHandle) {
        self.set_status(SidecarStatus::Starting, None, app);

        if Self::is_healthy(self.port).await {
            tracing::info!("Python Sidecar already running on port {}", self.port);
            self.set_status(SidecarStatus::Ready, None, app);
            return;
        }

        for attempt in 1..=self.max_retries {
            tracing::info!(
                "Starting Python Sidecar (attempt {}/{}) on port {}...",
                attempt,
                self.max_retries,
                self.port,
            );

            let spawn_result = std::process::Command::new("python")
                .args([
                    "-m",
                    "uvicorn",
                    "app.main:app",
                    "--host",
                    "127.0.0.1",
                    "--port",
                    &self.port.to_string(),
                ])
                .current_dir(&self.agent_dir)
                .spawn();

            match spawn_result {
                Ok(child) => {
                    {
                        let mut guard = self.child.lock().unwrap();
                        *guard = Some(child);
                    }

                    if self.wait_for_healthy(app).await {
                        tracing::info!("Python Sidecar ready on port {}", self.port);
                        self.set_status(SidecarStatus::Ready, None, app);
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
                    let msg = format!("Spawn failed: {} (attempt {}/{})", e, attempt, self.max_retries);
                    tracing::error!("{}", msg);
                }
            }

            if attempt < self.max_retries {
                tokio::time::sleep(Duration::from_secs(2)).await;
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
        for _ in 0..20 {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(500)) => {}
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
        let url = format!("http://127.0.0.1:{}/health", port);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
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
        {
            let mut guard = self.status.lock().unwrap();
            *guard = new_status;
        }
        let event = SidecarStatusEvent {
            status: new_status,
            message,
            port: self.port,
        };
        tracing::debug!("Sidecar status → {:?}", new_status);
        let _ = app.emit("sidecar:status", &event);
    }
}

impl Drop for SidecarManager {
    fn drop(&mut self) {
        self.stop_process();
    }
}
