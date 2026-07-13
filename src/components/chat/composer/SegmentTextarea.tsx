import { useCallback, useEffect, useRef, type ClipboardEvent, type KeyboardEvent } from "react";
import { cn } from "@/lib/utils";
import type { LocalTextSelection } from "./composerTextSelection";

/** 单行固定 32px，与发送钮 size-8 对齐 */
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
  const hardLines = Math.max(1, value.split("\n").length);
  const isMultiline = hardLines > 1;
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

    // wrap=off：仅硬换行增高。勿用 scrollHeight 判定单行——leading-8 下会虚高，把框撑高后文字贴顶。
    if (!isMultiline) {
      el.style.height = `${COMPOSER_SINGLE_LINE_HEIGHT_PX}px`;
      return;
    }

    el.style.height = "0px";
    const next = Math.min(
      Math.max(el.scrollHeight, COMPOSER_SINGLE_LINE_HEIGHT_PX),
      MAX_HEIGHT_PX
    );
    el.style.height = `${next}px`;
  }, [isLastText, isMultiline]);

  useEffect(() => {
    syncSize();
  }, [value, isLastText, isMultiline, syncSize]);

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
        "box-border resize-none text-sm leading-8",
        "py-0 [field-sizing:fixed]",
        "text-foreground outline-none placeholder:text-muted-foreground/55",
        "disabled:cursor-not-allowed disabled:opacity-50",
        hasHighlight ? "bg-primary/15" : "bg-transparent",
        isMultiline
          ? "max-h-[200px] min-h-8 overflow-y-auto"
          : "h-8 max-h-8 min-h-8 overflow-hidden",
        isLastText
          ? "min-w-[2rem] flex-1"
          : "shrink-0 overflow-hidden whitespace-nowrap"
      )}
      style={{
        height: isMultiline ? undefined : `${COMPOSER_SINGLE_LINE_HEIGHT_PX}px`,
      }}
    />
  );
}
