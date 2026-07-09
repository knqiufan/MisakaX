import { describe, it, expect, vi, beforeEach } from "vitest";
import { useChatStore } from "@/stores/chat-store";
import { parseToolCalls } from "@/lib/ipc/chat";
import type { ToolCall } from "@/lib/ipc";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const sampleToolCall: ToolCall = {
  id: "tc1",
  server_id: "srv",
  server_name: "Filesystem",
  tool_name: "read_file",
  arguments: { path: "package.json" },
  result: { content: "ok" },
  status: "complete",
  error: null,
  started_at: 1000,
  completed_at: 1200,
};

describe("parseToolCalls", () => {
  it("parses a JSON string into ToolCall[]", () => {
    const parsed = parseToolCalls(JSON.stringify([sampleToolCall]));
    expect(parsed).toHaveLength(1);
    expect(parsed?.[0].tool_name).toBe("read_file");
  });

  it("returns undefined for null / undefined / empty string", () => {
    expect(parseToolCalls(null)).toBeUndefined();
    expect(parseToolCalls(undefined)).toBeUndefined();
    expect(parseToolCalls("")).toBeUndefined();
    expect(parseToolCalls("   ")).toBeUndefined();
  });

  it("returns undefined for invalid JSON without throwing", () => {
    expect(parseToolCalls("{not valid json")).toBeUndefined();
  });

  it("returns undefined when JSON is not an array", () => {
    expect(parseToolCalls('{"tool_name":"x"}')).toBeUndefined();
  });

  it("passes through an already-parsed array", () => {
    const arr = [sampleToolCall];
    expect(parseToolCalls(arr)).toBe(arr);
  });
});

describe("loadMessages tool_calls deserialization", () => {
  beforeEach(() => {
    useChatStore.setState({ messages: [], loading: false });
  });

  const rawMessage = (toolCalls: unknown) => ({
    id: "a1",
    session_id: "s1",
    role: "assistant",
    content: "done",
    token_usage: null,
    model: null,
    thinking_content: null,
    attachments: null,
    status: "complete",
    created_at: "2026-01-01T00:00:00Z",
    tool_calls: toolCalls,
  });

  it("parses tool_calls JSON string returned by get_messages", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce([
      rawMessage(JSON.stringify([sampleToolCall])),
    ]);

    await useChatStore.getState().loadMessages("s1");

    const [msg] = useChatStore.getState().messages;
    expect(msg.tool_calls).toHaveLength(1);
    expect(msg.tool_calls?.[0].tool_name).toBe("read_file");
    expect(msg.tool_calls?.[0].arguments.path).toBe("package.json");
  });

  it("leaves tool_calls undefined when backend returns null", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce([rawMessage(null)]);

    await useChatStore.getState().loadMessages("s1");

    expect(useChatStore.getState().messages[0].tool_calls).toBeUndefined();
  });

  it("does not throw on malformed tool_calls JSON", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce([rawMessage("{broken")]);

    await useChatStore.getState().loadMessages("s1");

    expect(useChatStore.getState().messages).toHaveLength(1);
    expect(useChatStore.getState().messages[0].tool_calls).toBeUndefined();
  });
});
