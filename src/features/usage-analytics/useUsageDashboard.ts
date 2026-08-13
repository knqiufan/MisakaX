import { useCallback, useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import {
  USAGE_RECORDED_EVENT,
  usageIpc,
  type UsageRecordedEvent,
} from "@/lib/ipc/usage";
import type { UsageDashboardV1 } from "@/lib/ipc/types";

type DashboardStatus = "loading" | "success" | "error";

export interface UsageDashboardState {
  dashboard: UsageDashboardV1 | null;
  status: DashboardStatus;
  error: string | null;
  refresh: () => Promise<void>;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function useUsageDashboard(): UsageDashboardState {
  const [dashboard, setDashboard] = useState<UsageDashboardV1 | null>(null);
  const [status, setStatus] = useState<DashboardStatus>("loading");
  const [error, setError] = useState<string | null>(null);
  const requestSequence = useRef(0);
  const dashboardRef = useRef<UsageDashboardV1 | null>(null);

  useEffect(() => {
    dashboardRef.current = dashboard;
  }, [dashboard]);

  const refresh = useCallback(async () => {
    const requestId = ++requestSequence.current;
    if (!dashboardRef.current) setStatus("loading");
    setError(null);
    try {
      const nextDashboard = await usageIpc.getDashboard();
      if (requestId !== requestSequence.current) return;
      dashboardRef.current = nextDashboard;
      setDashboard(nextDashboard);
      setStatus("success");
    } catch (refreshError) {
      if (requestId !== requestSequence.current) return;
      setStatus("error");
      setError(errorMessage(refreshError));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    let disposed = false;
    let unlisten: UnlistenFn | undefined;
    let refreshTimer: ReturnType<typeof setTimeout> | undefined;

    void listen<UsageRecordedEvent>(USAGE_RECORDED_EVENT, (event) => {
      const currentProfileId = dashboardRef.current?.profile_id;
      if (currentProfileId && event.payload.profile_id !== currentProfileId) return;
      if (refreshTimer) clearTimeout(refreshTimer);
      refreshTimer = setTimeout(() => void refresh(), 350);
    }).then((stopListening) => {
      if (disposed) stopListening();
      else unlisten = stopListening;
    }).catch(() => {
      // Dashboard reads remain usable in browser previews without Tauri events.
    });

    return () => {
      disposed = true;
      if (refreshTimer) clearTimeout(refreshTimer);
      unlisten?.();
    };
  }, [refresh]);

  return { dashboard, status, error, refresh };
}
