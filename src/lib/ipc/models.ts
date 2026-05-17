import { invoke } from "./invoke";
import type {
  CreateCustomModel,
  CustomModel,
  FetchModelsResult,
  FetchProviderModelsRequest,
  ModelTestResult,
  RouterConfigView,
  TestModelRequest,
} from "./types";

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
  provider: RouterConfigView;
  models: ModelInfo[];
}

export const modelsIpc = {
  listAvailable: (routerConfigId?: string) =>
    invoke<ProviderModels[]>("list_available_models", { routerConfigId }),

  listCustom: (routerConfigId: string) =>
    invoke<CustomModel[]>("list_custom_models", { routerConfigId }),

  replaceCustom: (routerConfigId: string, models: CreateCustomModel[]) =>
    invoke<void>("replace_custom_models", { routerConfigId, models }),

  fetchProviderModels: (request: FetchProviderModelsRequest) =>
    invoke<FetchModelsResult>("fetch_provider_models", { request }),

  testModel: (request: TestModelRequest) =>
    invoke<ModelTestResult>("test_model", { request }),
};
