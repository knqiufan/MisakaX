import { useCallback, useEffect, useRef, useState, type KeyboardEvent } from "react";
import type { ComposerCursor, ComposerSegment } from "./composer-segment";
import {
  buildFullTextSelection,
  getLocalSelectionForSegment,
  getTextSegmentEntries,
  isFullTextSelection,
  isSelectAllKey,
  normalizeTextSelection,
  selectionFromTextarea,
  type ComposerTextSelection,
  type LocalTextSelection,
} from "./composer-text-selection";

interface UseComposerTextSelectionOptions {
  segments: ComposerSegment[];
  onCursorChange: (cursor: ComposerCursor) => void;
}

export function useComposerTextSelection({
  segments,
  onCursorChange,
}: UseComposerTextSelectionOptions) {
  const [textSelection, setTextSelection] = useState<ComposerTextSelection | null>(null);
  const textareaRefs = useRef<Map<string, HTMLTextAreaElement>>(new Map());
  const activeSegmentRef = useRef<string | null>(null);

  const registerRef = useCallback((id: string, el: HTMLTextAreaElement | null) => {
    if (el) textareaRefs.current.set(id, el);
    else textareaRefs.current.delete(id);
  }, []);

  const applyDomSelection = useCallback(
    (selection: ComposerTextSelection, focusSegmentId: string) => {
      const bounds = normalizeTextSelection(segments, selection);
      for (const entry of getTextSegmentEntries(segments)) {
        const local = getLocalSelectionForSegment(
          segments,
          entry.id,
          entry.value,
          bounds.start,
          bounds.end
        );
        const el = textareaRefs.current.get(entry.id);
        if (!el || !local) continue;
        if (entry.id === focusSegmentId) {
          el.focus();
          el.setSelectionRange(local.start, local.end);
        }
      }
    },
    [segments]
  );

  const focusSegment = useCallback((segmentId: string, offset?: number) => {
    const el = textareaRefs.current.get(segmentId);
    if (!el) return;
    el.focus();
    const pos = offset ?? el.value.length;
    el.setSelectionRange(pos, pos);
    setTextSelection(null);
  }, []);

  const handleSelectAll = useCallback(
    (segmentId: string) => {
      const full = buildFullTextSelection(segments);
      if (!full) return;
      setTextSelection(full);
      applyDomSelection(full, segmentId);
      onCursorChange({ segmentId, offset: 0 });
    },
    [segments, applyDomSelection, onCursorChange]
  );

  const handleSelectReport = useCallback(
    (segmentId: string, el: HTMLTextAreaElement) => {
      activeSegmentRef.current = segmentId;
      const range = selectionFromTextarea(
        segmentId,
        el.selectionStart,
        el.selectionEnd
      );
      setTextSelection(range);
      onCursorChange({ segmentId, offset: el.selectionStart });
    },
    [onCursorChange]
  );

  const handleSelectAllKey = useCallback(
    (segmentId: string, e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (!isSelectAllKey(e.key, e.ctrlKey || e.metaKey)) return false;
      e.preventDefault();
      handleSelectAll(segmentId);
      return true;
    },
    [handleSelectAll]
  );

  const getSegmentHighlight = useCallback(
    (segmentId: string, value: string): LocalTextSelection | null => {
      if (!textSelection) return null;
      const bounds = normalizeTextSelection(segments, textSelection);
      return getLocalSelectionForSegment(
        segments,
        segmentId,
        value,
        bounds.start,
        bounds.end
      );
    },
    [segments, textSelection]
  );

  const clearSelection = useCallback(() => setTextSelection(null), []);

  const isAllTextSelected = isFullTextSelection(segments, textSelection);

  useEffect(() => {
    if (!textSelection || !activeSegmentRef.current) return;
    const focusId = activeSegmentRef.current;
    requestAnimationFrame(() => applyDomSelection(textSelection, focusId));
  }, [textSelection, applyDomSelection]);

  return {
    textareaRefs,
    registerRef,
    focusSegment,
    textSelection,
    isAllTextSelected,
    handleSelectAllKey,
    handleSelectReport,
    getSegmentHighlight,
    clearSelection,
  };
}
