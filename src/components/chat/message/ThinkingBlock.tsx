import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown } from "lucide-react";
import { cn } from "@/lib/utils";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { MessageResponse } from "../markdown/MessageResponse";
import { Shimmer } from "./Shimmer";

const AUTO_CLOSE_DELAY = 1000;

interface ThinkingBlockProps {
  content: string;
  isStreaming?: boolean;
}

export function ThinkingBlock({ content, isStreaming }: ThinkingBlockProps) {
  const { t } = useTranslation("chat");
  const [open, setOpen] = useState(Boolean(isStreaming));
  const [durationSec, setDurationSec] = useState(0);
  const startedAtRef = useRef<number | null>(null);
  const userClosedRef = useRef(false);
  const autoClosedRef = useRef(false);

  useEffect(() => {
    if (isStreaming) {
      if (startedAtRef.current === null) {
        startedAtRef.current = Date.now();
      }
      autoClosedRef.current = false;
      // Streaming always re-opens unless the user explicitly collapsed mid-stream.
      if (!userClosedRef.current) {
        setOpen(true);
      }
      return;
    }

    let hadLiveStream = false;
    if (startedAtRef.current !== null) {
      hadLiveStream = true;
      const elapsed = Math.ceil((Date.now() - startedAtRef.current) / 1000);
      setDurationSec(Math.max(elapsed, 1));
      startedAtRef.current = null;
    }

    // History mounts with isStreaming=false — keep collapsed, no timer.
    if (!hadLiveStream) {
      autoClosedRef.current = true;
      return;
    }

    if (autoClosedRef.current) return;
    if (userClosedRef.current) {
      autoClosedRef.current = true;
      return;
    }

    const timer = window.setTimeout(() => {
      setOpen(false);
      autoClosedRef.current = true;
    }, AUTO_CLOSE_DELAY);

    return () => window.clearTimeout(timer);
  }, [isStreaming]);

  if (!content && !isStreaming) return null;

  const label = isStreaming
    ? null
    : durationSec > 0
      ? t("thinkingDuration", { count: durationSec })
      : t("thinkingFewSeconds");

  return (
    <Collapsible
      open={open}
      onOpenChange={(next) => {
        userClosedRef.current = !next;
        setOpen(next);
      }}
      className="not-prose mb-4"
    >
      <CollapsibleTrigger
        className={cn(
          "flex w-full items-center gap-2 text-sm text-muted-foreground",
          "transition-colors duration-[var(--ds-dur-fast)] hover:text-foreground"
        )}
      >
        {isStreaming ? (
          <Shimmer>{t("thinkingStreaming")}</Shimmer>
        ) : (
          <span>{label ?? t("thinkingFewSeconds")}</span>
        )}
        <ChevronDown
          className={cn(
            "size-4 transition-transform duration-[var(--ds-dur-fast)]",
            open && "rotate-180"
          )}
        />
      </CollapsibleTrigger>
      <CollapsibleContent className="data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=closed]:animate-out data-[state=closed]:fade-out-0">
        <div className="mt-4 text-sm text-muted-foreground">
          {content ? (
            <MessageResponse
              content={content}
              isStreaming={isStreaming}
              className="text-muted-foreground"
            />
          ) : null}
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
}
