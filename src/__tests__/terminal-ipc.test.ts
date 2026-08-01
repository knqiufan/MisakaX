import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { terminalIpc } from "@/lib/ipc/terminal";

const mockInvoke = vi.mocked(tauriInvoke);

describe("terminal IPC", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue(undefined);
  });

  it("sends only ownership, dimensions, and an allowlisted profile to spawn", async () => {
    await terminalIpc.spawn({
      chatSessionId: "chat-1",
      workspaceGeneration: 4,
      rows: 24,
      cols: 80,
      shellProfile: "auto",
    });

    expect(mockInvoke).toHaveBeenCalledWith("terminal_spawn", {
      chatSessionId: "chat-1",
      workspaceGeneration: 4,
      rows: 24,
      cols: 80,
      shellProfile: "auto",
    });
    expect(mockInvoke.mock.calls[0]?.[1]).not.toHaveProperty("cwd");
    expect(mockInvoke.mock.calls[0]?.[1]).not.toHaveProperty("executable");
    expect(mockInvoke.mock.calls[0]?.[1]).not.toHaveProperty("env");
  });

  it("preserves binary input through base64 without exposing a text command API", async () => {
    await terminalIpc.writeBytes(
      {
        terminalId: "terminal-1",
        chatSessionId: "chat-1",
        workspaceGeneration: 4,
      },
      new Uint8Array([0, 1, 255, 10]),
    );

    expect(mockInvoke).toHaveBeenCalledWith("terminal_write", {
      terminalId: "terminal-1",
      chatSessionId: "chat-1",
      workspaceGeneration: 4,
      dataBase64: "AAH/Cg==",
    });
  });
});
