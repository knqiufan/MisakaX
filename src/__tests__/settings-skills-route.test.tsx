import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsSidebar } from "@/components/settings/SettingsSidebar";
import { SESSION_LIST_DEFAULT_WIDTH, useAppStore } from "@/stores/app-store";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("Settings Skills navigation", () => {
  beforeEach(() => {
    useAppStore.setState({
      route: { page: "settings", tab: "mcp" },
      sessionListWidth: SESSION_LIST_DEFAULT_WIDTH,
      globalLoading: false,
    });
  });

  it("places Skills after MCP and preserves the selected deep link", () => {
    render(<SettingsSidebar />);
    const labels = screen.getAllByRole("button").map((button) => button.textContent);
    expect(labels).toEqual([
      "settings:general.title",
      "settings:models.title",
      "settings:mcp.title",
      "settings:skills.title",
      "settings:appearance.title",
      "settings:about.title",
    ]);

    fireEvent.click(screen.getByRole("button", { name: "settings:skills.title" }));
    expect(useAppStore.getState().route).toEqual({ page: "settings", tab: "skills" });
    expect(screen.getByRole("button", { name: "settings:skills.title" }).getAttribute("aria-current")).toBe("page");
  });
});
