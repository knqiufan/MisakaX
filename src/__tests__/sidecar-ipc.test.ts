import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { getSidecarStatus, restartSidecar } from "@/lib/ipc/sidecar";

const mockTauriInvoke = vi.mocked(tauriInvoke);

describe("sidecar IPC", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("getSidecarStatus", () => {
    it("should call get_sidecar_status command", async () => {
      mockTauriInvoke.mockResolvedValue("ready");

      const status = await getSidecarStatus();

      expect(mockTauriInvoke).toHaveBeenCalledWith(
        "get_sidecar_status",
        undefined,
      );
      expect(status).toBe("ready");
    });

    it("should return all valid status values", async () => {
      for (const status of [
        "stopped",
        "starting",
        "ready",
        "error",
        "restarting",
      ]) {
        mockTauriInvoke.mockResolvedValue(status);
        const result = await getSidecarStatus();
        expect(result).toBe(status);
      }
    });
  });

  describe("restartSidecar", () => {
    it("should call restart_sidecar command", async () => {
      mockTauriInvoke.mockResolvedValue(undefined);

      await restartSidecar();

      expect(mockTauriInvoke).toHaveBeenCalledWith(
        "restart_sidecar",
        undefined,
      );
    });
  });
});
