import type { CreateCustomModel, CustomModel } from "@/lib/ipc";
import type { ModelInfo } from "@/lib/ipc/models";
import { inferManualModelTypes } from "./model-types";

export function customModelToCreate(model: CustomModel): CreateCustomModel {
  return {
    model_id: model.model_id,
    display_name: model.display_name,
    supports_vision: model.supports_vision,
    supports_thinking: model.supports_thinking,
    model_types: model.model_types,
    max_tokens: model.max_tokens,
    context_window: model.context_window,
    enabled: model.enabled,
    sort_order: model.sort_order,
    thinking_off_model_id: model.thinking_off_model_id ?? null,
  };
}

export function modelInfoToCreate(
  model: ModelInfo,
  sortOrder: number,
): CreateCustomModel {
  return {
    model_id: model.model_id,
    display_name: model.display_name,
    supports_vision: model.supports_vision,
    supports_thinking: model.supports_thinking,
    model_types: model.model_types,
    max_tokens: model.max_tokens,
    context_window: model.context_window,
    enabled: true,
    sort_order: sortOrder,
  };
}

export function manualModelToCreate(
  modelId: string,
  sortOrder: number,
): CreateCustomModel {
  return {
    model_id: modelId.trim(),
    display_name: modelId.trim(),
    model_types: inferManualModelTypes(modelId),
    enabled: true,
    sort_order: sortOrder,
  };
}

export function mergeModels(
  current: CreateCustomModel[],
  incoming: CreateCustomModel[],
): CreateCustomModel[] {
  const seen = new Set(current.map((model) => model.model_id));
  const additions = incoming.filter((model) => !seen.has(model.model_id));
  return [...current, ...additions];
}
