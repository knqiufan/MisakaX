import type { ContentBlock } from "@/lib/ipc";

export type ChartType = "line" | "bar" | "area" | "scatter" | "pie" | "metric";

export interface ChartDatum {
  x: string;
  y: number;
  label?: string;
}

export interface ChartSeries {
  name: string;
  values: ChartDatum[];
}

export interface ChartSpecV1 {
  chart_type: ChartType;
  title: string;
  summary?: string;
  x_label?: string;
  y_label?: string;
  unit?: string;
  series: ChartSeries[];
}

const MAX_SERIES = 24;
const MAX_POINTS = 5_000;
const MAX_LABEL_CHARS = 512;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isString(value: unknown): value is string {
  return typeof value === "string";
}

export function readChartSpec(block: ContentBlock): ChartSpecV1 | null {
  if (!isRecord(block.payload)) return null;
  const candidate = isRecord(block.payload.spec) ? block.payload.spec : block.payload;
  if (!isString(candidate.chart_type) || !isString(candidate.title) || !Array.isArray(candidate.series)) {
    return null;
  }
  const chartTypes: ChartType[] = ["line", "bar", "area", "scatter", "pie", "metric"];
  if (
    !chartTypes.includes(candidate.chart_type as ChartType)
    || candidate.title.trim().length === 0
    || candidate.title.length > MAX_LABEL_CHARS
    || candidate.series.length === 0
    || candidate.series.length > MAX_SERIES
  ) {
    return null;
  }
  let points = 0;
  const series = candidate.series.flatMap((rawSeries) => {
    if (
      !isRecord(rawSeries)
      || !isString(rawSeries.name)
      || rawSeries.name.trim().length === 0
      || rawSeries.name.length > MAX_LABEL_CHARS
      || !Array.isArray(rawSeries.values)
    ) {
      return [];
    }
    const values = rawSeries.values.flatMap((item) => {
      if (
        !isRecord(item)
        || !isString(item.x)
        || item.x.length > MAX_LABEL_CHARS
        || typeof item.y !== "number"
        || !Number.isFinite(item.y)
        || (item.label !== undefined && !isString(item.label))
        || (isString(item.label) && item.label.length > MAX_LABEL_CHARS)
      ) {
        return [];
      }
      points += 1;
      if (points > MAX_POINTS) return [];
      return [{ x: item.x, y: item.y, label: isString(item.label) ? item.label : undefined }];
    });
    if (values.length !== rawSeries.values.length) return [];
    return [{ name: rawSeries.name, values }];
  });
  if (series.length !== candidate.series.length || points > MAX_POINTS) return null;
  return {
    chart_type: candidate.chart_type as ChartType,
    title: candidate.title,
    summary: isString(candidate.summary) ? candidate.summary : undefined,
    x_label: isString(candidate.x_label) ? candidate.x_label : undefined,
    y_label: isString(candidate.y_label) ? candidate.y_label : undefined,
    unit: isString(candidate.unit) ? candidate.unit : undefined,
    series,
  };
}
