import { describe, expect, it } from "vitest";

import {
  buildUsageChart,
  inclusiveDateSequence,
} from "@/features/usage-analytics/usage-chart-options";
import type { ModelUsagePointV1, ModelUsageSeriesV1 } from "@/lib/ipc/types";

const LABELS = {
  yAxis: "Tokens",
  unknownOperations: "unknown operations",
  estimatedTokens: "estimated",
  legacyTokens: "legacy",
};

function point(
  localDate: string,
  totalTokens: string | null,
  unknownOperationCount = 0,
  estimatedTokens = "0"
): ModelUsagePointV1 {
  return {
    local_date: localDate,
    total_tokens: totalTokens,
    unknown_operation_count: unknownOperationCount,
    estimated_tokens: estimatedTokens,
    legacy_tokens: "0",
  };
}

function series(
  key: string,
  name: string,
  points: ModelUsagePointV1[],
  providerId: string | null = "provider"
): ModelUsageSeriesV1 {
  return {
    series_key: key,
    display_name: name,
    provider_config_id: `${key}-config`,
    provider_id: providerId,
    effective_model_id: name,
    points,
  };
}

describe("usage chart options", () => {
  it("builds 30 continuous categories and distinguishes zero, gap, and mixed points", () => {
    const dates = inclusiveDateSequence("2026-07-15", "2026-08-13");
    expect(dates).toHaveLength(30);
    const chart = buildUsageChart(
      [
        series("a", "Model A", [
          point("2026-07-15", "0"),
          point("2026-07-16", null, 1),
          point("2026-07-17", "1200", 2, "200"),
        ]),
      ],
      null,
      { start: "2026-07-15", end: "2026-08-13" },
      "en",
      LABELS
    );

    expect(chart.option.xAxis.data).toHaveLength(30);
    expect(chart.preparedSeries[0].points[0].value).toBe(0);
    expect(chart.preparedSeries[0].points[1].value).toBeNull();
    expect(chart.preparedSeries[0].points[2]).toMatchObject({
      rawValue: "1200",
      unknownOperationCount: 2,
      symbol: "diamond",
      symbolSize: 9,
    });
    expect(chart.preparedSeries[0].points[3]).toMatchObject({
      value: 0,
      rawValue: "0",
      unknownOperationCount: 0,
    });
    expect(chart.option.yAxis.min).toBe(0);
    expect(chart.option.series[0]).toMatchObject({ smooth: false, connectNulls: false });
    expect(chart.option.animation).toBe(false);
    expect(chart.option.aria).toEqual({ enabled: true, decal: { show: true } });
  });

  it("scales values above Number.MAX_SAFE_INTEGER without losing raw tooltip data", () => {
    const raw = "90071992547409930";
    const chart = buildUsageChart(
      [series("large", "Large", [point("2026-08-13", raw, 1)])],
      null,
      { start: "2026-08-13", end: "2026-08-13" },
      "en",
      LABELS
    );
    const plotted = chart.preparedSeries[0].points[0];
    expect(chart.scale).toBeGreaterThan(1n);
    expect(plotted.value).toBeLessThanOrEqual(Number.MAX_SAFE_INTEGER);
    expect(plotted.rawValue).toBe(raw);
    const tooltip = chart.option.tooltip.formatter({
      axisValue: "2026-08-13",
      seriesName: "Large",
      data: plotted,
    });
    expect(tooltip).toContain("90,071,992,547,409,930");
    expect(tooltip).toContain("1 unknown operations");
    expect(chart.option.yAxis.axisLabel.formatter(plotted.value ?? 0)).toMatch(/[KMB]$/);
  });

  it("keeps stable visual mappings, disambiguates duplicate names, and includes others", () => {
    const models = Array.from({ length: 5 }, (_, index) =>
      series(`key-${index}`, index < 2 ? "Shared" : `Model ${index}`, [point("2026-08-13", String(index))], `p${index}`)
    );
    const other = series("others", "Others", [point("2026-08-13", "10")], null);
    const sourceBeforeRender = structuredClone(models);
    const first = buildUsageChart(
      models,
      other,
      { start: "2026-08-13", end: "2026-08-13" },
      "en",
      LABELS
    );
    const second = buildUsageChart(
      [...models].reverse(),
      other,
      { start: "2026-08-13", end: "2026-08-13" },
      "en",
      LABELS
    );

    expect(first.preparedSeries).toHaveLength(6);
    expect(models).toEqual(sourceBeforeRender);
    expect(first.option.legend.type).toBe("scroll");
    expect(first.preparedSeries[0].label).toContain("p0");
    const firstColor = first.preparedSeries.find((item) => item.seriesKey === "key-0")?.color;
    const secondColor = second.preparedSeries.find((item) => item.seriesKey === "key-0")?.color;
    expect(firstColor).toBe(secondColor);
  });

  it("keeps duplicate name/provider labels unique and formats million/billion axes", () => {
    const firstSeries = series(
      "shared-key-a",
      "Shared",
      [point("2026-08-12", "1000000")],
      "same-provider"
    );
    const secondSeries = series(
      "shared-key-b",
      "Shared",
      [point("2026-08-13", "1000000000")],
      "same-provider"
    );
    firstSeries.provider_config_id = null;
    secondSeries.provider_config_id = null;
    const chart = buildUsageChart(
      [firstSeries, secondSeries],
      null,
      { start: "2026-08-12", end: "2026-08-13" },
      "en",
      LABELS
    );

    expect(chart.preparedSeries[0].label).not.toBe(chart.preparedSeries[1].label);
    expect(chart.option.yAxis.axisLabel.formatter(1_000_000)).toBe("1M");
    expect(chart.option.yAxis.axisLabel.formatter(1_000_000_000)).toBe("1B");
  });

  it("supports one-point and all-zero series", () => {
    const chart = buildUsageChart(
      [series("zero", "Zero", [point("2026-08-13", "0")])],
      null,
      { start: "2026-08-13", end: "2026-08-13" },
      "en",
      LABELS
    );
    expect(chart.dates).toEqual(["2026-08-13"]);
    expect(chart.preparedSeries[0].points[0].value).toBe(0);
  });
});
