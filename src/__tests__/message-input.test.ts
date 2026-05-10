import { describe, it, expect } from "vitest";

/**
 * MessageInput logic tests (pure unit tests, no DOM rendering needed).
 * Tests keyboard shortcut logic and auto-resize constraints.
 */

describe("MessageInput keyboard logic", () => {
  it("Enter without shift should trigger send (logic check)", () => {
    const event = { key: "Enter", shiftKey: false, isComposing: false };
    const shouldSend = event.key === "Enter" && !event.shiftKey && !event.isComposing;
    expect(shouldSend).toBe(true);
  });

  it("Shift+Enter should not trigger send", () => {
    const event = { key: "Enter", shiftKey: true, isComposing: false };
    const shouldSend = event.key === "Enter" && !event.shiftKey && !event.isComposing;
    expect(shouldSend).toBe(false);
  });

  it("Enter during IME composition should not trigger send", () => {
    const event = { key: "Enter", shiftKey: false, isComposing: true };
    const shouldSend = event.key === "Enter" && !event.shiftKey && !event.isComposing;
    expect(shouldSend).toBe(false);
  });

  it("non-Enter key should not trigger send", () => {
    const event = { key: "a", shiftKey: false, isComposing: false };
    const shouldSend = event.key === "Enter" && !event.shiftKey && !event.isComposing;
    expect(shouldSend).toBe(false);
  });
});

describe("MessageInput auto-resize constraints", () => {
  const MIN_HEIGHT = 40;
  const MAX_HEIGHT = 200;

  function computeHeight(scrollHeight: number): number {
    return Math.min(Math.max(scrollHeight, MIN_HEIGHT), MAX_HEIGHT);
  }

  it("should use MIN_HEIGHT when content is shorter", () => {
    expect(computeHeight(20)).toBe(MIN_HEIGHT);
  });

  it("should use scrollHeight when within bounds", () => {
    expect(computeHeight(80)).toBe(80);
    expect(computeHeight(150)).toBe(150);
  });

  it("should cap at MAX_HEIGHT for tall content", () => {
    expect(computeHeight(300)).toBe(MAX_HEIGHT);
    expect(computeHeight(500)).toBe(MAX_HEIGHT);
  });

  it("should use exactly MIN_HEIGHT at boundary", () => {
    expect(computeHeight(MIN_HEIGHT)).toBe(MIN_HEIGHT);
  });

  it("should use exactly MAX_HEIGHT at boundary", () => {
    expect(computeHeight(MAX_HEIGHT)).toBe(MAX_HEIGHT);
  });
});

describe("ModelSelector groupByProvider logic", () => {
  interface FlatModel {
    id: string;
    label: string;
    provider: string;
  }

  function groupByProvider(
    models: FlatModel[]
  ): { provider: string; items: FlatModel[] }[] {
    const map = new Map<string, FlatModel[]>();
    for (const m of models) {
      const list = map.get(m.provider) ?? [];
      list.push(m);
      map.set(m.provider, list);
    }
    return Array.from(map.entries()).map(([provider, items]) => ({
      provider,
      items,
    }));
  }

  it("should group models by provider", () => {
    const models: FlatModel[] = [
      { id: "a:gpt-4o", label: "GPT-4o", provider: "OpenAI" },
      { id: "a:gpt-4", label: "GPT-4", provider: "OpenAI" },
      { id: "b:claude-sonnet", label: "Claude Sonnet", provider: "Anthropic" },
    ];

    const groups = groupByProvider(models);
    expect(groups).toHaveLength(2);
    expect(groups[0].provider).toBe("OpenAI");
    expect(groups[0].items).toHaveLength(2);
    expect(groups[1].provider).toBe("Anthropic");
    expect(groups[1].items).toHaveLength(1);
  });

  it("should handle empty model list", () => {
    const groups = groupByProvider([]);
    expect(groups).toHaveLength(0);
  });

  it("should handle single provider", () => {
    const models: FlatModel[] = [
      { id: "x:m1", label: "M1", provider: "P1" },
      { id: "x:m2", label: "M2", provider: "P1" },
    ];
    const groups = groupByProvider(models);
    expect(groups).toHaveLength(1);
    expect(groups[0].items).toHaveLength(2);
  });
});

describe("Send validation logic (with images)", () => {
  function canSend(
    content: string,
    hasImages: boolean,
    disabled: boolean,
    isStreaming: boolean
  ): boolean {
    return (content.trim().length > 0 || hasImages) && !disabled && !isStreaming;
  }

  it("should allow send when content is present and not streaming", () => {
    expect(canSend("hello", false, false, false)).toBe(true);
  });

  it("should disallow send with empty content and no images", () => {
    expect(canSend("", false, false, false)).toBe(false);
    expect(canSend("   ", false, false, false)).toBe(false);
  });

  it("should allow send with images even if text is empty", () => {
    expect(canSend("", true, false, false)).toBe(true);
    expect(canSend("  ", true, false, false)).toBe(true);
  });

  it("should disallow send when disabled", () => {
    expect(canSend("hello", false, true, false)).toBe(false);
  });

  it("should disallow send when streaming", () => {
    expect(canSend("hello", false, false, true)).toBe(false);
  });

  it("should disallow send with images when streaming", () => {
    expect(canSend("", true, false, true)).toBe(false);
  });
});

describe("Image validation logic", () => {
  const MAX_IMAGE_SIZE = 10 * 1024 * 1024;
  const ALLOWED_IMAGE_TYPES = [
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
  ];

  function validateImage(type: string, size: number): { valid: boolean; reason?: string } {
    if (!ALLOWED_IMAGE_TYPES.includes(type)) {
      return { valid: false, reason: "unsupported_type" };
    }
    if (size > MAX_IMAGE_SIZE) {
      return { valid: false, reason: "too_large" };
    }
    return { valid: true };
  }

  it("should accept valid PNG", () => {
    expect(validateImage("image/png", 1024).valid).toBe(true);
  });

  it("should accept valid JPEG", () => {
    expect(validateImage("image/jpeg", 5 * 1024 * 1024).valid).toBe(true);
  });

  it("should accept valid WebP", () => {
    expect(validateImage("image/webp", 100).valid).toBe(true);
  });

  it("should accept valid GIF", () => {
    expect(validateImage("image/gif", 2 * 1024 * 1024).valid).toBe(true);
  });

  it("should reject unsupported type", () => {
    const result = validateImage("image/svg+xml", 1024);
    expect(result.valid).toBe(false);
    expect(result.reason).toBe("unsupported_type");
  });

  it("should reject text file", () => {
    expect(validateImage("text/plain", 100).valid).toBe(false);
  });

  it("should reject image that is too large", () => {
    const result = validateImage("image/png", 11 * 1024 * 1024);
    expect(result.valid).toBe(false);
    expect(result.reason).toBe("too_large");
  });

  it("should accept image at exactly 10MB boundary", () => {
    expect(validateImage("image/png", MAX_IMAGE_SIZE).valid).toBe(true);
  });

  it("should reject image at 10MB + 1 byte", () => {
    expect(validateImage("image/png", MAX_IMAGE_SIZE + 1).valid).toBe(false);
  });
});

describe("Attachment parsing logic", () => {
  interface AttachmentImage {
    data: string;
    mime_type: string;
  }

  function parseAttachments(raw: string | null): AttachmentImage[] {
    if (!raw) return [];
    try {
      const parsed = JSON.parse(raw) as AttachmentImage[];
      return Array.isArray(parsed) ? parsed : [];
    } catch {
      return [];
    }
  }

  it("should return empty array for null", () => {
    expect(parseAttachments(null)).toEqual([]);
  });

  it("should return empty array for invalid JSON", () => {
    expect(parseAttachments("not-json")).toEqual([]);
  });

  it("should parse valid attachment array", () => {
    const json = JSON.stringify([
      { data: "abc123", mime_type: "image/png" },
      { data: "def456", mime_type: "image/jpeg" },
    ]);
    const result = parseAttachments(json);
    expect(result).toHaveLength(2);
    expect(result[0].mime_type).toBe("image/png");
  });

  it("should return empty array for non-array JSON", () => {
    expect(parseAttachments(JSON.stringify({ key: "value" }))).toEqual([]);
  });
});

describe("TokenUsage parsing logic", () => {
  interface TokenUsage {
    input_tokens: number;
    output_tokens: number;
    total_tokens: number;
  }

  function parseTokenUsage(raw: string | null): TokenUsage | null {
    if (!raw) return null;
    try {
      return JSON.parse(raw) as TokenUsage;
    } catch {
      return null;
    }
  }

  it("should return null for null input", () => {
    expect(parseTokenUsage(null)).toBeNull();
  });

  it("should return null for invalid JSON", () => {
    expect(parseTokenUsage("bad")).toBeNull();
  });

  it("should parse valid usage JSON", () => {
    const json = JSON.stringify({
      input_tokens: 100,
      output_tokens: 200,
      total_tokens: 300,
    });
    const result = parseTokenUsage(json);
    expect(result).not.toBeNull();
    expect(result!.input_tokens).toBe(100);
    expect(result!.output_tokens).toBe(200);
    expect(result!.total_tokens).toBe(300);
  });
});
