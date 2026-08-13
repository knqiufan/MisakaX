import { KeyboardEvent, Ref } from "react";
import { useTranslation } from "react-i18next";

import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { formatFullTokens } from "./usage-format";
import { parseLocalDate, type ActivityCalendarCell } from "./usage-calendar";
import { cn } from "@/lib/utils";

const HEAT_CLASSES = [
  "bg-[var(--usage-heat-0)] border-[var(--usage-heat-border)]",
  "bg-[var(--usage-heat-1)] border-transparent",
  "bg-[var(--usage-heat-2)] border-transparent",
  "bg-[var(--usage-heat-3)] border-transparent",
  "bg-[var(--usage-heat-4)] border-transparent",
] as const;

interface ActivityCellProps {
  cell: ActivityCalendarCell;
  tabIndex: 0 | -1;
  buttonRef: Ref<HTMLButtonElement>;
  onFocus: () => void;
  onKeyDown: (event: KeyboardEvent<HTMLButtonElement>) => void;
}

function formatTokens(value: string | null, locale: string, unknownLabel: string): string {
  return value === null ? unknownLabel : formatFullTokens(value, locale);
}

export function ActivityCell({
  cell,
  tabIndex,
  buttonRef,
  onFocus,
  onKeyDown,
}: ActivityCellProps) {
  const { t, i18n } = useTranslation("profile");
  const { day, visualState, heatLevel } = cell;
  const date = parseLocalDate(day.local_date);
  const dateLabel = new Intl.DateTimeFormat(i18n.language, { dateStyle: "long", timeZone: "UTC" }).format(date);
  const tokenLabel = formatTokens(day.total_tokens, i18n.language, t("activity.unknownTokens"));
  const qualityLabel = t("activity.qualitySummary", {
    exact: formatFullTokens(day.quality.exact_tokens, i18n.language),
    estimated: formatFullTokens(day.quality.estimated_tokens, i18n.language),
    legacy: formatFullTokens(day.quality.legacy_tokens, i18n.language),
    unknown: day.quality.unknown_operation_count,
  });
  const ariaLabel = t("activity.cellLabel", {
    date: dateLabel,
    tokens: tokenLabel,
    calls: day.operation_count,
    model: day.primary_model ?? t("activity.noPrimaryModel"),
    quality: qualityLabel,
  });

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          ref={buttonRef}
          type="button"
          role="gridcell"
          tabIndex={tabIndex}
          aria-label={ariaLabel}
          aria-rowindex={cell.rowIndex + 1}
          aria-colindex={cell.weekIndex + 1}
          data-local-date={day.local_date}
          data-visual-state={visualState}
          data-heat-level={heatLevel}
          onFocus={onFocus}
          onKeyDown={onKeyDown}
          className={cn(
            "size-3 rounded-[3px] border outline-none transition-[border-color,box-shadow] duration-[var(--ds-dur-fast)]",
            "hover:border-foreground/60 focus-visible:z-10 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-card",
            HEAT_CLASSES[heatLevel],
            visualState === "unknown" && "usage-cell-unknown",
            visualState === "mixed" && "usage-cell-mixed",
            visualState === "future" && "cursor-default opacity-35"
          )}
        />
      </TooltipTrigger>
      <TooltipContent className="grid max-w-xs gap-1.5 p-3 text-left font-normal" side="top">
        <strong className="font-medium text-popover-foreground">{dateLabel}</strong>
        <span>{t("activity.tooltip.total", { value: tokenLabel })}</span>
        <span>
          {t("activity.tooltip.io", {
            input: formatTokens(day.input_tokens, i18n.language, t("activity.unknownShort")),
            output: formatTokens(day.output_tokens, i18n.language, t("activity.unknownShort")),
          })}
        </span>
        <span>{t("activity.tooltip.calls", { count: day.operation_count })}</span>
        <span>{t("activity.tooltip.model", { model: day.primary_model ?? t("activity.noPrimaryModel") })}</span>
        <span className="text-muted-foreground">{qualityLabel}</span>
      </TooltipContent>
    </Tooltip>
  );
}
