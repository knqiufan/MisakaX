use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::contracts::{AppErrorCode, AppErrorPayload, DomainEvent};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerminalOwner {
    pub window_label: String,
    pub chat_session_id: String,
    pub workspace_generation: u64,
}

#[derive(Debug, Clone)]
pub struct TerminalSpawnRequest {
    pub owner: TerminalOwner,
    pub cwd: PathBuf,
    pub rows: u16,
    pub cols: u16,
    pub shell_profile: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct TerminalLimits {
    pub max_global_sessions: usize,
    pub max_sessions_per_window: usize,
    pub max_sessions_per_chat: usize,
    pub min_rows: u16,
    pub max_rows: u16,
    pub min_cols: u16,
    pub max_cols: u16,
    pub max_input_bytes: usize,
    pub max_input_bytes_per_second: usize,
    pub max_writes_per_second: u32,
    pub max_resizes_per_second: u32,
    pub max_commands_per_second: u32,
    pub max_spawns_per_minute: u32,
    pub max_output_bytes_per_second: usize,
    pub output_chunk_bytes: usize,
    pub output_batch_bytes: usize,
    pub output_channel_chunks: usize,
    pub output_batch_delay: Duration,
}

impl Default for TerminalLimits {
    fn default() -> Self {
        Self {
            max_global_sessions: 8,
            max_sessions_per_window: 4,
            max_sessions_per_chat: 1,
            min_rows: 2,
            max_rows: 512,
            min_cols: 2,
            max_cols: 512,
            max_input_bytes: 64 * 1024,
            max_input_bytes_per_second: 512 * 1024,
            max_writes_per_second: 200,
            max_resizes_per_second: 30,
            max_commands_per_second: 300,
            max_spawns_per_minute: 6,
            max_output_bytes_per_second: 8 * 1024 * 1024,
            output_chunk_bytes: 8 * 1024,
            output_batch_bytes: 32 * 1024,
            output_channel_chunks: 64,
            output_batch_delay: Duration::from_millis(16),
        }
    }
}

impl TerminalLimits {
    pub fn validate_size(&self, rows: u16, cols: u16) -> Result<(), TerminalServiceError> {
        if !(self.min_rows..=self.max_rows).contains(&rows)
            || !(self.min_cols..=self.max_cols).contains(&cols)
        {
            return Err(TerminalServiceError::InvalidRequest("dimensions"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalStatus {
    Running,
    Exited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellFallbackReason {
    RequestedUnavailable,
    UserShellInvalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalState {
    pub terminal_id: String,
    pub chat_session_id: String,
    pub workspace_generation: u64,
    pub shell_profile: String,
    pub shell_name: String,
    pub fallback_reason: Option<ShellFallbackReason>,
    pub rows: u16,
    pub cols: u16,
    pub status: TerminalStatus,
    pub started_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalKillReason {
    User,
    PanelClosed,
    WorkspaceChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalExitReason {
    ProcessExited,
    User,
    PanelClosed,
    WorkspaceChanged,
    SessionDeleted,
    WindowClosed,
    AppExit,
    OutputLimit,
    ReaderError,
}

impl From<TerminalKillReason> for TerminalExitReason {
    fn from(value: TerminalKillReason) -> Self {
        match value {
            TerminalKillReason::User => Self::User,
            TerminalKillReason::PanelClosed => Self::PanelClosed,
            TerminalKillReason::WorkspaceChanged => Self::WorkspaceChanged,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalOutputPayload {
    pub terminal_id: String,
    pub seq: u64,
    pub data_base64: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalExitedPayload {
    pub terminal_id: String,
    pub last_seq: u64,
    pub exit_code: Option<u32>,
    pub signal: Option<String>,
    pub reason: TerminalExitReason,
}

#[derive(Debug, Clone)]
pub enum TerminalDomainEvent {
    Output(DomainEvent<TerminalOutputPayload>),
    Exited(DomainEvent<TerminalExitedPayload>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalServiceError {
    NotFound,
    OwnershipMismatch,
    InvalidRequest(&'static str),
    LimitExceeded(&'static str),
    SpawnFailed(&'static str),
    Internal(&'static str),
}

impl TerminalServiceError {
    pub fn public_payload(self) -> AppErrorPayload {
        let (code, key, retryable, field) = match self {
            Self::NotFound => (
                AppErrorCode::TerminalSessionNotFound,
                "terminal.sessionNotFound",
                false,
                None,
            ),
            Self::OwnershipMismatch => (
                AppErrorCode::TerminalSessionOwnershipMismatch,
                "terminal.ownershipMismatch",
                false,
                None,
            ),
            Self::InvalidRequest(field) => (
                AppErrorCode::TerminalInvalidRequest,
                "terminal.invalidRequest",
                false,
                Some(field),
            ),
            Self::LimitExceeded(field) => (
                AppErrorCode::TerminalLimitExceeded,
                "terminal.limitExceeded",
                true,
                Some(field),
            ),
            Self::SpawnFailed(field) => (
                AppErrorCode::TerminalSpawnFailed,
                "terminal.spawnFailed",
                true,
                Some(field),
            ),
            Self::Internal(field) => (
                AppErrorCode::InternalError,
                "errors.internal",
                false,
                Some(field),
            ),
        };
        let mut params = BTreeMap::new();
        if let Some(field) = field {
            params.insert("field".to_string(), field.to_string());
        }
        AppErrorPayload {
            code,
            message_key: key.to_string(),
            params,
            retryable,
            correlation_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}
