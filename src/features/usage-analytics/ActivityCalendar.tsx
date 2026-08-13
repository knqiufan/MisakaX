import { KeyboardEvent, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { DailyUsageV1 } from "@/lib/ipc/types";
import { ActivityCell } from "./ActivityCell";
import { formatFullTokens } from "./usage-format";
import {
  buildActivityCalendar,
  parseLocalDate,
  type ActivityCalendarCell,
} from "./usage-calendar";

interface ActivityCalendarProps {
  days: DailyUsageV1[];
  weekStart: 0 | 1;
  todayLocalDate?: string;
  status: "loading" | "success" | "error";
  hasSnapshot: boolean;
  onRetry: () => void;
}

const LEGEND_HEAT_CLASSES = [
  "bg-[var(--usage-heat-0)]",
  "bg-[var(--usage-heat-1)]",
  "bg-[var(--usage-heat-2)]",
  "bg-[var(--usage-heat-3)]",
  "bg-[var(--usage-heat-4)]",
] as const;

function ActivityCalendarSkeleton() {
  return (
    <Card className="min-h-80 gap-4 shadow-sm" aria-hidden="true">
      <CardHeader><Skeleton className="h-5 w-36" /></CardHeader>
      <CardContent className="grid gap-4">
        <Skeleton className="h-4 w-64 max-w-full" />
        <Skeleton className="h-36 w-full" />
      </CardContent>
    </Card>
  );
}

function weekdayLabels(locale: string, weekStart: 0 | 1): string[] {
  const sunday = new Date(Date.UTC(2024, 0, 7));
  return Array.from({ length: 7 }, (_, index) => {
    const date = new Date(sunday);
    date.setUTCDate(sunday.getUTCDate() + ((weekStart + index) % 7));
    return new Intl.DateTimeFormat(locale, { weekday: "narrow", timeZone: "UTC" }).format(date);
  });
}

export function ActivityCalendar({
  days,
  weekStart,
  todayLocalDate,
  status,
  hasSnapshot,
  onRetry,
}: ActivityCalendarProps) {
  const { t, i18n } = useTranslation("profile");
  const grid = useMemo(
    () => buildActivityCalendar(
      days,
      weekStart,
      todayLocalDate ? parseLocalDate(todayLocalDate) : new Date()
    ),
    [days, todayLocalDate, weekStart]
  );
  const focusableDays = useMemo(
    () => grid.days.filter((cell) => cell.visualState !== "future"),
    [grid.days]
  );
  const defaultFocusedDate = focusableDays[focusableDays.length - 1]?.day.local_date ?? null;
  const [focusedDate, setFocusedDate] = useState<string | null>(defaultFocusedDate);
  const buttonRefs = useRef(new Map<string, HTMLButtonElement>());
  const activeFocusedDate = focusedDate && focusableDays.some((cell) => cell.day.local_date === focusedDate)
    ? focusedDate
    : defaultFocusedDate;

  useEffect(() => {
    if (focusedDate && !focusableDays.some((cell) => cell.day.local_date === focusedDate)) {
      setFocusedDate(defaultFocusedDate);
    }
  }, [defaultFocusedDate, focusableDays, focusedDate]);

  function moveFocus(current: ActivityCalendarCell, event: KeyboardEvent<HTMLButtonElement>) {
    const currentIndex = focusableDays.findIndex(
      (cell) => cell.day.local_date === current.day.local_date
    );
    if (currentIndex < 0) return;
    let targetIndex = currentIndex;
    switch (event.key) {
      case "ArrowUp":
        targetIndex -= 1;
        break;
      case "ArrowDown":
        targetIndex += 1;
        break;
      case "ArrowLeft":
        targetIndex -= 7;
        break;
      case "ArrowRight":
        targetIndex += 7;
        break;
      case "Home":
        targetIndex -= current.rowIndex;
        break;
      case "End":
        targetIndex += 6 - current.rowIndex;
        break;
      case "Escape":
        event.currentTarget.blur();
        return;
      default:
        return;
    }
    event.preventDefault();
    const target = focusableDays[targetIndex];
    if (!target) return;
    setFocusedDate(target.day.local_date);
    buttonRefs.current.get(target.day.local_date)?.focus();
  }

  if (status === "loading" && !hasSnapshot) return <ActivityCalendarSkeleton />;
  if (status === "error" && !hasSnapshot) {
    return (
      <Card className="min-h-80 items-center justify-center gap-3 p-6 text-center shadow-sm" role="alert">
        <p className="text-sm font-medium">{t("activity.loadError")}</p>
        <p className="text-xs text-muted-foreground">{t("activity.loadErrorDescription")}</p>
        <Button variant="outline" size="sm" onClick={onRetry}>{t("retry")}</Button>
      </Card>
    );
  }
  if (!hasSnapshot) return null;

  const labels = weekdayLabels(i18n.language, weekStart);
  const gridWidth = grid.weekCount * 12 + Math.max(0, grid.weekCount - 1) * 3;

  return (
    <Card className="min-h-80 gap-4 shadow-sm">
      <CardHeader className="gap-1">
        <CardTitle role="heading" aria-level={2}>{t("activity.title")}</CardTitle>
        <p className="text-sm text-muted-foreground">{t("activity.description")}</p>
      </CardHeader>
      <CardContent className="grid gap-4">
        {status === "error" ? (
          <div className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-border bg-muted/40 px-3 py-2 text-xs text-muted-foreground" role="status">
            <span>{t("activity.refreshError")}</span>
            <Button variant="ghost" size="xs" onClick={onRetry}>{t("retry")}</Button>
          </div>
        ) : null}

        <div className="overflow-x-auto pb-2" data-activity-scroll-region>
          <div className="min-w-max p-1">
            <div className="relative ml-7 h-5 text-[10px] text-muted-foreground" style={{ width: gridWidth }} aria-hidden="true">
              {grid.monthLabels.map((label) => (
                <span
                  key={label.monthKey}
                  className="absolute whitespace-nowrap"
                  style={{ left: label.weekIndex * 15 }}
                >
                  {new Intl.DateTimeFormat(i18n.language, { month: "short", timeZone: "UTC" }).format(label.date)}
                </span>
              ))}
            </div>
            <div className="flex gap-1.5">
              <div className="grid w-5 shrink-0 grid-rows-7 gap-[3px] text-[9px] leading-3 text-muted-foreground" aria-hidden="true">
                {labels.map((label, index) => <span key={`${label}-${index}`}>{index % 2 === 1 ? label : ""}</span>)}
              </div>
              <div
                role="grid"
                aria-label={t("activity.gridLabel")}
                aria-rowcount={7}
                aria-colcount={grid.weekCount}
                className="grid grid-flow-col grid-rows-7 gap-[3px]"
                style={{ gridTemplateColumns: `repeat(${grid.weekCount}, 12px)` }}
              >
                {grid.cells.map((cell, slotIndex) =>
                  cell ? (
                    <ActivityCell
                      key={cell.day.local_date}
                      cell={cell}
                      tabIndex={activeFocusedDate === cell.day.local_date ? 0 : -1}
                      buttonRef={(button) => {
                        if (button) buttonRefs.current.set(cell.day.local_date, button);
                        else buttonRefs.current.delete(cell.day.local_date);
                      }}
                      onFocus={() => setFocusedDate(cell.day.local_date)}
                      onKeyDown={(event) => moveFocus(cell, event)}
                    />
                  ) : (
                    <span key={`empty-${slotIndex}`} aria-hidden="true" className="size-3" />
                  )
                )}
              </div>
            </div>
          </div>
        </div>

        <div className="flex flex-wrap items-center justify-between gap-3 text-xs text-muted-foreground">
          <div className="flex items-center gap-1.5" aria-label={t("activity.legendLabel")}>
            <span>{t("activity.less")}</span>
            {LEGEND_HEAT_CLASSES.map((heatClass, level) => (
              <span
                key={level}
                className={`size-3 rounded-[3px] border border-[var(--usage-heat-border)] ${heatClass}`}
                aria-hidden="true"
              />
            ))}
            <span>{t("activity.more")}</span>
            <span className="usage-cell-unknown ml-2 size-3 rounded-[3px] border" aria-hidden="true" />
            <span>{t("activity.unknownLegend")}</span>
          </div>
          <span>{t("activity.relativeScale")}</span>
        </div>

        <details className="group rounded-lg border border-border bg-muted/20">
          <summary className="cursor-pointer rounded-lg px-3 py-2 text-sm font-medium outline-none focus-visible:ring-2 focus-visible:ring-ring/40">
            {t("activity.tableSummary")}
          </summary>
          <div className="max-h-80 overflow-auto border-t border-border">
            <table className="w-full min-w-[640px] border-collapse text-left text-xs">
              <thead className="sticky top-0 bg-card text-muted-foreground">
                <tr>
                  <th className="px-3 py-2 font-medium">{t("activity.table.date")}</th>
                  <th className="px-3 py-2 font-medium">{t("activity.table.tokens")}</th>
                  <th className="px-3 py-2 font-medium">{t("activity.table.calls")}</th>
                  <th className="px-3 py-2 font-medium">{t("activity.table.model")}</th>
                  <th className="px-3 py-2 font-medium">{t("activity.table.quality")}</th>
                </tr>
              </thead>
              <tbody>
                {grid.days.map((cell) => (
                  <tr key={cell.day.local_date} className="border-t border-border/60">
                    <td className="whitespace-nowrap px-3 py-2">
                      {new Intl.DateTimeFormat(i18n.language, { dateStyle: "medium", timeZone: "UTC" }).format(parseLocalDate(cell.day.local_date))}
                    </td>
                    <td className="px-3 py-2">
                      {cell.day.total_tokens === null
                        ? t("activity.unknownTokens")
                        : formatFullTokens(cell.day.total_tokens, i18n.language)}
                    </td>
                    <td className="px-3 py-2">{cell.day.operation_count}</td>
                    <td className="px-3 py-2">{cell.day.primary_model ?? t("activity.noPrimaryModel")}</td>
                    <td className="px-3 py-2" data-quality={cell.visualState}>{t(`activity.states.${cell.visualState}`)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </details>
      </CardContent>
    </Card>
  );
}
