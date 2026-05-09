import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useChatStore } from "@/stores/chat-store";

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
  message_id: string;
  error: string;
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
      const { updateMessageContent, setMessageStatus, setStreaming } =
        useChatStore.getState();

      const u1 = await listen<StreamTokenEvent>("stream_token", (event) => {
        const { session_id, message_id, delta } = event.payload;
        if (session_id !== sessionId) return;
        updateMessageContent(message_id, delta);
      });
      unlisteners.push(u1);

      const u2 = await listen<StreamThinkingEvent>(
        "stream_thinking",
        (event) => {
          const { session_id, message_id, thinking_delta } = event.payload;
          if (session_id !== sessionId) return;
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
        const { session_id, message_id } = event.payload;
        if (session_id !== sessionId) return;
        if (message_id) {
          setMessageStatus(message_id, "error");
        }
        setStreaming(false);
      });
      unlisteners.push(u4);
    };

    setup();

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  }, [sessionId]);
}
