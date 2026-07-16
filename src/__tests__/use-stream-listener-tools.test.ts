import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { useChatStore } from "@/stores/chat-store";
import { useStreamListener } from "@/hooks/use-stream-listener";
import type { Message } from "@/lib/ipc";

type Handler = (event: { payload: unknown }) => void;

// 捕获每个事件名注册的回调，便于测试手动派发。
const handlers = new Map<string, Handler>();

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((eventName: string, cb: Handler) => {
    handlers.set(eventName, cb);
    return Promise.resolve(() => handlers.delete(eventName));
  }),
}));

vi.mock("@/locales/i18n", () => ({
  i18n: { t: (key: string) => key },
}));

const SESSION_ID = "s1";
const MESSAGE_ID = "assistant-1";

const assistantMessage: Message = {
  id: MESSAGE_ID,
  session_id: SESSION_ID,
  role: "assistant",
  content: "",
  token_usage: null,
  model: null,
  thinking_content: null,
  attachments: null,
  status: "streaming",
  created_at: "2026-01-01T00:00:00Z",
};

const toolCallPayload = {
  session_id: SESSION_ID,
  message_id: MESSAGE_ID,
  tool_call_id: "tc1",
  server_id: "srv",
  server_name: "Filesystem",
  tool_name: "read_file",
  arguments: { path: "package.json" },
  status: "running" as const,
};

// setup() 内部 await 了多个 listen，需要把这些微任务清空后 handler 才注册完成。
async function flushListeners() {
  for (let i = 0; i < 10; i += 1) {
    await Promise.resolve();
  }
}

async function mountListener() {
  const view = renderHook(() => useStreamListener(SESSION_ID));
  await flushListeners();
  return view;
}

function dispatch(eventName: string, payload: unknown) {
  const handler = handlers.get(eventName);
  if (!handler) throw new Error(`No handler registered for ${eventName}`);
  handler({ payload });
}

describe("useStreamListener tool events", () => {
  beforeEach(() => {
    handlers.clear();
    useChatStore.setState({
      messages: [{ ...assistantMessage }],
      isStreaming: true,
      streamingMessageId: MESSAGE_ID,
      isThinkingStreaming: false,
    });
  });

  it("appends a tool call on stream:tool_call for the active session", async () => {
    await mountListener();

    dispatch("stream:tool_call", toolCallPayload);

    const msg = useChatStore.getState().messages.find((m) => m.id === MESSAGE_ID);
    expect(msg?.tool_calls).toHaveLength(1);
    const tc = msg?.tool_calls?.[0];
    expect(tc?.id).toBe("tc1");
    expect(tc?.tool_name).toBe("read_file");
    expect(tc?.status).toBe("running");
    expect(tc?.result).toBeNull();
    expect(tc?.completed_at).toBeNull();
  });

  it("upserts duplicate stream:tool_call with the same tool_call_id", async () => {
    await mountListener();

    dispatch("stream:tool_call", toolCallPayload);
    dispatch("stream:tool_call", {
      ...toolCallPayload,
      arguments: { path: "updated.json" },
    });

    const msg = useChatStore.getState().messages.find((m) => m.id === MESSAGE_ID);
    expect(msg?.tool_calls).toHaveLength(1);
    expect(msg?.tool_calls?.[0].arguments).toEqual({ path: "updated.json" });
  });

  it("updates the matching tool call on stream:tool_result", async () => {
    await mountListener();

    dispatch("stream:tool_call", toolCallPayload);
    dispatch("stream:tool_result", {
      session_id: SESSION_ID,
      message_id: MESSAGE_ID,
      tool_call_id: "tc1",
      result: { content: "ok" },
      error: null,
      status: "complete" as const,
    });

    const msg = useChatStore.getState().messages.find((m) => m.id === MESSAGE_ID);
    const tc = msg?.tool_calls?.[0];
    expect(tc?.status).toBe("complete");
    expect(tc?.result).toEqual({ content: "ok" });
    expect(tc?.error).toBeNull();
    expect(tc?.completed_at).not.toBeNull();
  });

  it("ignores events for a different session", async () => {
    await mountListener();

    dispatch("stream:tool_call", { ...toolCallPayload, session_id: "other" });

    const msg = useChatStore.getState().messages.find((m) => m.id === MESSAGE_ID);
    expect(msg?.tool_calls ?? []).toHaveLength(0);
  });
});
