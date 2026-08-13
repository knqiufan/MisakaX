import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { clearHistory } = vi.hoisted(() => ({ clearHistory: vi.fn() }));
vi.mock("@/lib/ipc/usage", () => ({ usageIpc: { clearHistory } }));

import { UsageDataControls } from "@/features/usage-analytics/UsageDataControls";
import { i18n } from "@/locales/i18n";

describe("UsageDataControls", () => {
  beforeEach(async () => {
    await i18n.changeLanguage("en");
    clearHistory.mockReset();
  });

  it("explains preservation semantics and refreshes after an atomic clear", async () => {
    const onCleared = vi.fn().mockResolvedValue(undefined);
    clearHistory.mockResolvedValue(3);
    render(<UsageDataControls onCleared={onCleared} />);
    fireEvent.click(screen.getByRole("button", { name: "Clear usage history" }));
    expect(await screen.findByText(/Chat messages and their stored per-message Token badges remain/)).toBeTruthy();
    fireEvent.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Clear usage history",
      })
    );
    await waitFor(() => expect(clearHistory).toHaveBeenCalledOnce());
    await waitFor(() => expect(onCleared).toHaveBeenCalledOnce());
    expect(screen.getByText(/Cleared 3 usage events/)).toBeTruthy();
  });

  it("keeps the dialog open with a stable rollback error", async () => {
    clearHistory.mockRejectedValue(new Error("transaction failed"));
    render(<UsageDataControls onCleared={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Clear usage history" }));
    fireEvent.click(
      within(screen.getByRole("dialog")).getByRole("button", {
        name: "Clear usage history",
      })
    );
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.getByRole("dialog")).toBeTruthy();
  });
});
