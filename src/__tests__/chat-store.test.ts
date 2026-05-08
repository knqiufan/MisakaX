import { describe, it, expect, beforeEach } from "vitest";
import { useChatStore } from "@/stores/chat-store";

describe("useChatStore", () => {
  beforeEach(() => {
    useChatStore.setState({
      sessions: [],
      activeSessionId: null,
      loading: false,
    });
  });

  it("should have correct initial state", () => {
    const state = useChatStore.getState();
    expect(state.sessions).toEqual([]);
    expect(state.activeSessionId).toBeNull();
    expect(state.loading).toBe(false);
  });

  it("should set active session", () => {
    useChatStore.getState().setActiveSession("session-1");
    expect(useChatStore.getState().activeSessionId).toBe("session-1");
  });

  it("should clear active session", () => {
    useChatStore.getState().setActiveSession("session-1");
    useChatStore.getState().setActiveSession(null);
    expect(useChatStore.getState().activeSessionId).toBeNull();
  });

  it("should set sessions list", () => {
    const sessions = [
      {
        id: "s1",
        title: "Test Chat",
        model: "gpt-4o",
        system_prompt: null,
        working_directory: null,
        project_name: null,
        status: "active",
        mode: "agent",
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
      },
      {
        id: "s2",
        title: "Another Chat",
        model: "claude-sonnet-4-20250514",
        system_prompt: "You are helpful",
        working_directory: "/home/user",
        project_name: "my-project",
        status: "active",
        mode: "agent",
        created_at: "2026-01-02T00:00:00Z",
        updated_at: "2026-01-02T00:00:00Z",
      },
    ];

    useChatStore.getState().setSessions(sessions);
    expect(useChatStore.getState().sessions).toHaveLength(2);
    expect(useChatStore.getState().sessions[0].title).toBe("Test Chat");
    expect(useChatStore.getState().sessions[1].id).toBe("s2");
  });

  it("should replace sessions list entirely", () => {
    useChatStore.getState().setSessions([
      {
        id: "old",
        title: "Old",
        model: null,
        system_prompt: null,
        working_directory: null,
        project_name: null,
        status: "active",
        mode: "agent",
        created_at: "",
        updated_at: "",
      },
    ]);

    useChatStore.getState().setSessions([]);
    expect(useChatStore.getState().sessions).toHaveLength(0);
  });

  it("should set loading state", () => {
    useChatStore.getState().setLoading(true);
    expect(useChatStore.getState().loading).toBe(true);
    useChatStore.getState().setLoading(false);
    expect(useChatStore.getState().loading).toBe(false);
  });

  it("should maintain independent state between actions", () => {
    useChatStore.getState().setActiveSession("s1");
    useChatStore.getState().setLoading(true);

    expect(useChatStore.getState().activeSessionId).toBe("s1");
    expect(useChatStore.getState().loading).toBe(true);
    expect(useChatStore.getState().sessions).toEqual([]);
  });
});
