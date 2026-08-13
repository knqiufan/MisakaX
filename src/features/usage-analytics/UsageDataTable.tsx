import { useTranslation } from "react-i18next";

import { formatFullTokens } from "./usage-format";
import { parseLocalDate } from "./usage-calendar";
import type { BuiltUsageChart } from "./usage-chart-options";

interface UsageDataTableProps {
  chart: BuiltUsageChart;
  label?: string;
}

export function UsageDataTable({ chart, label }: UsageDataTableProps) {
  const { t, i18n } = useTranslation("profile");

  return (
    <div className="overflow-x-auto" data-usage-data-table>
      {label ? <p className="mb-2 text-xs text-muted-foreground">{label}</p> : null}
      <table className="w-full min-w-[640px] border-collapse text-left text-xs">
        <thead className="text-muted-foreground">
          <tr>
            <th className="border-b border-border px-3 py-2 font-medium">{t("trend.table.date")}</th>
            {chart.preparedSeries.map((series) => (
              <th key={series.seriesKey} className="border-b border-border px-3 py-2 font-medium">
                {series.label}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {chart.dates.map((date, dateIndex) => (
            <tr key={date} className="border-b border-border/60">
              <th scope="row" className="whitespace-nowrap px-3 py-2 font-normal text-muted-foreground">
                {new Intl.DateTimeFormat(i18n.language, { dateStyle: "medium", timeZone: "UTC" }).format(parseLocalDate(date))}
              </th>
              {chart.preparedSeries.map((series) => {
                const point = series.points[dateIndex];
                return (
                  <td key={series.seriesKey} className="px-3 py-2 text-foreground">
                    <span>{point.rawValue === null ? "—" : formatFullTokens(point.rawValue, i18n.language)}</span>
                    {point.unknownOperationCount > 0 ? (
                      <span className="ml-1 text-muted-foreground">
                        {t("trend.table.unknown", { count: point.unknownOperationCount })}
                      </span>
                    ) : null}
                    {BigInt(point.estimatedTokens) > 0n ? (
                      <span className="block text-[10px] text-muted-foreground">
                        {t("trend.table.estimated", {
                          value: formatFullTokens(point.estimatedTokens, i18n.language),
                        })}
                      </span>
                    ) : null}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
