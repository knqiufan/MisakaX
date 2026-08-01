use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine;
use misaka_x_lib::services::terminal::{
    TerminalDomainEvent, TerminalExitReason, TerminalLimits, TerminalManager, TerminalOwner,
    TerminalServiceError, TerminalSpawnRequest,
};
#[cfg(windows)]
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

fn owner(chat: &str) -> TerminalOwner {
    TerminalOwner {
        window_label: "test-window".to_string(),
        chat_session_id: chat.to_string(),
        workspace_generation: 7,
    }
}

#[cfg(windows)]
#[test]
fn portable_pty_direct_conpty_smoke() {
    let cr = direct_cmd_input(b"\r");
    assert!(cr
        .as_ref()
        .is_some_and(|output| output.contains("DIRECT_PTY_OK")));
}

#[cfg(windows)]
fn direct_cmd_input(ending: &[u8]) -> Option<String> {
    use std::io::{Read, Write};

    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let shell = std::env::var_os("SystemRoot")
        .map(std::path::PathBuf::from)
        .unwrap()
        .join("System32")
        .join("cmd.exe");
    let mut command = CommandBuilder::new(shell);
    command.arg("/Q");
    let mut child = pair.slave.spawn_command(command).unwrap();
    let mut killer = child.clone_killer();
    drop(pair.slave);
    let mut reader = pair.master.try_clone_reader().unwrap();
    let (output_tx, output_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut output = Vec::new();
        let _ = reader.read_to_end(&mut output);
        let _ = output_tx.send(output);
    });
    let mut writer = pair.master.take_writer().unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let mut input = b"\x1b[1;1Recho DIRECT_PTY_OK".to_vec();
    input.extend_from_slice(ending);
    input.extend_from_slice(b"exit 9");
    input.extend_from_slice(ending);
    writer.write_all(&input).unwrap();
    writer.flush().unwrap();
    let (status_tx, status_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = status_tx.send(child.wait());
    });
    let status = match status_rx.recv_timeout(Duration::from_secs(3)) {
        Ok(Ok(status)) => Some(status.exit_code()),
        _ => {
            let _ = killer.kill();
            None
        }
    };
    drop(writer);
    drop(pair.master);
    let output = output_rx.recv_timeout(Duration::from_secs(3)).ok()?;
    Some(format!(
        "status={status:?}; output={}",
        String::from_utf8_lossy(&output)
    ))
}

fn spawn(
    manager: &Arc<TerminalManager>,
    cwd: &std::path::Path,
    chat: &str,
) -> misaka_x_lib::services::terminal::TerminalState {
    spawn_profile(manager, cwd, chat, "auto")
}

fn spawn_profile(
    manager: &Arc<TerminalManager>,
    cwd: &std::path::Path,
    chat: &str,
    profile: &str,
) -> misaka_x_lib::services::terminal::TerminalState {
    let state = manager
        .spawn(TerminalSpawnRequest {
            owner: owner(chat),
            cwd: cwd.to_path_buf(),
            rows: 24,
            cols: 80,
            shell_profile: Some(profile.to_string()),
        })
        .expect("PTY should spawn");
    // A real xterm replies only after parsing ConPTY's initial DSR request.
    std::thread::sleep(Duration::from_millis(300));
    state
}

fn collect_until_exit(
    events_rx: &mpsc::Receiver<TerminalDomainEvent>,
    deadline: Instant,
) -> (
    Vec<u8>,
    misaka_x_lib::services::terminal::TerminalExitedPayload,
) {
    let mut output = Vec::new();
    let mut exit = None;
    while Instant::now() < deadline && exit.is_none() {
        match events_rx.recv_timeout(Duration::from_millis(250)) {
            Ok(TerminalDomainEvent::Output(event)) => {
                let chunk = base64::engine::general_purpose::STANDARD
                    .decode(event.payload.data_base64)
                    .unwrap();
                output.extend(chunk);
            }
            Ok(TerminalDomainEvent::Exited(event)) => exit = Some(event.payload),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(error) => panic!("event stream closed: {error}"),
        }
    }
    let exit = exit.unwrap_or_else(|| {
        let tail_start = output.len().saturating_sub(1024);
        panic!(
            "terminal should exit; captured_bytes={}; tail={:?}",
            output.len(),
            String::from_utf8_lossy(&output[tail_start..])
        )
    });
    (output, exit)
}

fn shell_line(shell_name: &str, powershell: &str, cmd: &str, unix: &str) -> String {
    if matches!(shell_name, "pwsh" | "powershell") {
        format!("{powershell}\r")
    } else if shell_name == "cmd" {
        format!("{cmd}\r")
    } else {
        format!("{unix}\n")
    }
}

fn with_terminal_handshake(line: String) -> Vec<u8> {
    #[cfg(windows)]
    let mut input = b"\x1b[1;1R".to_vec();
    #[cfg(not(windows))]
    let mut input = Vec::new();
    input.extend_from_slice(line.as_bytes());
    input
}

#[cfg(windows)]
struct VerbatimDirGuard(std::path::PathBuf);

#[cfg(windows)]
impl Drop for VerbatimDirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn conpty_round_trip_resize_unicode_owner_tamper_and_exit_code() {
    let cwd = tempfile::tempdir().unwrap();
    let manager = Arc::new(TerminalManager::default());
    let (events_tx, events_rx) = mpsc::channel();
    manager.start(move |event| events_tx.send(event).unwrap(), |_| {});

    let state = spawn(&manager, cwd.path(), "chat-a");
    assert!(uuid::Uuid::parse_str(&state.terminal_id).is_ok());
    assert_eq!(
        manager.resize(&state.terminal_id, &owner("chat-b"), 30, 100),
        Err(TerminalServiceError::OwnershipMismatch)
    );
    manager
        .resize(&state.terminal_id, &owner("chat-a"), 30, 100)
        .unwrap();

    let line = shell_line(
        &state.shell_name,
        "[Console]::Write(\"$([char]27)[31mW3_COLOR$([char]27)[0m`r`n$([char]27)[?1049hW3_TUI$([char]27)[?1049l`r`n\"); Write-Output 'W3_UNICODE_你好'; exit 7",
        "echo W3_COLOR & echo W3_TUI & echo W3_UNICODE_你好 & exit 7",
        "printf '\\033[31mW3_COLOR\\033[0m\\n\\033[?1049hW3_TUI\\033[?1049l\\nW3_UNICODE_你好\\n'; exit 7",
    );
    manager
        .write(
            &state.terminal_id,
            &owner("chat-a"),
            &with_terminal_handshake(line),
        )
        .unwrap();

    let (output, exit) = collect_until_exit(&events_rx, Instant::now() + Duration::from_secs(10));
    assert_eq!(exit.exit_code, Some(7));
    assert_eq!(exit.reason, TerminalExitReason::ProcessExited);
    assert!(exit.last_seq >= 1);
    let output = String::from_utf8_lossy(&output);
    assert!(output.contains("W3_UNICODE_你好"));
    if state.shell_name != "cmd" {
        assert!(output.contains("\x1b[31mW3_COLOR"));
        assert!(output.contains("\x1b[?1049h"));
        assert!(output.contains("W3_TUI"));
    }
    assert_eq!(manager.active_count(), 0);
}

#[cfg(unix)]
#[test]
fn unix_shell_profiles_round_trip_and_report_fallbacks() {
    let cwd = tempfile::tempdir().unwrap();
    let manager = Arc::new(TerminalManager::default());
    let (events_tx, events_rx) = mpsc::channel();
    manager.start(move |event| events_tx.send(event).unwrap(), |_| {});

    for requested in ["auto", "zsh", "bash", "sh", "fish"] {
        let chat = format!("unix-{requested}");
        let state = spawn_profile(&manager, cwd.path(), &chat, requested);
        assert!(matches!(
            state.shell_name.as_str(),
            "zsh" | "bash" | "sh" | "fish"
        ));
        if requested != "auto" && requested != state.shell_name {
            assert!(state.fallback_reason.is_some());
        }

        let marker = format!("W6_UNIX_PROFILE_{}={}", requested, state.shell_name);
        manager
            .write(
                &state.terminal_id,
                &owner(&chat),
                format!("printf '{marker}\\n'; exit 0\n").as_bytes(),
            )
            .unwrap();
        let (output, exit) =
            collect_until_exit(&events_rx, Instant::now() + Duration::from_secs(10));
        assert_eq!(exit.exit_code, Some(0));
        assert!(String::from_utf8_lossy(&output).contains(&marker));
        println!(
            "requested={requested}; resolved={}; fallback={:?}",
            state.shell_name, state.fallback_reason
        );
    }

    assert_eq!(manager.active_count(), 0);
}

#[cfg(windows)]
#[test]
fn windows_profiles_and_long_unicode_workspace_round_trip() {
    let base = tempfile::tempdir().unwrap();
    let mut long_cwd = base.path().to_path_buf();
    for index in 0..7 {
        long_cwd.push(format!("中文路径段{index}{}", "x".repeat(32)));
    }
    let verbatim_cwd = std::path::PathBuf::from(format!(r"\\?\{}", long_cwd.display()));
    std::fs::create_dir_all(&verbatim_cwd).expect("create a >MAX_PATH Unicode workspace");
    let _long_path_guard = VerbatimDirGuard(verbatim_cwd.clone());
    assert!(long_cwd.as_os_str().len() > 260);

    let manager = Arc::new(TerminalManager::default());
    let (events_tx, events_rx) = mpsc::channel();
    manager.start(move |event| events_tx.send(event).unwrap(), |_| {});
    let state = spawn_profile(&manager, &verbatim_cwd, "long-powershell", "powershell");
    assert_eq!(state.shell_name, "powershell");
    manager
        .write(
            &state.terminal_id,
            &owner("long-powershell"),
            &with_terminal_handshake(shell_line(
                &state.shell_name,
                "Write-Output \"W6_PS_MAJOR=$($PSVersionTable.PSVersion.Major)\"; Write-Output 'W6_LONG_UNICODE_OK'; exit 0",
                "echo W6_LONG_UNICODE_OK & exit 0",
                "printf 'W6_LONG_UNICODE_OK\\n'; exit 0",
            )),
        )
        .unwrap();
    let (output, exit) = collect_until_exit(&events_rx, Instant::now() + Duration::from_secs(12));
    assert_eq!(exit.exit_code, Some(0));
    let output = String::from_utf8_lossy(&output);
    assert!(output.contains("W6_PS_MAJOR=5"));
    assert!(output.contains("W6_LONG_UNICODE_OK"));

    let cmd_long_path = Arc::new(TerminalManager::default());
    let result = cmd_long_path.spawn(TerminalSpawnRequest {
        owner: owner("cmd-long-path"),
        cwd: verbatim_cwd.clone(),
        rows: 24,
        cols: 80,
        shell_profile: Some("cmd".to_string()),
    });
    assert!(matches!(
        result,
        Err(TerminalServiceError::SpawnFailed("shell_workspace_path"))
    ));
    assert_eq!(cmd_long_path.active_count(), 0);

    let manager = Arc::new(TerminalManager::default());
    let (events_tx, events_rx) = mpsc::channel();
    manager.start(move |event| events_tx.send(event).unwrap(), |_| {});
    let state = spawn_profile(&manager, base.path(), "cmd-profile", "cmd");
    assert_eq!(state.shell_name, "cmd");
    manager
        .write(
            &state.terminal_id,
            &owner("cmd-profile"),
            &with_terminal_handshake(shell_line(
                &state.shell_name,
                "Write-Output 'W6_CMD_OK'; exit 0",
                "echo W6_CMD_OK & exit 0",
                "printf 'W6_CMD_OK\\n'; exit 0",
            )),
        )
        .unwrap();
    let (output, exit) = collect_until_exit(&events_rx, Instant::now() + Duration::from_secs(10));
    assert_eq!(exit.exit_code, Some(0));
    assert!(String::from_utf8_lossy(&output).contains("W6_CMD_OK"));

    let manager = Arc::new(TerminalManager::default());
    let (events_tx, events_rx) = mpsc::channel();
    manager.start(move |event| events_tx.send(event).unwrap(), |_| {});
    let pwsh = spawn_profile(&manager, base.path(), "pwsh-profile", "pwsh");
    manager
        .write(
            &pwsh.terminal_id,
            &owner("pwsh-profile"),
            &with_terminal_handshake(shell_line(
                &pwsh.shell_name,
                "Write-Output \"W6_PWSH_MAJOR=$($PSVersionTable.PSVersion.Major)\"; Write-Output 'W6_PWSH_OK'; exit 0",
                "echo W6_PWSH_UNEXPECTED_CMD & exit 1",
                "printf 'W6_PWSH_UNEXPECTED_UNIX\\n'; exit 1",
            )),
        )
        .unwrap();
    let (output, exit) = collect_until_exit(&events_rx, Instant::now() + Duration::from_secs(10));
    assert_eq!(exit.exit_code, Some(0));
    let output = String::from_utf8_lossy(&output);
    assert!(output.contains("W6_PWSH_OK"));
    if pwsh.shell_name == "pwsh" {
        assert!(output.contains("W6_PWSH_MAJOR=7"));
        assert!(pwsh.fallback_reason.is_none());
    } else {
        assert_eq!(pwsh.shell_name, "powershell");
        assert!(output.contains("W6_PWSH_MAJOR=5"));
        assert!(pwsh.fallback_reason.is_some());
    }
}

#[test]
fn rejects_missing_cwd_before_starting_a_host_process() {
    let manager = Arc::new(TerminalManager::default());
    let missing = tempfile::tempdir().unwrap().path().join("missing");
    let result = manager.spawn(TerminalSpawnRequest {
        owner: owner("cwd-race"),
        cwd: missing,
        rows: 24,
        cols: 80,
        shell_profile: None,
    });
    assert!(matches!(
        result,
        Err(TerminalServiceError::SpawnFailed("workspace_missing"))
    ));
    assert_eq!(manager.active_count(), 0);
}

#[test]
fn output_flood_is_rate_limited_and_process_is_stopped() {
    let cwd = tempfile::tempdir().unwrap();
    let mut limits = TerminalLimits::default();
    limits.max_output_bytes_per_second = 1024;
    let manager = Arc::new(TerminalManager::new(limits));
    let (events_tx, events_rx) = mpsc::channel();
    manager.start(move |event| events_tx.send(event).unwrap(), |_| {});
    let state = spawn(&manager, cwd.path(), "flood");
    let line = shell_line(
        &state.shell_name,
        "1..500 | ForEach-Object { 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' }",
        "for /L %i in (1,1,500) do @echo xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        "yes xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx | head -c 20000",
    );
    manager
        .write(
            &state.terminal_id,
            &owner("flood"),
            &with_terminal_handshake(line),
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut reason = None;
    while Instant::now() < deadline && reason.is_none() {
        if let Ok(TerminalDomainEvent::Exited(event)) =
            events_rx.recv_timeout(Duration::from_millis(250))
        {
            reason = Some(event.payload.reason);
        }
    }
    assert_eq!(reason, Some(TerminalExitReason::OutputLimit));
    assert_eq!(manager.active_count(), 0);
}

#[test]
fn ten_mibibyte_long_line_is_bounded_and_reaps_the_process() {
    let cwd = tempfile::tempdir().unwrap();
    let mut limits = TerminalLimits::default();
    limits.max_output_bytes_per_second = 256 * 1024;
    let manager = Arc::new(TerminalManager::new(limits));
    let (events_tx, events_rx) = mpsc::channel();
    manager.start(move |event| events_tx.send(event).unwrap(), |_| {});
    let state = spawn(&manager, cwd.path(), "ten-mib-long-line");
    let line = shell_line(
        &state.shell_name,
        "$line = 'x' * (10 * 1024 * 1024); [Console]::Out.Write($line)",
        "powershell -NoLogo -NoProfile -Command \"$line = 'x' * (10 * 1024 * 1024); [Console]::Out.Write($line)\"",
        "head -c 10485760 /dev/zero | tr '\\0' x",
    );
    let started = Instant::now();
    manager
        .write(
            &state.terminal_id,
            &owner("ten-mib-long-line"),
            &with_terminal_handshake(line),
        )
        .unwrap();

    let (output, exit) = collect_until_exit(&events_rx, started + Duration::from_secs(20));
    assert_eq!(exit.reason, TerminalExitReason::OutputLimit);
    assert!(
        output.len() <= 512 * 1024,
        "bounded output should stop near the 256 KiB/s test limit, got {} bytes",
        output.len()
    );
    assert!(started.elapsed() < Duration::from_secs(20));
    assert_eq!(manager.active_count(), 0);
}

#[test]
fn nominal_ten_mibibytes_per_second_source_is_bounded_and_reaped() {
    let cwd = tempfile::tempdir().unwrap();
    let manager = Arc::new(TerminalManager::default());
    let (events_tx, events_rx) = mpsc::channel();
    manager.start(move |event| events_tx.send(event).unwrap(), |_| {});
    let state = spawn(&manager, cwd.path(), "ten-mib-per-second");
    let line = shell_line(
        &state.shell_name,
        "$chunk = 'x' * (512 * 1024); 1..24 | ForEach-Object { [Console]::Out.Write($chunk); Start-Sleep -Milliseconds 50 }; exit 0",
        "powershell -NoLogo -NoProfile -Command \"$chunk = 'x' * (512 * 1024); 1..24 | ForEach-Object { [Console]::Out.Write($chunk); Start-Sleep -Milliseconds 50 }\" & exit 0",
        "i=0; while [ $i -lt 24 ]; do head -c 524288 /dev/zero | tr '\\0' x; sleep 0.05; i=$((i + 1)); done; exit 0",
    );
    let started = Instant::now();
    manager
        .write(
            &state.terminal_id,
            &owner("ten-mib-per-second"),
            &with_terminal_handshake(line),
        )
        .unwrap();

    let (output, exit) = collect_until_exit(&events_rx, started + Duration::from_secs(20));
    assert!(matches!(
        exit.reason,
        TerminalExitReason::OutputLimit | TerminalExitReason::ProcessExited
    ));
    if exit.reason == TerminalExitReason::ProcessExited {
        assert_eq!(exit.exit_code, Some(0));
    }
    assert!(
        output.len() >= 1024 * 1024,
        "the paced source should exercise sustained output, got {} bytes",
        output.len()
    );
    assert!(
        output.len() <= 32 * 1024 * 1024,
        "PTY transformation and buffering should remain bounded, got {} bytes",
        output.len()
    );
    assert!(started.elapsed() < Duration::from_secs(20));
    assert_eq!(manager.active_count(), 0);
}

#[cfg(windows)]
#[test]
fn crash_helper_process_holds_terminal_job() {
    if std::env::var_os("MISAKAX_W6_TERMINAL_CRASH_HELPER").is_none() {
        return;
    }
    let cwd = std::path::PathBuf::from(
        std::env::var_os("MISAKAX_W6_TERMINAL_CRASH_CWD").expect("helper cwd"),
    );
    let pid_file = std::path::PathBuf::from(
        std::env::var_os("MISAKAX_W6_TERMINAL_CRASH_PID").expect("helper pid file"),
    );
    let manager = Arc::new(TerminalManager::default());
    manager.start(|_| {}, |_| {});
    let state = spawn(&manager, &cwd, "crash-helper");
    let quoted = pid_file.to_string_lossy().replace('\'', "''");
    let line = shell_line(
        &state.shell_name,
        &format!(
            "$p = Start-Process -FilePath powershell.exe -ArgumentList '-NoLogo','-NoProfile','-Command','Start-Sleep -Seconds 60' -PassThru; Set-Content -LiteralPath '{quoted}' -Value $p.Id -NoNewline"
        ),
        &format!(
            "powershell -NoLogo -NoProfile -Command \"$p=Start-Process powershell.exe -ArgumentList '-NoLogo','-NoProfile','-Command','Start-Sleep 60' -PassThru; Set-Content -LiteralPath '{quoted}' -Value $p.Id -NoNewline\""
        ),
        "sleep 60",
    );
    manager
        .write(
            &state.terminal_id,
            &owner("crash-helper"),
            &with_terminal_handshake(line),
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    while !pid_file.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(pid_file.exists(), "helper grandchild pid should be written");
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(windows)]
#[test]
fn abrupt_app_termination_reaps_job_and_a_new_manager_can_reopen() {
    use std::process::{Command, Stdio};

    let cwd = tempfile::tempdir().unwrap();
    let pid_file = cwd.path().join("crash-grandchild.pid");
    let current_test = std::env::current_exe().expect("current integration test executable");
    let mut helper = Command::new(current_test)
        .arg("--exact")
        .arg("crash_helper_process_holds_terminal_job")
        .arg("--nocapture")
        .env("MISAKAX_W6_TERMINAL_CRASH_HELPER", "1")
        .env("MISAKAX_W6_TERMINAL_CRASH_CWD", cwd.path())
        .env("MISAKAX_W6_TERMINAL_CRASH_PID", &pid_file)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn crash helper test process");

    let deadline = Instant::now() + Duration::from_secs(20);
    while !pid_file.exists() && Instant::now() < deadline {
        if let Some(status) = helper.try_wait().expect("poll crash helper") {
            panic!("crash helper exited before pid handoff: {status}");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let grandchild_pid = std::fs::read_to_string(&pid_file)
        .expect("crash helper grandchild pid file")
        .trim()
        .parse::<u32>()
        .expect("numeric crash grandchild pid");
    assert!(process_exists(grandchild_pid));

    helper.kill().expect("terminate helper like an app crash");
    helper.wait().expect("reap crash helper");
    let deadline = Instant::now() + Duration::from_secs(8);
    while process_exists(grandchild_pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        !process_exists(grandchild_pid),
        "KILL_ON_JOB_CLOSE must reap the grandchild after abrupt app termination"
    );

    let manager = Arc::new(TerminalManager::default());
    let (events_tx, events_rx) = mpsc::channel();
    manager.start(move |event| events_tx.send(event).unwrap(), |_| {});
    let state = spawn(&manager, cwd.path(), "reopened-after-crash");
    manager
        .write(
            &state.terminal_id,
            &owner("reopened-after-crash"),
            &with_terminal_handshake(shell_line(
                &state.shell_name,
                "Write-Output 'W6_REOPEN_OK'; exit 0",
                "echo W6_REOPEN_OK & exit 0",
                "printf 'W6_REOPEN_OK\\n'; exit 0",
            )),
        )
        .unwrap();
    let (output, exit) = collect_until_exit(&events_rx, Instant::now() + Duration::from_secs(10));
    assert_eq!(exit.exit_code, Some(0));
    assert!(String::from_utf8_lossy(&output).contains("W6_REOPEN_OK"));
}

#[test]
fn shutdown_reaps_shell_and_grandchild_process_tree() {
    let cwd = tempfile::tempdir().unwrap();
    let pid_file = cwd.path().join("grandchild.pid");
    let manager = Arc::new(TerminalManager::default());
    manager.start(|_| {}, |_| {});
    let state = spawn(&manager, cwd.path(), "shutdown");
    let quoted = pid_file.to_string_lossy().replace('\'', "''");
    let line = shell_line(
        &state.shell_name,
        &format!(
            "$exe = Join-Path $PSHOME '{}.exe'; $p = Start-Process -FilePath $exe -ArgumentList '-NoLogo','-Command','Start-Sleep -Seconds 30' -PassThru; Set-Content -LiteralPath '{}' -Value $p.Id -NoNewline",
            state.shell_name, quoted
        ),
        &format!(
            "powershell -NoLogo -Command \"$p=Start-Process powershell -ArgumentList '-NoLogo','-Command','Start-Sleep 30' -PassThru; Set-Content -LiteralPath '{}' -Value $p.Id -NoNewline\"",
            quoted
        ),
        &format!("sleep 30 & echo $! > '{}'", quoted),
    );
    manager
        .write(
            &state.terminal_id,
            &owner("shutdown"),
            &with_terminal_handshake(line),
        )
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(8);
    while !pid_file.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    let pid = std::fs::read_to_string(&pid_file)
        .expect("grandchild pid file")
        .trim()
        .parse::<u32>()
        .expect("numeric grandchild pid");
    assert!(process_exists(pid));

    manager.shutdown();
    let deadline = Instant::now() + Duration::from_secs(5);
    while process_exists(pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!process_exists(pid), "grandchild process must be reaped");
    assert_eq!(manager.active_count(), 0);
}

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return false;
    }
    unsafe { CloseHandle(handle) };
    true
}

#[cfg(unix)]
fn process_exists(pid: u32) -> bool {
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}
