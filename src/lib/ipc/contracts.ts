export type AppErrorCode =
  | "SKILL_DISABLED"
  | "SKILL_SCAN_REQUIRED"
  | "SKILL_SCAN_STALE"
  | "SKILL_REVIEW_REQUIRED"
  | "SKILL_POLICY_BLOCKED"
  | "SKILL_APPROVAL_EXPIRED"
  | "SKILL_SCAN_CANCELLED"
  | "SKILL_SCAN_TIMEOUT"
  | "SKILL_QUARANTINE_QUOTA"
  | "SKILL_PATH_INVALID"
  | "FILE_PREVIEW_TOO_LARGE"
  | "WORKSPACE_NOT_FOUND"
  | "GIT_NOT_AVAILABLE"
  | "TERMINAL_SESSION_NOT_FOUND"
  | "TERMINAL_SESSION_OWNERSHIP_MISMATCH"
  | "TERMINAL_INVALID_REQUEST"
  | "TERMINAL_LIMIT_EXCEEDED"
  | "TERMINAL_SPAWN_FAILED"
  | "INTERNAL_ERROR";

export interface AppErrorPayload {
  code: AppErrorCode;
  message_key: string;
  params: Record<string, string>;
  retryable: boolean;
  correlation_id: string;
}

export type SkillId = string;
export type TerminalSessionId = string;

export type ScanState =
  | "unscanned"
  | "queued"
  | "scanning"
  | "passed"
  | "warnings"
  | "review_required"
  | "blocked"
  | "error"
  | "stale"
  | "cancelled";

export type ScanDecision =
  | "unknown"
  | "allow"
  | "allow_with_ack"
  | "review"
  | "block";

export type FindingSeverity = "info" | "low" | "medium" | "high" | "critical";

export type WorkspaceKind = "git" | "local";

export interface WorkspaceContextDiagnostic {
  code: AppErrorCode;
  message_key: string;
  retryable: boolean;
  correlation_id: string;
}

export interface WorkspaceContext {
  workspace_path: string;
  kind: WorkspaceKind;
  repository_root: string | null;
  branch: string | null;
  detached_head: string | null;
  generation: number;
  diagnostic: WorkspaceContextDiagnostic | null;
}

export type WorkspacePanelMode = "explorer" | "terminal";

export interface DomainEvent<T> {
  eventId: string;
  aggregateId: string;
  generation: number;
  occurredAt: string;
  payload: T;
}
