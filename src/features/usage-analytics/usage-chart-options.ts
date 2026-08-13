import type {
  ModelUsagePointV1,
  ModelUsageSeriesV1,
} from "@/lib/ipc/types";
import { formatCompactTokens, formatFullTokens } from "./usage-format";
import { parseLocalDate } from "./usage-calendar";

const CHART_COLORS = [
  "var(--chart-1)",
  "var(--chart-2)",
  "var(--chart-3)",
  "var(--chart-4)",
  "var(--chart-5)",
] as const;
const LINE_TYPES = ["solid", "dashed", "dotted"] as const;
const SYMBOLS = ["circle", "rect", "triangle", "diamond"] as const;
const MAX_SAFE_BIGINT = BigInt(Number.MAX_SAFE_INTEGER);

export interface UsageChartLabels {
  yAxis: string;
  unknownOperations: string;
  estimatedTokens: string;
  legacyTokens: string;
}

export interface UsageChartDataPoint {
  value: number | null;
  rawValue: string | null;
  localDate: string;
  unknownOperationCount: number;
  estimatedTokens: string;
  legacyTokens: string;
  symbol?: string;
  symbolSize?: number;
  itemStyle?: { borderColor: string; borderWidth: number };
}

export interface PreparedUsageSeries {
  seriesKey: string;
  label: string;
  color: string;
  lineType: (typeof LINE_TYPES)[number];
  symbol: (typeof SYMBOLS)[number];
  source: ModelUsageSeriesV1;
  points: UsageChartDataPoint[];
}

export interface UsageChartOption {
  animation: false;
  aria: { enabled: true; decal: { show: true } };
  color: string[];
  grid: object;
  tooltip: {
    trigger: "axis";
    renderMode: "richText";
    formatter: (params: UsageTooltipParam | UsageTooltipParam[]) => string;
  };
  legend: { type: "scroll"; bottom: number; data: string[] };
  xAxis: {
    type: "category";
    boundaryGap: false;
    data: string[];
    axisLabel: { hideOverlap: true; formatter: (value: string) => string };
  };
  yAxis: {
    type: "value";
    min: 0;
    name: string;
    axisLabel: { formatter: (value: number) => string };
  };
  series: Array<{
    id: string;
    name: string;
    type: "line";
    data: UsageChartDataPoint[];
    smooth: false;
    connectNulls: false;
    showSymbol: true;
    symbol: string;
    symbolSize: number;
    lineStyle: { color: string; type: string; width: number };
    itemStyle: { color: string };
    emphasis: { focus: "series" };
  }>;
}

interface UsageTooltipParam {
  axisValue?: string;
  seriesName?: string;
  data?: UsageChartDataPoint;
}

export interface BuiltUsageChart {
  dates: string[];
  scale: bigint;
  preparedSeries: PreparedUsageSeries[];
  option: UsageChartOption;
}

export function inclusiveDateSequence(start: string, end: string): string[] {
  const startDate = parseLocalDate(start);
  const endDate = parseLocalDate(end);
  if (startDate > endDate) throw new Error("Trend range start must not be after end");
  const dates: string[] = [];
  for (const cursor = new Date(startDate); cursor <= endDate; cursor.setUTCDate(cursor.getUTCDate() + 1)) {
    dates.push(cursor.toISOString().slice(0, 10));
    if (dates.length > 366) throw new Error("Trend range exceeds 366 days");
  }
  return dates;
}

function stableHash(value: string): number {
  let hash = 2_166_136_261;
  for (const character of value) {
    hash ^= character.codePointAt(0) ?? 0;
    hash = Math.imul(hash, 16_777_619);
  }
  return hash >>> 0;
}

export function usageSeriesLabels(series: ModelUsageSeriesV1[]): Map<string, string> {
  const nameCounts = new Map<string, number>();
  for (const item of series) {
    nameCounts.set(item.display_name, (nameCounts.get(item.display_name) ?? 0) + 1);
  }
  const candidates = series.map((item) => {
    const provider = item.provider_id ?? "local";
    const config = item.provider_config_id ? `/${item.provider_config_id.slice(-6)}` : "";
    return {
      item,
      label: (nameCounts.get(item.display_name) ?? 0) === 1
        ? item.display_name
        : `${item.display_name} · ${provider}${config}`,
    };
  });
  const candidateCounts = new Map<string, number>();
  for (const candidate of candidates) {
    candidateCounts.set(
      candidate.label,
      (candidateCounts.get(candidate.label) ?? 0) + 1
    );
  }
  return new Map(
    candidates.map(({ item, label }) => {
      const uniqueLabel = (candidateCounts.get(label) ?? 0) === 1
        ? label
        : `${label} · ${item.series_key}`;
      return [item.series_key, uniqueLabel];
    })
  );
}

function chartScale(series: ModelUsageSeriesV1[]): bigint {
  let maximum = 0n;
  for (const item of series) {
    for (const point of item.points) {
      if (point.total_tokens !== null) {
        const value = BigInt(point.total_tokens);
        if (value > maximum) maximum = value;
      }
    }
  }
  let scale = 1n;
  while (maximum / scale > MAX_SAFE_BIGINT) scale *= 10n;
  return scale;
}

function scaledNumber(value: string, scale: bigint): number {
  const raw = BigInt(value);
  const quotient = raw / scale;
  const remainder = raw % scale;
  return Number(quotient) + Number(remainder) / Number(scale);
}

function pointForDate(
  point: ModelUsagePointV1 | undefined,
  localDate: string,
  scale: bigint
): UsageChartDataPoint {
  if (!point) {
    return {
      value: 0,
      rawValue: "0",
      localDate,
      unknownOperationCount: 0,
      estimatedTokens: "0",
      legacyTokens: "0",
    };
  }
  const mixed = point.total_tokens !== null && point.unknown_operation_count > 0;
  return {
    value: point.total_tokens === null ? null : scaledNumber(point.total_tokens, scale),
    rawValue: point.total_tokens,
    localDate,
    unknownOperationCount: point.unknown_operation_count,
    estimatedTokens: point.estimated_tokens,
    legacyTokens: point.legacy_tokens,
    symbol: mixed ? "diamond" : undefined,
    symbolSize: mixed ? 9 : undefined,
    itemStyle: mixed
      ? { borderColor: "var(--foreground)", borderWidth: 2 }
      : undefined,
  };
}

function axisRawValue(value: number, scale: bigint): bigint {
  const thousandths = BigInt(Math.max(0, Math.round(value * 1000)));
  return (thousandths * scale) / 1000n;
}

export function buildUsageChart(
  modelSeries: ModelUsageSeriesV1[],
  otherSeries: ModelUsageSeriesV1 | null,
  range: { start: string; end: string },
  locale: string,
  labels: UsageChartLabels
): BuiltUsageChart {
  const dates = inclusiveDateSequence(range.start, range.end);
  const allSeries = [...modelSeries, ...(otherSeries ? [otherSeries] : [])];
  const scale = chartScale(allSeries);
  const displayLabels = usageSeriesLabels(allSeries);
  const preparedSeries = allSeries.map((item): PreparedUsageSeries => {
    const hash = stableHash(item.series_key);
    const pointByDate = new Map(item.points.map((point) => [point.local_date, point]));
    return {
      seriesKey: item.series_key,
      label: displayLabels.get(item.series_key) ?? item.display_name,
      color: CHART_COLORS[hash % CHART_COLORS.length],
      lineType: LINE_TYPES[Math.floor(hash / CHART_COLORS.length) % LINE_TYPES.length],
      symbol: SYMBOLS[Math.floor(hash / 17) % SYMBOLS.length],
      source: item,
      points: dates.map((date) => pointForDate(pointByDate.get(date), date, scale)),
    };
  });

  const tooltipFormatter = (input: UsageTooltipParam | UsageTooltipParam[]): string => {
    const params = Array.isArray(input) ? input : [input];
    const dateValue = params[0]?.axisValue ?? params[0]?.data?.localDate ?? "";
    const dateLabel = dateValue
      ? new Intl.DateTimeFormat(locale, { dateStyle: "medium", timeZone: "UTC" }).format(parseLocalDate(dateValue))
      : "";
    const lines = params.map((param) => {
      const data = param.data;
      const raw = data?.rawValue;
      const value = raw === null || raw === undefined
        ? "—"
        : formatFullTokens(raw, locale);
      const details: string[] = [];
      if (data && BigInt(data.estimatedTokens) > 0n) {
        details.push(`${labels.estimatedTokens} ${formatFullTokens(data.estimatedTokens, locale)}`);
      }
      if (data && BigInt(data.legacyTokens) > 0n) {
        details.push(`${labels.legacyTokens} ${formatFullTokens(data.legacyTokens, locale)}`);
      }
      if (data?.unknownOperationCount) {
        details.push(`${data.unknownOperationCount} ${labels.unknownOperations}`);
      }
      return `${param.seriesName ?? ""}: ${value}${details.length ? ` (${details.join(" · ")})` : ""}`;
    });
    return [dateLabel, ...lines].filter(Boolean).join("\n");
  };

  const option: UsageChartOption = {
    animation: false,
    aria: { enabled: true, decal: { show: true } },
    color: preparedSeries.map((series) => series.color),
    grid: { top: 28, right: 24, bottom: 56, left: 24, containLabel: true },
    tooltip: { trigger: "axis", renderMode: "richText", formatter: tooltipFormatter },
    legend: {
      type: "scroll",
      bottom: 0,
      data: preparedSeries.map((series) => series.label),
    },
    xAxis: {
      type: "category",
      boundaryGap: false,
      data: dates,
      axisLabel: {
        hideOverlap: true,
        formatter: (value) =>
          new Intl.DateTimeFormat(locale, { month: "short", day: "numeric", timeZone: "UTC" }).format(parseLocalDate(value)),
      },
    },
    yAxis: {
      type: "value",
      min: 0,
      name: labels.yAxis,
      axisLabel: {
        formatter: (value) => formatCompactTokens(axisRawValue(value, scale).toString()),
      },
    },
    series: preparedSeries.map((series) => ({
      id: series.seriesKey,
      name: series.label,
      type: "line",
      data: series.points,
      smooth: false,
      connectNulls: false,
      showSymbol: true,
      symbol: series.symbol,
      symbolSize: 6,
      lineStyle: { color: series.color, type: series.lineType, width: 2 },
      itemStyle: { color: series.color },
      emphasis: { focus: "series" },
    })),
  };

  return { dates, scale, preparedSeries, option };
}
