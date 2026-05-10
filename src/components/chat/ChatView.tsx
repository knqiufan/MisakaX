import { useCallback, useEffect } from "react";
import { useChatStore } from "@/stores/chat-store";
import { chatIpc } from "@/lib/ipc";
import type { Session, ImageAttachment } from "@/lib/ipc";
import { useStreamListener } from "@/hooks/use-stream-listener";
import { WorkspaceBar } from "./WorkspaceBar";
import { MessageList } from "./MessageList";
import { MessageInput } from "./MessageInput";
import { ChatEmptyState } from "./ChatEmptyState";

interface ChatViewProps {
  session: Session;
  onChangeDir: () => void;
}

export function ChatView({ session, onChangeDir }: ChatViewProps) {
  const {
    messages,
    streamingMessageId,
    isStreaming,
    isThinkingStreaming,
    setStreaming,
    addMessage,
    removeMessagesFrom,
    loadMessages,
    requestAutoTitle,
  } = useChatStore();

  useStreamListener(session.id);

  useEffect(() => {
    loadMessages(session.id);
  }, [session.id, loadMessages]);

  const handleSend = useCallback(
    async (
      content: string,
      modelOverride?: string,
      images?: ImageAttachment[]
    ) => {
      const attachmentsJson =
        images && images.length > 0 ? JSON.stringify(images) : null;

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
          images,
          model_override: modelOverride,
        });
      } catch (err) {
        console.error("Failed to send message:", err);
        setStreaming(false);
      }

      if (isFirstMessage) {
        requestAutoTitle(session.id, content);
      }
    },
    [session.id, addMessage, setStreaming, messages.length, requestAutoTitle]
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
        setStreaming(false);
      }
    },
    [session.id, addMessage, removeMessagesFrom, setStreaming]
  );

  return (
    <div className="flex h-full flex-col">
      <WorkspaceBar workingDir={session.working_directory} onChangeDir={onChangeDir} />
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
      <MessageInput onSend={handleSend} onStop={handleStop} />
    </div>
  );
}
