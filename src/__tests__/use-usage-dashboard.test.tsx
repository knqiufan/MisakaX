import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getDashboard: vi.fn(),
  listener: undefined as ((event: { payload: { profile_id: string } }) => void) | undefined,
  unlisten: vi.fn(),
}));

vi.mock("@/lib/ipc/usage", () => ({
  USAGE_RECORDED_EVENT: "usage:recorded",
  usageIpc: { getDashboard: mocks.getDashboard },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((_name: string, listener: typeof mocks.listener) => {
    mocks.listener = listener;
    return Promise.resolve(mocks.unlisten);
  }),
}));

import { useUsageDashboard } from "@/features/usage-analytics/useUsageDashboard";
import type { UsageDashboardV1 } from "@/lib/ipc/types";

const DASHBOARD = {
  schema_version: 1,
  profile_id: "local",
  overview: {},
  daily_activity: [],
  model_series: [],
  other_series: null,
} as unknown as UsageDashboardV1;

describe("useUsageDashboard", () => {
  beforeEach(() => {
    vi.useRealTimers();
    mocks.getDashboard.mockReset().mockResolvedValue(DASHBOARD);
    mocks.listener = undefined;
    mocks.unlisten.mockReset();
  });

  it("loads one snapshot and debounces matching usage events", async () => {
    const { result, unmount } = renderHook(() => useUsageDashboard());
    await waitFor(() => expect(result.current.status).toBe("success"));
    expect(mocks.getDashboard).toHaveBeenCalledTimes(1);
    await waitFor(() => expect(mocks.listener).toBeTypeOf("function"));

    vi.useFakeTimers();
    act(() => {
      mocks.listener?.({ payload: { profile_id: "other" } });
      mocks.listener?.({ payload: { profile_id: "local" } });
      mocks.listener?.({ payload: { profile_id: "local" } });
      vi.advanceTimersByTime(349);
    });
    expect(mocks.getDashboard).toHaveBeenCalledTimes(1);
    await act(async () => {
      vi.advanceTimersByTime(1);
      await Promise.resolve();
    });
    expect(mocks.getDashboard).toHaveBeenCalledTimes(2);

    unmount();
    expect(mocks.unlisten).toHaveBeenCalledOnce();
  });
});
