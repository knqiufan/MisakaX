import { create } from "zustand";
import type {
  AppConfig,
  RouterConfigView,
  CreateRouterConfig,
  UpdateRouterConfig,
} from "@/lib/ipc";
import { settingsIpc, routerConfigsIpc } from "@/lib/ipc";

interface SettingsState {
  config: AppConfig | null;
  providers: RouterConfigView[];
  loading: boolean;
  error: string | null;

  loadConfig: () => Promise<void>;
  updateConfig: (updates: Partial<AppConfig>) => Promise<void>;
  loadProviders: () => Promise<void>;
  addProvider: (provider: CreateRouterConfig) => Promise<string>;
  updateProvider: (id: string, updates: UpdateRouterConfig) => Promise<void>;
  deleteProvider: (id: string) => Promise<void>;
  clearError: () => void;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  config: null,
  providers: [],
  loading: false,
  error: null,

  loadConfig: async () => {
    set({ loading: true, error: null });
    try {
      const config = await settingsIpc.getAppConfig();
      set({ config, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  updateConfig: async (updates) => {
    const current = get().config;
    if (!current) return;

    const merged: AppConfig = { ...current, ...updates };
    set({ config: merged });

    try {
      await settingsIpc.updateAppConfig(merged);
    } catch (e) {
      set({ config: current, error: String(e) });
    }
  },

  loadProviders: async () => {
    set({ loading: true, error: null });
    try {
      const providers = await routerConfigsIpc.list();
      set({ providers, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  addProvider: async (provider) => {
    set({ loading: true, error: null });
    try {
      const id = await routerConfigsIpc.create(provider);
      const providers = await routerConfigsIpc.list();
      set({ providers, loading: false });
      return id;
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  updateProvider: async (id, updates) => {
    set({ loading: true, error: null });
    try {
      await routerConfigsIpc.update(id, updates);
      const providers = await routerConfigsIpc.list();
      set({ providers, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  deleteProvider: async (id) => {
    set({ loading: true, error: null });
    try {
      await routerConfigsIpc.delete(id);
      set((state) => ({
        providers: state.providers.filter((p) => p.id !== id),
        loading: false,
      }));
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  clearError: () => set({ error: null }),
}));
