import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { RouterConfigView } from "@/lib/ipc";
import {
  createInitialProviderForm,
  toProviderPayload,
  useProviderForm,
  validateProviderForm,
} from "@/pages/settings/provider-dialog/hooks/useProviderForm";

describe("useProviderForm", () => {
  it("requires api key only when creating a provider", () => {
    const values = createInitialProviderForm();

    expect(validateProviderForm(values, false)).toContain("apiKey");
    expect(validateProviderForm(values, true)).not.toContain("apiKey");
  });

  it("keeps existing api key unchanged when editing and apiKey is empty", () => {
    const values = createInitialProviderForm(providerFixture());
    const payload = toProviderPayload(values, true);

    expect(payload).not.toHaveProperty("api_key");
    expect(payload.provider).toBe("openai");
    expect(payload.vendor).toBe("zhipu");
  });

  it("submits normalized provider payload and selected models", async () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => useProviderForm({ onSubmit }));

    act(() => {
      result.current.update("name", "  Zhipu  ");
      result.current.update("apiKey", " sk-test ");
      result.current.update("vendor", "zhipu");
      result.current.update("baseUrl", " https://open.bigmodel.cn/api/paas/v4 ");
      result.current.setModels([
        { model_id: "glm-4.5", display_name: "GLM-4.5", enabled: true },
      ]);
    });

    await act(async () => {
      await result.current.submit();
    });

    expect(onSubmit).toHaveBeenCalledWith({
      config: expect.objectContaining({
        name: "Zhipu",
        api_key: "sk-test",
        vendor: "zhipu",
        base_url: "https://open.bigmodel.cn/api/paas/v4",
      }),
      models: [{ model_id: "glm-4.5", display_name: "GLM-4.5", enabled: true }],
    });
  });

  it("does not replace models on edit until models are explicitly changed", async () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    const editingProvider = providerFixture();
    const { result } = renderHook(() =>
      useProviderForm({ editingProvider, onSubmit }),
    );

    await act(async () => {
      await result.current.submit();
    });

    expect(onSubmit).toHaveBeenCalledWith({
      config: expect.any(Object),
      models: undefined,
    });
  });

  it("can load existing edit models without marking them dirty", async () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    const editingProvider = providerFixture();
    const { result } = renderHook(() =>
      useProviderForm({ editingProvider, onSubmit }),
    );

    act(() => {
      result.current.setModels(
        [{ model_id: "glm-4.5", display_name: "GLM-4.5", enabled: true }],
        false,
      );
    });

    await act(async () => {
      await result.current.submit();
    });

    expect(onSubmit).toHaveBeenCalledWith({
      config: expect.any(Object),
      models: undefined,
    });
  });

  it("resets form values when resetKey changes", () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    const { result, rerender } = renderHook(
      ({ resetKey }) => useProviderForm({ onSubmit, resetKey }),
      { initialProps: { resetKey: 1 } },
    );

    act(() => {
      result.current.update("name", "Draft");
    });

    rerender({ resetKey: 2 });

    expect(result.current.values.name).toBe("");
  });
});

function providerFixture(): RouterConfigView {
  return {
    id: "rc-1",
    name: "Zhipu",
    provider: "openai",
    vendor: "zhipu",
    api_key_masked: "sk-***",
    model: null,
    base_url: "https://open.bigmodel.cn/api/paas/v4",
    api_compat: "openai",
    config_json: null,
    advanced: { temperature: 0.7, max_tokens: null, proxy: null },
    is_active: true,
    created_at: "2026-01-01T00:00:00Z",
  };
}
