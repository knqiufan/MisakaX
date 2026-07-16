import { describe, it, expect, beforeEach } from "vitest";
import { useChatStore } from "@/stores/chat-store";
import type { Message } from "@/lib/ipc";

const makeMessage = (overrides: Partial<Message> = {}): Message => ({
  id: crypto.randomUUID(),
  session_id: "test-session",
  role: "user",
  content: "Hello",
  token_usage: null,
  model: null,
  thinking_content: null,
  attachments: null,
  status: "complete",
  created_at: new Date().toISOString(),
  ...overrides,
});

describe("useChatStore", () => {
  beforeEach(() => {
    useChatStore.setState({
      sessions: [],
      activeSessionId: null,
      activeSession: null,
      loading: false,
      showWorkspaceSelector: false,
      workspaceSelectorIntent: null,
      workspaceSelectorTargetSessionId: null,
      thinkingEnabled: true,
      researchEnabled: false,
      messages: [],
      isStreaming: false,
      streamingMessageId: null,
      isThinkingStreaming: false,
      selectedModel: null,
    });
    try {
      localStorage.removeItem("misakax:thinkingEnabled");
      localStorage.removeItem("misakax:researchEnabled");
    } catch { /* noop */ }
  });

  it("should have correct initial state", () => {
    const state = useChatStore.getState();
    expect(state.sessions).toEqual([]);
    expect(state.activeSessionId).toBeNull();
    expect(state.loading).toBe(false);
    expect(state.messages).toEqual([]);
    expect(state.isStreaming).toBe(false);
    expect(state.streamingMessageId).toBeNull();
    expect(state.selectedModel).toBeNull();
    expect(state.thinkingEnabled).toBe(true);
    expect(state.workspaceSelectorIntent).toBeNull();
  });

  describe("thinkingEnabled", () => {
    it("should persist thinkingEnabled to localStorage", () => {
      useChatStore.getState().setThinkingEnabled(false);
      expect(useChatStore.getState().thinkingEnabled).toBe(false);
      expect(localStorage.getItem("misakax:thinkingEnabled")).toBe("false");

      useChatStore.getState().setThinkingEnabled(true);
      expect(useChatStore.getState().thinkingEnabled).toBe(true);
      expect(localStorage.getItem("misakax:thinkingEnabled")).toBe("true");
    });
  });

  describe("researchEnabled", () => {
    it("should persist researchEnabled to localStorage and default off", () => {
      expect(useChatStore.getState().researchEnabled).toBe(false);
      useChatStore.getState().setResearchEnabled(true);
      expect(useChatStore.getState().researchEnabled).toBe(true);
      expect(localStorage.getItem("misakax:researchEnabled")).toBe("true");
      useChatStore.getState().setResearchEnabled(false);
      expect(localStorage.getItem("misakax:researchEnabled")).toBe("false");
    });
  });

  describe("workspace selector intent", () => {
    it("should open with new-session intent", () => {
      useChatStore.getState().openWorkspaceSelector("new-session");
      const state = useChatStore.getState();
      expect(state.showWorkspaceSelector).toBe(true);
      expect(state.workspaceSelectorIntent).toBe("new-session");
      expect(state.workspaceSelectorTargetSessionId).toBeNull();
    });

    it("should open with change-session intent and target id", () => {
      useChatStore.getState().openWorkspaceSelector("change-session", "sess-1");
      const state = useChatStore.getState();
      expect(state.showWorkspaceSelector).toBe(true);
      expect(state.workspaceSelectorIntent).toBe("change-session");
      expect(state.workspaceSelectorTargetSessionId).toBe("sess-1");
    });

    it("should clear intent on closeWorkspaceSelector", () => {
      useChatStore.getState().openWorkspaceSelector("change-session", "sess-1");
      useChatStore.getState().closeWorkspaceSelector();
      const state = useChatStore.getState();
      expect(state.showWorkspaceSelector).toBe(false);
      expect(state.workspaceSelectorIntent).toBeNull();
      expect(state.workspaceSelectorTargetSessionId).toBeNull();
    });
  });

  describe("upsertSession", () => {
    it("should prepend a new session and activate it", () => {
      useChatStore.getState().setSessions([
        {
          id: "old",
          title: "Old",
          model: null,
          system_prompt: null,
          working_directory: null,
          project_name: null,
          workspace_kind: "default",
          status: "active",
          mode: "agent",
          total_input_tokens: 0,
          total_output_tokens: 0,
          last_message_at: null,
          pinned: false,
          group_name: null,
          created_at: "",
          updated_at: "",
        },
      ]);
      useChatStore.getState().upsertSession({
        id: "new",
        title: "New",
        model: null,
        system_prompt: null,
        working_directory: "/ws",
        project_name: "ws",
        workspace_kind: "custom",
        status: "active",
        mode: "agent",
        total_input_tokens: 0,
        total_output_tokens: 0,
        last_message_at: null,
        pinned: false,
        group_name: null,
        created_at: "",
        updated_at: "",
      });
      const state = useChatStore.getState();
      expect(state.activeSessionId).toBe("new");
      expect(state.sessions[0].id).toBe("new");
      expect(state.sessions).toHaveLength(2);
    });
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
        workspace_kind: "default",
        status: "active",
        mode: "agent",
        total_input_tokens: 0,
        total_output_tokens: 0,
        last_message_at: null,
        pinned: false,
        group_name: null,
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
        workspace_kind: "custom",
        status: "active",
        mode: "agent",
        total_input_tokens: 100,
        total_output_tokens: 200,
        last_message_at: "2026-01-02T00:00:00Z",
        pinned: false,
        group_name: null,
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
        workspace_kind: "default",
        status: "active",
        mode: "agent",
        total_input_tokens: 0,
        total_output_tokens: 0,
        last_message_at: null,
        pinned: false,
        group_name: null,
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

  describe("messages", () => {
    it("should add a message", () => {
      const msg = makeMessage({ content: "Hi" });
      useChatStore.getState().addMessage(msg);
      expect(useChatStore.getState().messages).toHaveLength(1);
      expect(useChatStore.getState().messages[0].content).toBe("Hi");
    });

    it("should set messages list", () => {
      const msgs = [makeMessage({ id: "m1" }), makeMessage({ id: "m2" })];
      useChatStore.getState().setMessages(msgs);
      expect(useChatStore.getState().messages).toHaveLength(2);
    });

    it("should update message content (append delta)", () => {
      const msg = makeMessage({ id: "m1", content: "Hello" });
      useChatStore.getState().addMessage(msg);
      useChatStore.getState().updateMessageContent("m1", " World");
      expect(useChatStore.getState().messages[0].content).toBe("Hello World");
    });

    it("should not modify other messages when updating content", () => {
      useChatStore.getState().setMessages([
        makeMessage({ id: "m1", content: "First" }),
        makeMessage({ id: "m2", content: "Second" }),
      ]);
      useChatStore.getState().updateMessageContent("m2", "++");
      expect(useChatStore.getState().messages[0].content).toBe("First");
      expect(useChatStore.getState().messages[1].content).toBe("Second++");
    });

    it("should set message status", () => {
      const msg = makeMessage({ id: "m1", status: "streaming" });
      useChatStore.getState().addMessage(msg);
      useChatStore.getState().setMessageStatus("m1", "complete");
      expect(useChatStore.getState().messages[0].status).toBe("complete");
    });

    it("should clear messages", () => {
      useChatStore.getState().setMessages([makeMessage(), makeMessage()]);
      useChatStore.getState().setStreaming(true, "x");
      useChatStore.getState().clearMessages();
      expect(useChatStore.getState().messages).toHaveLength(0);
      expect(useChatStore.getState().isStreaming).toBe(false);
      expect(useChatStore.getState().streamingMessageId).toBeNull();
    });
  });

  describe("streaming state", () => {
    it("should set streaming with message id", () => {
      useChatStore.getState().setStreaming(true, "msg-42");
      const state = useChatStore.getState();
      expect(state.isStreaming).toBe(true);
      expect(state.streamingMessageId).toBe("msg-42");
    });

    it("should clear streaming", () => {
      useChatStore.getState().setStreaming(true, "msg-42");
      useChatStore.getState().setStreaming(false);
      const state = useChatStore.getState();
      expect(state.isStreaming).toBe(false);
      expect(state.streamingMessageId).toBeNull();
    });
  });

  describe("model selection", () => {
    it("should set selected model", () => {
      useChatStore.getState().setSelectedModel("provider-1:gpt-4o");
      expect(useChatStore.getState().selectedModel).toBe("provider-1:gpt-4o");
    });

    it("should clear selected model", () => {
      useChatStore.getState().setSelectedModel("x:y");
      useChatStore.getState().setSelectedModel(null);
      expect(useChatStore.getState().selectedModel).toBeNull();
    });
  });

  describe("thinking streaming state", () => {
    it("should set thinking streaming", () => {
      useChatStore.getState().setThinkingStreaming(true);
      expect(useChatStore.getState().isThinkingStreaming).toBe(true);
    });

    it("should reset thinking streaming when stream completes", () => {
      useChatStore.getState().setThinkingStreaming(true);
      useChatStore.getState().setStreaming(false);
      expect(useChatStore.getState().isThinkingStreaming).toBe(false);
    });

    it("should reset thinking streaming on clearMessages", () => {
      useChatStore.getState().setThinkingStreaming(true);
      useChatStore.getState().clearMessages();
      expect(useChatStore.getState().isThinkingStreaming).toBe(false);
    });

    it("should reset thinking streaming on setActiveSessionData", () => {
      useChatStore.getState().setThinkingStreaming(true);
      useChatStore.getState().setActiveSessionData({
        id: "s",
        title: null,
        model: null,
        system_prompt: null,
        working_directory: null,
        project_name: null,
        workspace_kind: "default",
        status: "active",
        mode: "agent",
        total_input_tokens: 0,
        total_output_tokens: 0,
        last_message_at: null,
        pinned: false,
        group_name: null,
        created_at: "",
        updated_at: "",
      });
      expect(useChatStore.getState().isThinkingStreaming).toBe(false);
    });
  });

  describe("updateMessageError", () => {
    it("should set status error and replace content", () => {
      useChatStore.getState().addMessage(
        makeMessage({ id: "e1", role: "assistant", status: "streaming" })
      );
      useChatStore.getState().updateMessageError("e1", "API error");
      const m = useChatStore.getState().messages[0];
      expect(m.status).toBe("error");
      expect(m.content).toBe("API error");
    });
  });

  describe("removeMessagesFrom", () => {
    it("should remove target message and all after it", () => {
      useChatStore.getState().setMessages([
        makeMessage({ id: "m1" }),
        makeMessage({ id: "m2" }),
        makeMessage({ id: "m3" }),
      ]);
      useChatStore.getState().removeMessagesFrom("m2");
      const msgs = useChatStore.getState().messages;
      expect(msgs).toHaveLength(1);
      expect(msgs[0].id).toBe("m1");
    });

    it("should remove all messages when first is targeted", () => {
      useChatStore.getState().setMessages([
        makeMessage({ id: "m1" }),
        makeMessage({ id: "m2" }),
      ]);
      useChatStore.getState().removeMessagesFrom("m1");
      expect(useChatStore.getState().messages).toHaveLength(0);
    });

    it("should do nothing if id not found", () => {
      useChatStore.getState().setMessages([
        makeMessage({ id: "m1" }),
      ]);
      useChatStore.getState().removeMessagesFrom("not-found");
      expect(useChatStore.getState().messages).toHaveLength(1);
    });
  });
});
