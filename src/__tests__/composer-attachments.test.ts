import { describe, expect, it } from "vitest";
import {
  ACCEPTED_ATTACHMENT_TYPES,
  canSendComposerMessage,
  classifyAttachment,
} from "@/components/chat/composer/attachment-utils";

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
    expect(
      classifyAttachment(
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "brief.docx"
      )
    ).toEqual({ kind: "planned", supported: false });
  });

  it("publishes accept strings without planned PDF/Office extensions", () => {
    expect(ACCEPTED_ATTACHMENT_TYPES).toContain("image/png");
    expect(ACCEPTED_ATTACHMENT_TYPES).toContain(".md");
    expect(ACCEPTED_ATTACHMENT_TYPES).not.toContain(".pdf");
    expect(ACCEPTED_ATTACHMENT_TYPES).not.toContain(".docx");
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
