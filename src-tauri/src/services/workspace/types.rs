use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;

use crate::contracts::AppErrorCode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalWorkspace {
    path: PathBuf,
}

impl CanonicalWorkspace {
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = std::fs::canonicalize(path)?;
        if !path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                "workspace is not a directory",
            ));
        }
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn display_path(&self) -> String {
        strip_windows_verbatim_prefix(&self.path)
            .to_string_lossy()
            .into_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcsContext {
    pub repository_root: PathBuf,
    pub branch: Option<String>,
    pub detached_head: Option<String>,
    pub git_dir: PathBuf,
    pub git_common_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcsDiagnostic {
    pub code: AppErrorCode,
    pub message_key: &'static str,
    pub retryable: bool,
    pub internal_reason: &'static str,
}

impl VcsDiagnostic {
    pub fn git_unavailable(internal_reason: &'static str) -> Self {
        Self {
            code: AppErrorCode::GitNotAvailable,
            message_key: "workspace.gitUnavailable",
            retryable: true,
            internal_reason,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DetectionCancellation {
    cancelled: Arc<AtomicBool>,
}

impl DetectionCancellation {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[async_trait]
pub trait VcsProvider: Send + Sync {
    async fn detect(
        &self,
        workspace: &CanonicalWorkspace,
        cancellation: DetectionCancellation,
    ) -> Result<Option<VcsContext>, VcsDiagnostic>;
}

pub fn strip_windows_verbatim_prefix(path: &Path) -> PathBuf {
    let value = path.to_string_lossy();
    value
        .strip_prefix(r"\\?\")
        .map(PathBuf::from)
        .unwrap_or_else(|| path.to_path_buf())
}
