use std::ffi::OsString;
use std::path::{Path, PathBuf};

use portable_pty::CommandBuilder;

#[cfg(windows)]
use crate::services::workspace::strip_windows_verbatim_prefix;

use super::types::{ShellFallbackReason, TerminalServiceError};

#[derive(Debug, Clone)]
pub struct ResolvedShell {
    pub executable: PathBuf,
    pub profile: String,
    pub shell_name: String,
    pub args: Vec<OsString>,
    pub fallback_reason: Option<ShellFallbackReason>,
}

impl ResolvedShell {
    pub fn command(&self, cwd: &Path) -> Result<CommandBuilder, TerminalServiceError> {
        let mut command = CommandBuilder::new(&self.executable);
        command.args(&self.args);
        #[cfg(windows)]
        configure_windows_working_directory(&mut command, &self.shell_name, cwd)?;
        #[cfg(not(windows))]
        command.cwd(cwd);
        for key in protected_environment_keys() {
            command.env_remove(key);
        }
        Ok(command)
    }
}

#[cfg(windows)]
fn configure_windows_working_directory(
    command: &mut CommandBuilder,
    shell_name: &str,
    canonical_cwd: &Path,
) -> Result<(), TerminalServiceError> {
    use std::os::windows::ffi::OsStrExt;

    let display_cwd = strip_windows_verbatim_prefix(canonical_cwd);
    if display_cwd.as_os_str().encode_wide().count() < 248 {
        command.cwd(display_cwd);
        return Ok(());
    }

    if matches!(shell_name, "pwsh" | "powershell") {
        // CreateProcessW still caps lpCurrentDirectory at MAX_PATH. Start the
        // profile-free shell from the drive/share root, then synchronously move
        // into the already-canonicalized workspace before showing a prompt.
        let launch_cwd = display_cwd
            .ancestors()
            .last()
            .unwrap_or(display_cwd.as_path());
        command.cwd(launch_cwd);
        let quoted = canonical_cwd.to_string_lossy().replace('\'', "''");
        command.arg("-NoExit");
        command.arg("-Command");
        command.arg(format!("Set-Location -LiteralPath '{quoted}'"));
        Ok(())
    } else {
        // cmd.exe cannot use an extended-length path as its current directory.
        // Fail closed with a stable diagnostic rather than silently starting
        // outside the requested workspace.
        Err(TerminalServiceError::SpawnFailed("shell_workspace_path"))
    }
}

pub fn resolve_shell(requested: Option<&str>) -> Result<ResolvedShell, TerminalServiceError> {
    let requested = requested
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("auto");
    if requested.len() > 32 {
        return Err(TerminalServiceError::InvalidRequest("shell_profile"));
    }
    let requested = requested.to_ascii_lowercase();

    #[cfg(windows)]
    {
        resolve_windows_shell(&requested)
    }
    #[cfg(unix)]
    {
        resolve_unix_shell(&requested)
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = requested;
        Err(TerminalServiceError::SpawnFailed("platform"))
    }
}

fn protected_environment_keys() -> Vec<OsString> {
    let mut keys = std::env::vars_os()
        .filter_map(|(key, _)| {
            let normalized = key.to_string_lossy().to_ascii_uppercase();
            (normalized.starts_with("MISAKAX_")
                || normalized.starts_with("CODEX_SANDBOX_")
                || normalized.starts_with("SANDBOX_")
                || matches!(
                    normalized.as_str(),
                    "TAURI_CHANNEL"
                        | "TAURI_PRIVATE_KEY"
                        | "TAURI_PRIVATE_KEY_PASSWORD"
                        | "MCP_BRIDGE_TOKEN"
                        | "SIDECAR_BRIDGE_TOKEN"
                ))
            .then_some(key)
        })
        .collect::<Vec<_>>();
    for known in [
        "MISAKAX_MCP_BRIDGE_TOKEN",
        "MISAKAX_SIDECAR_TOKEN",
        "MCP_BRIDGE_TOKEN",
        "SIDECAR_BRIDGE_TOKEN",
        "TAURI_CHANNEL",
        "TAURI_PRIVATE_KEY",
        "TAURI_PRIVATE_KEY_PASSWORD",
    ] {
        if !keys.iter().any(|key| key == known) {
            keys.push(known.into());
        }
    }
    keys
}

#[cfg(windows)]
fn resolve_windows_shell(requested: &str) -> Result<ResolvedShell, TerminalServiceError> {
    if !matches!(requested, "auto" | "pwsh" | "powershell" | "cmd") {
        return Err(TerminalServiceError::InvalidRequest("shell_profile"));
    }

    let mut candidates = Vec::<(&str, PathBuf, Vec<OsString>)>::new();
    let program_files = std::env::var_os("ProgramFiles").map(PathBuf::from);
    let system_root = std::env::var_os("SystemRoot").map(PathBuf::from);

    if matches!(requested, "auto" | "pwsh") {
        if let Some(root) = program_files.as_ref() {
            candidates.push((
                "pwsh",
                root.join("PowerShell").join("7").join("pwsh.exe"),
                vec!["-NoLogo".into(), "-NoProfile".into()],
            ));
        }
    }
    if matches!(requested, "auto" | "pwsh" | "powershell") {
        if let Some(root) = system_root.as_ref() {
            candidates.push((
                "powershell",
                root.join("System32")
                    .join("WindowsPowerShell")
                    .join("v1.0")
                    .join("powershell.exe"),
                vec!["-NoLogo".into(), "-NoProfile".into()],
            ));
        }
    }
    if let Some(root) = system_root.as_ref() {
        candidates.push((
            "cmd",
            root.join("System32").join("cmd.exe"),
            vec!["/D".into(), "/Q".into()],
        ));
    }

    let (name, executable, args) = candidates
        .into_iter()
        .find(|(_, path, _)| is_regular_file(path))
        .ok_or(TerminalServiceError::SpawnFailed("shell_unavailable"))?;
    let fallback_reason = (requested != "auto" && requested != name)
        .then_some(ShellFallbackReason::RequestedUnavailable);
    Ok(ResolvedShell {
        executable,
        profile: requested.to_string(),
        shell_name: name.to_string(),
        args,
        fallback_reason,
    })
}

#[cfg(unix)]
fn resolve_unix_shell(requested: &str) -> Result<ResolvedShell, TerminalServiceError> {
    if !matches!(requested, "auto" | "zsh" | "bash" | "sh" | "fish") {
        return Err(TerminalServiceError::InvalidRequest("shell_profile"));
    }

    let user_shell = (requested == "auto")
        .then(|| std::env::var_os("SHELL"))
        .flatten()
        .map(PathBuf::from)
        .filter(|path| trusted_unix_shell(path));
    if let Some(executable) = user_shell {
        let name = executable
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("sh")
            .to_string();
        return Ok(ResolvedShell {
            executable,
            profile: requested.to_string(),
            shell_name: name,
            args: vec!["-l".into()],
            fallback_reason: None,
        });
    }

    let names: &[&str] = match requested {
        "zsh" => &["zsh", "bash", "sh"],
        "bash" => &["bash", "sh"],
        "fish" => &["fish", "zsh", "bash", "sh"],
        "sh" => &["sh"],
        _ => &["zsh", "bash", "sh"],
    };
    let prefixes = ["/bin", "/usr/bin", "/usr/local/bin", "/opt/homebrew/bin"];
    for name in names {
        for prefix in prefixes {
            let path = Path::new(prefix).join(name);
            if trusted_unix_shell(&path) {
                let fallback_reason = if requested == "auto" {
                    std::env::var_os("SHELL")
                        .is_some()
                        .then_some(ShellFallbackReason::UserShellInvalid)
                } else if requested != *name {
                    Some(ShellFallbackReason::RequestedUnavailable)
                } else {
                    None
                };
                return Ok(ResolvedShell {
                    executable: path,
                    profile: requested.to_string(),
                    shell_name: (*name).to_string(),
                    args: vec!["-l".into()],
                    fallback_reason,
                });
            }
        }
    }
    Err(TerminalServiceError::SpawnFailed("shell_unavailable"))
}

#[cfg(unix)]
fn trusted_unix_shell(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    if !path.is_absolute() {
        return false;
    }
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if !matches!(name, "zsh" | "bash" | "sh" | "fish") {
        return false;
    }
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_regular_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_arbitrary_executable_profile() {
        assert!(matches!(
            resolve_shell(Some("../../malware")),
            Err(TerminalServiceError::InvalidRequest("shell_profile"))
        ));
    }

    #[test]
    fn protected_environment_removes_internal_tokens_but_not_normal_dev_keys() {
        let keys = protected_environment_keys()
            .into_iter()
            .map(|key| key.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert!(keys.iter().any(|key| key == "MISAKAX_MCP_BRIDGE_TOKEN"));
        assert!(!keys.iter().any(|key| key == "PATH"));
    }
}
