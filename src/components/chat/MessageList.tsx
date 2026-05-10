import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ArrowDown } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Message } from "@/lib/ipc";
import { MessageItem } from "./MessageItem";

interface MessageListProps {
  messages: Message[];
  streamingMessageId: string | null;
  isThinkingStreaming?: boolean;
  onRegenerate?: (messageId: string) => void;
}

const SCROLL_THRESHOLD = 80;

export function MessageList({
  messages,
  streamingMessageId,
  isThinkingStreaming,
  onRegenerate,
}: MessageListProps) {
  const { t } = useTranslation("chat");
  const containerRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const [isAtBottom, setIsAtBottom] = useState(true);

  const checkIsAtBottom = useCallback(() => {
    const el = containerRef.current;
    if (!el) return true;
    return el.scrollHeight - el.scrollTop - el.clientHeight < SCROLL_THRESHOLD;
  }, []);

  const scrollToBottom = useCallback((behavior: ScrollBehavior = "smooth") => {
    bottomRef.current?.scrollIntoView({ behavior });
  }, []);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const handleScroll = () => {
      setIsAtBottom(checkIsAtBottom());
    };

    el.addEventListener("scroll", handleScroll, { passive: true });
    return () => el.removeEventListener("scroll", handleScroll);
  }, [checkIsAtBottom]);

  useEffect(() => {
    if (isAtBottom) {
      scrollToBottom("instant");
    }
  }, [messages, isAtBottom, scrollToBottom]);

  return (
    <div className="relative min-h-0 flex-1">
      <div
        ref={containerRef}
        className="h-full overflow-y-auto scroll-smooth"
      >
        <div className="mx-auto max-w-3xl py-4">
          {messages.map((msg) => {
            const isCurrentStreaming = msg.id === streamingMessageId;
            return (
              <MessageItem
                key={msg.id}
                message={msg}
                isStreaming={isCurrentStreaming}
                isThinkingStreaming={isCurrentStreaming && isThinkingStreaming}
                onRegenerate={
                  msg.role === "assistant" ? onRegenerate : undefined
                }
              />
            );
          })}
          <div ref={bottomRef} className="h-px" />
        </div>
      </div>

      {!isAtBottom && (
        <button
          type="button"
          onClick={() => scrollToBottom()}
          className={cn(
            "absolute bottom-4 left-1/2 -translate-x-1/2",
            "flex size-8 items-center justify-center rounded-full",
            "border border-[color:var(--border-muted)]",
            "bg-[color:var(--surface-popover)] text-muted-foreground",
            "shadow-[0_4px_12px_rgba(0,0,0,0.2)]",
            "transition-colors duration-[var(--ds-dur-fast)]",
            "hover:bg-[color:var(--surface-card-strong)] hover:text-foreground"
          )}
          aria-label={t("scrollToBottom")}
        >
          <ArrowDown className="size-4" />
        </button>
      )}
    </div>
  );
}
