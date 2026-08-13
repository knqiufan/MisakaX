import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Table2 } from "lucide-react";

import { EChartCanvas } from "@/components/charts";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { ModelUsageSeriesV1 } from "@/lib/ipc/types";
import { UsageDataTable } from "./UsageDataTable";
import { buildUsageChart } from "./usage-chart-options";

interface UsageTrendChartProps {
  modelSeries: ModelUsageSeriesV1[];
  otherSeries: ModelUsageSeriesV1 | null;
  range: { start: string; end: string } | null;
  status: "loading" | "success" | "error";
  hasSnapshot: boolean;
  onRetry: () => void;
}

function TrendSkeleton() {
  return (
    <Card className="min-h-[360px] gap-4 shadow-sm" aria-hidden="true">
      <CardHeader><Skeleton className="h-5 w-48" /></CardHeader>
      <CardContent><Skeleton className="h-[260px] w-full" /></CardContent>
    </Card>
  );
}

export function UsageTrendChart({
  modelSeries,
  otherSeries,
  range,
  status,
  hasSnapshot,
  onRetry,
}: UsageTrendChartProps) {
  const { t, i18n } = useTranslation("profile");
  const [showTable, setShowTable] = useState(false);
  const [chartFailed, setChartFailed] = useState(false);
  const handleChartError = useCallback(() => setChartFailed(true), []);
  const chart = useMemo(
    () => range
      ? buildUsageChart(modelSeries, otherSeries, range, i18n.language, {
          yAxis: t("trend.yAxis"),
          unknownOperations: t("trend.unknownOperations"),
          estimatedTokens: t("trend.estimatedTokens"),
          legacyTokens: t("trend.legacyTokens"),
        })
      : null,
    [i18n.language, modelSeries, otherSeries, range, t]
  );

  if (status === "loading" && !hasSnapshot) return <TrendSkeleton />;
  if (status === "error" && !hasSnapshot) {
    return (
      <Card className="min-h-[360px] items-center justify-center gap-3 p-6 text-center shadow-sm" role="alert">
        <p className="text-sm font-medium">{t("trend.loadError")}</p>
        <p className="text-xs text-muted-foreground">{t("trend.loadErrorDescription")}</p>
        <Button variant="outline" size="sm" onClick={onRetry}>{t("retry")}</Button>
      </Card>
    );
  }
  if (!hasSnapshot || !chart) return null;

  const hasSeries = chart.preparedSeries.length > 0;
  return (
    <Card className="min-h-[360px] gap-4 shadow-sm">
      <CardHeader className="grid grid-cols-[1fr_auto] items-start gap-3">
        <div className="grid gap-1">
          <CardTitle role="heading" aria-level={2}>{t("trend.title")}</CardTitle>
          <p className="text-sm text-muted-foreground">{t("trend.description")}</p>
        </div>
        <Button
          variant="ghost"
          size="sm"
          aria-pressed={showTable || chartFailed}
          onClick={() => setShowTable((value) => !value)}
          disabled={!hasSeries || chartFailed}
        >
          <Table2 className="size-3.5" />
          {t("trend.toggleTable")}
        </Button>
      </CardHeader>
      <CardContent className="grid gap-4">
        {status === "error" ? (
          <div className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-border bg-muted/40 px-3 py-2 text-xs text-muted-foreground" role="status">
            <span>{t("trend.refreshError")}</span>
            <Button variant="ghost" size="xs" onClick={onRetry}>{t("retry")}</Button>
          </div>
        ) : null}

        {!hasSeries ? (
          <div className="flex min-h-[260px] items-center justify-center rounded-lg border border-dashed border-border text-sm text-muted-foreground">
            {t("trend.empty")}
          </div>
        ) : (
          <>
            <EChartCanvas
              option={chart.option}
              className="h-[280px]"
              ariaLabel={t("trend.chartLabel")}
              onError={handleChartError}
              fallback={
                <div className="flex min-h-20 items-center justify-center rounded-lg border border-border bg-muted/30 p-4 text-xs text-muted-foreground" role="alert">
                  {t("trend.chartFallback")}
                </div>
              }
            />
            {showTable || chartFailed ? (
              <UsageDataTable
                chart={chart}
                label={chartFailed ? t("trend.chartFallbackTable") : undefined}
              />
            ) : null}
          </>
        )}
      </CardContent>
    </Card>
  );
}
