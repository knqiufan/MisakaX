import { useTranslation } from "react-i18next";
import { CalendarDays, Flame, Hash } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { UsageDashboardV1 } from "@/lib/ipc/types";
import { formatCompactTokens, formatFullTokens, parseTokenDecimal } from "./usage-format";

interface UsageOverviewCardsProps {
  dashboard: UsageDashboardV1 | null;
  status: "loading" | "success" | "error";
  error: string | null;
  onRetry: () => void;
}

function OverviewSkeleton() {
  return (
    <div className="grid gap-4 md:grid-cols-3" aria-hidden="true">
      {[0, 1, 2].map((index) => (
        <Card key={index} className="min-h-40 gap-4 shadow-sm">
          <CardHeader className="gap-3">
            <Skeleton className="h-4 w-28" />
            <Skeleton className="h-9 w-24" />
          </CardHeader>
          <CardContent>
            <Skeleton className="h-4 w-40" />
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

interface MetricCardProps {
  title: string;
  value: string;
  fullValue: string;
  description: string;
  icon: typeof Hash;
}

function MetricCard({ title, value, fullValue, description, icon: Icon }: MetricCardProps) {
  return (
    <Card className="min-h-40 gap-4 shadow-sm">
      <CardHeader className="grid grid-cols-[1fr_auto] items-center gap-2">
        <CardTitle className="text-sm font-medium text-muted-foreground">{title}</CardTitle>
        <Icon className="size-4 text-muted-foreground" aria-hidden="true" />
      </CardHeader>
      <CardContent className="grid gap-2">
        <Tooltip>
          <TooltipTrigger asChild>
            <span
              tabIndex={0}
              className="w-fit rounded-sm text-3xl font-semibold tracking-tight text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
              aria-label={`${title}: ${fullValue}`}
            >
              {value}
            </span>
          </TooltipTrigger>
          <TooltipContent>{fullValue}</TooltipContent>
        </Tooltip>
        <p className="min-h-9 text-xs leading-5 text-muted-foreground">{description}</p>
      </CardContent>
    </Card>
  );
}

export function UsageOverviewCards({
  dashboard,
  status,
  onRetry,
}: UsageOverviewCardsProps) {
  const { t, i18n } = useTranslation("profile");

  if (status === "loading" && !dashboard) return <OverviewSkeleton />;

  if (status === "error" && !dashboard) {
    return (
      <div className="flex min-h-40 flex-col items-center justify-center gap-3 rounded-[var(--radius-ui-lg)] border border-border bg-card p-6 text-center" role="alert">
        <p className="text-sm font-medium">{t("overview.loadError")}</p>
        <p className="max-w-lg text-xs text-muted-foreground">{t("overview.loadErrorDescription")}</p>
        <Button variant="outline" size="sm" onClick={onRetry}>{t("retry")}</Button>
      </div>
    );
  }

  if (!dashboard) return null;

  const { overview } = dashboard;
  const totalTokens = parseTokenDecimal(overview.total_tokens);
  const hasActivity = overview.total_days > 0 || overview.unknown_operation_count > 0;
  const hasKnownTokens = totalTokens > 0n || (hasActivity && overview.unknown_operation_count === 0);
  const emptyValue = t("overview.emptyValue");
  const totalValue = hasKnownTokens ? formatCompactTokens(overview.total_tokens) : emptyValue;
  const fullTotal = hasKnownTokens
    ? t("overview.fullTokens", {
        value: formatFullTokens(overview.total_tokens, i18n.language),
      })
    : t("overview.unknownTotal");
  const qualityDescription = t("overview.quality", {
    estimated: formatCompactTokens(overview.estimated_tokens),
    legacy: formatCompactTokens(overview.legacy_tokens),
    unknown: overview.unknown_operation_count,
  });

  return (
    <div className="grid gap-3">
      {status === "error" ? (
        <div
          className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-border bg-muted/40 px-3 py-2 text-xs text-muted-foreground"
          role="status"
          data-overview-refresh-error
        >
          <span>{t("overview.refreshError")}</span>
          <Button variant="ghost" size="xs" onClick={onRetry}>{t("retry")}</Button>
        </div>
      ) : null}
      <div className="grid gap-4 md:grid-cols-3" data-overview-state={hasActivity ? "ready" : "empty"}>
      <MetricCard
        title={t("overview.totalTokens")}
        value={totalValue}
        fullValue={fullTotal}
        description={hasActivity ? qualityDescription : t("overview.noUsageYet")}
        icon={Hash}
      />
      <MetricCard
        title={t("overview.totalDays")}
        value={hasActivity ? String(overview.total_days) : emptyValue}
        fullValue={hasActivity ? t("overview.fullDays", { count: overview.total_days }) : t("overview.noUsageYet")}
        description={t("overview.totalDaysDescription")}
        icon={CalendarDays}
      />
      <MetricCard
        title={t("overview.currentStreak")}
        value={hasActivity ? String(overview.current_streak) : emptyValue}
        fullValue={hasActivity ? t("overview.fullStreak", { count: overview.current_streak }) : t("overview.noUsageYet")}
        description={hasActivity
          ? t("overview.longestStreak", { count: overview.longest_streak })
          : t("overview.currentStreakDescription")}
        icon={Flame}
      />
      </div>
    </div>
  );
}
