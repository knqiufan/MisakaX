import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const open = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/plugin-shell", () => ({ open }));

import { MessageResponse } from "@/components/chat/markdown";
import { normalizeExternalUrl, openExternalUrl } from "@/lib/external-links";

describe("W5 safe external links", () => {
  beforeEach(() => open.mockReset().mockResolvedValue(undefined));

  it("allows credential-free HTTP(S) URLs and rejects executable or ambiguous schemes", async () => {
    expect(normalizeExternalUrl("https://example.com/docs?q=1")).toBe(
      "https://example.com/docs?q=1",
    );
    expect(normalizeExternalUrl("javascript:alert(1)")).toBeNull();
    expect(normalizeExternalUrl("data:text/html,<script>alert(1)</script>")).toBeNull();
    expect(normalizeExternalUrl("https://user:secret@example.com/")).toBeNull();
    expect(normalizeExternalUrl("https://example.com/\nnext")).toBeNull();

    await expect(openExternalUrl("javascript:alert(1)")).rejects.toThrow();
    expect(open).not.toHaveBeenCalled();
  });

  it("renders unsafe markdown URLs as inert text and opens safe links through shell open", async () => {
    render(
      <MessageResponse content="[unsafe](javascript:alert(1)) [safe](https://example.com/docs)" />,
    );

    expect(screen.queryByRole("link", { name: "unsafe" })).toBeNull();
    const safe = screen.getByRole("link", { name: "safe" });
    fireEvent.click(safe);
    await waitFor(() =>
      expect(open).toHaveBeenCalledWith("https://example.com/docs"),
    );
  });
});
