import { useCallback, useEffect, useRef, type ClipboardEvent, type KeyboardEvent } from "react";
import { cn } from "@/lib/utils";
import type { LocalTextSelection } from "./composer-text-selection";

const MIN_HEIGHT = 36;
const MAX_HEIGHT = 200;

interface SegmentTextareaProps {
  segmentId: string;
  value: string;
  isLastText: boolean;
  showPlaceholder: boolean;
  placeholder: string;
  disabled: boolean;
  highlightRange?: LocalTextSelection | null;
  onChange: (value: string) => void;
  onKeyDown: (e: KeyboardEvent<HTMLTextAreaElement>) => void;
  onPaste: (e: ClipboardEvent<HTMLTextAreaElement>) => void;
  onSelectReport: (el: HTMLTextAreaElement) => void;
  registerRef: (id: string, el: HTMLTextAreaElement | null) => void;
}

export function SegmentTextarea({
  segmentId,
  value,
  isLastText,
  showPlaceholder,
  placeholder,
  disabled,
  highlightRange,
  onChange,
  onKeyDown,
  onPaste,
  onSelectReport,
  registerRef,
}: SegmentTextareaProps) {
  const localRef = useRef<HTMLTextAreaElement>(null);
  const hasHighlight =
    highlightRange !== null &&
    highlightRange !== undefined &&
    highlightRange.end > highlightRange.start;

  const syncSize = useCallback(() => {
    const el = localRef.current;
    if (!el) return;

    el.style.height = "auto";
    el.style.height = `${Math.min(Math.max(el.scrollHeight, MIN_HEIGHT), MAX_HEIGHT)}px`;

    if (isLastText) {
      el.style.width = "";
      return;
    }

    el.style.width = "0px";
    el.style.width = `${Math.max(el.scrollWidth + 4, 8)}px`;
  }, [isLastText]);

  useEffect(() => {
    syncSize();
  }, [value, isLastText, syncSize]);

  return (
    <textarea
      ref={(el) => {
        localRef.current = el;
        registerRef(segmentId, el);
        if (el) requestAnimationFrame(syncSize);
      }}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      onKeyDown={onKeyDown}
      onKeyUp={(e) => onSelectReport(e.currentTarget)}
      onSelect={(e) => onSelectReport(e.currentTarget)}
      onFocus={(e) => onSelectReport(e.currentTarget)}
      onPaste={onPaste}
      placeholder={showPlaceholder ? placeholder : ""}
      disabled={disabled}
      rows={1}
      wrap="off"
      className={cn(
        "max-h-[200px] resize-none py-2 text-sm leading-[20px]",
        "text-foreground outline-none placeholder:text-muted-foreground/55",
        "disabled:cursor-not-allowed disabled:opacity-50",
        hasHighlight ? "bg-sky-500/20" : "bg-transparent",
        isLastText
          ? "min-h-[36px] min-w-[2rem] flex-1"
          : "min-h-[36px] shrink-0 overflow-hidden whitespace-nowrap"
      )}
      style={{ height: `${MIN_HEIGHT}px` }}
    />
  );
}
