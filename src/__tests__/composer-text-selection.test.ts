import { describe, expect, it } from "vitest";
import {
  buildFullTextSelection,
  getLocalSelectionForSegment,
  isFullTextSelection,
  normalizeTextSelection,
} from "@/components/chat/composer/composer-text-selection";
import type { ComposerSegment } from "@/components/chat/composer/composer-segment";

function text(id: string, value: string): ComposerSegment {
  return { type: "text", id, value };
}

function mention(id: string): ComposerSegment {
  return {
    type: "mention",
    mention: {
      id,
      absPath: `/tmp/${id}`,
      relPath: id,
      name: id,
      status: "ready",
    },
  };
}

describe("composer-text-selection", () => {
  const segments: ComposerSegment[] = [
    text("t1", "阅读"),
    mention("m1"),
    text("t2", "告诉我"),
    mention("m2"),
    text("t3", "这是什么"),
  ];

  it("builds full text selection across mentions", () => {
    const full = buildFullTextSelection(segments);
    expect(full).toEqual({
      anchor: { segmentId: "t1", offset: 0 },
      focus: { segmentId: "t3", offset: 4 },
    });
  });

  it("normalizes reversed anchor and focus", () => {
    const normalized = normalizeTextSelection(segments, {
      anchor: { segmentId: "t3", offset: 2 },
      focus: { segmentId: "t1", offset: 1 },
    });
    expect(normalized).toEqual({
      start: { segmentId: "t1", offset: 1 },
      end: { segmentId: "t3", offset: 2 },
    });
  });

  it("maps local selection slices per text segment", () => {
    const bounds = normalizeTextSelection(segments, buildFullTextSelection(segments)!);
    expect(getLocalSelectionForSegment(segments, "t1", "阅读", bounds.start, bounds.end)).toEqual({
      start: 0,
      end: 2,
    });
    expect(getLocalSelectionForSegment(segments, "t2", "告诉我", bounds.start, bounds.end)).toEqual({
      start: 0,
      end: 3,
    });
    expect(getLocalSelectionForSegment(segments, "t3", "这是什么", bounds.start, bounds.end)).toEqual({
      start: 0,
      end: 4,
    });
  });

  it("detects full text selection", () => {
    const full = buildFullTextSelection(segments)!;
    expect(isFullTextSelection(segments, full)).toBe(true);
    expect(
      isFullTextSelection(segments, {
        anchor: { segmentId: "t1", offset: 0 },
        focus: { segmentId: "t1", offset: 1 },
      })
    ).toBe(false);
  });
});
