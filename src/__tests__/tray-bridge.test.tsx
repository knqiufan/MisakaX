import type {
  ButtonHTMLAttributes,
  InputHTMLAttributes,
  LabelHTMLAttributes,
  ReactNode,
} from "react";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Session } from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";
import { useChatStore } from "@/stores/chat-store";

const mocks = vi.hoisted(() => ({
  handlers: new Map<string, (event: { payload: unknown }) => void>(),
  listen: vi.fn(),
  updateContext: vi.fn(),
  resolveCloseRequest: vi.fn(),
  createSession: vi.fn(),
  toastError: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (event: string, handler: (event: { payload: unknown }) => void) => {
    mocks.handlers.set(event, handler);
    mocks.listen(event, handler);
    return Promise.resolve(() => mocks.handlers.delete(event));
  },
}));

vi.mock("@/lib/ipc", () => ({
  sessionsIpc: { create: mocks.createSession },
  trayIpc: {
    updateContext: mocks.updateContext,
    resolveCloseRequest: mocks.resolveCloseRequest,
  },
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) =>
      ({
        closeTitle: "Close MisakaX?",
        closeDescription: "Choose how to close.",
        rememberChoice: "Remember my choice",
        cancel: "Cancel",
        minimize: "Minimize to tray",
        quit: "Quit MisakaX",
        newTaskFailed: "Couldn't create a new task",
        openProjectFailed: "Couldn't open project",
      })[key] ?? key,
  }),
}));

vi.mock("sonner", () => ({
  toast: { error: mocks.toastError },
}));

vi.mock("@/components/ui/dialog", () => ({
  Dialog: ({ open, children }: { open: boolean; children: ReactNode }) =>
    open ? <div>{children}</div> : null,
  DialogContent: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  DialogDescription: ({ children }: { children: ReactNode }) => <p>{children}</p>,
  DialogFooter: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  DialogHeader: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  DialogTitle: ({ children }: { children: ReactNode }) => <h2>{children}</h2>,
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({ children, ...props }: ButtonHTMLAttributes<HTMLButtonElement> & { children?: ReactNode }) => (
    <button type="button" {...props}>{children}</button>
  ),
}));

vi.mock("@/components/ui/label", () => ({
  Label: ({ children, ...props }: LabelHTMLAttributes<HTMLLabelElement>) => (
    <label {...props}>{children}</label>
  ),
}));

vi.mock("@/components/ui/switch", () => ({
  Switch: ({
    checked,
    onCheckedChange,
    ...props
  }: InputHTMLAttributes<HTMLInputElement> & {
    checked: boolean;
    onCheckedChange: (checked: boolean) => void;
  }) => (
    <input
      type="checkbox"
      checked={checked}
      onChange={(event) => onCheckedChange(event.target.checked)}
      {...props}
    />
  ),
}));

import { TrayBridge } from "@/components/layout/TrayBridge";

const activeSession = (): Session => ({
  id: "session-alpha",
  title: "Alpha task",
  model: null,
  system_prompt: null,
  working_directory: "C:/work/alpha",
  project_name: "Alpha",
  workspace_kind: "custom",
  status: "active",
  mode: "chat",
  total_input_tokens: 0,
  total_output_tokens: 0,
  last_message_at: null,
  pinned: false,
  group_name: null,
  created_at: "2026-07-23T00:00:00Z",
  updated_at: "2026-07-23T00:00:00Z",
});

async function emit(event: string, payload: unknown = undefined) {
  await act(async () => {
    mocks.handlers.get(event)?.({ payload });
  });
}

describe("TrayBridge", () => {
  beforeEach(() => {
    mocks.handlers.clear();
    vi.clearAllMocks();
    mocks.updateContext.mockResolvedValue(undefined);
    mocks.resolveCloseRequest.mockResolvedValue(undefined);
    mocks.createSession.mockResolvedValue(activeSession());
    useAppStore.setState({ route: { page: "chat" } });
    useChatStore.setState({
      sessions: [],
      activeSession: null,
      activeSessionId: null,
      showWorkspaceSelector: false,
      workspaceSelectorIntent: null,
      workspaceSelectorTargetSessionId: null,
    });
  });

  afterEach(() => {
    cleanup();
    mocks.handlers.clear();
  });

  it("synchronizes active project context and routes native menu actions", async () => {
    const view = render(<TrayBridge />);
    await waitFor(() => expect(mocks.listen).toHaveBeenCalledTimes(4));
    expect(mocks.updateContext).toHaveBeenLastCalledWith({
      workingDirectory: null,
      projectName: null,
      workspaceKind: null,
    });

    useChatStore.setState({
      activeSession: activeSession(),
      activeSessionId: "session-alpha",
    });
    view.rerender(<TrayBridge />);
    await waitFor(() => {
      expect(mocks.updateContext).toHaveBeenLastCalledWith({
        workingDirectory: "C:/work/alpha",
        projectName: "Alpha",
        workspaceKind: "custom",
      });
    });

    await emit("tray:new-task", { workingDirectory: "C:/work/alpha" });
    await waitFor(() => {
      expect(mocks.createSession).toHaveBeenCalledWith({
        workingDirectory: "C:/work/alpha",
      });
    });
    expect(useChatStore.getState().activeSessionId).toBe("session-alpha");

    await emit("tray:open-settings");
    expect(useAppStore.getState().route).toEqual({
      page: "settings",
      tab: "general",
    });

    await emit("tray:new-task", {});
    expect(useChatStore.getState().showWorkspaceSelector).toBe(true);
    expect(useChatStore.getState().workspaceSelectorIntent).toBe("new-session");
  });

  it("offers cancel, remembered minimize, and quit close outcomes", async () => {
    render(<TrayBridge />);
    await waitFor(() => expect(mocks.listen).toHaveBeenCalledTimes(4));

    await emit("tray:close-requested");
    expect(screen.getByText("Close MisakaX?")).toBeTruthy();
    fireEvent.click(screen.getByText("Cancel"));
    expect(screen.queryByText("Close MisakaX?")).toBeNull();
    expect(mocks.resolveCloseRequest).not.toHaveBeenCalled();

    await emit("tray:close-requested");
    fireEvent.click(screen.getByLabelText("Remember my choice"));
    fireEvent.click(screen.getByText("Minimize to tray"));
    await waitFor(() => {
      expect(mocks.resolveCloseRequest).toHaveBeenCalledWith(
        "minimize_to_tray",
        true
      );
    });

    await emit("tray:close-requested");
    fireEvent.click(screen.getByText("Quit MisakaX"));
    await waitFor(() => {
      expect(mocks.resolveCloseRequest).toHaveBeenLastCalledWith("quit", false);
    });
  });
});
