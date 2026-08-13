import { invoke } from "./invoke";
import type { UsageDashboardV1 } from "./types";

export interface UsageDashboardParams {
  activityDays?: number;
  trendDays?: number;
  maxSeries?: number;
}

export const USAGE_RECORDED_EVENT = "usage:recorded";

export interface UsageRecordedEvent {
  profile_id: string;
  operation_key: string;
  occurred_at: string;
}

export const usageIpc = {
  getDashboard: (params: UsageDashboardParams = {}) =>
    invoke<UsageDashboardV1>("usage_get_dashboard", {
      activityDays: params.activityDays,
      trendDays: params.trendDays,
      maxSeries: params.maxSeries,
    }),
  clearHistory: () => invoke<number>("usage_clear_history"),
};
