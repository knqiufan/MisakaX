import { StrictMode, type ReactNode } from "react";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const clipboardHarness = vi.hoisted(() => ({
  readText: vi.fn<() => Promise<string>>(),
  writeText: vi.fn<(value: string) => Promise<void>>(),
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => clipboardHarness);

const harness = vi.hoisted(() => ({
  instances: [] as Array<Record<string, any>>,
  listeners: new Map<string, (event: { payload: unknown }) => void>(),
  resizeCallbacks: [] as Array<() => void>,
  spawn: vi.fn(),
  writeBytes: vi.fn(),
  resize: vi.fn(),
  kill: vi.fn(),
  getState: vi.fn(),
}));

vi.mock("@xterm/xterm", () => ({
  Terminal: class MockTerminal {
    rows = 24;
    cols = 80;
    options: Record<string, unknown>;
    write = vi.fn();
    focus = vi.fn();
    clear = vi.fn();
    paste = vi.fn();
    dispose = vi.fn();
    hasSelection = vi.fn(() => false);
    getSelection = vi.fn(() => "");
    loadAddon = vi.fn();
    open = vi.fn();
    attachCustomKeyEventHandler = vi.fn();
    dataHandler: ((data: string) => void) | null = null;
    binaryHandler: ((data: string) => void) | null = null;
    selectionHandler: (() => void) | null = null;
    keyHandler: ((event: KeyboardEvent) => boolean) | null = null;
    onData = vi.fn((handler: (data: string) => void) => {
      this.dataHandler = handler;
      return { dispose: vi.fn() };
    });
    onBinary = vi.fn((handler: (data: string) => void) => {
      this.binaryHandler = handler;
      return { dispose: vi.fn() };
    });
    onSelectionChange = vi.fn((handler: () => void) => {
      this.selectionHandler = handler;
      return { dispose: vi.fn() };
    });

    constructor(options: Record<string, unknown>) {
      this.options = options;
      this.attachCustomKeyEventHandler.mockImplementation(
        (handler: (event: KeyboardEvent) => boolean) => {
          this.keyHandler = handler;
        },
      );
      harness.instances.push(this as unknown as Record<string, any>);
    }
  },
}));

vi.mock("@xterm/addon-fit", () => ({
  FitAddon: class MockFitAddon {
    fit = vi.fn();
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, handler: (event: { payload: unknown }) => void) => {
    harness.listeners.set(name, handler);
    return () => {
      if (harness.listeners.get(name) === handler) harness.listeners.delete(name);
    };
  }),
}));

vi.mock("@/lib/ipc/terminal", () => ({
  TERMINAL_OUTPUT_EVENT: "terminal:output",
  TERMINAL_EXITED_EVENT: "terminal:exited",
  terminalIpc: {
    spawn: harness.spawn,
    writeBytes: harness.writeBytes,
    resize: harness.resize,
    kill: harness.kill,
    getState: harness.getState,
  },
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: Record<string, unknown>) => {
      if (key === "terminal.shellSummary") {
        return `${values?.shell} · ${values?.workspace}`;
      }
      if (key === "terminal.exitedWithCode") return `Exited ${values?.code}`;
      if (key === "terminal.workspaceChangedDescription") {
        return `Changed to ${values?.workspace}`;
      }
      return key;
    },
  }),
}));

vi.mock("@/components/ui/tooltip", () => ({
  Tooltip: ({ children }: { children: ReactNode }) => <>{children}</>,
  TooltipTrigger: ({ children }: { children: ReactNode }) => <>{children}</>,
  TooltipContent: ({ children }: { children: ReactNode }) => <>{children}</>,
}));

vi.mock("@/components/ui/dialog", () => ({
  Dialog: ({ open, children }: { open: boolean; children: ReactNode }) =>
    open ? <div role="dialog">{children}</div> : null,
  DialogContent: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  DialogDescription: ({ children }: { children: ReactNode }) => <p>{children}</p>,
  DialogFooter: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  DialogHeader: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  DialogTitle: ({ children }: { children: ReactNode }) => <h2>{children}</h2>,
}));

import { TerminalPanel } from "@/components/chat/workspace/TerminalPanel";
import { resetTerminalRuntimeForTests } from "@/stores/terminal-store";

const SESSION = {
  terminal_id: "terminal-1",
  chat_session_id: "chat-1",
  workspace_generation: 4,
  shell_profile: "powershell",
  shell_name: "PowerShell",
  fallback_reason: null,
  rows: 24,
  cols: 80,
  status: "running" as const,
  started_at: "2026-08-01T00:00:00Z",
};

describe("Workspace W4 TerminalPanel", () => {
  beforeEach(() => {
    resetTerminalRuntimeForTests();
    harness.instances.length = 0;
    harness.listeners.clear();
    harness.resizeCallbacks.length = 0;
    harness.spawn.mockReset().mockResolvedValue(SESSION);
    harness.writeBytes.mockReset().mockResolvedValue(undefined);
    harness.resize.mockReset().mockResolvedValue(undefined);
    harness.kill.mockReset().mockResolvedValue(undefined);
    harness.getState.mockReset();

    Object.defineProperty(globalThis, "ResizeObserver", {
      configurable: true,
      value: class ResizeObserverMock {
        constructor(callback: () => void) {
          harness.resizeCallbacks.push(callback);
        }
        observe() {}
        disconnect() {}
      },
    });
    clipboardHarness.readText.mockReset().mockResolvedValue("");
    clipboardHarness.writeText.mockReset().mockResolvedValue(undefined);
  });

  it("spawns once in StrictMode, disables link activation, and gates binary output", async () => {
    render(
      <StrictMode>
        <TerminalPanel
          active
          chatSessionId="chat-1"
          workspaceGeneration={4}
          workingDir="D:/code/Misaka-Tauri"
          onClose={vi.fn()}
        />
      </StrictMode>,
    );

    await waitFor(() => expect(harness.spawn).toHaveBeenCalledTimes(1));
    const terminal = harness.instances[harness.instances.length - 1];
    expect(terminal.options).toMatchObject({
      allowProposedApi: false,
      allowTransparency: false,
      screenReaderMode: false,
      scrollback: 5_000,
    });
    const preventDefault = vi.fn();
    (terminal.options.linkHandler as { activate: (event: { preventDefault: () => void }) => void })
      .activate({ preventDefault });
    expect(preventDefault).toHaveBeenCalledOnce();

    await waitFor(() => expect(harness.listeners.has("terminal:output")).toBe(true));
    act(() => {
      harness.listeners.get("terminal:output")?.({
        payload: outputEvent(1, 4, "5L2g5aW9"),
      });
      harness.listeners.get("terminal:output")?.({
        payload: outputEvent(1, 4, "aWdub3JlZA=="),
      });
      harness.listeners.get("terminal:output")?.({
        payload: outputEvent(2, 3, "c3RhbGU="),
      });
    });

    expect(terminal.write).toHaveBeenCalledTimes(1);
    expect(Array.from(terminal.write.mock.calls[0][0] as Uint8Array)).toEqual(
      Array.from(new TextEncoder().encode("你好")),
    );
    expect(screen.getByText("PowerShell · Misaka-Tauri")).toBeTruthy();
  });

  it("replays bounded output that races the spawn response", async () => {
    let resolveSpawn!: (session: typeof SESSION) => void;
    harness.spawn.mockReturnValue(
      new Promise<typeof SESSION>((resolve) => {
        resolveSpawn = resolve;
      }),
    );

    render(
      <TerminalPanel
        active
        chatSessionId="chat-1"
        workspaceGeneration={4}
        workingDir="D:/code/Misaka-Tauri"
        onClose={vi.fn()}
      />,
    );
    await waitFor(() => expect(harness.spawn).toHaveBeenCalledOnce());
    await waitFor(() => expect(harness.listeners.has("terminal:output")).toBe(true));
    const terminal = harness.instances[harness.instances.length - 1];

    act(() => {
      harness.listeners.get("terminal:output")?.({
        payload: outputEvent(1, 4, encodeBase64("initial prompt")),
      });
    });
    expect(terminal.write).not.toHaveBeenCalled();

    act(() => resolveSpawn(SESSION));
    await waitFor(() => expect(terminal.write).toHaveBeenCalledOnce());
    expect(new TextDecoder().decode(terminal.write.mock.calls[0][0])).toBe(
      "initial prompt",
    );
  });

  it("announces exit and requires an explicit workspace switch decision", async () => {
    const view = render(
      <TerminalPanel
        active
        chatSessionId="chat-1"
        workspaceGeneration={4}
        workingDir="D:/code/Misaka-Tauri"
        onClose={vi.fn()}
      />,
    );
    await waitFor(() => expect(harness.spawn).toHaveBeenCalledOnce());
    await waitFor(() => expect(harness.listeners.has("terminal:exited")).toBe(true));

    act(() => {
      harness.listeners.get("terminal:exited")?.({ payload: exitEvent(0, 7) });
    });
    expect(screen.getAllByText("Exited 7").length).toBeGreaterThan(0);
    expect(screen.getByRole("button", { name: "terminal.restart" })).toBeTruthy();

    view.rerender(
      <TerminalPanel
        active
        chatSessionId="chat-2"
        workspaceGeneration={8}
        workingDir="D:/code/new-project"
        onClose={vi.fn()}
      />,
    );
    expect(screen.getByRole("dialog")).toBeTruthy();
    expect(screen.getByText("Changed to new-project")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "terminal.keepOld" }));
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(screen.getByText("terminal.retained")).toBeTruthy();
    expect(harness.spawn).toHaveBeenCalledTimes(1);
  });

  it("preserves UTF-8 and binary input, clipboard shortcuts, and cell resize", async () => {
    render(
      <TerminalPanel
        active
        chatSessionId="chat-1"
        workspaceGeneration={4}
        workingDir="D:/code/Misaka-Tauri"
        onClose={vi.fn()}
      />,
    );
    await waitFor(() => expect(harness.spawn).toHaveBeenCalledOnce());
    const terminal = harness.instances[harness.instances.length - 1];

    act(() => {
      terminal.dataHandler?.("中");
      terminal.binaryHandler?.("\u0000\u00ff");
    });
    await waitFor(() => expect(harness.writeBytes).toHaveBeenCalledTimes(2));
    expect(Array.from(harness.writeBytes.mock.calls[0][1] as Uint8Array)).toEqual(
      Array.from(new TextEncoder().encode("中")),
    );
    expect(Array.from(harness.writeBytes.mock.calls[1][1] as Uint8Array)).toEqual([0, 255]);

    terminal.hasSelection.mockReturnValue(true);
    terminal.getSelection.mockReturnValue("selected text");
    act(() => terminal.selectionHandler?.());
    fireEvent.click(screen.getByRole("button", { name: "terminal.copySelection" }));
    await waitFor(() =>
      expect(clipboardHarness.writeText).toHaveBeenCalledWith("selected text"),
    );

    terminal.hasSelection.mockReturnValue(false);
    expect(
      terminal.keyHandler?.({
        ctrlKey: true,
        metaKey: false,
        key: "c",
      } as KeyboardEvent),
    ).toBe(true);

    clipboardHarness.readText.mockResolvedValue("pasted text");
    fireEvent.click(screen.getByRole("button", { name: "terminal.paste" }));
    await waitFor(() => expect(harness.writeBytes).toHaveBeenCalledTimes(3));
    expect(
      new TextDecoder().decode(
        harness.writeBytes.mock.calls[2][1] as Uint8Array,
      ),
    ).toBe("pasted text");

    const host = screen.getByTestId("workspace-terminal").querySelector(
      ".misaka-terminal-viewport",
    ) as HTMLElement;
    Object.defineProperty(host, "clientWidth", { configurable: true, value: 480 });
    terminal.rows = 32;
    terminal.cols = 120;
    act(() => harness.resizeCallbacks.forEach((callback) => callback()));
    await waitFor(
      () =>
        expect(harness.resize).toHaveBeenCalledWith(
          expect.objectContaining({ terminalId: "terminal-1" }),
          32,
          120,
        ),
      { timeout: 500 },
    );
  });

  it("keeps shell, workspace, OSC, and HTML-shaped output out of the DOM", async () => {
    harness.spawn.mockResolvedValue({
      ...SESSION,
      shell_name: "<img src=x onerror=alert(1)>",
    });
    render(
      <TerminalPanel
        active
        chatSessionId="chat-1"
        workspaceGeneration={4}
        workingDir="D:/code/<svg onload=alert(1)>"
        onClose={vi.fn()}
      />,
    );
    await waitFor(() => expect(harness.spawn).toHaveBeenCalledOnce());
    await waitFor(() => expect(harness.listeners.has("terminal:output")).toBe(true));

    const payload =
      "\u001b]0;<img src=x onerror=alert(1)>\u0007" +
      "\u001b]8;;javascript:alert(1)\u0007click\u001b]8;;\u0007";
    act(() => {
      harness.listeners.get("terminal:output")?.({
        payload: outputEvent(1, 4, encodeBase64(payload)),
      });
    });

    const terminal = harness.instances[harness.instances.length - 1];
    expect(terminal.write).toHaveBeenCalledWith(expect.any(Uint8Array));
    expect(document.querySelector("img[src='x'], svg[onload], script")).toBeNull();
    expect(
      screen.getByText("<img src=x onerror=alert(1)> · <svg onload=alert(1)>")
    ).toBeTruthy();
  });
});

function encodeBase64(value: string): string {
  const bytes = new TextEncoder().encode(value);
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function outputEvent(seq: number, generation: number, data: string) {
  return {
    eventId: `output-${seq}`,
    aggregateId: "terminal-1",
    generation,
    occurredAt: "2026-08-01T00:00:00Z",
    payload: { terminal_id: "terminal-1", seq, data_base64: data },
  };
}

function exitEvent(lastSeq: number, exitCode: number) {
  return {
    eventId: "exit-1",
    aggregateId: "terminal-1",
    generation: 4,
    occurredAt: "2026-08-01T00:00:01Z",
    payload: {
      terminal_id: "terminal-1",
      last_seq: lastSeq,
      exit_code: exitCode,
      signal: null,
      reason: "process_exited" as const,
    },
  };
}
