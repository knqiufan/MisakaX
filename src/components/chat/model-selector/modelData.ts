import type { ProviderModels } from "@/lib/ipc";
import type { VendorId } from "@/lib/providers/catalog";
import type { FlatModel } from "./types";

const VENDOR_LABELS: Record<VendorId, string> = {
  openai: "OpenAI",
  anthropic: "Anthropic",
  google: "Google",
  deepseek: "DeepSeek",
  zhipu: "GLM",
  minimax: "MiniMax",
  stepfun: "StepFun",
  moonshot: "Kimi",
  siliconflow: "SiliconFlow",
  custom: "Custom",
};

export function toFlatModels(
  providerModels: ProviderModels[],
  fallbackLabel: string,
): FlatModel[] {
  return providerModels.flatMap((providerModel) =>
    providerModel.models.map((model) => {
      const vendorId = providerModel.provider.vendor ?? "custom";
      return {
        id: `${providerModel.provider.id}:${model.model_id}`,
        label: model.display_name?.trim() || model.model_id?.trim() || fallbackLabel,
        modelId: model.model_id,
        provider: providerModel.provider.provider,
        routerName: providerModel.provider.name?.trim() || providerModel.provider.id,
        vendorId,
        vendorLabel: VENDOR_LABELS[vendorId],
      };
    }),
  );
}

export function selectedModelLabel(
  selectedModel: string | null,
  models: FlatModel[],
  fallbackLabel: string,
): string {
  if (!selectedModel) return fallbackLabel;
  return models.find((model) => model.id === selectedModel)?.label?.trim() || fallbackLabel;
}

export function isSelectedModelValid(
  selectedModel: string | null,
  models: FlatModel[],
): boolean {
  return !selectedModel || models.some((model) => model.id === selectedModel);
}
