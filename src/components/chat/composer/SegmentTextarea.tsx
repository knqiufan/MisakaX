import { useCallback, useEffect, useRef, type ClipboardEvent, type KeyboardEvent } from "react";
import { cn } from "@/lib/utils";
import type { LocalTextSelection } from "./composerTextSelection";

/** leading-5 (20) + py-1.5×2 (12) = 32，与发送钮 size-8 对齐 */
export const COMPOSER_SINGLE_LINE_HEIGHT_PX = 32;
const MAX_HEIGHT_PX = 200;

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

    if (isLastText) {
      el.style.width = "";
    } else {
      el.style.width = "0px";
      el.style.width = `${Math.max(el.scrollWidth + 4, 8)}px`;
    }

    // 先锁单行高，避免 UA 默认 rows 高度 / scrollHeight 虚高把空态撑成「瘦高胶囊」
    el.style.height = `${COMPOSER_SINGLE_LINE_HEIGHT_PX}px`;
    const hardLines = value.split("\n").length;
    const needsGrow =
      hardLines > 1 || el.scrollHeight > COMPOSER_SINGLE_LINE_HEIGHT_PX + 2;
    if (!needsGrow) return;

    el.style.height = "0px";
    const next = Math.min(
      Math.max(el.scrollHeight, COMPOSER_SINGLE_LINE_HEIGHT_PX),
      MAX_HEIGHT_PX
    );
    el.style.height = `${next}px`;
  }, [isLastText, value]);

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
        "box-border max-h-[200px] min-h-8 resize-none overflow-y-auto",
        "py-1.5 text-sm leading-5",
        "text-foreground outline-none placeholder:text-muted-foreground/55",
        "disabled:cursor-not-allowed disabled:opacity-50",
        hasHighlight ? "bg-primary/15" : "bg-transparent",
        isLastText
          ? "min-w-[2rem] flex-1"
          : "shrink-0 overflow-hidden whitespace-nowrap"
      )}
      style={{ height: `${COMPOSER_SINGLE_LINE_HEIGHT_PX}px` }}
    />
  );
}
