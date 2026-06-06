import { useCallback, useEffect, useRef, type ClipboardEvent, type KeyboardEvent } from "react";
import { cn } from "@/lib/utils";
import {
  findLastTextSegmentIndex,
  getSegmentId,
  handleSegmentBackspace,
  handleSegmentDelete,
  type ComposerCursor,
  type ComposerSegment,
} from "./composer-segment";
import { InlineMentionChip } from "./MentionPill";

const MIN_HEIGHT = 36;
const MAX_HEIGHT = 200;

interface ComposerInlineFieldProps {
  segments: ComposerSegment[];
  composerCursor: ComposerCursor | null;
  focusRequestId: number;
  onTextChange: (id: string, value: string) => void;
  onCursorChange: (cursor: ComposerCursor) => void;
  onSegmentsChange: (segments: ComposerSegment[], cursor: ComposerCursor) => void;
  onKeyDown: (e: KeyboardEvent<HTMLTextAreaElement>) => void;
  onPaste: (e: ClipboardEvent<HTMLTextAreaElement>) => void;
  placeholder: string;
  disabled: boolean;
}

export function ComposerInlineField(props: ComposerInlineFieldProps) {
  return <ComposerInlineFieldInner {...props} />;
}

function ComposerInlineFieldInner({
  segments,
  composerCursor,
  focusRequestId,
  onTextChange,
  onCursorChange,
  onSegmentsChange,
  onKeyDown,
  onPaste,
  placeholder,
  disabled,
}: ComposerInlineFieldProps) {
  const textareaRefs = useRef<Map<string, HTMLTextAreaElement>>(new Map());
  const lastTextIndex = findLastTextSegmentIndex(segments);
  const hasMentions = segments.some((s) => s.type === "mention");

  const adjustHeight = useCallback((el: HTMLTextAreaElement) => {
    el.style.height = "auto";
    el.style.height = `${Math.min(Math.max(el.scrollHeight, MIN_HEIGHT), MAX_HEIGHT)}px`;
  }, []);

  const focusSegment = useCallback((segmentId: string, offset?: number) => {
    const el = textareaRefs.current.get(segmentId);
    if (!el) return;
    el.focus();
    const pos = offset ?? el.value.length;
    el.setSelectionRange(pos, pos);
    adjustHeight(el);
  }, [adjustHeight]);

  useEffect(() => {
    if (focusRequestId <= 0) return;
    const targetId = composerCursor?.segmentId;
    if (targetId) focusSegment(targetId, composerCursor?.offset);
  }, [focusRequestId, composerCursor, focusSegment]);

  useEffect(() => {
    textareaRefs.current.forEach((el) => adjustHeight(el));
  }, [segments, adjustHeight]);

  const reportCursor = useCallback((segmentId: string, el: HTMLTextAreaElement) => {
    onCursorChange({ segmentId, offset: el.selectionStart });
  }, [onCursorChange]);

  const handleTextKeyDown = useCallback((segmentId: string, e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Backspace" && !e.nativeEvent.isComposing) {
      const cursor = { segmentId, offset: e.currentTarget.selectionStart };
      const result = handleSegmentBackspace(segments, cursor);
      if (result.handled) {
        e.preventDefault();
        onSegmentsChange(result.segments, result.cursor);
        requestAnimationFrame(() => focusSegment(result.cursor.segmentId, result.cursor.offset));
        return;
      }
    }
    if (e.key === "Delete" && !e.nativeEvent.isComposing) {
      const cursor = { segmentId, offset: e.currentTarget.selectionStart };
      const result = handleSegmentDelete(segments, cursor);
      if (result.handled) {
        e.preventDefault();
        onSegmentsChange(result.segments, result.cursor);
        requestAnimationFrame(() => focusSegment(result.cursor.segmentId, result.cursor.offset));
        return;
      }
    }
    onKeyDown(e);
  }, [segments, onSegmentsChange, focusSegment, onKeyDown]);

  return (
    <div className={cn("flex min-w-0 flex-1 flex-wrap items-center gap-1", hasMentions && "py-0.5")}>
      {segments.map((segment, index) => renderSegment(segment, index))}
    </div>
  );

  function renderSegment(segment: ComposerSegment, index: number) {
    if (segment.type === "mention") {
      return <InlineMentionChip key={getSegmentId(segment)} mention={segment.mention} />;
    }
    const isLastText = index === lastTextIndex;
    const showPlaceholder = isLastText && !hasMentions && segment.value.length === 0;
    const inlineWidth =
      !isLastText && segment.value.length > 0
        ? `${Math.max(1, segment.value.length)}ch`
        : undefined;

    return (
      <textarea
        key={segment.id}
        ref={(el) => {
          if (el) textareaRefs.current.set(segment.id, el);
          else textareaRefs.current.delete(segment.id);
        }}
        value={segment.value}
        onChange={(e) => onTextChange(segment.id, e.target.value)}
        onKeyDown={(e) => handleTextKeyDown(segment.id, e)}
        onKeyUp={(e) => reportCursor(segment.id, e.currentTarget)}
        onSelect={(e) => reportCursor(segment.id, e.currentTarget)}
        onFocus={(e) => reportCursor(segment.id, e.currentTarget)}
        onPaste={onPaste}
        placeholder={showPlaceholder ? placeholder : ""}
        disabled={disabled}
        rows={1}
        cols={isLastText ? undefined : 1}
        className={cn(
          "max-h-[200px] resize-none bg-transparent py-2 text-sm leading-[20px]",
          "text-foreground outline-none placeholder:text-muted-foreground/55",
          "disabled:cursor-not-allowed disabled:opacity-50",
          isLastText
            ? "min-h-[36px] min-w-[2rem] flex-1 [field-sizing:content]"
            : "min-h-[36px] min-w-0 max-w-full [field-sizing:content]"
        )}
        style={{
          height: `${MIN_HEIGHT}px`,
          width: inlineWidth,
        }}
      />
    );
  }
}
