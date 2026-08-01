use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;

use super::types::{
    strip_windows_verbatim_prefix, CanonicalWorkspace, DetectionCancellation, VcsContext,
    VcsDiagnostic, VcsProvider,
};

const GIT_TIMEOUT: Duration = Duration::from_secs(2);
const GIT_OUTPUT_LIMIT: usize = 8 * 1024;

#[derive(Debug, Clone)]
pub struct GitCliProvider {
    explicit_executable: Option<PathBuf>,
    timeout: Duration,
}

impl Default for GitCliProvider {
    fn default() -> Self {
        Self {
            explicit_executable: None,
            timeout: GIT_TIMEOUT,
        }
    }
}

impl GitCliProvider {
    pub fn with_executable(executable: impl Into<PathBuf>) -> Self {
        Self {
            explicit_executable: Some(executable.into()),
            timeout: GIT_TIMEOUT,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn resolve_executable(&self, workspace: &CanonicalWorkspace) -> Result<PathBuf, GitRunError> {
        if let Some(executable) = &self.explicit_executable {
            return canonical_executable(executable).ok_or(GitRunError::ExecutableMissing);
        }

        resolve_trusted_git(workspace.path()).ok_or(GitRunError::ExecutableMissing)
    }

    async fn run(
        &self,
        workspace: &CanonicalWorkspace,
        cancellation: &DetectionCancellation,
        args: &[&str],
    ) -> Result<String, GitRunError> {
        if cancellation.is_cancelled() {
            return Err(GitRunError::Cancelled);
        }

        let executable = self.resolve_executable(workspace)?;
        let mut command = Command::new(executable);
        for (key, _) in std::env::vars_os() {
            if key
                .to_str()
                .is_some_and(|key| key.to_ascii_uppercase().starts_with("GIT_"))
            {
                command.env_remove(key);
            }
        }
        command
            .arg("-C")
            .arg(workspace.path())
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C");

        let mut child = command.spawn().map_err(|_| GitRunError::SpawnFailed)?;
        let stdout = child.stdout.take().ok_or(GitRunError::PipeUnavailable)?;
        let stderr = child.stderr.take().ok_or(GitRunError::PipeUnavailable)?;
        let stdout_task = tokio::spawn(read_bounded(stdout));
        let stderr_task = tokio::spawn(read_bounded(stderr));

        let status = tokio::select! {
            status = child.wait() => status.map_err(|_| GitRunError::WaitFailed)?,
            _ = wait_for_cancellation(cancellation.clone()) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                return Err(GitRunError::Cancelled);
            }
            _ = tokio::time::sleep(self.timeout) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                return Err(GitRunError::TimedOut);
            }
        };

        let stdout = stdout_task.await.map_err(|_| GitRunError::ReadFailed)??;
        let _stderr = stderr_task.await.map_err(|_| GitRunError::ReadFailed)??;

        if !status.success() {
            return Err(GitRunError::CommandFailed);
        }

        let output = String::from_utf8(stdout).map_err(|_| GitRunError::InvalidOutput)?;
        Ok(output.trim().to_string())
    }
}

#[async_trait]
impl VcsProvider for GitCliProvider {
    async fn detect(
        &self,
        workspace: &CanonicalWorkspace,
        cancellation: DetectionCancellation,
    ) -> Result<Option<VcsContext>, VcsDiagnostic> {
        let inside = match self
            .run(
                workspace,
                &cancellation,
                &["rev-parse", "--is-inside-work-tree"],
            )
            .await
        {
            Ok(inside) => inside,
            Err(GitRunError::CommandFailed) => return Ok(None),
            Err(error) => return Err(map_diagnostic(error)),
        };
        if inside != "true" {
            return Ok(None);
        }

        let repository_root = self
            .run(workspace, &cancellation, &["rev-parse", "--show-toplevel"])
            .await
            .map_err(map_diagnostic)?;
        let repository_root = canonical_output_path(workspace.path(), &repository_root)
            .map_err(|_| map_diagnostic(GitRunError::InvalidOutput))?;

        let branch = match self
            .run(
                workspace,
                &cancellation,
                &["symbolic-ref", "-q", "--short", "HEAD"],
            )
            .await
        {
            Ok(branch) if !branch.is_empty() => Some(validate_ref_label(branch)?),
            Ok(_) | Err(GitRunError::CommandFailed) => None,
            Err(error) => return Err(map_diagnostic(error)),
        };
        let detached_head = if branch.is_none() {
            Some(validate_short_sha(
                self.run(workspace, &cancellation, &["rev-parse", "--short", "HEAD"])
                    .await
                    .map_err(map_diagnostic)?,
            )?)
        } else {
            None
        };

        let dirs = match self
            .run(
                workspace,
                &cancellation,
                &[
                    "rev-parse",
                    "--path-format=absolute",
                    "--git-dir",
                    "--git-common-dir",
                ],
            )
            .await
        {
            Ok(dirs) => dirs,
            Err(GitRunError::CommandFailed) => self
                .run(
                    workspace,
                    &cancellation,
                    &["rev-parse", "--git-dir", "--git-common-dir"],
                )
                .await
                .map_err(map_diagnostic)?,
            Err(error) => return Err(map_diagnostic(error)),
        };
        let mut dirs = dirs.lines().filter(|line| !line.trim().is_empty());
        let git_dir = dirs
            .next()
            .ok_or_else(|| map_diagnostic(GitRunError::InvalidOutput))?;
        let git_common_dir = dirs
            .next()
            .ok_or_else(|| map_diagnostic(GitRunError::InvalidOutput))?;

        Ok(Some(VcsContext {
            repository_root,
            branch,
            detached_head,
            git_dir: canonical_output_path(workspace.path(), git_dir)
                .map_err(|_| map_diagnostic(GitRunError::InvalidOutput))?,
            git_common_dir: canonical_output_path(workspace.path(), git_common_dir)
                .map_err(|_| map_diagnostic(GitRunError::InvalidOutput))?,
        }))
    }
}

async fn read_bounded(reader: impl AsyncRead + Unpin) -> Result<Vec<u8>, GitRunError> {
    let mut bytes = Vec::new();
    reader
        .take((GIT_OUTPUT_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| GitRunError::ReadFailed)?;
    if bytes.len() > GIT_OUTPUT_LIMIT {
        return Err(GitRunError::OutputLimitExceeded);
    }
    Ok(bytes)
}

async fn wait_for_cancellation(cancellation: DetectionCancellation) {
    while !cancellation.is_cancelled() {
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

fn canonical_output_path(base: &Path, output: &str) -> std::io::Result<PathBuf> {
    let path = Path::new(output.trim());
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    std::fs::canonicalize(joined).map(|path| strip_windows_verbatim_prefix(&path))
}

fn validate_ref_label(value: String) -> Result<String, VcsDiagnostic> {
    if value.len() > 512 || value.chars().any(char::is_control) {
        return Err(map_diagnostic(GitRunError::InvalidOutput));
    }
    Ok(value)
}

fn validate_short_sha(value: String) -> Result<String, VcsDiagnostic> {
    if !(4..=64).contains(&value.len()) || !value.chars().all(|char| char.is_ascii_hexdigit()) {
        return Err(map_diagnostic(GitRunError::InvalidOutput));
    }
    Ok(value)
}

fn resolve_trusted_git(workspace: &Path) -> Option<PathBuf> {
    let workspace = strip_windows_verbatim_prefix(&std::fs::canonicalize(workspace).ok()?);
    let temp = std::fs::canonicalize(std::env::temp_dir())
        .ok()
        .map(|path| strip_windows_verbatim_prefix(&path));
    let trusted_roots = trusted_git_roots();
    let mut candidates = trusted_git_candidates();

    if let Some(path) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&path) {
            if directory.is_absolute() {
                candidates.push(directory.join(git_executable_name()));
            }
        }
    }

    let mut seen = HashSet::new();
    candidates.into_iter().find_map(|candidate| {
        let executable = canonical_executable(&candidate)?;
        if !seen.insert(executable.clone())
            || executable.starts_with(&workspace)
            || temp
                .as_ref()
                .is_some_and(|temp| executable.starts_with(temp))
            || !is_trusted_installation(&executable, &trusted_roots)
        {
            return None;
        }
        Some(executable)
    })
}

fn canonical_executable(candidate: &Path) -> Option<PathBuf> {
    let executable = std::fs::canonicalize(candidate).ok()?;
    executable
        .is_file()
        .then(|| strip_windows_verbatim_prefix(&executable))
}

fn git_executable_name() -> &'static str {
    if cfg!(windows) {
        "git.exe"
    } else {
        "git"
    }
}

#[cfg(windows)]
fn trusted_git_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for key in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        if let Some(root) = std::env::var_os(key) {
            let root = PathBuf::from(root);
            let path = if key == "LOCALAPPDATA" {
                root.join("Programs/Git/cmd/git.exe")
            } else {
                root.join("Git/cmd/git.exe")
            };
            candidates.push(path);
        }
    }
    candidates
}

#[cfg(not(windows))]
fn trusted_git_candidates() -> Vec<PathBuf> {
    [
        "/usr/bin/git",
        "/usr/local/bin/git",
        "/opt/homebrew/bin/git",
        "/opt/local/bin/git",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

#[cfg(windows)]
fn trusted_git_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for key in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        if let Some(root) = std::env::var_os(key) {
            let root = PathBuf::from(root);
            let git_root = if key == "LOCALAPPDATA" {
                root.join("Programs/Git")
            } else {
                root.join("Git")
            };
            if let Ok(canonical) = std::fs::canonicalize(git_root) {
                roots.push(strip_windows_verbatim_prefix(&canonical));
            }
        }
    }
    roots
}

#[cfg(windows)]
fn is_trusted_installation(executable: &Path, trusted_roots: &[PathBuf]) -> bool {
    if trusted_roots
        .iter()
        .any(|root| executable.starts_with(root))
    {
        return true;
    }
    let Some(command_dir) = executable.parent() else {
        return false;
    };
    if !matches!(
        command_dir.file_name().and_then(|name| name.to_str()),
        Some(name) if name.eq_ignore_ascii_case("cmd") || name.eq_ignore_ascii_case("bin")
    ) {
        return false;
    }
    let Some(install_root) = command_dir.parent() else {
        return false;
    };
    install_root.join("mingw64/bin/git.exe").is_file()
        && install_root.join("mingw64/libexec/git-core").is_dir()
        && install_root.join("ReleaseNotes.html").is_file()
}

#[cfg(not(windows))]
fn is_trusted_installation(executable: &Path, trusted_roots: &[PathBuf]) -> bool {
    use std::os::unix::fs::PermissionsExt;

    if trusted_roots
        .iter()
        .any(|root| executable.starts_with(root))
    {
        return true;
    }
    let Some(parent) = executable.parent() else {
        return false;
    };
    let Ok(directory) = std::fs::metadata(parent) else {
        return false;
    };
    let Ok(file) = std::fs::metadata(executable) else {
        return false;
    };
    directory.permissions().mode() & 0o022 == 0 && file.permissions().mode() & 0o111 != 0
}

#[cfg(not(windows))]
fn trusted_git_roots() -> Vec<PathBuf> {
    ["/usr", "/opt/homebrew", "/opt/local"]
        .into_iter()
        .filter_map(|root| std::fs::canonicalize(root).ok())
        .collect()
}

fn map_diagnostic(error: GitRunError) -> VcsDiagnostic {
    let reason = match error {
        GitRunError::ExecutableMissing => "trusted_git_not_found",
        GitRunError::TimedOut => "git_query_timeout",
        GitRunError::Cancelled => "git_query_cancelled",
        GitRunError::OutputLimitExceeded => "git_output_limit",
        GitRunError::InvalidOutput => "git_invalid_output",
        GitRunError::CommandFailed => "git_command_failed",
        GitRunError::SpawnFailed => "git_spawn_failed",
        GitRunError::PipeUnavailable => "git_pipe_unavailable",
        GitRunError::ReadFailed => "git_read_failed",
        GitRunError::WaitFailed => "git_wait_failed",
    };
    VcsDiagnostic::git_unavailable(reason)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GitRunError {
    ExecutableMissing,
    SpawnFailed,
    PipeUnavailable,
    TimedOut,
    Cancelled,
    WaitFailed,
    ReadFailed,
    OutputLimitExceeded,
    CommandFailed,
    InvalidOutput,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bounded_reader_rejects_oversized_output() {
        let bytes = vec![b'x'; GIT_OUTPUT_LIMIT + 1];
        let error = read_bounded(bytes.as_slice()).await.unwrap_err();
        assert_eq!(error, GitRunError::OutputLimitExceeded);
    }

    #[test]
    fn workspace_and_temp_candidates_are_never_trusted() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = CanonicalWorkspace::new(temp.path()).unwrap();
        let candidate = temp.path().join(git_executable_name());
        std::fs::write(&candidate, b"not git").unwrap();
        let provider = GitCliProvider::default();

        assert!(matches!(
            provider.resolve_executable(&workspace),
            Ok(_) | Err(GitRunError::ExecutableMissing)
        ));
        assert_ne!(
            provider.resolve_executable(&workspace).ok(),
            canonical_executable(&candidate)
        );
    }
}
