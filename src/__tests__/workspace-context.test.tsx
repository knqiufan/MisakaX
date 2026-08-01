import { act, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { DomainEvent, WorkspaceContext } from "@/lib/ipc/contracts";
import { WorkspaceContextBadge, middleEllipsis } from "@/components/chat/composer/WorkspaceContextBadge";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listeners: new Map<string, (event: { payload: unknown }) => void>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, callback: (event: { payload: unknown }) => void) => {
    mocks.listeners.set(name, callback);
    return () => {
      if (mocks.listeners.get(name) === callback) mocks.listeners.delete(name);
    };
  }),
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, string>) =>
      ({
        "context.loading": "Loading workspace context",
        "context.localProject": "Local project",
        "context.localHint": "This workspace is not a Git work tree.",
        "context.gitUnavailable": `Git unavailable ${params?.id ?? ""}`,
        "context.ariaLabel": `Workspace context: ${params?.value ?? ""}`,
      }[key] ?? key),
  }),
}));
vi.mock("@/components/ui/tooltip", () => ({
  Tooltip: ({ children }: { children: ReactNode }) => <>{children}</>,
  TooltipTrigger: ({ children }: { children: ReactNode }) => <>{children}</>,
  TooltipContent: ({ children }: { children: ReactNode }) => <span>{children}</span>,
}));

const gitContext = (overrides: Partial<WorkspaceContext> = {}): WorkspaceContext => ({
  workspace_path: "D:/code/project",
  kind: "git",
  repository_root: "D:/code/project",
  branch: "main",
  detached_head: null,
  generation: 1,
  diagnostic: null,
  ...overrides,
});

describe("WorkspaceContextBadge", () => {
  beforeEach(() => {
    mocks.invoke.mockReset();
    mocks.listeners.clear();
  });

  it("renders a read-only branch badge and invokes only the fixed context command", async () => {
    mocks.invoke.mockResolvedValue(gitContext({ branch: "feature/workspace-context" }));

    render(
      <WorkspaceContextBadge
        chatSessionId="session-1"
        workingDirectory="D:/code/project"
      />,
    );

    await waitFor(() =>
      expect(screen.getByTestId("workspace-context-badge").textContent).toContain(
        "feature/workspace-context",
      ),
    );
    expect(mocks.invoke).toHaveBeenCalledWith("workspace_get_context", {
      chatSessionId: "session-1",
      refresh: false,
    });
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("labels detached HEAD without presenting it as a branch", async () => {
    mocks.invoke.mockResolvedValue(
      gitContext({ branch: null, detached_head: "a1b2c3d" }),
    );
    render(
      <WorkspaceContextBadge
        chatSessionId="session-1"
        workingDirectory="D:/code/project"
      />,
    );

    await waitFor(() =>
      expect(screen.getByTestId("workspace-context-badge").textContent).toContain(
        "detached:a1b2c3d",
      ),
    );
  });

  it("falls back to a local project with a path-free diagnostic", async () => {
    mocks.invoke.mockResolvedValue(
      gitContext({
        kind: "local",
        repository_root: null,
        branch: null,
        diagnostic: {
          code: "GIT_NOT_AVAILABLE",
          message_key: "workspace.gitUnavailable",
          retryable: true,
          correlation_id: "deadbeef-private-path-is-not-exposed",
        },
      }),
    );
    render(
      <WorkspaceContextBadge
        chatSessionId="session-1"
        workingDirectory="D:/secret/project"
      />,
    );

    expect(await screen.findByText("Local project")).toBeTruthy();
    expect(screen.getByText("Git unavailable deadbeef")).toBeTruthy();
    expect(screen.queryByText(/D:\/secret/)).toBeNull();
  });

  it("drops late requests and older watcher generations during rapid switching", async () => {
    let resolveFirst!: (context: WorkspaceContext) => void;
    const first = new Promise<WorkspaceContext>((resolve) => {
      resolveFirst = resolve;
    });
    mocks.invoke.mockImplementation(
      (_command: string, args: { chatSessionId: string }) =>
        args.chatSessionId === "session-1"
          ? first
          : Promise.resolve(gitContext({ branch: "second", generation: 7 })),
    );
    const { rerender } = render(
      <WorkspaceContextBadge
        chatSessionId="session-1"
        workingDirectory="D:/first"
      />,
    );
    rerender(
      <WorkspaceContextBadge
        chatSessionId="session-2"
        workingDirectory="D:/second"
      />,
    );
    await waitFor(() =>
      expect(screen.getByTestId("workspace-context-badge").textContent).toContain(
        "second",
      ),
    );

    await act(async () => {
      resolveFirst(gitContext({ branch: "late-first", generation: 1 }));
      await first;
    });
    expect(screen.queryByText("late-first")).toBeNull();

    const listener = mocks.listeners.get("workspace.context.changed");
    expect(listener).toBeDefined();
    const staleEvent: DomainEvent<WorkspaceContext> = {
      eventId: "event-stale",
      aggregateId: "session-2",
      generation: 6,
      occurredAt: "2026-08-01T00:00:00Z",
      payload: gitContext({ branch: "stale", generation: 6 }),
    };
    act(() => listener?.({ payload: staleEvent }));
    expect(screen.queryByText("stale")).toBeNull();

    const freshEvent: DomainEvent<WorkspaceContext> = {
      ...staleEvent,
      eventId: "event-fresh",
      generation: 8,
      payload: gitContext({ branch: "fresh", generation: 8 }),
    };
    act(() => listener?.({ payload: freshEvent }));
    await waitFor(() =>
      expect(screen.getByTestId("workspace-context-badge").textContent).toContain(
        "fresh",
      ),
    );
  });

  it("uses middle ellipsis for long branch names", () => {
    expect(middleEllipsis("feature/a-very-long-branch-name", 18)).toBe(
      "feature/a…nch-name",
    );
  });
});
