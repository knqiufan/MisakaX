import { describe, it, expect, vi } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useIpc } from "@/hooks/use-ipc";
import { IpcError } from "@/lib/ipc/invoke";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("useIpc", () => {
  it("should have correct initial state", () => {
    const mockFn = vi.fn().mockResolvedValue("result");
    const { result } = renderHook(() => useIpc(mockFn));

    expect(result.current.data).toBeNull();
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it("should set loading during execution", async () => {
    let resolvePromise: (value: string) => void;
    const mockFn = vi.fn(
      () => new Promise<string>((resolve) => { resolvePromise = resolve; })
    );

    const { result } = renderHook(() => useIpc(mockFn));

    let executePromise: Promise<string>;
    act(() => {
      executePromise = result.current.execute();
    });

    expect(result.current.loading).toBe(true);

    await act(async () => {
      resolvePromise!("done");
      await executePromise;
    });

    expect(result.current.loading).toBe(false);
  });

  it("should return data on success", async () => {
    const mockFn = vi.fn().mockResolvedValue({ id: 1, name: "test" });
    const { result } = renderHook(() => useIpc(mockFn));

    await act(async () => {
      await result.current.execute();
    });

    expect(result.current.data).toEqual({ id: 1, name: "test" });
    expect(result.current.error).toBeNull();
    expect(result.current.loading).toBe(false);
  });

  it("should pass arguments to the function", async () => {
    const mockFn = vi.fn().mockResolvedValue("ok");
    const { result } = renderHook(() => useIpc(mockFn));

    await act(async () => {
      await result.current.execute("arg1", 42);
    });

    expect(mockFn).toHaveBeenCalledWith("arg1", 42);
  });

  it("should set error on IpcError", async () => {
    const ipcErr = new IpcError("test_cmd", "oops");
    const mockFn = vi.fn().mockRejectedValue(ipcErr);
    const { result } = renderHook(() => useIpc(mockFn));

    await act(async () => {
      try {
        await result.current.execute();
      } catch {
        // expected
      }
    });

    expect(result.current.error).toBeInstanceOf(IpcError);
    expect(result.current.error?.command).toBe("test_cmd");
    expect(result.current.loading).toBe(false);
  });

  it("should wrap non-IpcError into IpcError", async () => {
    const mockFn = vi.fn().mockRejectedValue(new Error("network failure"));
    const { result } = renderHook(() => useIpc(mockFn));

    await act(async () => {
      try {
        await result.current.execute();
      } catch {
        // expected
      }
    });

    expect(result.current.error).toBeInstanceOf(IpcError);
    expect(result.current.error?.command).toBe("unknown");
    expect(result.current.error?.originalError).toContain("network failure");
  });

  it("should reset state", async () => {
    const mockFn = vi.fn().mockResolvedValue("data");
    const { result } = renderHook(() => useIpc(mockFn));

    await act(async () => {
      await result.current.execute();
    });

    expect(result.current.data).toBe("data");

    act(() => {
      result.current.reset();
    });

    expect(result.current.data).toBeNull();
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it("should clear previous error on new execution", async () => {
    const mockFn = vi.fn()
      .mockRejectedValueOnce(new IpcError("cmd", "fail"))
      .mockResolvedValueOnce("success");

    const { result } = renderHook(() => useIpc(mockFn));

    await act(async () => {
      try { await result.current.execute(); } catch { /* expected */ }
    });

    expect(result.current.error).not.toBeNull();

    await act(async () => {
      await result.current.execute();
    });

    expect(result.current.error).toBeNull();
    expect(result.current.data).toBe("success");
  });
});
