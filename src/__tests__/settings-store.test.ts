import { describe, it, expect, beforeEach, vi } from "vitest";
import type { RouterConfigView } from "@/lib/ipc";
import { useChatStore } from "@/stores/chat-store";
import { useSettingsStore } from "@/stores/settings-store";

vi.mock("@/lib/ipc", () => ({
  settingsIpc: {
    getAppConfig: vi.fn(),
    updateAppConfig: vi.fn(),
  },
  routerConfigsIpc: {
    list: vi.fn(),
    create: vi.fn(),
    createWithModels: vi.fn(),
    update: vi.fn(),
    delete: vi.fn(),
    revealApiKey: vi.fn(),
  },
  modelsIpc: {
    fetchProviderModels: vi.fn(),
    testModel: vi.fn(),
    replaceCustom: vi.fn(),
  },
}));

import { settingsIpc, routerConfigsIpc, modelsIpc } from "@/lib/ipc";

const mockSettingsIpc = vi.mocked(settingsIpc);
const mockRouterConfigsIpc = vi.mocked(routerConfigsIpc);
const mockModelsIpc = vi.mocked(modelsIpc);

const mockConfig = {
  language: "en",
  theme: "system" as const,
  accent_color: "#6366f1",
  reduced_transparency: false,
  ui_font_size: 14,
  default_model: "claude-sonnet-4-20250514",
  log_level: "info",
  sidecar_port: 9527,
  auto_start_sidecar: false,
  use_sidecar: true,
  mcp_bridge_port: 9528,
  close_behavior: "ask" as const,
};

const mockProvider: RouterConfigView = {
  id: "prov-1",
  name: "OpenAI",
  provider: "openai",
  vendor: "openai",
  api_key_masked: "sk-...***",
  model: "gpt-4o",
  base_url: null,
  api_compat: "openai",
  config_json: null,
  advanced: { temperature: 0.7, max_tokens: null, proxy: null },
  is_active: true,
  created_at: "2026-01-01T00:00:00Z",
};

describe("useSettingsStore", () => {
  beforeEach(() => {
    useSettingsStore.setState({
      config: null,
      providers: [],
      modelsVersion: 0,
      loading: false,
      error: null,
    });
    useChatStore.setState({ modelsVersion: 0 });
    vi.clearAllMocks();
  });

  describe("loadConfig", () => {
    it("should load config successfully", async () => {
      mockSettingsIpc.getAppConfig.mockResolvedValue(mockConfig);

      await useSettingsStore.getState().loadConfig();

      const state = useSettingsStore.getState();
      expect(state.config).toEqual(mockConfig);
      expect(state.loading).toBe(false);
      expect(state.error).toBeNull();
    });

    it("should handle config load error", async () => {
      mockSettingsIpc.getAppConfig.mockRejectedValue(new Error("Network error"));

      await useSettingsStore.getState().loadConfig();

      const state = useSettingsStore.getState();
      expect(state.config).toBeNull();
      expect(state.loading).toBe(false);
      expect(state.error).toContain("Network error");
    });
  });

  describe("updateConfig", () => {
    it("should optimistically update config", async () => {
      useSettingsStore.setState({ config: mockConfig });
      mockSettingsIpc.updateAppConfig.mockResolvedValue(undefined);

      await useSettingsStore.getState().updateConfig({ theme: "dark" });

      const state = useSettingsStore.getState();
      expect(state.config?.theme).toBe("dark");
    });

    it("should save the close behavior selection", async () => {
      useSettingsStore.setState({ config: mockConfig });
      mockSettingsIpc.updateAppConfig.mockResolvedValue(undefined);

      await useSettingsStore
        .getState()
        .updateConfig({ close_behavior: "minimize_to_tray" });

      expect(stateConfig().close_behavior).toBe("minimize_to_tray");
      expect(mockSettingsIpc.updateAppConfig).toHaveBeenCalledWith(
        expect.objectContaining({ close_behavior: "minimize_to_tray" })
      );
    });

    it("should rollback on update failure", async () => {
      useSettingsStore.setState({ config: mockConfig });
      mockSettingsIpc.updateAppConfig.mockRejectedValue(new Error("Save failed"));

      await useSettingsStore.getState().updateConfig({ theme: "dark" });

      const state = useSettingsStore.getState();
      expect(state.config?.theme).toBe("system");
      expect(state.error).toContain("Save failed");
    });

    it("should do nothing if config is null", async () => {
      await useSettingsStore.getState().updateConfig({ theme: "dark" });
      expect(mockSettingsIpc.updateAppConfig).not.toHaveBeenCalled();
    });
  });

  describe("loadProviders", () => {
    it("should load providers successfully", async () => {
      mockRouterConfigsIpc.list.mockResolvedValue([mockProvider]);

      await useSettingsStore.getState().loadProviders();

      const state = useSettingsStore.getState();
      expect(state.providers).toEqual([mockProvider]);
      expect(state.loading).toBe(false);
    });
  });

  describe("deleteProvider", () => {
    it("should delete provider and remove from local state", async () => {
      useSettingsStore.setState({ providers: [mockProvider] });
      mockRouterConfigsIpc.delete.mockResolvedValue(undefined);

      await useSettingsStore.getState().deleteProvider("prov-1");

      const state = useSettingsStore.getState();
      expect(state.providers).toEqual([]);
      expect(state.loading).toBe(false);
    });

    it("should bump modelsVersion after deleting provider", async () => {
      useSettingsStore.setState({ providers: [mockProvider], modelsVersion: 3 });
      mockRouterConfigsIpc.delete.mockResolvedValue(undefined);

      await useSettingsStore.getState().deleteProvider("prov-1");

      expect(useSettingsStore.getState().modelsVersion).toBe(4);
    });
  });

  describe("model list versioning", () => {
    it("should bump modelsVersion after updating provider metadata", async () => {
      mockRouterConfigsIpc.update.mockResolvedValue(undefined);
      mockRouterConfigsIpc.list.mockResolvedValue([mockProvider]);

      await useSettingsStore.getState().updateProvider("prov-1", {
        name: "Renamed",
      });

      expect(useSettingsStore.getState().modelsVersion).toBe(1);
    });

    it("should bump modelsVersion after replacing custom models", async () => {
      mockModelsIpc.replaceCustom.mockResolvedValue(undefined);

      await useSettingsStore.getState().replaceModels("prov-1", [
        { model_id: "gpt-4o", display_name: "GPT-4o", enabled: true },
      ]);

      expect(useSettingsStore.getState().modelsVersion).toBe(1);
      expect(useChatStore.getState().modelsVersion).toBe(1);
    });
  });

  describe("clearError", () => {
    it("should clear error state", () => {
      useSettingsStore.setState({ error: "some error" });
      useSettingsStore.getState().clearError();
      expect(useSettingsStore.getState().error).toBeNull();
    });
  });
});

function stateConfig() {
  return useSettingsStore.getState().config!;
}
