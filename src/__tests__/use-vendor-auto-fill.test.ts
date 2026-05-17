import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  defaultBaseUrl,
  shouldAutoFillBaseUrl,
  useVendorAutoFill,
} from "@/pages/settings/provider-dialog/hooks/useVendorAutoFill";

describe("useVendorAutoFill", () => {
  it("returns catalog default base URL for supported combinations", () => {
    expect(defaultBaseUrl("openai", "zhipu")).toBe(
      "https://open.bigmodel.cn/api/paas/v4",
    );
  });

  it("does not overwrite dirty non-empty base URL", () => {
    expect(
      shouldAutoFillBaseUrl(
        "https://api.deepseek.com/v1",
        "https://proxy.example.com/v1",
        true,
      ),
    ).toBe(false);
  });

  it("does not clear dirty custom base URL when the next default is empty", () => {
    expect(shouldAutoFillBaseUrl("", "https://proxy.example.com/v1", true)).toBe(
      false,
    );
  });

  it("fills dirty base URL when the current value is empty", () => {
    expect(shouldAutoFillBaseUrl("https://api.deepseek.com/v1", "", true)).toBe(
      true,
    );
  });

  it("runs the hook effect when dirty base URL becomes empty", () => {
    const onBaseUrlChange = vi.fn();
    const { rerender } = renderHook(
      ({ baseUrl }) =>
        useVendorAutoFill({
          provider: "openai",
          vendor: "deepseek",
          baseUrl,
          baseUrlDirty: true,
          onBaseUrlChange,
        }),
      { initialProps: { baseUrl: "https://proxy.example.com/v1" } },
    );

    rerender({ baseUrl: "" });

    expect(onBaseUrlChange).toHaveBeenCalledWith("https://api.deepseek.com/v1");
  });

  it("clears controlled base URL when switching to custom", () => {
    const onBaseUrlChange = vi.fn();

    renderHook(() =>
      useVendorAutoFill({
        provider: "openai",
        vendor: "custom",
        baseUrl: "https://api.deepseek.com/v1",
        baseUrlDirty: false,
        onBaseUrlChange,
      }),
    );

    expect(onBaseUrlChange).toHaveBeenCalledWith("");
  });
});
