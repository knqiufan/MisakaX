import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke, IpcError } from "@/lib/ipc/invoke";

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
