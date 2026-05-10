import { describe, it, expect, vi, beforeEach } from "vitest";
import { useChatStore } from "@/stores/chat-store";
import type { Message } from "@/lib/ipc";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

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

describe("ChatView store integration", () => {
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
      isThinkingStreaming: false,
      selectedModel: null,
    });
  });

  describe("loadMessages", () => {
    it("should set loading true during load", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValueOnce([]);

      const promise = useChatStore.getState().loadMessages("s1");
      expect(useChatStore.getState().loading).toBe(true);
      await promise;
      expect(useChatStore.getState().loading).toBe(false);
    });

    it("should populate messages on success", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const msgs = [makeMessage({ id: "m1" }), makeMessage({ id: "m2" })];
      vi.mocked(invoke).mockResolvedValueOnce(msgs);

      await useChatStore.getState().loadMessages("s1");
      expect(useChatStore.getState().messages).toHaveLength(2);
      expect(useChatStore.getState().messages[0].id).toBe("m1");
    });

    it("should handle error gracefully", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockRejectedValueOnce(new Error("DB error"));

      await useChatStore.getState().loadMessages("s1");
      expect(useChatStore.getState().loading).toBe(false);
      expect(useChatStore.getState().messages).toEqual([]);
    });
  });

  describe("streaming workflow", () => {
    it("should simulate full send → stream → complete lifecycle", () => {
      const store = useChatStore.getState();

      const userMsg = makeMessage({ id: "u1", role: "user", content: "Hi" });
      store.addMessage(userMsg);

      const assistantMsg = makeMessage({
        id: "a1",
        role: "assistant",
        content: "",
        status: "streaming",
      });
      store.addMessage(assistantMsg);
      store.setStreaming(true, "a1");

      expect(useChatStore.getState().isStreaming).toBe(true);
      expect(useChatStore.getState().streamingMessageId).toBe("a1");

      store.updateMessageContent("a1", "Hello");
      store.updateMessageContent("a1", " World");
      expect(useChatStore.getState().messages[1].content).toBe("Hello World");

      store.setMessageStatus("a1", "complete");
      store.setStreaming(false);

      expect(useChatStore.getState().isStreaming).toBe(false);
      expect(useChatStore.getState().messages[1].status).toBe("complete");
    });

    it("should handle abort correctly", () => {
      const store = useChatStore.getState();

      const msg = makeMessage({ id: "a2", role: "assistant", content: "partial", status: "streaming" });
      store.addMessage(msg);
      store.setStreaming(true, "a2");

      store.setMessageStatus("a2", "aborted");
      store.setStreaming(false);

      expect(useChatStore.getState().messages[0].status).toBe("aborted");
      expect(useChatStore.getState().isStreaming).toBe(false);
    });

    it("should handle error status correctly", () => {
      const store = useChatStore.getState();

      const msg = makeMessage({ id: "a3", role: "assistant", content: "", status: "streaming" });
      store.addMessage(msg);
      store.setStreaming(true, "a3");

      store.updateMessageError("a3", "Stream failed");
      store.setStreaming(false);

      expect(useChatStore.getState().messages[0].status).toBe("error");
      expect(useChatStore.getState().messages[0].content).toBe("Stream failed");
    });
  });

  describe("setActiveSessionData clears messages", () => {
    it("should clear messages and streaming state when switching sessions", () => {
      useChatStore.setState({
        messages: [makeMessage(), makeMessage()],
        isStreaming: true,
        streamingMessageId: "x",
      });

      useChatStore.getState().setActiveSessionData({
        id: "new-session",
        title: "New",
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
      });

      expect(useChatStore.getState().messages).toEqual([]);
      expect(useChatStore.getState().isStreaming).toBe(false);
      expect(useChatStore.getState().streamingMessageId).toBeNull();
    });
  });
});

describe("TokenBadge formatting logic", () => {
  function formatTokenCount(count: number): string {
    if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`;
    if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K`;
    return String(count);
  }

  it("should format small numbers directly", () => {
    expect(formatTokenCount(42)).toBe("42");
    expect(formatTokenCount(999)).toBe("999");
  });

  it("should format thousands with K", () => {
    expect(formatTokenCount(1000)).toBe("1.0K");
    expect(formatTokenCount(2500)).toBe("2.5K");
    expect(formatTokenCount(15300)).toBe("15.3K");
  });

  it("should format millions with M", () => {
    expect(formatTokenCount(1000000)).toBe("1.0M");
    expect(formatTokenCount(2300000)).toBe("2.3M");
  });
});

describe("MessageList scroll logic", () => {
  const SCROLL_THRESHOLD = 80;

  function isAtBottom(scrollHeight: number, scrollTop: number, clientHeight: number): boolean {
    return scrollHeight - scrollTop - clientHeight < SCROLL_THRESHOLD;
  }

  it("should be at bottom when fully scrolled", () => {
    expect(isAtBottom(1000, 600, 400)).toBe(true);
  });

  it("should be at bottom within threshold", () => {
    expect(isAtBottom(1000, 530, 400)).toBe(true);
  });

  it("should not be at bottom when scrolled up beyond threshold", () => {
    expect(isAtBottom(1000, 400, 400)).toBe(false);
  });
});
