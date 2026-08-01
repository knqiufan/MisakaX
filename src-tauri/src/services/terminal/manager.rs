use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

use base64::Engine;
use portable_pty::{native_pty_system, ChildKiller, MasterPty, PtySize};

use crate::contracts::DomainEvent;
use crate::services::workspace::strip_windows_verbatim_prefix;

use super::process_tree::ProcessTreeGuard;
use super::shell::resolve_shell;
use super::types::{
    TerminalDomainEvent, TerminalExitReason, TerminalExitedPayload, TerminalLimits,
    TerminalOutputPayload, TerminalOwner, TerminalServiceError, TerminalSpawnRequest,
    TerminalState, TerminalStatus,
};

type EventSink = Arc<dyn Fn(TerminalDomainEvent) + Send + Sync>;
type WorkspaceRefreshSink = Arc<dyn Fn(String) + Send + Sync>;

struct TerminalSession {
    owner: TerminalOwner,
    state: Mutex<TerminalState>,
    master: Mutex<Option<Box<dyn MasterPty + Send>>>,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
    killer: Mutex<Box<dyn ChildKiller + Send + Sync>>,
    process_tree: ProcessTreeGuard,
    requested_reason: Mutex<Option<TerminalExitReason>>,
    exit: ExitSignal,
    next_seq: AtomicU64,
    stopped: AtomicBool,
    command_rates: Mutex<CommandRates>,
}

impl TerminalSession {
    fn authorize(&self, owner: &TerminalOwner) -> Result<(), TerminalServiceError> {
        if &self.owner == owner {
            Ok(())
        } else {
            Err(TerminalServiceError::OwnershipMismatch)
        }
    }

    fn terminate(&self, reason: TerminalExitReason) {
        if let Ok(mut requested) = self.requested_reason.lock() {
            if requested.is_none() {
                *requested = Some(reason);
            }
        }
        if self.stopped.swap(true, Ordering::AcqRel) {
            return;
        }
        if let Ok(mut writer) = self.writer.lock() {
            writer.take();
        }
        self.process_tree.terminate();
        if let Ok(mut killer) = self.killer.lock() {
            let _ = killer.kill();
        }
    }
}

#[derive(Debug, Clone)]
struct ExitOutcome {
    exit_code: Option<u32>,
    signal: Option<String>,
}

struct ExitSignal {
    outcome: Mutex<Option<ExitOutcome>>,
    ready: Condvar,
}

impl ExitSignal {
    fn new() -> Self {
        Self {
            outcome: Mutex::new(None),
            ready: Condvar::new(),
        }
    }

    fn complete(&self, outcome: ExitOutcome) {
        let mut slot = self.outcome.lock().expect("terminal exit mutex poisoned");
        *slot = Some(outcome);
        self.ready.notify_all();
    }

    fn wait(&self) -> ExitOutcome {
        let mut slot = self.outcome.lock().expect("terminal exit mutex poisoned");
        while slot.is_none() {
            let (next, timeout) = self
                .ready
                .wait_timeout(slot, Duration::from_secs(2))
                .expect("terminal exit mutex poisoned");
            slot = next;
            if timeout.timed_out() && slot.is_none() {
                return ExitOutcome {
                    exit_code: None,
                    signal: None,
                };
            }
        }
        slot.clone().expect("terminal exit outcome checked")
    }
}

#[derive(Default)]
struct CommandRates {
    commands: FixedWindow,
    writes: FixedWindow,
    input_bytes: FixedWindow,
    resizes: FixedWindow,
}

#[derive(Default)]
struct FixedWindow {
    started: Option<Instant>,
    value: u64,
}

impl FixedWindow {
    fn allow(&mut self, amount: u64, maximum: u64, window: Duration) -> bool {
        let now = Instant::now();
        if self
            .started
            .is_none_or(|started| now.duration_since(started) >= window)
        {
            self.started = Some(now);
            self.value = 0;
        }
        if self.value.saturating_add(amount) > maximum {
            return false;
        }
        self.value += amount;
        true
    }
}

pub struct TerminalManager {
    limits: TerminalLimits,
    sessions: Mutex<HashMap<String, Arc<TerminalSession>>>,
    spawn_history: Mutex<HashMap<String, VecDeque<Instant>>>,
    spawn_guard: Mutex<()>,
    event_sink: Mutex<Option<EventSink>>,
    workspace_refresh_sink: Mutex<Option<WorkspaceRefreshSink>>,
    shutting_down: AtomicBool,
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new(TerminalLimits::default())
    }
}

impl TerminalManager {
    pub fn new(limits: TerminalLimits) -> Self {
        Self {
            limits,
            sessions: Mutex::new(HashMap::new()),
            spawn_history: Mutex::new(HashMap::new()),
            spawn_guard: Mutex::new(()),
            event_sink: Mutex::new(None),
            workspace_refresh_sink: Mutex::new(None),
            shutting_down: AtomicBool::new(false),
        }
    }

    pub fn start(
        &self,
        event_sink: impl Fn(TerminalDomainEvent) + Send + Sync + 'static,
        workspace_refresh_sink: impl Fn(String) + Send + Sync + 'static,
    ) {
        *self
            .event_sink
            .lock()
            .expect("terminal event mutex poisoned") = Some(Arc::new(event_sink));
        *self
            .workspace_refresh_sink
            .lock()
            .expect("terminal refresh mutex poisoned") = Some(Arc::new(workspace_refresh_sink));
    }

    pub fn spawn(
        self: &Arc<Self>,
        request: TerminalSpawnRequest,
    ) -> Result<TerminalState, TerminalServiceError> {
        if self.shutting_down.load(Ordering::Acquire) {
            return Err(TerminalServiceError::SpawnFailed("app_exiting"));
        }
        self.limits.validate_size(request.rows, request.cols)?;
        let canonical_cwd = std::fs::canonicalize(&request.cwd)
            .map_err(|_| TerminalServiceError::SpawnFailed("workspace_missing"))?;
        if !canonical_cwd.is_dir() {
            return Err(TerminalServiceError::SpawnFailed("workspace_missing"));
        }

        let _spawn_guard = self
            .spawn_guard
            .lock()
            .expect("terminal spawn mutex poisoned");
        self.check_session_limits(&request.owner)?;
        self.check_spawn_rate(&request.owner.window_label)?;

        let shell = resolve_shell(request.shell_profile.as_deref())?;
        let pty = native_pty_system();
        let pair = pty
            .openpty(PtySize {
                rows: request.rows,
                cols: request.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|_| TerminalServiceError::SpawnFailed("pty_open"))?;
        // Windows canonicalization adds a `\\?\` prefix. Keep it for
        // identity validation, then pass the equivalent display-form cwd to
        // the shell so PowerShell does not render a verbose provider path.
        let shell_cwd = strip_windows_verbatim_prefix(&canonical_cwd);
        let command = shell.command(&shell_cwd);
        let mut child = pair
            .slave
            .spawn_command(command)
            .map_err(|_| TerminalServiceError::SpawnFailed("shell_start"))?;
        // ConPTY requires the slave-side startup handles to remain intact until
        // after process creation, then to be released before host I/O begins.
        drop(pair.slave);
        let process_tree = match ProcessTreeGuard::attach(child.as_ref(), pair.master.as_ref()) {
            Ok(guard) => guard,
            Err(error) => {
                let _ = child.kill();
                return Err(error);
            }
        };
        let killer = child.clone_killer();
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|_| TerminalServiceError::SpawnFailed("pty_reader"))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|_| TerminalServiceError::SpawnFailed("pty_writer"))?;

        let terminal_id = self.unique_terminal_id();
        let state = TerminalState {
            terminal_id: terminal_id.clone(),
            chat_session_id: request.owner.chat_session_id.clone(),
            workspace_generation: request.owner.workspace_generation,
            shell_profile: shell.profile,
            shell_name: shell.shell_name,
            fallback_reason: shell.fallback_reason,
            rows: request.rows,
            cols: request.cols,
            status: TerminalStatus::Running,
            started_at: chrono::Utc::now().to_rfc3339(),
        };
        let session = Arc::new(TerminalSession {
            owner: request.owner,
            state: Mutex::new(state.clone()),
            master: Mutex::new(Some(pair.master)),
            writer: Mutex::new(Some(writer)),
            killer: Mutex::new(killer),
            process_tree,
            requested_reason: Mutex::new(None),
            exit: ExitSignal::new(),
            next_seq: AtomicU64::new(1),
            stopped: AtomicBool::new(false),
            command_rates: Mutex::new(CommandRates::default()),
        });
        self.sessions
            .lock()
            .expect("terminal sessions mutex poisoned")
            .insert(terminal_id.clone(), Arc::clone(&session));

        self.start_wait_thread(Arc::clone(&session), child);
        self.start_output_threads(session, reader);
        Ok(state)
    }

    pub fn write(
        &self,
        terminal_id: &str,
        owner: &TerminalOwner,
        bytes: &[u8],
    ) -> Result<(), TerminalServiceError> {
        if bytes.is_empty() || bytes.len() > self.limits.max_input_bytes {
            return Err(TerminalServiceError::InvalidRequest("input_size"));
        }
        let session = self.authorized_session(terminal_id, owner)?;
        {
            let mut rates = session
                .command_rates
                .lock()
                .map_err(|_| TerminalServiceError::Internal("rate_lock"))?;
            if !rates.commands.allow(
                1,
                self.limits.max_commands_per_second as u64,
                Duration::from_secs(1),
            ) || !rates.writes.allow(
                1,
                self.limits.max_writes_per_second as u64,
                Duration::from_secs(1),
            ) || !rates.input_bytes.allow(
                bytes.len() as u64,
                self.limits.max_input_bytes_per_second as u64,
                Duration::from_secs(1),
            ) {
                return Err(TerminalServiceError::LimitExceeded("input_rate"));
            }
        }
        let mut writer = session
            .writer
            .lock()
            .map_err(|_| TerminalServiceError::Internal("writer_lock"))?;
        let writer = writer.as_mut().ok_or(TerminalServiceError::NotFound)?;
        writer
            .write_all(bytes)
            .and_then(|_| writer.flush())
            .map_err(|_| TerminalServiceError::NotFound)
    }

    pub fn write_base64(
        &self,
        terminal_id: &str,
        owner: &TerminalOwner,
        data_base64: &str,
    ) -> Result<(), TerminalServiceError> {
        let maximum_encoded = self.limits.max_input_bytes.div_ceil(3) * 4;
        if data_base64.is_empty() || data_base64.len() > maximum_encoded {
            return Err(TerminalServiceError::InvalidRequest("input_size"));
        }
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(data_base64)
            .map_err(|_| TerminalServiceError::InvalidRequest("input_encoding"))?;
        self.write(terminal_id, owner, &bytes)
    }

    pub fn resize(
        &self,
        terminal_id: &str,
        owner: &TerminalOwner,
        rows: u16,
        cols: u16,
    ) -> Result<(), TerminalServiceError> {
        self.limits.validate_size(rows, cols)?;
        let session = self.authorized_session(terminal_id, owner)?;
        let mut rates = session
            .command_rates
            .lock()
            .map_err(|_| TerminalServiceError::Internal("rate_lock"))?;
        if !rates.commands.allow(
            1,
            self.limits.max_commands_per_second as u64,
            Duration::from_secs(1),
        ) || !rates.resizes.allow(
            1,
            self.limits.max_resizes_per_second as u64,
            Duration::from_secs(1),
        ) {
            return Err(TerminalServiceError::LimitExceeded("resize_rate"));
        }
        drop(rates);
        session
            .master
            .lock()
            .map_err(|_| TerminalServiceError::Internal("pty_lock"))?
            .as_ref()
            .ok_or(TerminalServiceError::NotFound)?
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|_| TerminalServiceError::NotFound)?;
        let mut state = session
            .state
            .lock()
            .map_err(|_| TerminalServiceError::Internal("state_lock"))?;
        state.rows = rows;
        state.cols = cols;
        Ok(())
    }

    pub fn kill(
        &self,
        terminal_id: &str,
        owner: &TerminalOwner,
        reason: TerminalExitReason,
    ) -> Result<(), TerminalServiceError> {
        let session = self.authorized_session(terminal_id, owner)?;
        self.check_command_rate(&session)?;
        session.terminate(reason);
        Ok(())
    }

    pub fn get_state(
        &self,
        terminal_id: &str,
        owner: &TerminalOwner,
    ) -> Result<TerminalState, TerminalServiceError> {
        let session = self.authorized_session(terminal_id, owner)?;
        self.check_command_rate(&session)?;
        let state = session
            .state
            .lock()
            .map_err(|_| TerminalServiceError::Internal("state_lock"))?
            .clone();
        Ok(state)
    }

    pub fn kill_window(&self, window_label: &str) {
        self.kill_matching(TerminalExitReason::WindowClosed, |owner| {
            owner.window_label == window_label
        });
    }

    pub fn kill_chat_session(&self, chat_session_id: &str) {
        self.kill_matching(TerminalExitReason::SessionDeleted, |owner| {
            owner.chat_session_id == chat_session_id
        });
    }

    pub fn shutdown(&self) {
        if self.shutting_down.swap(true, Ordering::AcqRel) {
            return;
        }
        self.kill_matching(TerminalExitReason::AppExit, |_| true);
        let deadline = Instant::now() + Duration::from_secs(2);
        while self.active_count() > 0 && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn active_count(&self) -> usize {
        self.sessions
            .lock()
            .expect("terminal sessions mutex poisoned")
            .len()
    }

    fn authorized_session(
        &self,
        terminal_id: &str,
        owner: &TerminalOwner,
    ) -> Result<Arc<TerminalSession>, TerminalServiceError> {
        let session = self
            .sessions
            .lock()
            .map_err(|_| TerminalServiceError::Internal("sessions_lock"))?
            .get(terminal_id)
            .cloned()
            .ok_or(TerminalServiceError::NotFound)?;
        session.authorize(owner)?;
        Ok(session)
    }

    fn check_command_rate(&self, session: &TerminalSession) -> Result<(), TerminalServiceError> {
        if session
            .command_rates
            .lock()
            .map_err(|_| TerminalServiceError::Internal("rate_lock"))?
            .commands
            .allow(
                1,
                self.limits.max_commands_per_second as u64,
                Duration::from_secs(1),
            )
        {
            Ok(())
        } else {
            Err(TerminalServiceError::LimitExceeded("command_rate"))
        }
    }

    fn check_session_limits(&self, owner: &TerminalOwner) -> Result<(), TerminalServiceError> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| TerminalServiceError::Internal("sessions_lock"))?;
        if sessions.len() >= self.limits.max_global_sessions {
            return Err(TerminalServiceError::LimitExceeded("global_sessions"));
        }
        let window_count = sessions
            .values()
            .filter(|session| session.owner.window_label == owner.window_label)
            .count();
        if window_count >= self.limits.max_sessions_per_window {
            return Err(TerminalServiceError::LimitExceeded("window_sessions"));
        }
        let chat_count = sessions
            .values()
            .filter(|session| {
                session.owner.window_label == owner.window_label
                    && session.owner.chat_session_id == owner.chat_session_id
            })
            .count();
        if chat_count >= self.limits.max_sessions_per_chat {
            return Err(TerminalServiceError::LimitExceeded("chat_sessions"));
        }
        Ok(())
    }

    fn check_spawn_rate(&self, window_label: &str) -> Result<(), TerminalServiceError> {
        let mut history = self
            .spawn_history
            .lock()
            .map_err(|_| TerminalServiceError::Internal("spawn_rate_lock"))?;
        let now = Instant::now();
        let entries = history.entry(window_label.to_string()).or_default();
        while entries
            .front()
            .is_some_and(|started| now.duration_since(*started) >= Duration::from_secs(60))
        {
            entries.pop_front();
        }
        if entries.len() >= self.limits.max_spawns_per_minute as usize {
            return Err(TerminalServiceError::LimitExceeded("spawn_rate"));
        }
        entries.push_back(now);
        Ok(())
    }

    fn unique_terminal_id(&self) -> String {
        loop {
            let id = uuid::Uuid::new_v4().to_string();
            if !self
                .sessions
                .lock()
                .expect("terminal sessions mutex poisoned")
                .contains_key(&id)
            {
                return id;
            }
        }
    }

    fn start_wait_thread(
        self: &Arc<Self>,
        session: Arc<TerminalSession>,
        mut child: Box<dyn portable_pty::Child + Send + Sync>,
    ) {
        thread::Builder::new()
            .name("misakax-terminal-wait".to_string())
            .spawn(move || {
                let outcome = child.wait().map_or(
                    ExitOutcome {
                        exit_code: None,
                        signal: None,
                    },
                    |status| ExitOutcome {
                        exit_code: Some(status.exit_code()),
                        signal: status.signal().map(str::to_string),
                    },
                );
                session.exit.complete(outcome);
                // Releasing the remaining master handle after child exit lets
                // the cloned reader observe EOF and drain all queued output.
                if let Ok(mut writer) = session.writer.lock() {
                    writer.take();
                }
                if let Ok(mut master) = session.master.lock() {
                    master.take();
                }
            })
            .expect("failed to start terminal wait thread");
    }

    fn start_output_threads(
        self: &Arc<Self>,
        session: Arc<TerminalSession>,
        mut reader: Box<dyn Read + Send>,
    ) {
        let (sender, receiver) = mpsc::sync_channel::<Vec<u8>>(self.limits.output_channel_chunks);
        let read_session = Arc::clone(&session);
        let read_manager = Arc::downgrade(self);
        let read_chunk_bytes = self.limits.output_chunk_bytes;
        thread::Builder::new()
            .name("misakax-terminal-read".to_string())
            .spawn(move || {
                read_pty(
                    &mut reader,
                    sender,
                    read_chunk_bytes,
                    read_session,
                    read_manager,
                )
            })
            .expect("failed to start terminal reader thread");

        let weak = Arc::downgrade(self);
        let output_limits = self.limits;
        thread::Builder::new()
            .name("misakax-terminal-output".to_string())
            .spawn(move || {
                let mut output_rate = FixedWindow::default();
                loop {
                    let first = match receiver.recv() {
                        Ok(chunk) => chunk,
                        Err(_) => break,
                    };
                    let mut batch = first;
                    let deadline = Instant::now() + output_limits.output_batch_delay;
                    while batch.len() < output_limits.output_batch_bytes {
                        let remaining = deadline.saturating_duration_since(Instant::now());
                        if remaining.is_zero() {
                            break;
                        }
                        match receiver.recv_timeout(remaining) {
                            Ok(chunk) => {
                                let available = output_limits.output_batch_bytes - batch.len();
                                if chunk.len() <= available {
                                    batch.extend_from_slice(&chunk);
                                } else {
                                    batch.extend_from_slice(&chunk[..available]);
                                    // A PTY read chunk is bounded and batches never retain more than
                                    // output_batch_bytes; preserve the tail as the next emitted batch.
                                    if !emit_output_chunk(&weak, &session, &batch, &mut output_rate)
                                    {
                                        break;
                                    }
                                    batch = chunk[available..].to_vec();
                                }
                            }
                            Err(RecvTimeoutError::Timeout) => break,
                            Err(RecvTimeoutError::Disconnected) => break,
                        }
                    }
                    if !batch.is_empty()
                        && !emit_output_chunk(&weak, &session, &batch, &mut output_rate)
                    {
                        break;
                    }
                }
                if let Some(manager) = weak.upgrade() {
                    manager.finalize_session(&session);
                }
            })
            .expect("failed to start terminal output thread");
    }

    fn request_reader_failure(&self, session: &TerminalSession) {
        session.terminate(TerminalExitReason::ReaderError);
    }

    fn request_output_limit(&self, session: &TerminalSession) {
        session.terminate(TerminalExitReason::OutputLimit);
    }

    fn emit_output(&self, session: &TerminalSession, bytes: &[u8]) {
        let seq = session.next_seq.fetch_add(1, Ordering::AcqRel);
        let terminal_id = session
            .state
            .lock()
            .expect("terminal state mutex poisoned")
            .terminal_id
            .clone();
        let event = DomainEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            aggregate_id: terminal_id.clone(),
            generation: session.owner.workspace_generation,
            occurred_at: chrono::Utc::now().to_rfc3339(),
            payload: TerminalOutputPayload {
                terminal_id,
                seq,
                data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            },
        };
        self.dispatch(TerminalDomainEvent::Output(event));
    }

    fn finalize_session(&self, session: &Arc<TerminalSession>) {
        let terminal_id = session
            .state
            .lock()
            .expect("terminal state mutex poisoned")
            .terminal_id
            .clone();
        let outcome = session.exit.wait();
        let removed = self
            .sessions
            .lock()
            .expect("terminal sessions mutex poisoned")
            .remove(&terminal_id)
            .is_some();
        if !removed {
            return;
        }
        if let Ok(mut state) = session.state.lock() {
            state.status = TerminalStatus::Exited;
        }
        let reason = session
            .requested_reason
            .lock()
            .ok()
            .and_then(|reason| *reason)
            .unwrap_or(TerminalExitReason::ProcessExited);
        let last_seq = session.next_seq.load(Ordering::Acquire).saturating_sub(1);
        self.dispatch(TerminalDomainEvent::Exited(DomainEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            aggregate_id: terminal_id.clone(),
            generation: session.owner.workspace_generation,
            occurred_at: chrono::Utc::now().to_rfc3339(),
            payload: TerminalExitedPayload {
                terminal_id,
                last_seq,
                exit_code: outcome.exit_code,
                signal: outcome.signal,
                reason,
            },
        }));
        if let Some(sink) = self
            .workspace_refresh_sink
            .lock()
            .expect("terminal refresh mutex poisoned")
            .as_ref()
            .cloned()
        {
            sink(session.owner.chat_session_id.clone());
        }
    }

    fn dispatch(&self, event: TerminalDomainEvent) {
        if let Some(sink) = self
            .event_sink
            .lock()
            .expect("terminal event mutex poisoned")
            .as_ref()
            .cloned()
        {
            sink(event);
        }
    }

    fn kill_matching(
        &self,
        reason: TerminalExitReason,
        predicate: impl Fn(&TerminalOwner) -> bool,
    ) {
        let sessions = self
            .sessions
            .lock()
            .expect("terminal sessions mutex poisoned")
            .values()
            .filter(|session| predicate(&session.owner))
            .cloned()
            .collect::<Vec<_>>();
        for session in sessions {
            session.terminate(reason);
        }
    }
}

fn read_pty(
    reader: &mut dyn Read,
    sender: SyncSender<Vec<u8>>,
    chunk_bytes: usize,
    session: Arc<TerminalSession>,
    manager: Weak<TerminalManager>,
) {
    let mut buffer = vec![0u8; chunk_bytes];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                if sender.send(buffer[..read].to_vec()).is_err() {
                    break;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => {
                if let Some(manager) = manager.upgrade() {
                    manager.request_reader_failure(&session);
                }
                break;
            }
        }
    }
}

fn emit_output_chunk(
    manager: &Weak<TerminalManager>,
    session: &TerminalSession,
    bytes: &[u8],
    output_rate: &mut FixedWindow,
) -> bool {
    let Some(manager) = manager.upgrade() else {
        return false;
    };
    if !output_rate.allow(
        bytes.len() as u64,
        manager.limits.max_output_bytes_per_second as u64,
        Duration::from_secs(1),
    ) {
        manager.request_output_limit(session);
        return false;
    }
    manager.emit_output(session, bytes);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_window_rejects_over_limit_without_unbounded_growth() {
        let mut window = FixedWindow::default();
        assert!(window.allow(8, 10, Duration::from_secs(1)));
        assert!(!window.allow(3, 10, Duration::from_secs(1)));
        assert_eq!(window.value, 8);
    }

    #[test]
    fn dimensions_and_input_limits_are_bounded() {
        let limits = TerminalLimits::default();
        assert!(limits.validate_size(24, 80).is_ok());
        assert_eq!(
            limits.validate_size(1, 80),
            Err(TerminalServiceError::InvalidRequest("dimensions"))
        );
        assert!(limits.max_input_bytes < limits.max_input_bytes_per_second);
        assert_eq!(
            limits.output_chunk_bytes * limits.output_channel_chunks,
            512 * 1024
        );
    }
}
