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

describe("Send validation logic", () => {
  function canSend(content: string, disabled: boolean, isStreaming: boolean): boolean {
    return content.trim().length > 0 && !disabled && !isStreaming;
  }

  it("should allow send when content is present and not streaming", () => {
    expect(canSend("hello", false, false)).toBe(true);
  });

  it("should disallow send with empty content", () => {
    expect(canSend("", false, false)).toBe(false);
    expect(canSend("   ", false, false)).toBe(false);
  });

  it("should disallow send when disabled", () => {
    expect(canSend("hello", true, false)).toBe(false);
  });

  it("should disallow send when streaming", () => {
    expect(canSend("hello", false, true)).toBe(false);
  });

  it("should allow content with only whitespace trimmed to empty", () => {
    expect(canSend("\n\n  \t", false, false)).toBe(false);
  });
});
