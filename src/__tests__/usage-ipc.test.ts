import { beforeEach, describe, expect, it, vi } from "vitest";

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));

import { usageIpc } from "@/lib/ipc/usage";

describe("usage IPC", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("maps camelCase parameters to Tauri command arguments", async () => {
    mockInvoke.mockResolvedValue({
      schema_version: 1,
      overview: { total_tokens: "9007199254740993" },
    });
    const dashboard = await usageIpc.getDashboard({
      activityDays: 365,
      trendDays: 30,
      maxSeries: 5,
    });
    expect(mockInvoke).toHaveBeenCalledWith("usage_get_dashboard", {
      activityDays: 365,
      trendDays: 30,
      maxSeries: 5,
    });
    expect(dashboard.overview.total_tokens).toBe("9007199254740993");
  });

  it("clears history without path or content arguments", async () => {
    mockInvoke.mockResolvedValue(4);
    await expect(usageIpc.clearHistory()).resolves.toBe(4);
    expect(mockInvoke).toHaveBeenCalledWith("usage_clear_history", undefined);
  });
});
