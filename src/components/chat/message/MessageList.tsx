import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { ArrowDown } from "lucide-react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { Message } from "@/lib/ipc";
import { MessageItem } from "./MessageItem";

export const MESSAGE_ROW_ESTIMATE = 220;
export const MESSAGE_ROW_OVERSCAN = 6;

interface MessageListProps {
  messages: Message[];
  streamingMessageId: string | null;
  isThinkingStreaming?: boolean;
  onRegenerate?: (messageId: string) => void;
  hasMoreEarlier?: boolean;
  loadingEarlier?: boolean;
  onLoadEarlier?: () => Promise<string | null>;
  scrollToMessageId?: string | null;
  onScrollToMessageHandled?: () => void;
  onOpenToolLogs?: () => void;
}

const SCROLL_THRESHOLD = 80;

export function MessageList({
  messages,
  streamingMessageId,
  isThinkingStreaming,
  onRegenerate,
  hasMoreEarlier = false,
  loadingEarlier = false,
  onLoadEarlier,
  scrollToMessageId = null,
  onScrollToMessageHandled,
  onOpenToolLogs,
}: MessageListProps) {
  const { t } = useTranslation("chat");
  const parentRef = useRef<HTMLDivElement>(null);
  const [isAtBottom, setIsAtBottom] = useState(true);
  const prevCountRef = useRef(messages.length);
  const prevFirstIdRef = useRef(messages[0]?.id ?? null);
  const pendingAnchorRef = useRef<string | null>(null);

  const virtualizer = useVirtualizer({
    count: messages.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => MESSAGE_ROW_ESTIMATE,
    overscan: MESSAGE_ROW_OVERSCAN,
    getItemKey: (index) => messages[index]?.id ?? index,
  });

  const checkIsAtBottom = useCallback(() => {
    const el = parentRef.current;
    if (!el) return true;
    return el.scrollHeight - el.scrollTop - el.clientHeight < SCROLL_THRESHOLD;
  }, []);

  const scrollToBottom = useCallback(
    (behavior: ScrollBehavior = "smooth") => {
      const last = messages.length - 1;
      if (last < 0) return;
      virtualizer.scrollToIndex(last, { align: "end", behavior });
    },
    [messages.length, virtualizer]
  );

  useEffect(() => {
    const el = parentRef.current;
    if (!el) return;
    const handleScroll = () => setIsAtBottom(checkIsAtBottom());
    el.addEventListener("scroll", handleScroll, { passive: true });
    return () => el.removeEventListener("scroll", handleScroll);
  }, [checkIsAtBottom]);

  useLayoutEffect(() => {
    const count = messages.length;
    const firstId = messages[0]?.id ?? null;
    const prevCount = prevCountRef.current;
    const prevFirst = prevFirstIdRef.current;

    if (pendingAnchorRef.current) {
      const anchorId = pendingAnchorRef.current;
      const idx = messages.findIndex((m) => m.id === anchorId);
      if (idx >= 0) {
        virtualizer.scrollToIndex(idx, { align: "start", behavior: "auto" });
      }
      pendingAnchorRef.current = null;
    } else if (count > prevCount && firstId === prevFirst) {
      if (isAtBottom || prevCount === 0) {
        scrollToBottom(prevCount === 0 ? "instant" : "auto");
      }
    } else if (prevCount === 0 && count > 0) {
      scrollToBottom("instant");
    }

    prevCountRef.current = count;
    prevFirstIdRef.current = firstId;
  }, [messages, isAtBottom, scrollToBottom, virtualizer]);

  useEffect(() => {
    if (!scrollToMessageId) return;
    const idx = messages.findIndex((m) => m.id === scrollToMessageId);
    if (idx >= 0) {
      virtualizer.scrollToIndex(idx, { align: "center", behavior: "smooth" });
    }
    onScrollToMessageHandled?.();
  }, [scrollToMessageId, messages, virtualizer, onScrollToMessageHandled]);

  const handleLoadEarlier = useCallback(async () => {
    if (!onLoadEarlier) return;
    const anchor = await onLoadEarlier();
    if (anchor) pendingAnchorRef.current = anchor;
  }, [onLoadEarlier]);

  const items = virtualizer.getVirtualItems();

  return (
    <div className="relative min-h-0 flex-1">
      <div
        ref={parentRef}
        className="h-full overflow-y-auto"
        role="log"
        aria-live="polite"
      >
        <div className="mx-auto w-full max-w-3xl px-4 py-6">
          {hasMoreEarlier ? (
            <div className="mb-4 flex justify-center">
              <Button
                type="button"
                variant="ghost"
                size="sm"
                disabled={loadingEarlier}
                onClick={handleLoadEarlier}
                className="text-muted-foreground hover:text-foreground"
              >
                {loadingEarlier
                  ? t("loadEarlierLoading")
                  : t("loadEarlier")}
              </Button>
            </div>
          ) : null}

          <div
            className="relative w-full"
            style={{ height: virtualizer.getTotalSize() }}
          >
            {items.map((item) => {
              const msg = messages[item.index];
              if (!msg) return null;
              const isCurrentStreaming = msg.id === streamingMessageId;
              return (
                <div
                  key={item.key}
                  data-index={item.index}
                  ref={virtualizer.measureElement}
                  className="absolute left-0 top-0 w-full"
                  style={{
                    transform: `translateY(${item.start}px)`,
                  }}
                >
                  <MessageItem
                    message={msg}
                    isStreaming={isCurrentStreaming}
                    isThinkingStreaming={
                      isCurrentStreaming && isThinkingStreaming
                    }
                    onRegenerate={
                      msg.role === "assistant" ? onRegenerate : undefined
                    }
                    highlight={scrollToMessageId === msg.id}
                    onOpenToolLogs={onOpenToolLogs}
                  />
                </div>
              );
            })}
          </div>
        </div>
      </div>

      {!isAtBottom ? (
        <Button
          type="button"
          variant="outline"
          size="icon"
          onClick={() => scrollToBottom()}
          className={cn(
            "absolute bottom-4 left-1/2 z-10 size-8 -translate-x-1/2 rounded-full",
            "dark:bg-background dark:hover:bg-muted"
          )}
          aria-label={t("scrollToBottom")}
        >
          <ArrowDown className="size-4" />
        </Button>
      ) : null}
    </div>
  );
}
