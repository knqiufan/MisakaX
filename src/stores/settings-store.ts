import { create } from "zustand";
import { useChatStore } from "./chat-store";
import type {
  AppConfig,
  CreateCustomModel,
  CreateRouterConfigWithModels,
  FetchModelsResult,
  FetchProviderModelsRequest,
  ModelTestResult,
  RouterConfigView,
  TestModelRequest,
  CreateRouterConfig,
  UpdateRouterConfig,
} from "@/lib/ipc";
import { settingsIpc, routerConfigsIpc, modelsIpc } from "@/lib/ipc";

function bumpChatModels() {
  useChatStore.getState().bumpModels();
}

interface SettingsState {
  config: AppConfig | null;
  providers: RouterConfigView[];
  modelsVersion: number;
  loading: boolean;
  error: string | null;

  loadConfig: () => Promise<void>;
  updateConfig: (updates: Partial<AppConfig>) => Promise<void>;
  loadProviders: () => Promise<void>;
  addProvider: (provider: CreateRouterConfig) => Promise<string>;
  addProviderWithModels: (request: CreateRouterConfigWithModels) => Promise<string>;
  updateProvider: (id: string, updates: UpdateRouterConfig) => Promise<void>;
  deleteProvider: (id: string) => Promise<void>;
  revealApiKey: (id: string) => Promise<string>;
  fetchModels: (request: FetchProviderModelsRequest) => Promise<FetchModelsResult>;
  testModel: (request: TestModelRequest) => Promise<ModelTestResult>;
  replaceModels: (routerConfigId: string, models: CreateCustomModel[]) => Promise<void>;
  bumpModelsVersion: () => void;
  clearError: () => void;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  config: null,
  providers: [],
  modelsVersion: 0,
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
      set((state) => ({
        providers,
        loading: false,
        modelsVersion: state.modelsVersion + 1,
      }));
      bumpChatModels();
      return id;
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  addProviderWithModels: async (request) => {
    set({ loading: true, error: null });
    try {
      const id = await routerConfigsIpc.createWithModels(request);
      const providers = await routerConfigsIpc.list();
      set((state) => ({
        providers,
        loading: false,
        modelsVersion: state.modelsVersion + 1,
      }));
      bumpChatModels();
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
      set((state) => ({
        providers,
        loading: false,
        modelsVersion: state.modelsVersion + 1,
      }));
      bumpChatModels();
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
        modelsVersion: state.modelsVersion + 1,
      }));
      bumpChatModels();
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  revealApiKey: async (id) => {
    try {
      return await routerConfigsIpc.revealApiKey(id);
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  fetchModels: async (request) => {
    try {
      return await modelsIpc.fetchProviderModels(request);
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  testModel: async (request) => {
    try {
      return await modelsIpc.testModel(request);
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  replaceModels: async (routerConfigId, models) => {
    set({ loading: true, error: null });
    try {
      await modelsIpc.replaceCustom(routerConfigId, models);
      set((state) => ({
        loading: false,
        modelsVersion: state.modelsVersion + 1,
      }));
      bumpChatModels();
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  bumpModelsVersion: () => {
    set((state) => ({ modelsVersion: state.modelsVersion + 1 }));
    bumpChatModels();
  },

  clearError: () => set({ error: null }),
}));
