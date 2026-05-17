import type { ProviderApi, VendorId } from "@/lib/providers/catalog";

export const PROVIDER_API_LABELS: Record<ProviderApi, string> = {
  openai: "OpenAI",
  anthropic: "Anthropic",
  google: "Google",
};

export const VENDOR_FALLBACK_LABELS: Record<VendorId, string> = {
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
