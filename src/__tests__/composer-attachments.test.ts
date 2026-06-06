import { describe, expect, it } from "vitest";
import {
  ACCEPTED_ATTACHMENT_TYPES,
  canSendComposerMessage,
  classifyAttachment,
} from "@/components/chat/composer/attachment-utils";
import {
  buildOutgoingContent,
  countSendableMentions,
  mentionsToWorkspaceAttachments,
} from "@/components/chat/composer/composer-mention-utils";
import {
  buildOutgoingFromSegments,
  createEmptyDocument,
  handleSegmentBackspace,
  handleSegmentDelete,
  insertMentionAtCursor,
  type ComposerSegment,
} from "@/components/chat/composer/composer-segment";
import type { PendingFileMention } from "@/stores/composer-store";

describe("composer attachment classification", () => {
  it("accepts supported image attachments", () => {
    expect(classifyAttachment("image/png", "shot.png")).toEqual({
      kind: "image",
      supported: true,
    });
  });

  it("accepts markdown and plain text attachments", () => {
    expect(classifyAttachment("text/markdown", "notes.md")).toEqual({
      kind: "text",
      supported: true,
    });
    expect(classifyAttachment("text/plain", "readme.txt")).toEqual({
      kind: "text",
      supported: true,
    });
  });

  it("marks pdf and office documents as planned but unsupported", () => {
    expect(classifyAttachment("application/pdf", "brief.pdf")).toEqual({
      kind: "planned",
      supported: false,
    });
  });

  it("publishes accept strings without planned PDF/Office extensions", () => {
    expect(ACCEPTED_ATTACHMENT_TYPES).toContain("image/png");
    expect(ACCEPTED_ATTACHMENT_TYPES).toContain(".md");
    expect(ACCEPTED_ATTACHMENT_TYPES).not.toContain(".pdf");
  });
});

describe("composer send validation", () => {
  it("allows sending with text attachments even when content is empty", () => {
    expect(
      canSendComposerMessage({
        content: "",
        attachmentCount: 1,
        disabled: false,
        isStreaming: false,
      })
    ).toBe(true);
  });

  it("blocks sending while streaming", () => {
    expect(
      canSendComposerMessage({
        content: "hello",
        attachmentCount: 0,
        disabled: false,
        isStreaming: true,
      })
    ).toBe(false);
  });
});

function makeMention(overrides: Partial<PendingFileMention> = {}): PendingFileMention {
  return {
    id: "test-id",
    absPath: "d:/proj/src/foo.tsx",
    relPath: "src/foo.tsx",
    name: "foo.tsx",
    status: "ready",
    extractedText: "const x = 1;",
    size: 12,
    mime: "text/plain",
    ...overrides,
  };
}

function textSeg(id: string, value: string): ComposerSegment {
  return { type: "text", id, value };
}

describe("insertMentionAtCursor", () => {
  it("inserts mention in the middle of text", () => {
    const segments = [textSeg("t1", "请修复 这段代码")];
    const result = insertMentionAtCursor(segments, { segmentId: "t1", offset: 4 }, makeMention());
    expect(result.segments).toHaveLength(3);
    expect(result.segments[0]).toEqual(textSeg("t1", "请修复 "));
    expect(result.segments[1]?.type).toBe("mention");
    if (result.segments[2]?.type === "text") {
      expect(result.segments[2].value).toBe("这段代码");
    }
  });

  it("appends mention at end when cursor is at end", () => {
    const segments = [textSeg("t1", "hello")];
    const result = insertMentionAtCursor(segments, { segmentId: "t1", offset: 5 }, makeMention());
    expect(result.segments[1]?.type).toBe("mention");
  });

  it("prunes empty text between adjacent mentions", () => {
    let segments = [textSeg("t1", "")];
    const first = insertMentionAtCursor(
      segments,
      { segmentId: "t1", offset: 0 },
      makeMention({
        id: "m1",
        absPath: "d:/proj/Makefile",
        relPath: "Makefile",
        name: "Makefile",
      })
    );
    segments = first.segments;
    const trailing = segments.find((s) => s.type === "text");
    const second = insertMentionAtCursor(
      segments,
      { segmentId: trailing?.type === "text" ? trailing.id : "t1", offset: 0 },
      makeMention({
        id: "m2",
        absPath: "d:/proj/README_JP.md",
        relPath: "README_JP.md",
        name: "README_JP.md",
      })
    );
    expect(second.segments.filter((s) => s.type === "mention")).toHaveLength(2);
    expect(second.segments.filter((s) => s.type === "text")).toHaveLength(1);
    expect(second.segments[0]?.type).toBe("mention");
    expect(second.segments[1]?.type).toBe("mention");
  });
});

describe("handleSegmentBackspace", () => {
  it("deletes previous mention when cursor at offset 0", () => {
    const mention = makeMention({ id: "m1" });
    const segments: ComposerSegment[] = [
      textSeg("t1", "请修复 "),
      { type: "mention", mention },
      textSeg("t2", "这段代码"),
    ];
    const result = handleSegmentBackspace(segments, { segmentId: "t2", offset: 0 });
    expect(result.handled).toBe(true);
    expect(result.segments).toHaveLength(1);
    if (result.segments[0]?.type === "text") {
      expect(result.segments[0].value).toBe("请修复 这段代码");
    }
  });
});

describe("handleSegmentDelete", () => {
  it("deletes next mention when cursor at end of text", () => {
    const mention = makeMention({ id: "m1" });
    const segments: ComposerSegment[] = [
      textSeg("t1", "请修复 "),
      { type: "mention", mention },
      textSeg("t2", "这段代码"),
    ];
    const result = handleSegmentDelete(segments, { segmentId: "t1", offset: 4 });
    expect(result.handled).toBe(true);
    expect(result.segments).toHaveLength(1);
    if (result.segments[0]?.type === "text") {
      expect(result.segments[0].value).toBe("请修复 这段代码");
    }
  });
});

describe("buildOutgoingFromSegments", () => {
  it("interleaves text and mentions in order", () => {
    const segments: ComposerSegment[] = [
      textSeg("t1", "请修复 "),
      { type: "mention", mention: makeMention() },
      textSeg("t2", "这段代码"),
    ];
    expect(buildOutgoingFromSegments(segments)).toBe("请修复 @src/foo.tsx这段代码");
  });
});

describe("mentionsToWorkspaceAttachments", () => {
  it("converts ready mentions from segments to text attachments", () => {
    const segments: ComposerSegment[] = [
      { type: "mention", mention: makeMention() },
      textSeg("t1", ""),
    ];
    const result = mentionsToWorkspaceAttachments(segments);
    expect(result).toHaveLength(1);
    expect(result[0].file_name).toBe("src/foo.tsx");
  });
});

describe("buildOutgoingContent", () => {
  it("returns empty for empty document", () => {
    expect(buildOutgoingContent(createEmptyDocument())).toBe("");
  });
});

describe("countSendableMentions", () => {
  it("counts ready and loading, excludes error", () => {
    const segments: ComposerSegment[] = [
      { type: "mention", mention: makeMention({ id: "a", status: "ready" }) },
      { type: "mention", mention: makeMention({ id: "b", status: "loading" }) },
      { type: "mention", mention: makeMention({ id: "c", status: "error" }) },
      textSeg("t1", ""),
    ];
    expect(countSendableMentions(segments)).toBe(2);
  });
});
