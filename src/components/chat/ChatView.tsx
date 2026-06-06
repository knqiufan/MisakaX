import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { listen } from "@tauri-apps/api/event";
import { useChatStore } from "@/stores/chat-store";
import { useToolLogsStore } from "@/stores/tool-logs-store";
import { chatIpc, IpcError } from "@/lib/ipc";
import type { Session, MessageAttachment, ToolCallRequestEvent } from "@/lib/ipc";
import { useStreamListener } from "@/hooks/use-stream-listener";
import { WorkspaceBar } from "./workspace/WorkspaceBar";
import { MessageList } from "./message/MessageList";
import { MessageInput } from "./composer/MessageInput";
import { ChatEmptyState } from "./ChatEmptyState";
import { ToolApprovalDialog } from "./ToolApprovalDialog";

interface ChatViewProps {
  session: Session;
  onChangeDir: () => void;
  onToggleExplorer?: () => void;
  explorerOpen?: boolean;
}

export function ChatView({
  session,
  onChangeDir,
  onToggleExplorer,
  explorerOpen = false,
}: ChatViewProps) {
  const { t } = useTranslation("chat");
  const {
    messages,
    streamingMessageId,
    isStreaming,
    isThinkingStreaming,
    setStreaming,
    addMessage,
    removeMessagesFrom,
    updateMessageError,
    loadMessages,
    requestAutoTitle,
  } = useChatStore();

  const [pendingApproval, setPendingApproval] =
    useState<ToolCallRequestEvent | null>(null);

  useStreamListener(session.id);

  useEffect(() => {
    loadMessages(session.id);
  }, [session.id, loadMessages]);

  const { addToolCall, clearToolCalls } = useToolLogsStore();

  useEffect(() => {
    clearToolCalls();
    for (const msg of messages) {
      if (msg.tool_calls) {
        for (const tc of msg.tool_calls) {
          addToolCall(tc);
        }
      }
    }
  }, [messages, clearToolCalls, addToolCall]);

  useEffect(() => {
    const unlisten = listen<ToolCallRequestEvent>(
      "mcp:tool_call_request",
      (event) => {
        setPendingApproval(event.payload);
      }
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleSend = useCallback(
    async (
      content: string,
      modelOverride?: string,
      attachments?: MessageAttachment[]
    ) => {
      const attachmentsJson =
        attachments && attachments.length > 0 ? JSON.stringify(attachments) : null;

      const userMessage = {
        id: crypto.randomUUID(),
        session_id: session.id,
        role: "user" as const,
        content,
        token_usage: null,
        model: null,
        thinking_content: null,
        attachments: attachmentsJson,
        status: "complete" as const,
        created_at: new Date().toISOString(),
      };
      addMessage(userMessage);

      const assistantId = crypto.randomUUID();
      const assistantPlaceholder = {
        id: assistantId,
        session_id: session.id,
        role: "assistant" as const,
        content: "",
        token_usage: null,
        model: modelOverride ?? null,
        thinking_content: null,
        attachments: null,
        status: "streaming" as const,
        created_at: new Date().toISOString(),
      };
      addMessage(assistantPlaceholder);
      setStreaming(true, assistantId);

      const isFirstMessage = messages.length === 0;

      try {
        await chatIpc.sendMessage({
          session_id: session.id,
          content,
          attachments,
          model_override: modelOverride,
        });
        if (isFirstMessage) {
          requestAutoTitle(session.id, content);
        }
      } catch (err) {
        console.error("Failed to send message:", err);
        const detail =
          err instanceof IpcError ? err.originalError : String(err);
        const shown = `${t("sendFailed")}: ${detail}`;
        const currentId =
          useChatStore.getState().streamingMessageId ?? assistantId;
        const assistant = useChatStore
          .getState()
          .messages.find((m) => m.id === currentId);
        if (assistant?.status !== "error") {
          updateMessageError(currentId, shown);
        }
        setStreaming(false);
        toast.error(t("errorOccurred"), {
          description:
            assistant?.status === "error"
              ? assistant.content
              : shown,
        });
      }
    },
    [
      session.id,
      addMessage,
      setStreaming,
      updateMessageError,
      messages.length,
      requestAutoTitle,
      t,
    ]
  );

  const handleStop = useCallback(async () => {
    try {
      await chatIpc.stopGeneration(session.id);
    } catch (err) {
      console.error("Failed to stop generation:", err);
    }
  }, [session.id]);

  const handleRegenerate = useCallback(
    async (messageId: string) => {
      removeMessagesFrom(messageId);

      const assistantId = crypto.randomUUID();
      const assistantPlaceholder = {
        id: assistantId,
        session_id: session.id,
        role: "assistant" as const,
        content: "",
        token_usage: null,
        model: null,
        thinking_content: null,
        attachments: null,
        status: "streaming" as const,
        created_at: new Date().toISOString(),
      };

      addMessage(assistantPlaceholder);
      setStreaming(true, assistantId);

      try {
        await chatIpc.regenerateMessage(session.id, messageId);
      } catch (err) {
        console.error("Failed to regenerate:", err);
        const detail =
          err instanceof IpcError ? err.originalError : String(err);
        const shown = `${t("sendFailed")}: ${detail}`;
        const currentId =
          useChatStore.getState().streamingMessageId ?? assistantId;
        const assistant = useChatStore
          .getState()
          .messages.find((m) => m.id === currentId);
        if (assistant?.status !== "error") {
          updateMessageError(currentId, shown);
        }
        setStreaming(false);
        toast.error(t("errorOccurred"), {
          description:
            assistant?.status === "error"
              ? assistant.content
              : shown,
        });
      }
    },
    [
      session.id,
      addMessage,
      removeMessagesFrom,
      setStreaming,
      updateMessageError,
      t,
    ]
  );

  return (
    <div className="flex h-full flex-col">
      <WorkspaceBar
        workingDir={session.working_directory}
        onChangeDir={onChangeDir}
        onToggleExplorer={onToggleExplorer}
        explorerOpen={explorerOpen}
        onToggleToolLogs={() => {}}
      />
      {messages.length === 0 && !isStreaming ? (
        <ChatEmptyState />
      ) : (
        <MessageList
          messages={messages}
          streamingMessageId={streamingMessageId}
          isThinkingStreaming={isThinkingStreaming}
          onRegenerate={handleRegenerate}
        />
      )}
      <MessageInput
        onSend={handleSend}
        onStop={handleStop}
        onPickWorkspaceFile={onToggleExplorer}
      />
      <ToolApprovalDialog
        request={pendingApproval}
        onDismiss={() => setPendingApproval(null)}
      />
    </div>
  );
}
