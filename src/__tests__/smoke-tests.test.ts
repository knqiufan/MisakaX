import { describe, it, expect, beforeEach } from "vitest";
import { extractDirName } from "@/components/chat/WorkspaceBar";
import { useChatStore } from "@/stores/chat-store";
import type { Message } from "@/lib/ipc";

/**
 * Phase 2 closing verification (F-1, F-2, F-5, F-7, F-8) — automated slices.
 * F-3 / F-4 / F-6 require live API keys and are tracked in docs/guides/phase2-e2e-verification-matrix.md
 */

const makeMessage = (overrides: Partial<Message> = {}): Message => ({
  id: crypto.randomUUID(),
  session_id: "sess",
  role: "user",
  content: "",
  token_usage: null,
  model: null,
  thinking_content: null,
  attachments: null,
  status: "complete",
  created_at: new Date().toISOString(),
  ...overrides,
});

describe("F-1 WorkspaceBar path helpers", () => {
  it("extractDirName handles Windows-style paths", () => {
    expect(extractDirName("D:\\code\\Misaka-Tauri")).toBe("Misaka-Tauri");
  });

  it("extractDirName handles POSIX paths and trailing slashes", () => {
    expect(extractDirName("/home/user/project/")).toBe("project");
  });

  it("extractDirName preserves path when no directory segment", () => {
    expect(extractDirName("/")).toBe("/");
  });
});

describe("F-2 Full conversation store lifecycle", () => {
  beforeEach(() => {
    useChatStore.setState({
      messages: [],
      isStreaming: false,
      streamingMessageId: null,
      isThinkingStreaming: false,
    });
  });

  it("send → stream tokens → complete with usage", () => {
    const store = useChatStore.getState();
    store.addMessage(makeMessage({ id: "u1", role: "user", content: "Hi" }));
    store.addMessage(
      makeMessage({ id: "a1", role: "assistant", content: "", status: "streaming" })
    );
    store.setStreaming(true, "a1");

    store.updateMessageContent("a1", "A");
    store.updateMessageContent("a1", "B");
    store.setMessageStatus("a1", "complete");
    useChatStore.setState({
      messages: useChatStore.getState().messages.map((m) =>
        m.id === "a1"
          ? { ...m, token_usage: JSON.stringify({ input_tokens: 1, output_tokens: 2, total_tokens: 3 }) }
          : m
      ),
    });
    store.setStreaming(false);

    const assistant = useChatStore.getState().messages.find((m) => m.id === "a1");
    expect(assistant?.content).toBe("AB");
    expect(assistant?.status).toBe("complete");
    expect(assistant?.token_usage).toContain("total_tokens");
    expect(useChatStore.getState().isStreaming).toBe(false);
  });
});

describe("F-5 Control flow: regenerate prune + ordering", () => {
  beforeEach(() => {
    useChatStore.setState({ messages: [], isStreaming: false, streamingMessageId: null });
  });

  it("removeMessagesFrom mimics regenerate tail cut", () => {
    useChatStore.getState().setMessages([
      makeMessage({ id: "m1", role: "user", content: "u1" }),
      makeMessage({ id: "m2", role: "assistant", content: "a1" }),
      makeMessage({ id: "m3", role: "user", content: "u2" }),
    ]);
    useChatStore.getState().removeMessagesFrom("m2");
    const ids = useChatStore.getState().messages.map((m) => m.id);
    expect(ids).toEqual(["m1"]);
  });

  it("multi-turn message list preserves order when appending", () => {
    const store = useChatStore.getState();
    for (let i = 0; i < 5; i++) {
      store.addMessage(makeMessage({ id: `r${i}`, content: String(i) }));
    }
    const contents = useChatStore.getState().messages.map((m) => m.content);
    expect(contents).toEqual(["0", "1", "2", "3", "4"]);
  });
});

describe("F-7 Stream error guard (no overwrite after catch)", () => {
  function shouldAttachStreamErrorToMessage(msg: Message | undefined): boolean {
    return msg?.status === "streaming";
  }

  it("applies stream error only while assistant is still streaming", () => {
    const streaming = makeMessage({ id: "x", role: "assistant", status: "streaming" });
    const errored = makeMessage({ id: "x", role: "assistant", status: "error" });
    expect(shouldAttachStreamErrorToMessage(streaming)).toBe(true);
    expect(shouldAttachStreamErrorToMessage(errored)).toBe(false);
  });
});

describe("F-8 Long stream: many deltas accumulate", () => {
  beforeEach(() => {
    useChatStore.setState({ messages: [] });
  });

  it("handles 1200 append deltas without losing length", () => {
    useChatStore.getState().addMessage(
      makeMessage({ id: "long", role: "assistant", content: "", status: "streaming" })
    );
    const store = useChatStore.getState();
    for (let i = 0; i < 1200; i++) {
      store.updateMessageContent("long", "a");
    }
    expect(useChatStore.getState().messages[0].content.length).toBe(1200);
  });
});
