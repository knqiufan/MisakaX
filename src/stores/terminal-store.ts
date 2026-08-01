import { create } from "zustand";
import { IpcError } from "@/lib/ipc/invoke";
import {
  terminalIpc,
  type TerminalExitedEvent,
  type TerminalExitedPayload,
  type TerminalKillReason,
  type TerminalOutputEvent,
  type TerminalState,
} from "@/lib/ipc/terminal";

export type TerminalUiStatus =
  | "idle"
  | "starting"
  | "running"
  | "exited"
  | "error";

export interface TerminalTarget {
  chatSessionId: string;
  workspaceGeneration: number;
  workspaceLabel: string;
}

export interface TerminalRuntimeError {
  code: string | null;
  correlationId: string | null;
  retryable: boolean;
}

interface TerminalRuntimeState {
  status: TerminalUiStatus;
  session: TerminalState | null;
  workspaceLabel: string | null;
  lastSeq: number;
  exit: TerminalExitedPayload | null;
  pendingExit: TerminalExitedPayload | null;
  error: TerminalRuntimeError | null;
  ensureStarted: (
    target: TerminalTarget,
    rows: number,
    cols: number,
  ) => Promise<TerminalState | null>;
  restart: (
    target: TerminalTarget,
    rows: number,
    cols: number,
  ) => Promise<TerminalState | null>;
  stop: (reason: TerminalKillReason) => Promise<void>;
  writeBytes: (bytes: Uint8Array) => Promise<void>;
  resize: (rows: number, cols: number) => Promise<void>;
  acceptOutput: (event: TerminalOutputEvent) => boolean;
  acceptExit: (event: TerminalExitedEvent) => boolean;
  clearError: () => void;
}

const INITIAL_RUNTIME = {
  status: "idle" as const,
  session: null,
  workspaceLabel: null,
  lastSeq: 0,
  exit: null,
  pendingExit: null,
  error: null,
};

let runtimeEpoch = 0;
let startPromise: Promise<TerminalState | null> | null = null;
let startKey: string | null = null;
let attachmentCount = 0;
let deferredDetach: ReturnType<typeof setTimeout> | null = null;

export const useTerminalStore = create<TerminalRuntimeState>((set, get) => ({
  ...INITIAL_RUNTIME,

  ensureStarted: async (target, rows, cols) => {
    if (target.workspaceGeneration <= 0) return null;

    const key = targetKey(target);
    const current = get();
    if (current.session) {
      return terminalMatchesTarget(current.session, target)
        ? current.session
        : null;
    }
    if (startPromise) {
      return startKey === key ? startPromise : null;
    }

    const attemptEpoch = runtimeEpoch;
    set({
      status: "starting",
      workspaceLabel: target.workspaceLabel,
      exit: null,
      pendingExit: null,
      error: null,
      lastSeq: 0,
    });

    startKey = key;
    startPromise = terminalIpc
      .spawn({
        chatSessionId: target.chatSessionId,
        workspaceGeneration: target.workspaceGeneration,
        rows,
        cols,
        shellProfile: "auto",
      })
      .then(async (session) => {
        if (attemptEpoch !== runtimeEpoch) {
          await terminalIpc
            .kill(ownershipOf(session), "panel_closed")
            .catch(() => undefined);
          return null;
        }
        set({
          status: "running",
          session,
          workspaceLabel: target.workspaceLabel,
          lastSeq: 0,
          exit: null,
          pendingExit: null,
          error: null,
        });
        return session;
      })
      .catch((error: unknown) => {
        if (attemptEpoch === runtimeEpoch) {
          set({ status: "error", error: runtimeError(error) });
        }
        return null;
      })
      .finally(() => {
        startPromise = null;
        startKey = null;
      });

    return startPromise;
  },

  restart: async (target, rows, cols) => {
    await get().stop("workspace_changed");
    return get().ensureStarted(target, rows, cols);
  },

  stop: async (reason) => {
    runtimeEpoch += 1;
    const pending = startPromise;
    if (pending) await pending.catch(() => null);

    const session = get().session;
    if (session && get().status === "running") {
      await terminalIpc.kill(ownershipOf(session), reason).catch(() => undefined);
    }
    set(INITIAL_RUNTIME);
  },

  writeBytes: async (bytes) => {
    const session = get().session;
    if (!session || get().status !== "running" || bytes.length === 0) return;
    try {
      await terminalIpc.writeBytes(ownershipOf(session), bytes);
    } catch (error) {
      set({ error: runtimeError(error) });
    }
  },

  resize: async (rows, cols) => {
    const session = get().session;
    if (!session || get().status !== "running") return;
    if (session.rows === rows && session.cols === cols) return;
    try {
      await terminalIpc.resize(ownershipOf(session), rows, cols);
      set({ session: { ...session, rows, cols } });
    } catch (error) {
      set({ error: runtimeError(error) });
    }
  },

  acceptOutput: (event) => {
    const state = get();
    const session = state.session;
    if (!session || !eventMatchesSession(event, session)) return false;
    if (state.status === "exited" && state.pendingExit === null) return false;
    if (event.payload.seq <= state.lastSeq) return false;
    if (state.pendingExit && event.payload.seq > state.pendingExit.last_seq) {
      return false;
    }

    const pendingExit = state.pendingExit;
    const completesExit =
      pendingExit !== null && event.payload.seq >= pendingExit.last_seq;
    set({
      lastSeq: event.payload.seq,
      ...(completesExit
        ? {
            status: "exited" as const,
            session: { ...session, status: "exited" as const },
            exit: pendingExit,
            pendingExit: null,
          }
        : null),
    });
    return true;
  },

  acceptExit: (event) => {
    const state = get();
    const session = state.session;
    if (!session || !eventMatchesSession(event, session)) return false;
    if (state.status === "exited") return false;
    if (event.payload.last_seq > state.lastSeq) {
      set({ pendingExit: event.payload });
      return true;
    }
    set({
      status: "exited",
      session: { ...session, status: "exited" },
      exit: event.payload,
      pendingExit: null,
    });
    return true;
  },

  clearError: () => set({ error: null }),
}));

export function terminalMatchesTarget(
  session: TerminalState,
  target: Pick<TerminalTarget, "chatSessionId" | "workspaceGeneration">,
): boolean {
  return (
    session.chat_session_id === target.chatSessionId &&
    session.workspace_generation === target.workspaceGeneration
  );
}

/**
 * Keeps the owner-bound process alive across React StrictMode's probe unmount,
 * while still turning a real panel close into an explicit backend kill.
 */
export function attachTerminalRuntime(): void {
  attachmentCount += 1;
  if (deferredDetach) {
    clearTimeout(deferredDetach);
    deferredDetach = null;
  }
}

export function detachTerminalRuntime(): void {
  attachmentCount = Math.max(0, attachmentCount - 1);
  if (attachmentCount !== 0 || deferredDetach) return;
  deferredDetach = setTimeout(() => {
    deferredDetach = null;
    if (attachmentCount === 0) {
      void useTerminalStore.getState().stop("panel_closed");
    }
  }, 50);
}

export function resetTerminalRuntimeForTests(): void {
  if (deferredDetach) clearTimeout(deferredDetach);
  deferredDetach = null;
  attachmentCount = 0;
  runtimeEpoch += 1;
  startPromise = null;
  startKey = null;
  useTerminalStore.setState(INITIAL_RUNTIME);
}

function targetKey(target: TerminalTarget): string {
  return `${target.chatSessionId}\u0000${target.workspaceGeneration}`;
}

function ownershipOf(session: TerminalState) {
  return {
    terminalId: session.terminal_id,
    chatSessionId: session.chat_session_id,
    workspaceGeneration: session.workspace_generation,
  };
}

function eventMatchesSession(
  event: TerminalOutputEvent | TerminalExitedEvent,
  session: TerminalState,
): boolean {
  return (
    event.aggregateId === session.terminal_id &&
    event.payload.terminal_id === session.terminal_id &&
    event.generation === session.workspace_generation
  );
}

function runtimeError(error: unknown): TerminalRuntimeError {
  if (error instanceof IpcError) {
    return {
      code: error.code,
      correlationId:
        typeof error.payload?.correlation_id === "string"
          ? error.payload.correlation_id
          : null,
      retryable:
        typeof error.payload?.retryable === "boolean"
          ? error.payload.retryable
          : true,
    };
  }
  return { code: null, correlationId: null, retryable: true };
}
