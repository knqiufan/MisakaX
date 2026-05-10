import { useEffect } from "react";
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

/**
 * Listens to Tauri backend stream events and updates chat store accordingly.
 * Should be mounted once per active session view.
 */
export function useStreamListener(sessionId: string | null) {
  useEffect(() => {
    if (!sessionId) return;

    const unlisteners: UnlistenFn[] = [];

    const setup = async () => {
      const {
        updateMessageContent,
        updateMessageError,
        setMessageStatus,
        setStreaming,
        setThinkingStreaming,
        addToolCall,
        updateToolCall,
      } = useChatStore.getState();

      const u1 = await listen<StreamTokenEvent>("stream_token", (event) => {
        const { session_id, message_id, delta } = event.payload;
        if (session_id !== sessionId) return;

        if (useChatStore.getState().isThinkingStreaming) {
          setThinkingStreaming(false);
        }

        updateMessageContent(message_id, delta);
      });
      unlisteners.push(u1);

      const u2 = await listen<StreamThinkingEvent>(
        "stream_thinking",
        (event) => {
          const { session_id, message_id, thinking_delta } = event.payload;
          if (session_id !== sessionId) return;

          setThinkingStreaming(true);

          const store = useChatStore.getState();
          const msg = store.messages.find((m) => m.id === message_id);
          if (msg) {
            useChatStore.setState({
              messages: store.messages.map((m) =>
                m.id === message_id
                  ? {
                      ...m,
                      thinking_content:
                        (m.thinking_content ?? "") + thinking_delta,
                    }
                  : m
              ),
            });
          }
        }
      );
      unlisteners.push(u2);

      const u3 = await listen<StreamCompleteEvent>(
        "stream_complete",
        (event) => {
          const { session_id, message_id, usage, was_aborted } = event.payload;
          if (session_id !== sessionId) return;

          const status = was_aborted ? "aborted" : "complete";
          setMessageStatus(message_id, status);
          setStreaming(false);

          if (usage) {
            const store = useChatStore.getState();
            useChatStore.setState({
              messages: store.messages.map((m) =>
                m.id === message_id
                  ? { ...m, token_usage: JSON.stringify(usage) }
                  : m
              ),
            });
          }
        }
      );
      unlisteners.push(u3);

      const u4 = await listen<StreamErrorEvent>("stream_error", (event) => {
        const { session_id, message_id, error } = event.payload;
        if (session_id !== sessionId) return;

        const detail = error.trim() || i18n.t("chat:streamError");

        if (message_id) {
          const msg = useChatStore
            .getState()
            .messages.find((m) => m.id === message_id);
          if (msg?.status === "streaming") {
            updateMessageError(message_id, detail);
          }
        }

        setStreaming(false);
      });
      unlisteners.push(u4);

      const u5 = await listen<StreamToolCallEvent>(
        "stream:tool_call",
        (event) => {
          const { session_id, message_id, tool_call_id, server_id, server_name, tool_name, arguments: args, status } = event.payload;
          if (session_id !== sessionId) return;

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
        }
      );
      unlisteners.push(u5);

      const u6 = await listen<StreamToolResultEvent>(
        "stream:tool_result",
        (event) => {
          const { session_id, message_id, tool_call_id, result, error: toolError, status } = event.payload;
          if (session_id !== sessionId) return;

          updateToolCall(message_id, tool_call_id, {
            result,
            error: toolError,
            status,
            completed_at: Date.now(),
          });
        }
      );
      unlisteners.push(u6);
    };

    setup();

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  }, [sessionId]);
}
