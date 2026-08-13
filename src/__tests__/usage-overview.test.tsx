import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { TooltipProvider } from "@/components/ui/tooltip";
import { UsageOverviewCards } from "@/features/usage-analytics/UsageOverviewCards";
import {
  formatCompactTokens,
  formatFullTokens,
  parseTokenDecimal,
} from "@/features/usage-analytics/usage-format";
import { i18n } from "@/locales/i18n";
import type { UsageDashboardV1 } from "@/lib/ipc/types";

function dashboard(overrides: Partial<UsageDashboardV1["overview"]> = {}): UsageDashboardV1 {
  return {
    schema_version: 1,
    profile_id: "local",
    generated_at: "2026-08-13T00:00:00Z",
    timezone_mode: "system",
    timezone_id: null,
    utc_offset_minutes: 480,
    range: {
      activity_start: "2025-08-14",
      activity_end: "2026-08-13",
      trend_start: "2026-07-15",
      trend_end: "2026-08-13",
    },
    overview: {
      total_tokens: "1200",
      exact_tokens: "1000",
      estimated_tokens: "150",
      legacy_tokens: "50",
      unknown_operation_count: 2,
      total_days: 7,
      current_streak: 3,
      longest_streak: 5,
      ...overrides,
    },
    daily_activity: [],
    model_series: [],
    other_series: null,
  };
}

function renderCards(props: Partial<React.ComponentProps<typeof UsageOverviewCards>> = {}) {
  return render(
    <TooltipProvider>
      <UsageOverviewCards
        dashboard={dashboard()}
        status="success"
        error={null}
        onRetry={vi.fn()}
        {...props}
      />
    </TooltipProvider>
  );
}

describe("usage overview formatting", () => {
  beforeEach(async () => {
    await i18n.changeLanguage("en");
  });

  it("formats decimal strings with bigint precision", () => {
    expect(parseTokenDecimal("9007199254740993")).toBe(9007199254740993n);
    expect(formatCompactTokens("1200")).toBe("1.2K");
    expect(formatCompactTokens("2300000")).toBe("2.3M");
    expect(formatCompactTokens("4000000000")).toBe("4B");
    expect(formatFullTokens("9007199254740993", "en")).toContain(
      "9,007,199,254,740,993"
    );
    expect(() => parseTokenDecimal("1.2")).toThrow();
  });

  it("renders equal overview cards with compact and accessible full values", () => {
    const { container } = renderCards();
    expect(screen.getByText("1.2K")).toBeTruthy();
    expect(screen.getByLabelText("Total tokens: 1,200 tokens")).toBeTruthy();
    expect(screen.getByText(/2 unknown operations/)).toBeTruthy();
    expect(screen.getByText("Longest streak: 5 days")).toBeTruthy();

    const cards = container.querySelectorAll('[data-slot="card"]');
    expect(cards).toHaveLength(3);
    cards.forEach((card) => {
      expect(card.className).toContain("min-h-40");
      expect(card.className).toContain("bg-card");
      expect(card.className).not.toMatch(/gradient|hover:-translate|hover:scale/);
    });
  });

  it("uses a genuine empty state instead of zero-looking placeholder data", () => {
    const empty = dashboard({
      total_tokens: "0",
      exact_tokens: "0",
      estimated_tokens: "0",
      legacy_tokens: "0",
      unknown_operation_count: 0,
      total_days: 0,
      current_streak: 0,
      longest_streak: 0,
    });
    const { container } = renderCards({ dashboard: empty });
    expect(screen.getAllByText("—")).toHaveLength(3);
    expect(screen.getByText("No model activity has been recorded yet.")).toBeTruthy();
    expect(container.querySelector('[data-overview-state="empty"]')).toBeTruthy();
  });

  it("keeps loading and error heights stable and exposes retry", () => {
    const retry = vi.fn();
    const { rerender, container } = render(
      <TooltipProvider>
        <UsageOverviewCards dashboard={null} status="loading" error={null} onRetry={retry} />
      </TooltipProvider>
    );
    expect(container.querySelectorAll('[data-slot="skeleton"]')).toHaveLength(9);

    rerender(
      <TooltipProvider>
        <UsageOverviewCards dashboard={null} status="error" error="offline" onRetry={retry} />
      </TooltipProvider>
    );
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(retry).toHaveBeenCalledOnce();
    expect(screen.getByRole("alert").className).toContain("min-h-40");
  });

  it("keeps the last good snapshot visible when a refresh fails", () => {
    const retry = vi.fn();
    const { container } = renderCards({ status: "error", error: "offline", onRetry: retry });
    expect(screen.getByText("1.2K")).toBeTruthy();
    expect(container.querySelector("[data-overview-refresh-error]")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(retry).toHaveBeenCalledOnce();
  });
});
