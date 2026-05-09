import { invoke } from "./invoke";

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  context_length: number | null;
  is_custom: boolean;
}

export interface ProviderModels {
  provider: {
    id: string;
    name: string;
    provider: string;
    api_key_masked: string;
    model: string | null;
    base_url: string | null;
    is_active: boolean;
    created_at: string;
  };
  models: ModelInfo[];
}

export const modelsIpc = {
  listAvailable: (routerConfigId?: string) =>
    invoke<ProviderModels[]>("list_available_models", { routerConfigId }),
};
