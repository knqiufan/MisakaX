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
      messages: [],
      isStreaming: false,
      streamingMessageId: null,
      selectedModel: null,
    });
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
});
