import type { CreateCustomModel, CustomModel } from "@/lib/ipc";
import type { ModelInfo } from "@/lib/ipc/models";

export function customModelToCreate(model: CustomModel): CreateCustomModel {
  return {
    model_id: model.model_id,
    display_name: model.display_name,
    supports_vision: model.supports_vision,
    supports_thinking: model.supports_thinking,
    max_tokens: model.max_tokens,
    context_window: model.context_window,
    enabled: model.enabled,
    sort_order: model.sort_order,
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
