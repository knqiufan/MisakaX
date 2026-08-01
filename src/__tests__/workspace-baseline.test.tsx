import { fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { WorkspaceBar } from "@/components/chat/workspace/WorkspaceBar";
import { LEGACY_CHAT_PANEL_LAYOUT } from "@/components/chat/workspace/workspacePanelLayout";
import { useWorkspaceExplorerStore } from "@/stores/workspace-explorer-store";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => ({
      switchDir: "Switch directory",
      openExplorer: "Open explorer",
      "explorer.toolLogs": "Tool logs",
      defaultWorkspaceName: "Default workspace",
    }[key] ?? key),
  }),
}));

vi.mock("@/components/ui/tooltip", () => ({
  Tooltip: ({ children }: { children: ReactNode }) => <>{children}</>,
  TooltipTrigger: ({ children }: { children: ReactNode }) => <>{children}</>,
  TooltipContent: ({ children }: { children: ReactNode }) => <span>{children}</span>,
}));

describe("Workspace W0 behavior baseline", () => {
  beforeEach(() => {
    useWorkspaceExplorerStore.setState({ open: false, tabs: [], activePath: null });
  });

  it("keeps the Terminal glyph wired to Tool Logs before W2", () => {
    const toggleExplorer = vi.fn();
    const toggleToolLogs = vi.fn();
    render(
      <WorkspaceBar
        workingDir="D:/code/Misaka-Tauri"
        onChangeDir={vi.fn()}
        onToggleExplorer={toggleExplorer}
        onToggleToolLogs={toggleToolLogs}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Open explorer" }));
    fireEvent.click(screen.getByRole("button", { name: "Tool logs" }));
    expect(toggleExplorer).toHaveBeenCalledOnce();
    expect(toggleToolLogs).toHaveBeenCalledOnce();
  });

  it("preserves explorer tabs when the right panel is closed and reopened", () => {
    const store = useWorkspaceExplorerStore.getState();
    store.openTab("README.md", "hello");
    store.setOpen(true);
    store.setOpen(false);
    store.setOpen(true);

    const state = useWorkspaceExplorerStore.getState();
    expect(state.open).toBe(true);
    expect(state.tabs).toHaveLength(1);
    expect(state.activePath).toBe("README.md");
  });

  it("freezes the resizable Explorer bounds before WorkspacePanel migration", () => {
    expect(LEGACY_CHAT_PANEL_LAYOUT).toEqual({
      chatDefaultSize: "70%",
      chatMinSize: "55%",
      explorerDefaultSize: "30%",
      explorerMinSize: "18%",
      explorerMaxSize: "55%",
    });
  });
});
