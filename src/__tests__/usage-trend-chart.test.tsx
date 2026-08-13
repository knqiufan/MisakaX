import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@/components/charts", () => ({
  EChartCanvas: ({ ariaLabel, onError }: { ariaLabel: string; onError?: () => void }) => (
    <button type="button" aria-label={ariaLabel} onClick={onError}>chart-canvas</button>
  ),
}));

import { TooltipProvider } from "@/components/ui/tooltip";
import { UsageTrendChart } from "@/features/usage-analytics/UsageTrendChart";
import { i18n } from "@/locales/i18n";
import type { ModelUsageSeriesV1 } from "@/lib/ipc/types";

const SERIES: ModelUsageSeriesV1 = {
  series_key: "model-a",
  display_name: "Model A",
  provider_config_id: "config-a",
  provider_id: "provider",
  effective_model_id: "model-a",
  points: [
    {
      local_date: "2026-08-12",
      total_tokens: null,
      unknown_operation_count: 1,
      estimated_tokens: "0",
      legacy_tokens: "0",
    },
    {
      local_date: "2026-08-13",
      total_tokens: "1200",
      unknown_operation_count: 0,
      estimated_tokens: "200",
      legacy_tokens: "0",
    },
  ],
};

function renderTrend(overrides: Partial<React.ComponentProps<typeof UsageTrendChart>> = {}) {
  return render(
    <TooltipProvider>
      <UsageTrendChart
        modelSeries={[SERIES]}
        otherSeries={null}
        range={{ start: "2026-08-12", end: "2026-08-13" }}
        status="success"
        hasSnapshot
        onRetry={vi.fn()}
        {...overrides}
      />
    </TooltipProvider>
  );
}

describe("UsageTrendChart", () => {
  beforeEach(async () => {
    await i18n.changeLanguage("en");
  });

  it("renders the chart lazily and toggles an equivalent data table", () => {
    renderTrend();
    expect(screen.getByRole("button", { name: "Thirty-day Token usage line chart by model" })).toBeTruthy();
    expect(screen.queryByRole("table")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Data table" }));
    expect(screen.getByRole("table")).toBeTruthy();
    expect(screen.getByText("1,200")).toBeTruthy();
    expect(screen.getByText("· 1 unknown")).toBeTruthy();
    expect(screen.getByText("Estimated 200")).toBeTruthy();
  });

  it("automatically exposes the table when chart initialization fails", () => {
    renderTrend();
    fireEvent.click(screen.getByRole("button", { name: "Thirty-day Token usage line chart by model" }));
    expect(screen.getByRole("table")).toBeTruthy();
    expect(screen.getByText("Chart fallback data")).toBeTruthy();
  });

  it("keeps empty, loading, and error states stable", () => {
    const retry = vi.fn();
    const { rerender, container } = renderTrend({ modelSeries: [] });
    expect(screen.getByText("No model trend data has been recorded for this range.")).toBeTruthy();

    rerender(
      <TooltipProvider>
        <UsageTrendChart modelSeries={[]} otherSeries={null} range={null} status="loading" hasSnapshot={false} onRetry={retry} />
      </TooltipProvider>
    );
    expect(container.querySelector('[data-slot="skeleton"]')).toBeTruthy();

    rerender(
      <TooltipProvider>
        <UsageTrendChart modelSeries={[]} otherSeries={null} range={null} status="error" hasSnapshot={false} onRetry={retry} />
      </TooltipProvider>
    );
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(retry).toHaveBeenCalledOnce();
    expect(screen.getByRole("alert").className).toContain("min-h-[360px]");
  });
});
