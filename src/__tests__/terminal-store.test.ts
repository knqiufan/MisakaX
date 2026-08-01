import { beforeEach, describe, expect, it, vi } from "vitest";

const terminalMocks = vi.hoisted(() => ({
  spawn: vi.fn(),
  writeBytes: vi.fn(),
  resize: vi.fn(),
  kill: vi.fn(),
  getState: vi.fn(),
}));

vi.mock("@/lib/ipc/terminal", () => ({ terminalIpc: terminalMocks }));

import {
  attachTerminalRuntime,
  detachTerminalRuntime,
  resetTerminalRuntimeForTests,
  useTerminalStore,
} from "@/stores/terminal-store";
import type {
  TerminalExitedEvent,
  TerminalOutputEvent,
  TerminalState,
} from "@/lib/ipc/terminal";

const SESSION: TerminalState = {
  terminal_id: "terminal-1",
  chat_session_id: "chat-1",
  workspace_generation: 4,
  shell_profile: "powershell",
  shell_name: "PowerShell",
  fallback_reason: null,
  rows: 24,
  cols: 80,
  status: "running",
  started_at: "2026-08-01T00:00:00Z",
};

const TARGET = {
  chatSessionId: "chat-1",
  workspaceGeneration: 4,
  workspaceLabel: "Misaka-Tauri",
};

describe("Workspace W4 terminal runtime store", () => {
  beforeEach(() => {
    vi.useRealTimers();
    resetTerminalRuntimeForTests();
    Object.values(terminalMocks).forEach((mock) => mock.mockReset());
    terminalMocks.kill.mockResolvedValue(undefined);
    terminalMocks.resize.mockResolvedValue(undefined);
    terminalMocks.writeBytes.mockResolvedValue(undefined);
  });

  it("deduplicates concurrent starts for the same owner", async () => {
    let resolveSpawn: (session: TerminalState) => void = () => undefined;
    terminalMocks.spawn.mockReturnValue(
      new Promise<TerminalState>((resolve) => {
        resolveSpawn = resolve;
      }),
    );

    const first = useTerminalStore.getState().ensureStarted(TARGET, 24, 80);
    const second = useTerminalStore.getState().ensureStarted(TARGET, 30, 100);
    expect(terminalMocks.spawn).toHaveBeenCalledTimes(1);
    resolveSpawn(SESSION);

    await expect(first).resolves.toEqual(SESSION);
    await expect(second).resolves.toEqual(SESSION);
    expect(useTerminalStore.getState()).toMatchObject({
      status: "running",
      session: SESSION,
      workspaceLabel: "Misaka-Tauri",
    });
  });

  it("drops stale output and finalizes exit only after last_seq is observed", async () => {
    terminalMocks.spawn.mockResolvedValue(SESSION);
    await useTerminalStore.getState().ensureStarted(TARGET, 24, 80);

    expect(useTerminalStore.getState().acceptOutput(outputEvent(1, 3))).toBe(false);
    expect(useTerminalStore.getState().acceptOutput(outputEvent(2, 4))).toBe(true);
    expect(useTerminalStore.getState().acceptOutput(outputEvent(2, 4))).toBe(false);
    expect(useTerminalStore.getState().lastSeq).toBe(2);

    expect(useTerminalStore.getState().acceptExit(exitEvent(3))).toBe(true);
    expect(useTerminalStore.getState().status).toBe("running");
    expect(useTerminalStore.getState().acceptOutput(outputEvent(3, 4))).toBe(true);
    expect(useTerminalStore.getState()).toMatchObject({
      status: "exited",
      lastSeq: 3,
      exit: expect.objectContaining({ last_seq: 3, exit_code: 0 }),
    });
    expect(useTerminalStore.getState().acceptOutput(outputEvent(4, 4))).toBe(false);
  });

  it("restarts explicitly under the new workspace owner", async () => {
    const nextSession = {
      ...SESSION,
      terminal_id: "terminal-2",
      chat_session_id: "chat-2",
      workspace_generation: 8,
    };
    terminalMocks.spawn
      .mockResolvedValueOnce(SESSION)
      .mockResolvedValueOnce(nextSession);
    await useTerminalStore.getState().ensureStarted(TARGET, 24, 80);

    await useTerminalStore.getState().restart(
      {
        chatSessionId: "chat-2",
        workspaceGeneration: 8,
        workspaceLabel: "new-project",
      },
      32,
      120,
    );

    expect(terminalMocks.kill).toHaveBeenCalledWith(
      {
        terminalId: "terminal-1",
        chatSessionId: "chat-1",
        workspaceGeneration: 4,
      },
      "workspace_changed",
    );
    expect(terminalMocks.spawn).toHaveBeenLastCalledWith({
      chatSessionId: "chat-2",
      workspaceGeneration: 8,
      rows: 32,
      cols: 120,
      shellProfile: "auto",
    });
    expect(useTerminalStore.getState().session).toEqual(nextSession);
  });

  it("survives a StrictMode probe detach but kills after a real detach", async () => {
    vi.useFakeTimers();
    terminalMocks.spawn.mockResolvedValue(SESSION);
    await useTerminalStore.getState().ensureStarted(TARGET, 24, 80);

    attachTerminalRuntime();
    detachTerminalRuntime();
    await vi.advanceTimersByTimeAsync(30);
    attachTerminalRuntime();
    await vi.advanceTimersByTimeAsync(30);
    expect(terminalMocks.kill).not.toHaveBeenCalled();

    detachTerminalRuntime();
    await vi.advanceTimersByTimeAsync(60);
    expect(terminalMocks.kill).toHaveBeenCalledWith(
      expect.objectContaining({ terminalId: "terminal-1" }),
      "panel_closed",
    );
    expect(useTerminalStore.getState().status).toBe("idle");
  });
});

function outputEvent(seq: number, generation: number): TerminalOutputEvent {
  return {
    eventId: `output-${seq}`,
    aggregateId: "terminal-1",
    generation,
    occurredAt: "2026-08-01T00:00:00Z",
    payload: {
      terminal_id: "terminal-1",
      seq,
      data_base64: "b2s=",
    },
  };
}

function exitEvent(lastSeq: number): TerminalExitedEvent {
  return {
    eventId: "exit-1",
    aggregateId: "terminal-1",
    generation: 4,
    occurredAt: "2026-08-01T00:00:01Z",
    payload: {
      terminal_id: "terminal-1",
      last_seq: lastSeq,
      exit_code: 0,
      signal: null,
      reason: "process_exited",
    },
  };
}
