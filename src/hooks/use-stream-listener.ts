import { useEffect, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { i18n } from "@/locales/i18n";
import { useChatStore } from "@/stores/chat-store";
import type { ToolCallStatus } from "@/lib/ipc";

interface StreamTokenEvent {
  session_id: string;
  message_id: string;
  delta: string;
}

interface StreamThinkingEvent {
  session_id: string;
  message_id: string;
  thinking_delta: string;
}

interface StreamCompleteEvent {
  session_id: string;
  message_id: string;
  full_content: string;
  full_thinking: string;
  usage: {
    input_tokens: number;
    output_tokens: number;
    total_tokens: number;
  } | null;
  was_aborted: boolean;
}

interface StreamErrorEvent {
  session_id: string;
  message_id: string | null;
  error: string;
}

interface StreamToolCallEvent {
  session_id: string;
  message_id: string;
  tool_call_id: string;
  server_id: string;
  server_name: string;
  tool_name: string;
  arguments: Record<string, unknown>;
  status: ToolCallStatus;
}

interface StreamToolResultEvent {
  session_id: string;
  message_id: string;
  tool_call_id: string;
  result: unknown | null;
  error: string | null;
  status: ToolCallStatus;
}

interface PendingToken {
  messageId: string;
  delta: string;
}

/**
 * Listens to Tauri backend stream events and updates chat store accordingly.
 * Should be mounted once per active session view.
 *
 * Token deltas are coalesced via requestAnimationFrame so Markdown re-renders
 * grow smoothly instead of one-character typewriter flashes.
 */
export function useStreamListener(sessionId: string | null) {
  const pendingTokensRef = useRef<PendingToken[]>([]);
  const rafRef = useRef<number | null>(null);

  useEffect(() => {
    if (!sessionId) return;

    let cancelled = false;
    const unlisteners: UnlistenFn[] = [];

    const flushPendingTokens = () => {
      rafRef.current = null;
      const pending = pendingTokensRef.current;
      if (pending.length === 0) return;
      pendingTokensRef.current = [];

      const merged = new Map<string, string>();
      for (const item of pending) {
        merged.set(item.messageId, (merged.get(item.messageId) ?? "") + item.delta);
      }

      const { updateMessageContent, setThinkingStreaming } = useChatStore.getState();
      if (useChatStore.getState().isThinkingStreaming) {
        setThinkingStreaming(false);
      }
      for (const [messageId, delta] of merged) {
        updateMessageContent(messageId, delta);
      }
    };

    const scheduleTokenFlush = () => {
      if (rafRef.current !== null) return;
      rafRef.current = requestAnimationFrame(flushPendingTokens);
    };

    const ensureMessageId = (backendId: string) => {
      const state = useChatStore.getState();
      if (state.messages.some((m) => m.id === backendId)) return;

      const { streamingMessageId } = state;
      if (streamingMessageId && streamingMessageId !== backendId) {
        useChatStore.getState().remapMessageId(streamingMessageId, backendId);
      }
    };

    const setup = async () => {
      const {
        updateMessageError,
        setMessageStatus,
        setStreaming,
        setThinkingStreaming,
        addToolCall,
        updateToolCall,
      } = useChatStore.getState();

      const track = async (
        eventName: string,
        handler: (payload: never) => void
      ) => {
        const unlisten = await listen(eventName, (event) => {
          if (cancelled) return;
          handler(event.payload as never);
        });
        if (cancelled) {
          unlisten();
          return;
        }
        unlisteners.push(unlisten);
      };

      await track("stream_token", (payload: StreamTokenEvent) => {
        const { session_id, message_id, delta } = payload;
        if (session_id !== sessionId) return;
        ensureMessageId(message_id);
        pendingTokensRef.current.push({ messageId: message_id, delta });
        scheduleTokenFlush();
      });

      await track("stream_thinking", (payload: StreamThinkingEvent) => {
        const { session_id, message_id, thinking_delta } = payload;
        if (session_id !== sessionId) return;

        ensureMessageId(message_id);
        setThinkingStreaming(true);

        const store = useChatStore.getState();
        const msg = store.messages.find((m) => m.id === message_id);
        if (!msg) return;

        const previous = msg.thinking_content ?? "";
        const nextDelta = thinkingSuffixDelta(previous, thinking_delta);
        if (!nextDelta) return;

        useChatStore.setState({
          messages: store.messages.map((m) =>
            m.id === message_id
              ? {
                  ...m,
                  thinking_content: previous + nextDelta,
                }
              : m
          ),
        });
      });

      await track("stream_complete", (payload: StreamCompleteEvent) => {
        const {
          session_id,
          message_id,
          full_content,
          full_thinking,
          usage,
          was_aborted,
        } = payload;
        if (session_id !== sessionId) return;

        // Drain any coalesced tokens before applying the authoritative body.
        if (rafRef.current !== null) {
          cancelAnimationFrame(rafRef.current);
          rafRef.current = null;
        }
        pendingTokensRef.current = [];

        ensureMessageId(message_id);
        const status = was_aborted ? "aborted" : "complete";
        setMessageStatus(message_id, status);
        setStreaming(false);

        const store = useChatStore.getState();
        useChatStore.setState({
          messages: store.messages.map((m) =>
            m.id === message_id
              ? {
                  ...m,
                  content: full_content,
                  thinking_content: full_thinking || m.thinking_content,
                  ...(usage ? { token_usage: JSON.stringify(usage) } : {}),
                }
              : m
          ),
        });
      });

      await track("stream_error", (payload: StreamErrorEvent) => {
        const { session_id, message_id, error } = payload;
        if (session_id !== sessionId) return;

        if (rafRef.current !== null) {
          cancelAnimationFrame(rafRef.current);
          rafRef.current = null;
        }
        pendingTokensRef.current = [];

        const detail = error.trim() || i18n.t("chat:streamError");

        if (message_id) {
          ensureMessageId(message_id);
          const msg = useChatStore
            .getState()
            .messages.find((m) => m.id === message_id);
          if (msg?.status === "streaming") {
            updateMessageError(message_id, detail);
          }
        }

        setStreaming(false);
      });

      await track("stream:tool_call", (payload: StreamToolCallEvent) => {
        const {
          session_id,
          message_id,
          tool_call_id,
          server_id,
          server_name,
          tool_name,
          arguments: args,
          status,
        } = payload;
        if (session_id !== sessionId) return;

        ensureMessageId(message_id);
        addToolCall(message_id, {
          id: tool_call_id,
          server_id,
          server_name,
          tool_name,
          arguments: args,
          result: null,
          status,
          error: null,
          started_at: Date.now(),
          completed_at: null,
        });
      });

      await track("stream:tool_result", (payload: StreamToolResultEvent) => {
        const {
          session_id,
          message_id,
          tool_call_id,
          result,
          error: toolError,
          status,
        } = payload;
        if (session_id !== sessionId) return;

        ensureMessageId(message_id);
        updateToolCall(message_id, tool_call_id, {
          result,
          error: toolError,
          status,
          completed_at: Date.now(),
        });
      });
    };

    void setup();

    return () => {
      cancelled = true;
      if (rafRef.current !== null) {
        cancelAnimationFrame(rafRef.current);
        rafRef.current = null;
      }
      pendingTokensRef.current = [];
      unlisteners.forEach((fn) => fn());
    };
  }, [sessionId]);
}

/** Prefer suffix-only when the backend re-sends cumulative thinking text. */
export function thinkingSuffixDelta(previous: string, incoming: string): string {
  if (!incoming) return "";
  if (!previous) return incoming;
  if (incoming === previous) return "";
  if (incoming.startsWith(previous)) return incoming.slice(previous.length);
  return incoming;
}
