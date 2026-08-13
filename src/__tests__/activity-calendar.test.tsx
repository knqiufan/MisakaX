import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { TooltipProvider } from "@/components/ui/tooltip";
import { ActivityCalendar } from "@/features/usage-analytics/ActivityCalendar";
import { parseLocalDate } from "@/features/usage-analytics/usage-calendar";
import { i18n } from "@/locales/i18n";
import type { DailyUsageV1 } from "@/lib/ipc/types";

function days(count = 14): DailyUsageV1[] {
  const first = parseLocalDate("2026-07-01");
  return Array.from({ length: count }, (_, index) => {
    const date = new Date(first);
    date.setUTCDate(first.getUTCDate() + index);
    const unknown = index === 3;
    return {
      local_date: date.toISOString().slice(0, 10),
      total_tokens: unknown ? null : String(index * 100),
      input_tokens: unknown ? null : String(index * 60),
      output_tokens: unknown ? null : String(index * 40),
      operation_count: index === 0 ? 0 : 1,
      primary_model: index === 0 ? null : "model-a",
      quality: {
        exact_tokens: unknown ? "0" : String(index * 100),
        estimated_tokens: "0",
        legacy_tokens: "0",
        unknown_operation_count: unknown ? 1 : 0,
      },
    };
  });
}

function renderCalendar(overrides: Partial<React.ComponentProps<typeof ActivityCalendar>> = {}) {
  return render(
    <TooltipProvider delayDuration={0}>
      <ActivityCalendar
        days={days()}
        weekStart={1}
        status="success"
        hasSnapshot
        onRetry={vi.fn()}
        {...overrides}
      />
    </TooltipProvider>
  );
}

describe("ActivityCalendar", () => {
  beforeEach(async () => {
    await i18n.changeLanguage("en");
  });

  it("exposes grid semantics, one roving tab stop, quality texture, and no fake tabs", () => {
    const { container } = renderCalendar();
    expect(screen.getByRole("grid", { name: "Daily Token activity for the last 365 days" })).toBeTruthy();
    const cells = screen.getAllByRole("gridcell");
    expect(cells).toHaveLength(14);
    expect(cells.filter((cell) => cell.getAttribute("tabindex") === "0")).toHaveLength(1);
    expect(container.querySelector('[data-visual-state="unknown"].usage-cell-unknown')).toBeTruthy();
    expect(screen.queryAllByRole("tab")).toHaveLength(0);
    expect(container.querySelector("[data-activity-scroll-region]")?.className).toContain("overflow-x-auto");
  });

  it("moves by week with arrow keys and reveals the same data to keyboard focus", async () => {
    renderCalendar();
    const last = document.querySelector<HTMLButtonElement>('[data-local-date="2026-07-14"]');
    const previousWeek = document.querySelector<HTMLButtonElement>('[data-local-date="2026-07-07"]');
    expect(last).toBeTruthy();
    last?.focus();
    fireEvent.keyDown(last as HTMLButtonElement, { key: "ArrowLeft" });
    expect(document.activeElement).toBe(previousWeek);

    await waitFor(() => expect(screen.getAllByText("July 7, 2026").length).toBeGreaterThan(0));
    expect(screen.getAllByText(/Primary model: model-a/).length).toBeGreaterThan(0);
  });

  it("provides the date-sorted accessible table and stable loading/error states", () => {
    const retry = vi.fn();
    const { rerender, container } = renderCalendar();
    expect(screen.getByRole("table")).toBeTruthy();
    expect(screen.getAllByRole("row")).toHaveLength(15);

    rerender(
      <TooltipProvider delayDuration={0}>
        <ActivityCalendar days={[]} weekStart={1} status="loading" hasSnapshot={false} onRetry={retry} />
      </TooltipProvider>
    );
    expect(container.querySelector('[data-slot="skeleton"]')).toBeTruthy();

    rerender(
      <TooltipProvider delayDuration={0}>
        <ActivityCalendar days={[]} weekStart={1} status="error" hasSnapshot={false} onRetry={retry} />
      </TooltipProvider>
    );
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(retry).toHaveBeenCalledOnce();
    expect(screen.getByRole("alert").className).toContain("min-h-80");
  });
});
