import { invoke } from "./invoke";

/** Mirrors `crate::services::llm::registry::ModelInfo` (serde snake_case). */
export interface ModelInfo {
  model_id: string;
  display_name: string;
  supports_vision: boolean;
  supports_thinking: boolean;
  is_custom: boolean;
  max_tokens: number | null;
  context_window: number | null;
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
