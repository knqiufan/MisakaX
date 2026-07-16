import { describe, it, expect } from "vitest";
import type { Session } from "@/lib/ipc";
import {
  collapseKeyForGroup,
  groupSessionsByWorkspace,
  normalizeWorkingDirectory,
} from "@/components/chat/session/groupSessionsByWorkspace";

function makeSession(overrides: Partial<Session> = {}): Session {
  return {
    id: crypto.randomUUID(),
    title: "Chat",
    model: null,
    system_prompt: null,
    working_directory: null,
    project_name: null,
    workspace_kind: "custom",
    status: "active",
    mode: "agent",
    total_input_tokens: 0,
    total_output_tokens: 0,
    last_message_at: null,
    pinned: false,
    group_name: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

describe("normalizeWorkingDirectory", () => {
  it("normalizes separators and trailing slashes", () => {
    expect(normalizeWorkingDirectory("D:\\code\\App\\")).toBe("d:/code/app");
    expect(normalizeWorkingDirectory("/home/user/proj")).toBe("/home/user/proj");
  });

  it("uses a stable key for null/empty", () => {
    expect(normalizeWorkingDirectory(null)).toBe("__none__");
    expect(normalizeWorkingDirectory(undefined)).toBe("__none__");
  });
});

describe("groupSessionsByWorkspace", () => {
  it("groups by normalized working_directory not project_name alone", () => {
    const sessions = [
      makeSession({
        id: "a",
        working_directory: "D:\\code\\App",
        project_name: "App",
      }),
      makeSession({
        id: "b",
        working_directory: "d:/code/app/",
        project_name: "Different Name",
      }),
      makeSession({
        id: "c",
        working_directory: "/other",
        project_name: "App",
      }),
    ];

    const { workspaces } = groupSessionsByWorkspace(sessions);
    expect(workspaces).toHaveLength(2);
    const appGroup = workspaces.find((w) => w.key === "d:/code/app");
    expect(appGroup?.sessions.map((s) => s.id).sort()).toEqual(["a", "b"]);
  });

  it("keeps pinned sessions inside their workspace, sorted first", () => {
    const sessions = [
      makeSession({
        id: "u1",
        working_directory: "/ws",
        pinned: false,
      }),
      makeSession({
        id: "p1",
        working_directory: "/ws",
        pinned: true,
      }),
      makeSession({
        id: "p2",
        working_directory: "/other",
        pinned: true,
      }),
    ];

    const { workspaces } = groupSessionsByWorkspace(sessions);
    const ws = workspaces.find((w) => w.key === "/ws");
    expect(ws?.ungrouped.map((s) => s.id)).toEqual(["p1", "u1"]);
    expect(workspaces.find((w) => w.key === "/other")?.ungrouped[0]?.id).toBe("p2");
    expect(ws?.sessions.some((s) => s.id === "p2")).toBe(false);
  });

  it("splits manual group_name as secondary subgroups", () => {
    const sessions = [
      makeSession({
        id: "g1",
        working_directory: "/ws",
        group_name: "Feature",
        pinned: true,
      }),
      makeSession({
        id: "g2",
        working_directory: "/ws",
        group_name: "Feature",
      }),
      makeSession({
        id: "u1",
        working_directory: "/ws",
        group_name: null,
      }),
    ];

    const { workspaces } = groupSessionsByWorkspace(sessions);
    const ws = workspaces[0];
    expect(ws.ungrouped.map((s) => s.id)).toEqual(["u1"]);
    expect(ws.manualGroups.get("Feature")?.map((s) => s.id)).toEqual([
      "g1",
      "g2",
    ]);
  });

  it("builds namespaced collapse keys", () => {
    expect(collapseKeyForGroup("d:/code/app", "Feature")).toBe(
      "d:/code/app::Feature"
    );
  });
});
