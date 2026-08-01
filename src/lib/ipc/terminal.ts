import { invoke } from "./invoke";
import type { DomainEvent, TerminalSessionId } from "./contracts";

export const TERMINAL_OUTPUT_EVENT = "terminal.output";
export const TERMINAL_EXITED_EVENT = "terminal.exited";

export type ShellFallbackReason =
  | "requested_unavailable"
  | "user_shell_invalid";

export type TerminalStatus = "running" | "exited";

export interface TerminalState {
  terminal_id: TerminalSessionId;
  chat_session_id: string;
  workspace_generation: number;
  shell_profile: string;
  shell_name: string;
  fallback_reason: ShellFallbackReason | null;
  rows: number;
  cols: number;
  status: TerminalStatus;
  started_at: string;
}

export type TerminalKillReason = "user" | "panel_closed" | "workspace_changed";

export type TerminalExitReason =
  | "process_exited"
  | "user"
  | "panel_closed"
  | "workspace_changed"
  | "session_deleted"
  | "window_closed"
  | "app_exit"
  | "output_limit"
  | "reader_error";

export interface TerminalOutputPayload {
  terminal_id: TerminalSessionId;
  seq: number;
  data_base64: string;
}

export interface TerminalExitedPayload {
  terminal_id: TerminalSessionId;
  last_seq: number;
  exit_code: number | null;
  signal: string | null;
  reason: TerminalExitReason;
}

interface TerminalOwnership {
  terminalId: TerminalSessionId;
  chatSessionId: string;
  workspaceGeneration: number;
}

export const terminalIpc = {
  spawn: (request: {
    chatSessionId: string;
    workspaceGeneration: number;
    rows: number;
    cols: number;
    shellProfile?: string;
  }) => invoke<TerminalState>("terminal_spawn", request),

  writeBytes: (ownership: TerminalOwnership, bytes: Uint8Array) =>
    invoke<void>("terminal_write", {
      ...ownership,
      dataBase64: bytesToBase64(bytes),
    }),

  resize: (ownership: TerminalOwnership, rows: number, cols: number) =>
    invoke<void>("terminal_resize", { ...ownership, rows, cols }),

  kill: (ownership: TerminalOwnership, reason: TerminalKillReason) =>
    invoke<void>("terminal_kill", { ...ownership, reason }),

  getState: (ownership: TerminalOwnership) =>
    invoke<TerminalState>("terminal_get_state", { ...ownership }),
};

export type TerminalOutputEvent = DomainEvent<TerminalOutputPayload>;
export type TerminalExitedEvent = DomainEvent<TerminalExitedPayload>;

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunkSize = 0x8000;
  for (let offset = 0; offset < bytes.length; offset += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(offset, offset + chunkSize));
  }
  return btoa(binary);
}
