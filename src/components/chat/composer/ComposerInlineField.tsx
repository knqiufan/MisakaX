import { useCallback, useEffect, type ClipboardEvent, type KeyboardEvent } from "react";
import { cn } from "@/lib/utils";
import {
  clearAllTextContent,
  findLastTextSegmentIndex,
  getDocumentPlainText,
  getSegmentId,
  handleSegmentBackspace,
  handleSegmentDelete,
  type ComposerCursor,
  type ComposerSegment,
} from "./composerSegment";
import { InlineMentionChip } from "./MentionPill";
import { SegmentTextarea } from "./SegmentTextarea";
import { useComposerTextSelection } from "./useComposerTextSelection";

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
  const lastTextIndex = findLastTextSegmentIndex(segments);
  const hasMentions = segments.some((s) => s.type === "mention");

  const {
    registerRef,
    focusSegment,
    isAllTextSelected,
    handleSelectAllKey,
    handleSelectReport,
    getSegmentHighlight,
    clearSelection,
  } = useComposerTextSelection({ segments, onCursorChange });

  useEffect(() => {
    if (focusRequestId <= 0) return;
    const targetId = composerCursor?.segmentId;
    if (targetId) focusSegment(targetId, composerCursor?.offset);
  }, [focusRequestId, composerCursor, focusSegment]);

  const handleClearAllText = useCallback(() => {
    const result = clearAllTextContent(segments);
    onSegmentsChange(result.segments, result.cursor);
    clearSelection();
    requestAnimationFrame(() => focusSegment(result.cursor.segmentId, 0));
  }, [segments, onSegmentsChange, clearSelection, focusSegment]);

  const handleCopy = useCallback(
    (e: ClipboardEvent<HTMLDivElement>) => {
      if (!isAllTextSelected) return;
      e.preventDefault();
      e.clipboardData.setData("text/plain", getDocumentPlainText(segments));
    },
    [isAllTextSelected, segments]
  );

  const handleTextKeyDown = useCallback(
    (segmentId: string, e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (handleSelectAllKey(segmentId, e)) return;

      if (
        isAllTextSelected &&
        (e.key === "Backspace" || e.key === "Delete") &&
        !e.nativeEvent.isComposing
      ) {
        e.preventDefault();
        handleClearAllText();
        return;
      }

      if (e.key === "Backspace" && !e.nativeEvent.isComposing) {
        const cursor = { segmentId, offset: e.currentTarget.selectionStart };
        const result = handleSegmentBackspace(segments, cursor);
        if (result.handled) {
          e.preventDefault();
          clearSelection();
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
          clearSelection();
          onSegmentsChange(result.segments, result.cursor);
          requestAnimationFrame(() => focusSegment(result.cursor.segmentId, result.cursor.offset));
          return;
        }
      }
      onKeyDown(e);
    },
    [
      segments,
      isAllTextSelected,
      handleSelectAllKey,
      handleClearAllText,
      onSegmentsChange,
      focusSegment,
      clearSelection,
      onKeyDown,
    ]
  );

  return (
    <div
      className={cn("flex min-w-0 flex-1 flex-wrap items-center gap-x-0.5", hasMentions)}
      onCopy={handleCopy}
    >
      {segments.map((segment, index) => {
        if (segment.type === "mention") {
          return <InlineMentionChip key={getSegmentId(segment)} mention={segment.mention} />;
        }
        const isLastText = index === lastTextIndex;
        const showPlaceholder = isLastText && !hasMentions && segment.value.length === 0;
        return (
          <SegmentTextarea
            key={segment.id}
            segmentId={segment.id}
            value={segment.value}
            isLastText={isLastText}
            showPlaceholder={showPlaceholder}
            placeholder={placeholder}
            disabled={disabled}
            highlightRange={getSegmentHighlight(segment.id, segment.value)}
            onChange={(value) => {
              clearSelection();
              onTextChange(segment.id, value);
            }}
            onKeyDown={(e) => handleTextKeyDown(segment.id, e)}
            onPaste={onPaste}
            onSelectReport={(el) => handleSelectReport(segment.id, el)}
            registerRef={registerRef}
          />
        );
      })}
    </div>
  );
}
