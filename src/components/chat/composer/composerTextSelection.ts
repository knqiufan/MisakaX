import type { ComposerCursor, ComposerSegment } from "./composerSegment";
import { getSegmentId } from "./composerSegment";

export interface ComposerTextSelection {
  anchor: ComposerCursor;
  focus: ComposerCursor;
}

export interface TextSegmentEntry {
  id: string;
  value: string;
}

export interface LocalTextSelection {
  start: number;
  end: number;
}

export function isSelectAllKey(key: string, ctrlOrMeta: boolean): boolean {
  return ctrlOrMeta && key.toLowerCase() === "a";
}

export function getTextSegmentEntries(segments: ComposerSegment[]): TextSegmentEntry[] {
  return segments
    .filter((s): s is Extract<ComposerSegment, { type: "text" }> => s.type === "text")
    .map((s) => ({ id: s.id, value: s.value }));
}

export function buildFullTextSelection(segments: ComposerSegment[]): ComposerTextSelection | null {
  const entries = getTextSegmentEntries(segments);
  if (entries.length === 0) return null;
  const first = entries[0];
  const last = entries[entries.length - 1];
  return {
    anchor: { segmentId: first.id, offset: 0 },
    focus: { segmentId: last.id, offset: last.value.length },
  };
}

export function compareCursors(
  segments: ComposerSegment[],
  a: ComposerCursor,
  b: ComposerCursor
): number {
  const order = buildSegmentOrderMap(segments);
  const aOrder = order.get(a.segmentId) ?? 0;
  const bOrder = order.get(b.segmentId) ?? 0;
  if (aOrder !== bOrder) return aOrder < bOrder ? -1 : 1;
  if (a.offset === b.offset) return 0;
  return a.offset < b.offset ? -1 : 1;
}

export function normalizeTextSelection(
  segments: ComposerSegment[],
  selection: ComposerTextSelection
): { start: ComposerCursor; end: ComposerCursor } {
  const cmp = compareCursors(segments, selection.anchor, selection.focus);
  if (cmp <= 0) {
    return { start: selection.anchor, end: selection.focus };
  }
  return { start: selection.focus, end: selection.anchor };
}

export function getLocalSelectionForSegment(
  segments: ComposerSegment[],
  segmentId: string,
  segmentValue: string,
  start: ComposerCursor,
  end: ComposerCursor
): LocalTextSelection | null {
  const segStart: ComposerCursor = { segmentId, offset: 0 };
  const segEnd: ComposerCursor = { segmentId, offset: segmentValue.length };

  if (compareCursors(segments, segEnd, start) < 0) return null;
  if (compareCursors(segments, segStart, end) > 0) return null;

  const localStart =
    segmentId === start.segmentId ? Math.min(start.offset, segmentValue.length) : 0;
  const localEnd =
    segmentId === end.segmentId ? Math.min(end.offset, segmentValue.length) : segmentValue.length;

  if (localStart >= localEnd) return null;
  return { start: localStart, end: localEnd };
}

export function isFullTextSelection(
  segments: ComposerSegment[],
  selection: ComposerTextSelection | null
): boolean {
  if (!selection) return false;
  const full = buildFullTextSelection(segments);
  if (!full) return false;
  const a = normalizeTextSelection(segments, selection);
  const b = normalizeTextSelection(segments, full);
  return (
    a.start.segmentId === b.start.segmentId &&
    a.start.offset === b.start.offset &&
    a.end.segmentId === b.end.segmentId &&
    a.end.offset === b.end.offset
  );
}

export function selectionFromTextarea(
  segmentId: string,
  selectionStart: number,
  selectionEnd: number
): ComposerTextSelection | null {
  if (selectionStart === selectionEnd) return null;
  return {
    anchor: { segmentId, offset: selectionStart },
    focus: { segmentId, offset: selectionEnd },
  };
}

function buildSegmentOrderMap(segments: ComposerSegment[]): Map<string, number> {
  const order = new Map<string, number>();
  segments.forEach((seg, index) => order.set(getSegmentId(seg), index));
  return order;
}

