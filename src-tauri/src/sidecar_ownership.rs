//! Managed Sidecar process ownership records and port conflict helpers.
//!
//! Only processes recorded and verified as MisakaX-managed may be terminated.
//! Unknown listeners on the Sidecar port are never killed.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const RUNTIME_RECORD_FILE: &str = "sidecar-runtime.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedKind {
    PythonUvicorn,
    PackagedBinary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SidecarRuntimeRecord {
    pub nonce: String,
    pub pid: u32,
    pub port: u16,
    pub agent_dir: String,
    pub kind: ManagedKind,
    pub started_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortOccupancy {
    Free,
    Managed(SidecarRuntimeRecord),
    Unknown { pid: Option<u32> },
}

pub fn runtime_record_path(data_dir: &Path) -> PathBuf {
    data_dir.join(RUNTIME_RECORD_FILE)
}

pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn write_runtime_record(path: &Path, record: &SidecarRuntimeRecord) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create runtime dir: {e}"))?;
    }
    let json = serde_json::to_string_pretty(record)
        .map_err(|e| format!("Failed to serialize runtime record: {e}"))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json).map_err(|e| format!("Failed to write runtime record: {e}"))?;
    fs::rename(&tmp, path).map_err(|e| format!("Failed to commit runtime record: {e}"))?;
    Ok(())
}

pub fn read_runtime_record(path: &Path) -> Option<SidecarRuntimeRecord> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn clear_runtime_record(path: &Path) {
    let _ = fs::remove_file(path);
}

/// Decide whether a stored record still identifies a live managed process.
pub fn record_matches_process(
    record: &SidecarRuntimeRecord,
    pid_alive: bool,
    command_line: Option<&str>,
    listening_pids: &[u32],
) -> bool {
    if !pid_alive || !listening_pids.contains(&record.pid) {
        return false;
    }
    let Some(cmd) = command_line else {
        return false;
    };
    let cmd_lower = cmd.replace('\\', "/").to_ascii_lowercase();
    let agent_dir = record.agent_dir.replace('\\', "/").to_ascii_lowercase();
    let port_token = record.port.to_string();
    match record.kind {
        ManagedKind::PythonUvicorn => {
            let legacy_uvicorn = cmd_lower.contains("uvicorn")
                && cmd_lower.contains("app.main:app")
                && cmd_lower.contains(&port_token);
            let run_py = cmd_lower.contains("python")
                && cmd_lower.contains("run.py")
                && cmd_lower.contains(&agent_dir);
            legacy_uvicorn || run_py
        }
        // Packaged binary receives port via env; command line may omit it.
        ManagedKind::PackagedBinary => cmd_lower.contains("misaka-agent"),
    }
}

pub fn classify_port_occupancy(
    port_in_use: bool,
    listening_pids: &[u32],
    record: Option<&SidecarRuntimeRecord>,
    pid_alive: bool,
    command_line: Option<&str>,
) -> PortOccupancy {
    if !port_in_use {
        return PortOccupancy::Free;
    }
    if let Some(rec) = record {
        if record_matches_process(rec, pid_alive, command_line, listening_pids) {
            return PortOccupancy::Managed(rec.clone());
        }
    }
    PortOccupancy::Unknown {
        pid: listening_pids.first().copied(),
    }
}

pub fn unknown_occupant_message(port: u16, pid: Option<u32>) -> String {
    match pid {
        Some(pid) => format!(
            "Port {port} is already in use by unmanaged process PID {pid}. \
             Stop that process, then retry. MisakaX will not kill unknown listeners."
        ),
        None => format!(
            "Port {port} is already in use by an unmanaged process. \
             Stop the listener, then retry. MisakaX will not kill unknown listeners."
        ),
    }
}

/// Return PIDs that currently listen on `127.0.0.1:port` (best-effort).
pub fn listening_pids_on_port(port: u16) -> Vec<u32> {
    #[cfg(windows)]
    {
        listening_pids_on_port_windows(port)
    }
    #[cfg(not(windows))]
    {
        listening_pids_on_port_unix(port)
    }
}

pub fn process_is_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        process_is_alive_windows(pid)
    }
    #[cfg(not(windows))]
    {
        process_is_alive_unix(pid)
    }
}

pub fn process_command_line(pid: u32) -> Option<String> {
    #[cfg(windows)]
    {
        process_command_line_windows(pid)
    }
    #[cfg(not(windows))]
    {
        process_command_line_unix(pid)
    }
}

/// Terminate a process by PID. Only call after ownership verification.
pub fn terminate_pid(pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        terminate_pid_windows(pid)
    }
    #[cfg(not(windows))]
    {
        terminate_pid_unix(pid)
    }
}

#[cfg(windows)]
fn listening_pids_on_port_windows(port: u16) -> Vec<u32> {
    use std::process::Command;
    let output = Command::new("netstat")
        .args(["-ano", "-p", "TCP"])
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let needle = format!(":{port}");
    let mut pids = Vec::new();
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if !lower.contains("listening") {
            continue;
        }
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 5 || !cols[1].ends_with(&needle) {
            continue;
        }
        if let Ok(pid) = cols[cols.len() - 1].parse::<u32>() {
            if pid > 0 && !pids.contains(&pid) {
                pids.push(pid);
            }
        }
    }
    pids
}

#[cfg(windows)]
fn process_is_alive_windows(pid: u32) -> bool {
    use std::process::Command;
    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output();
    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            text.contains(&pid.to_string())
        }
        Err(_) => false,
    }
}

#[cfg(windows)]
fn process_command_line_windows(pid: u32) -> Option<String> {
    use std::process::Command;
    let script = format!(
        "(Get-CimInstance Win32_Process -Filter \"ProcessId={pid}\").CommandLine"
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() || text.eq_ignore_ascii_case("null") {
        None
    } else {
        Some(text)
    }
}

#[cfg(windows)]
fn terminate_pid_windows(pid: u32) -> Result<(), String> {
    use std::process::Command;
    let status = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .status()
        .map_err(|e| format!("taskkill failed: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("taskkill exited with {status}"))
    }
}

#[cfg(not(windows))]
fn listening_pids_on_port_unix(port: u16) -> Vec<u32> {
    use std::process::Command;
    let output = Command::new("lsof")
        .args(["-nP", &format!("-iTCP:{port}"), "-sTCP:LISTEN", "-t"])
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect()
}

#[cfg(not(windows))]
fn process_is_alive_unix(pid: u32) -> bool {
    use std::process::Command;
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn process_command_line_unix(pid: u32) -> Option<String> {
    use std::process::Command;
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "args="])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(not(windows))]
fn terminate_pid_unix(pid: u32) -> Result<(), String> {
    use std::process::Command;
    let status = Command::new("kill")
        .args([&pid.to_string()])
        .status()
        .map_err(|e| format!("kill failed: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("kill exited with {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record() -> SidecarRuntimeRecord {
        SidecarRuntimeRecord {
            nonce: "abc".into(),
            pid: 4242,
            port: 9527,
            agent_dir: r"D:\code\Misaka-Tauri\agent".into(),
            kind: ManagedKind::PythonUvicorn,
            started_at_ms: 1,
        }
    }

    #[test]
    fn free_when_port_not_in_use() {
        let occ = classify_port_occupancy(false, &[], None, false, None);
        assert_eq!(occ, PortOccupancy::Free);
    }

    #[test]
    fn managed_when_record_matches() {
        let rec = sample_record();
        let cmd = r#"python -m uvicorn app.main:app --host 127.0.0.1 --port 9527"#;
        let occ = classify_port_occupancy(true, &[4242], Some(&rec), true, Some(cmd));
        assert!(matches!(occ, PortOccupancy::Managed(_)));
    }

    #[test]
    fn managed_when_python_run_py_matches_recorded_agent_directory() {
        let rec = sample_record();
        let cmd = r#"C:\Python\python.exe D:\code\Misaka-Tauri\agent\run.py"#;
        let occ = classify_port_occupancy(true, &[4242], Some(&rec), true, Some(cmd));
        assert!(matches!(occ, PortOccupancy::Managed(_)));
    }

    #[test]
    fn run_py_outside_recorded_agent_directory_is_unknown() {
        let rec = sample_record();
        let cmd = r#"C:\Python\python.exe D:\other\run.py"#;
        let occ = classify_port_occupancy(true, &[4242], Some(&rec), true, Some(cmd));
        assert_eq!(occ, PortOccupancy::Unknown { pid: Some(4242) });
    }

    #[test]
    fn unknown_when_record_pid_not_listening() {
        let rec = sample_record();
        let cmd = r#"python -m uvicorn app.main:app --port 9527"#;
        let occ = classify_port_occupancy(true, &[9999], Some(&rec), true, Some(cmd));
        assert_eq!(occ, PortOccupancy::Unknown { pid: Some(9999) });
    }

    #[test]
    fn unknown_without_record() {
        let occ = classify_port_occupancy(true, &[111], None, true, Some("python -m http.server"));
        assert_eq!(occ, PortOccupancy::Unknown { pid: Some(111) });
    }

    #[test]
    fn packaged_binary_match_requires_binary_name() {
        let mut rec = sample_record();
        rec.kind = ManagedKind::PackagedBinary;
        assert!(record_matches_process(
            &rec,
            true,
            Some(r"D:\app\misaka-agent.exe"),
            &[4242]
        ));
        assert!(!record_matches_process(
            &rec,
            true,
            Some("python -m http.server"),
            &[4242]
        ));
    }

    #[test]
    fn runtime_record_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = runtime_record_path(dir.path());
        let rec = sample_record();
        write_runtime_record(&path, &rec).unwrap();
        assert_eq!(read_runtime_record(&path), Some(rec));
        clear_runtime_record(&path);
        assert!(read_runtime_record(&path).is_none());
    }
}
