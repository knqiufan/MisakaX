import { describe, it, expect } from "vitest";
import { thinkingSuffixDelta } from "@/hooks/use-stream-listener";
import { useChatStore } from "@/stores/chat-store";
import type { Message, ToolCall } from "@/lib/ipc";

describe("thinkingSuffixDelta", () => {
  it("returns full text when previous is empty", () => {
    expect(thinkingSuffixDelta("", "abc")).toBe("abc");
  });

  it("returns only the new suffix for cumulative resends", () => {
    expect(thinkingSuffixDelta("ab", "abcd")).toBe("cd");
  });

  it("returns empty string for identical payloads", () => {
    expect(thinkingSuffixDelta("same", "same")).toBe("");
  });

  it("treats non-prefix payloads as fresh deltas", () => {
    expect(thinkingSuffixDelta("old", "new")).toBe("new");
  });
});

describe("chat-store addToolCall upsert", () => {
  it("does not duplicate tool calls with the same id", () => {
    const message: Message = {
      id: "a1",
      session_id: "s1",
      role: "assistant",
      content: "",
      token_usage: null,
      model: null,
      thinking_content: null,
      attachments: null,
      status: "streaming",
      created_at: "2026-01-01T00:00:00Z",
    };
    useChatStore.setState({ messages: [message] });

    const base: ToolCall = {
      id: "tc1",
      server_id: "sidecar",
      server_name: "Sidecar",
      tool_name: "ls",
      arguments: { path: "/" },
      result: null,
      status: "running",
      error: null,
      started_at: 1,
      completed_at: null,
    };

    useChatStore.getState().addToolCall("a1", base);
    useChatStore.getState().addToolCall("a1", {
      ...base,
      arguments: { path: "/src" },
    });

    const msg = useChatStore.getState().messages[0];
    expect(msg.tool_calls).toHaveLength(1);
    expect(msg.tool_calls?.[0].arguments).toEqual({ path: "/src" });
  });
});
