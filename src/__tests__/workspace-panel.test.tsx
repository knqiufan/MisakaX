import { fireEvent, render, screen } from "@testing-library/react";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { WorkspaceBar } from "@/components/chat/workspace/WorkspaceBar";
import { WorkspacePanel } from "@/components/chat/workspace/WorkspacePanel";
import {
  resolveWorkspacePanelPresentation,
  WORKSPACE_PANEL_LAYOUT,
} from "@/components/chat/workspace/workspacePanelLayout";
import { resolveWorkspacePanelShortcut } from "@/components/chat/workspace/workspacePanelShortcuts";
import { ToolActionsGroup } from "@/components/chat/message/ToolActionsGroup";
import { useWorkspaceExplorerStore } from "@/stores/workspace-explorer-store";
import {
  LEGACY_WORKSPACE_EXPLORER_STORAGE_KEY,
  migrateLegacyWorkspacePanelStorage,
  useWorkspacePanelStore,
  WORKSPACE_PANEL_STORAGE_KEY,
} from "@/stores/workspace-panel-store";
import type { ToolCall } from "@/lib/ipc";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) =>
      ({
        switchDir: "Switch directory",
        openExplorer: "Open explorer",
        openExplorerShortcut: "Open explorer · Ctrl+Shift+E",
        openTerminal: "Open terminal",
        openTerminalShortcut: "Open terminal · Ctrl+`",
        defaultWorkspaceName: "Default workspace",
        "terminal.title": "Terminal · Local permissions",
        "terminal.collapse": "Collapse terminal",
        "terminal.preparing": "Terminal disabled",
        "toolGroup.done": "1 tool completed",
        "toolGroup.openLogs": "Open tool logs",
      })[key] ?? key,
  }),
}));

vi.mock("@/components/ui/tooltip", () => ({
  Tooltip: ({ children }: { children: ReactNode }) => <>{children}</>,
  TooltipTrigger: ({ children }: { children: ReactNode }) => <>{children}</>,
  TooltipContent: ({ children }: { children: ReactNode }) => <span>{children}</span>,
}));

vi.mock("@/components/chat/workspace/WorkspaceExplorer", () => ({
  WorkspaceExplorer: () => <div data-testid="explorer-content">Explorer</div>,
}));

const COMPLETE_TOOL_CALL: ToolCall = {
  id: "tool-1",
  server_id: "server-1",
  tool_name: "read_file",
  server_name: "workspace",
  arguments: {},
  result: null,
  status: "complete",
  error: null,
  started_at: 1,
  completed_at: 2,
};

describe("Workspace W2 panel contract", () => {
  beforeEach(() => {
    localStorage.clear();
    useWorkspacePanelStore.setState({
      open: false,
      mode: "explorer",
      size: 30,
      sessionId: null,
      workspaceGeneration: 0,
    });
    useWorkspaceExplorerStore.setState({ tabs: [], activePath: null });
  });

  it("routes Explorer and gated Terminal through one mode action", () => {
    const toggle = vi.fn();
    const { rerender } = render(
      <WorkspaceBar
        workingDir="D:/code/Misaka-Tauri"
        onChangeDir={vi.fn()}
        onTogglePanel={toggle}
        terminalEnabled
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Open explorer" }));
    fireEvent.click(screen.getByRole("button", { name: "Open terminal" }));
    expect(toggle.mock.calls).toEqual([["explorer"], ["terminal"]]);
    expect(screen.queryByRole("button", { name: "Tool logs" })).toBeNull();
    expect(
      screen
        .getByRole("button", { name: "Open explorer" })
        .getAttribute("aria-keyshortcuts"),
    ).toBe("Control+Shift+E Meta+Shift+E");

    rerender(
      <WorkspaceBar
        workingDir="D:/code/Misaka-Tauri"
        onChangeDir={vi.fn()}
        onTogglePanel={toggle}
        panelOpen
        panelMode="terminal"
        terminalEnabled
      />,
    );
    expect(
      screen
        .getByRole("button", { name: "Open terminal" })
        .getAttribute("aria-pressed"),
    ).toBe("true");
  });

  it("migrates the legacy Explorer open preference exactly once", () => {
    const memory = new Map<string, string>([
      [
        LEGACY_WORKSPACE_EXPLORER_STORAGE_KEY,
        JSON.stringify({ state: { open: true }, version: 0 }),
      ],
    ]);
    const storage = {
      getItem: (key: string) => memory.get(key) ?? null,
      setItem: (key: string, value: string) => memory.set(key, value),
      removeItem: (key: string) => memory.delete(key),
    };

    expect(migrateLegacyWorkspacePanelStorage(storage)).toBe(true);
    expect(memory.has(LEGACY_WORKSPACE_EXPLORER_STORAGE_KEY)).toBe(false);
    expect(JSON.parse(memory.get(WORKSPACE_PANEL_STORAGE_KEY)!)).toEqual({
      state: { open: true, mode: "explorer", size: 30 },
      version: 1,
    });
    expect(migrateLegacyWorkspacePanelStorage(storage)).toBe(false);
  });

  it("persists panel size, clamps resize, and keeps Explorer tabs separate", () => {
    useWorkspaceExplorerStore
      .getState()
      .openTab("README.md", "hello");
    const panel = useWorkspacePanelStore.getState();
    panel.setSize(72);
    panel.toggle("explorer");
    panel.toggle("terminal");

    expect(useWorkspacePanelStore.getState()).toMatchObject({
      open: true,
      mode: "terminal",
      size: 55,
    });
    expect(useWorkspaceExplorerStore.getState()).toMatchObject({
      activePath: "README.md",
      tabs: [expect.objectContaining({ path: "README.md" })],
    });
    expect(localStorage.getItem(WORKSPACE_PANEL_STORAGE_KEY)).toContain(
      '"size":55',
    );
  });

  it("binds session generations monotonically and handles rapid toggles", () => {
    const store = useWorkspacePanelStore.getState();
    store.bindSession("session-a", 4);
    store.bindSession("session-a", 2);
    expect(useWorkspacePanelStore.getState().workspaceGeneration).toBe(4);
    store.bindSession("session-a", 0);
    store.toggle("explorer");
    store.toggle("explorer");
    store.toggle("terminal");
    store.bindSession("session-b", 1);

    expect(useWorkspacePanelStore.getState()).toMatchObject({
      sessionId: "session-b",
      workspaceGeneration: 1,
      open: true,
      mode: "terminal",
    });
  });

  it("keeps a visited Terminal slot mounted while Explorer is visible", () => {
    let mounts = 0;
    function TerminalFixture() {
      useState(() => {
        mounts += 1;
        return 0;
      });
      return <div data-testid="terminal-content">Terminal</div>;
    }

    const { rerender } = render(
      <WorkspacePanel
        mode="terminal"
        chatSessionId="session-a"
        workspaceGeneration={4}
        workingDir="D:/code/Misaka-Tauri"
        terminalContent={<TerminalFixture />}
        onClose={vi.fn()}
      />,
    );
    rerender(
      <WorkspacePanel
        mode="explorer"
        chatSessionId="session-a"
        workspaceGeneration={4}
        workingDir="D:/code/Misaka-Tauri"
        terminalContent={<TerminalFixture />}
        onClose={vi.fn()}
      />,
    );

    expect(mounts).toBe(1);
    expect(
      screen.getByTestId("terminal-content").parentElement?.hasAttribute("hidden"),
    ).toBe(true);
    expect(
      screen.getByTestId("explorer-content").parentElement?.hasAttribute("hidden"),
    ).toBe(false);
  });

  it("moves Tool Logs to an explicit message tool-group action", () => {
    const openLogs = vi.fn();
    render(
      <ToolActionsGroup
        toolCalls={[COMPLETE_TOOL_CALL]}
        onOpenToolLogs={openLogs}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Open tool logs" }));
    expect(openLogs).toHaveBeenCalledOnce();
  });

  it("uses overlay on narrow windows and resolves keyboard toggles", () => {
    expect(resolveWorkspacePanelPresentation(960)).toBe("overlay");
    expect(resolveWorkspacePanelPresentation(1280)).toBe("split");
    expect(WORKSPACE_PANEL_LAYOUT).toMatchObject({
      panelDefaultSize: 30,
      panelMinSize: 18,
      panelMaxSize: 55,
    });
    expect(
      resolveWorkspacePanelShortcut(
        {
          key: "E",
          code: "KeyE",
          ctrlKey: true,
          metaKey: false,
          shiftKey: true,
          defaultPrevented: false,
        },
        { terminalEnabled: false, panelOpen: false, presentation: "split" },
      ),
    ).toBe("explorer");
    expect(
      resolveWorkspacePanelShortcut(
        {
          key: "Escape",
          code: "Escape",
          ctrlKey: false,
          metaKey: false,
          shiftKey: false,
          defaultPrevented: false,
        },
        { terminalEnabled: false, panelOpen: true, presentation: "overlay" },
      ),
    ).toBe("close");
    expect(
      resolveWorkspacePanelShortcut(
        {
          key: "`",
          code: "Backquote",
          ctrlKey: true,
          metaKey: false,
          shiftKey: false,
          defaultPrevented: false,
        },
        { terminalEnabled: true, panelOpen: false, presentation: "split" },
      ),
    ).toBe("terminal");
  });
});
