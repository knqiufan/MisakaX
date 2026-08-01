import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke, IpcError, sanitizeForLog } from "@/lib/ipc/invoke";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke as tauriInvoke } from "@tauri-apps/api/core";

const mockTauriInvoke = vi.mocked(tauriInvoke);

describe("invoke wrapper", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should call tauri invoke with correct args", async () => {
    mockTauriInvoke.mockResolvedValue({ key: "value" });

    const result = await invoke<{ key: string }>("test_command", { foo: "bar" });

    expect(mockTauriInvoke).toHaveBeenCalledWith("test_command", { foo: "bar" });
    expect(result).toEqual({ key: "value" });
  });

  it("should call tauri invoke without args", async () => {
    mockTauriInvoke.mockResolvedValue("ok");

    const result = await invoke<string>("simple_command");

    expect(mockTauriInvoke).toHaveBeenCalledWith("simple_command", undefined);
    expect(result).toBe("ok");
  });

  it("should throw IpcError on failure with string error", async () => {
    mockTauriInvoke.mockRejectedValue("Something went wrong");

    await expect(invoke("failing_command")).rejects.toThrow(IpcError);
    await expect(invoke("failing_command")).rejects.toMatchObject({
      command: "failing_command",
      originalError: "Something went wrong",
    });
  });

  it("should throw IpcError on failure with Error object", async () => {
    mockTauriInvoke.mockRejectedValue(new Error("Connection refused"));

    await expect(invoke("another_command")).rejects.toThrow(IpcError);
    await expect(invoke("another_command")).rejects.toMatchObject({
      command: "another_command",
      originalError: expect.stringContaining("Connection refused"),
    });
  });

  it("redacts sensitive values from IPC debug payloads", () => {
    expect(
      sanitizeForLog({
        api_key: "sk-secret",
        nested: { apiKey: "nested-secret" },
        headers: { Authorization: "Bearer secret", "x-api-key": "x-secret" },
        safe: "value",
      }),
    ).toEqual({
      api_key: "<redacted>",
      nested: { apiKey: "<redacted>" },
      headers: {
        Authorization: "<redacted>",
        "x-api-key": "<redacted>",
      },
      safe: "value",
    });
  });

  it("redacts revealed api keys from IPC debug results", async () => {
    const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
    mockTauriInvoke.mockResolvedValue("sk-revealed");

    await invoke<string>("reveal_router_api_key", { id: "router-1" });

    expect(logSpy).toHaveBeenLastCalledWith(
      "[IPC] ← reveal_router_api_key",
      "<redacted>",
    );
    expect(logSpy).not.toHaveBeenCalledWith(
      expect.any(String),
      expect.stringContaining("sk-revealed"),
    );

    logSpy.mockRestore();
  });

  it("never logs terminal input bytes", async () => {
    const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
    mockTauriInvoke.mockResolvedValue(undefined);

    await invoke<void>("terminal_write", {
      terminalId: "terminal-1",
      dataBase64: "c2VjcmV0IGNvbW1hbmQ=",
    });

    expect(logSpy.mock.calls[0]?.[1]).toMatchObject({
      terminalId: "terminal-1",
      dataBase64: "<redacted>",
    });
    expect(JSON.stringify(logSpy.mock.calls)).not.toContain("c2VjcmV0IGNvbW1hbmQ=");
    logSpy.mockRestore();
  });
});

describe("IpcError", () => {
  it("should have correct name and properties", () => {
    const error = new IpcError("test_cmd", "oops");
    expect(error.name).toBe("IpcError");
    expect(error.command).toBe("test_cmd");
    expect(error.originalError).toBe("oops");
    expect(error.message).toBe("IPC Error [test_cmd]: oops");
  });

  it("should be an instance of Error", () => {
    const error = new IpcError("cmd", "msg");
    expect(error).toBeInstanceOf(Error);
    expect(error).toBeInstanceOf(IpcError);
  });
});
