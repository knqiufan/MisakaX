use std::process::{Child, Command};
use std::time::Duration;

/// 构建健康检查 URL
pub fn health_check_url(port: u16) -> String {
    format!("http://127.0.0.1:{}/health", port)
}

/// Manages the Python Agent Sidecar process lifecycle.
pub struct SidecarManager {
    child: Option<Child>,
}

impl SidecarManager {
    /// Spawn the Python sidecar and wait for its health check.
    pub fn start(agent_dir: &str, port: u16) -> Result<Self, String> {
        if Self::is_healthy(port) {
            tracing::info!("Python Sidecar already running on port {}", port);
            return Ok(Self { child: None });
        }

        tracing::info!("Starting Python Sidecar on port {}...", port);

        let mut child = Command::new("python")
            .args([
                "-m",
                "uvicorn",
                "app.main:app",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
            ])
            .current_dir(agent_dir)
            .spawn()
            .map_err(|e| {
                format!(
                    "Failed to start Python Sidecar: {}. Is Python + uvicorn installed?",
                    e
                )
            })?;

        for _ in 0..20 {
            std::thread::sleep(Duration::from_millis(500));
            if Self::is_healthy(port) {
                tracing::info!("Python Sidecar ready on port {}", port);
                return Ok(Self { child: Some(child) });
            }
        }

        let _ = child.kill();
        let _ = child.wait();
        Err(format!(
            "Python Sidecar health check timed out after 10s on port {}",
            port
        ))
    }

    fn is_healthy(port: u16) -> bool {
        reqwest::blocking::Client::new()
            .get(health_check_url(port))
            .timeout(Duration::from_secs(2))
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

impl Drop for SidecarManager {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child {
            tracing::info!("Shutting down Python Sidecar...");
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
