import { useCallback, useMemo, useState } from "react";
import { BarChart3, Download, Table2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { EChartCanvas } from "@/components/charts";
import { artifactsIpc } from "@/lib/ipc";
import type { BlockRendererProps } from "../renderer-registry";
import { RichContentCard } from "../RichContentCard";
import { readChartSpec, type ChartSpecV1 } from "./chart-types";
import { NoticeBlockRenderer } from "./NoticeBlockRenderer";

export function ChartBlockRenderer({ block, sessionId }: BlockRendererProps) {
  const { t } = useTranslation("chat");
  const spec = readChartSpec(block);
  const [showTable, setShowTable] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const exportCsv = useCallback(async () => {
    if (!spec) return;
    setExporting(true);
    setError(null);
    let artifactId: string | null = null;
    try {
      const metadata = await artifactsIpc.exportChartCsv(sessionId, block.message_id, spec);
      artifactId = metadata.artifact_id;
      await artifactsIpc.export(sessionId, metadata.artifact_id);
    } catch {
      setError(t("richContent.chart.exportFailed"));
    } finally {
      if (artifactId) void artifactsIpc.expire(sessionId, artifactId);
      setExporting(false);
    }
  }, [block.message_id, sessionId, spec, t]);

  if (!spec) return <NoticeBlockRenderer block={block} sessionId={sessionId} isStreaming={false} />;

  return (
    <RichContentCard
      title={spec.title}
      icon={<BarChart3 className="size-4" />}
      status={block.status}
      actions={
        <>
          <Button
            size="icon-xs"
            variant="ghost"
            aria-label={t("richContent.chart.toggleData")}
            aria-pressed={showTable}
            onClick={() => setShowTable((value) => !value)}
          >
            <Table2 className="size-3.5" />
          </Button>
          <Button
            size="icon-xs"
            variant="ghost"
            aria-label={t("richContent.chart.exportCsv")}
            disabled={exporting}
            onClick={exportCsv}
          >
            <Download className="size-3.5" />
          </Button>
        </>
      }
      footer={
        error ? <span className="text-destructive" role="alert">{error}</span> : spec.summary || t("richContent.chart.localOnly")
      }
    >
      {spec.chart_type === "metric" ? <MetricPreview spec={spec} /> : <ChartCanvas spec={spec} />}
      {showTable ? <ChartDataTable spec={spec} /> : null}
    </RichContentCard>
  );
}

function ChartCanvas({ spec }: { spec: ChartSpecV1 }) {
  const { t } = useTranslation("chat");
  const option = useMemo(() => compileChartOption(spec), [spec]);
  return (
    <EChartCanvas
      option={option}
      className="h-[260px]"
      ariaLabel={t("richContent.chart.visualization")}
      fallback={<ChartDataTable spec={spec} label={t("richContent.chart.fallbackData")} />}
    />
  );
}

function MetricPreview({ spec }: { spec: ChartSpecV1 }) {
  const { t } = useTranslation("chat");
  const values = spec.series[0]?.values ?? [];
  const latest = values[values.length - 1];
  return (
    <div className="flex min-h-[180px] flex-col justify-center px-4 py-5">
      <p className="text-xs text-muted-foreground">{spec.series[0]?.name ?? t("richContent.chart.metric")}</p>
      <p className="mt-1 text-3xl font-semibold tracking-tight">{latest ? `${latest.y}${spec.unit ?? ""}` : "—"}</p>
      {latest?.x ? <p className="mt-2 text-xs text-muted-foreground">{latest.x}</p> : null}
    </div>
  );
}

function ChartDataTable({ spec, label }: { spec: ChartSpecV1; label?: string }) {
  const { t } = useTranslation("chat");
  return (
    <div className="overflow-x-auto border-t border-border/35 px-3 py-3">
      {label ? <p className="mb-2 text-xs text-muted-foreground">{label}</p> : null}
      <table className="w-full min-w-[360px] text-left text-xs">
        <thead className="text-muted-foreground">
          <tr>
            <th className="pb-2 pr-3 font-medium">{t("richContent.chart.series")}</th>
            <th className="pb-2 pr-3 font-medium">{spec.x_label ?? t("richContent.chart.x")}</th>
            <th className="pb-2 font-medium">{spec.y_label ?? t("richContent.chart.y")}</th>
          </tr>
        </thead>
        <tbody>
          {spec.series.flatMap((series) => series.values.map((datum, index) => (
            <tr key={`${series.name}-${datum.x}-${index}`} className="border-t border-border/25">
              <td className="py-1.5 pr-3 text-foreground">{series.name}</td>
              <td className="py-1.5 pr-3 text-muted-foreground">{datum.x}</td>
              <td className="py-1.5 text-foreground">{datum.y}{spec.unit ?? ""}</td>
            </tr>
          )))}
        </tbody>
      </table>
    </div>
  );
}

function compileChartOption(spec: ChartSpecV1) {
  const categories = [...new Set(spec.series.flatMap((series) => series.values.map((item) => item.x)))];
  const colors = ["var(--chart-1)", "var(--chart-2)", "var(--chart-3)", "var(--chart-4)", "var(--chart-5)"];
  const unitSuffix = spec.unit ? ` ${spec.unit}` : "";
  const series = spec.series.map((item, index) => {
    const values = item.values.map((datum) => ({ value: spec.chart_type === "scatter" ? [datum.x, datum.y] : datum.y, name: datum.label ?? datum.x }));
    if (spec.chart_type === "pie") return { type: "pie", name: item.name, radius: ["35%", "66%"], data: values.map((value, valueIndex) => ({ value: item.values[valueIndex].y, name: value.name })) };
    return {
      type: spec.chart_type === "area" ? "line" : spec.chart_type,
      name: item.name,
      data: values,
      smooth: spec.chart_type === "line" || spec.chart_type === "area",
      areaStyle: spec.chart_type === "area" ? { opacity: 0.16 } : undefined,
      symbolSize: spec.chart_type === "scatter" ? 8 : undefined,
      itemStyle: { color: colors[index % colors.length] },
      lineStyle: { color: colors[index % colors.length] },
    };
  });
  return {
    animation: false,
    aria: { enabled: true, decal: { show: true } },
    color: colors,
    grid: spec.chart_type === "pie" ? undefined : { top: 32, right: 24, bottom: 36, left: 48, containLabel: true },
    tooltip: {
      trigger: spec.chart_type === "pie" ? "item" : "axis",
      renderMode: "richText",
      valueFormatter: `{value}${unitSuffix}`,
    },
    legend: { show: spec.series.length > 1, bottom: 0, type: "scroll" },
    xAxis: spec.chart_type === "pie" ? undefined : { type: "category", name: spec.x_label, data: categories, axisLabel: { hideOverlap: true } },
    yAxis: spec.chart_type === "pie" ? undefined : { type: "value", name: spec.y_label, axisLabel: { formatter: `{value}${unitSuffix}` } },
    series,
  };
}
