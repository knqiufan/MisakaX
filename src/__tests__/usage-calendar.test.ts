import { describe, expect, it } from "vitest";

import {
  buildActivityCalendar,
  calculateUsageHeatLevels,
  parseLocalDate,
} from "@/features/usage-analytics/usage-calendar";
import type { DailyUsageV1 } from "@/lib/ipc/types";

function day(localDate: string, totalTokens: string | null = "0", operations = 0): DailyUsageV1 {
  return {
    local_date: localDate,
    total_tokens: totalTokens,
    input_tokens: totalTokens,
    output_tokens: totalTokens === null ? null : "0",
    operation_count: operations,
    primary_model: operations > 0 ? "model-a" : null,
    quality: {
      exact_tokens: totalTokens ?? "0",
      estimated_tokens: "0",
      legacy_tokens: "0",
      unknown_operation_count: totalTokens === null && operations > 0 ? operations : 0,
    },
  };
}

function dateRange(start: string, count: number): DailyUsageV1[] {
  const first = parseLocalDate(start);
  return Array.from({ length: count }, (_, index) => {
    const date = new Date(first);
    date.setUTCDate(first.getUTCDate() + index);
    return day(date.toISOString().slice(0, 10));
  });
}

describe("activity calendar grid", () => {
  it("builds a 365-day, 7-row, 53-week grid for Sunday and Monday starts", () => {
    const days = dateRange("2025-03-02", 365);
    const sunday = buildActivityCalendar(days, 0, parseLocalDate("2026-03-01"));
    const monday = buildActivityCalendar(days, 1, parseLocalDate("2026-03-01"));

    expect(sunday.days).toHaveLength(365);
    expect(sunday.cells).toHaveLength(sunday.weekCount * 7);
    expect(sunday.weekCount).toBe(53);
    expect(sunday.days[0].rowIndex).toBe(0);
    expect(monday.weekCount).toBe(53);
    expect(monday.days[0].rowIndex).toBe(6);
  });

  it("keeps leap day, year boundaries, month labels, and future dates deterministic", () => {
    const days = dateRange("2023-03-02", 365);
    const grid = buildActivityCalendar(days, 1, parseLocalDate("2024-02-27"));
    expect(grid.days.some((cell) => cell.day.local_date === "2024-02-29")).toBe(true);
    expect(grid.monthLabels.some((label) => label.monthKey === "2024-01")).toBe(true);
    expect(grid.monthLabels.some((label) => label.monthKey === "2024-02")).toBe(true);
    expect(grid.days.find((cell) => cell.day.local_date === "2024-02-29")?.visualState).toBe(
      "future"
    );
  });

  it("distinguishes no activity, known zero, unknown, and mixed usage", () => {
    const none = day("2026-01-01", "0", 0);
    const zero = day("2026-01-02", "0", 1);
    const unknown = day("2026-01-03", null, 1);
    const mixed = day("2026-01-04", "12", 2);
    mixed.quality.unknown_operation_count = 1;
    const grid = buildActivityCalendar(
      [none, zero, unknown, mixed],
      1,
      parseLocalDate("2026-01-04")
    );

    expect(grid.days.map((cell) => cell.visualState)).toEqual([
      "none",
      "known",
      "unknown",
      "mixed",
    ]);
    expect(grid.days.map((cell) => cell.heatLevel)).toEqual([0, 1, 0, 4]);
  });
});

describe("activity heat levels", () => {
  it("uses a P95 cap so a single outlier does not flatten ordinary active days", () => {
    const values = [1, 1, 1, 1, 1, 5, 5, 5, 25, 25, 25, 100, 100, 100, 100, 100, 100, 100, 100, 1_000_000];
    const days = values.map((value, index) =>
      day(`2026-01-${String(index + 1).padStart(2, "0")}`, String(value), 1)
    );
    const levels = calculateUsageHeatLevels(days);

    expect(levels.get("2026-01-01")).toBe(1);
    expect(levels.get("2026-01-09")).toBe(3);
    expect(levels.get("2026-01-12")).toBe(4);
    expect(levels.get("2026-01-20")).toBe(4);
  });

  it("maps all-zero active days to level one and equal positive values consistently", () => {
    const zeroLevels = calculateUsageHeatLevels([
      day("2026-01-01", "0", 1),
      day("2026-01-02", "0", 1),
    ]);
    expect(Array.from(zeroLevels.values())).toEqual([1, 1]);

    const equalLevels = calculateUsageHeatLevels([
      day("2026-01-01", "10", 1),
      day("2026-01-02", "10", 1),
    ]);
    expect(Array.from(equalLevels.values())).toEqual([4, 4]);
  });
});
