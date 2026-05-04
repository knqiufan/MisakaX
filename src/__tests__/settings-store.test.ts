import { describe, it, expect, beforeEach, vi } from "vitest";
import { useSettingsStore } from "@/stores/settings-store";

vi.mock("@/lib/ipc", () => ({
  settingsIpc: {
    getAppConfig: vi.fn(),
    updateAppConfig: vi.fn(),
  },
  routerConfigsIpc: {
    list: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    delete: vi.fn(),
  },
}));

import { settingsIpc, routerConfigsIpc } from "@/lib/ipc";

const mockSettingsIpc = vi.mocked(settingsIpc);
const mockRouterConfigsIpc = vi.mocked(routerConfigsIpc);

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
};

const mockProvider = {
  id: "prov-1",
  name: "OpenAI",
  provider: "openai",
  api_key_masked: "sk-...***",
  model: "gpt-4o",
  base_url: null,
  is_active: true,
  created_at: "2026-01-01T00:00:00Z",
};

describe("useSettingsStore", () => {
  beforeEach(() => {
    useSettingsStore.setState({
      config: null,
      providers: [],
      loading: false,
      error: null,
    });
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
  });

  describe("clearError", () => {
    it("should clear error state", () => {
      useSettingsStore.setState({ error: "some error" });
      useSettingsStore.getState().clearError();
      expect(useSettingsStore.getState().error).toBeNull();
    });
  });
});
