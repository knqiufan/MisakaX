import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ProviderDialog } from "@/pages/settings/ProviderDialog";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (_key: string, fallback?: string) => fallback ?? _key,
  }),
}));

class ResizeObserverMock {
  observe() {}
  unobserve() {}
  disconnect() {}
}

vi.stubGlobal("ResizeObserver", ResizeObserverMock);

describe("ProviderDialog layout", () => {
  it("keeps the outer dialog non-scrollable when a max-token preset receives focus", () => {
    render(
      <ProviderDialog
        open
        onOpenChange={vi.fn()}
        onSubmit={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Advanced configuration" }));
    fireEvent.click(screen.getByRole("radio", { name: "1M" }));

    const dialog = screen.getByRole("dialog");
    expect(dialog.className).toContain("overflow-clip");
    expect(dialog.className).not.toContain("overflow-hidden");
    expect(screen.getByRole("radio", { name: "1M" })).toHaveProperty("checked", true);
  });
});
