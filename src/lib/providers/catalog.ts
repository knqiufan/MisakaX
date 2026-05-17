export const PROVIDER_APIS = ["openai", "anthropic", "google"] as const;

export type ProviderApi = (typeof PROVIDER_APIS)[number];

export type VendorId =
  | "openai"
  | "anthropic"
  | "google"
  | "deepseek"
  | "zhipu"
  | "minimax"
  | "stepfun"
  | "moonshot"
  | "siliconflow"
  | "custom";

export interface VendorEndpoint {
  baseUrl: string;
  modelsPath?: string;
}

export interface VendorMeta {
  id: VendorId;
  labelKey: string;
  homepage?: string;
  apiKeyUrl?: string;
  endpoints: Partial<Record<ProviderApi, VendorEndpoint>>;
}

export const VENDOR_CATALOG: Readonly<Record<VendorId, VendorMeta>> = {
  openai: {
    id: "openai",
    labelKey: "providers.vendor.openai",
    homepage: "https://openai.com",
    apiKeyUrl: "https://platform.openai.com/api-keys",
    endpoints: { openai: { baseUrl: "https://api.openai.com/v1", modelsPath: "/models" } },
  },
  anthropic: {
    id: "anthropic",
    labelKey: "providers.vendor.anthropic",
    homepage: "https://anthropic.com",
    apiKeyUrl: "https://console.anthropic.com/settings/keys",
    endpoints: { anthropic: { baseUrl: "https://api.anthropic.com", modelsPath: "/v1/models" } },
  },
  google: {
    id: "google",
    labelKey: "providers.vendor.google",
    homepage: "https://ai.google.dev",
    apiKeyUrl: "https://aistudio.google.com/apikey",
    endpoints: { google: { baseUrl: "https://generativelanguage.googleapis.com", modelsPath: "/v1beta/models" } },
  },
  deepseek: {
    id: "deepseek",
    labelKey: "providers.vendor.deepseek",
    homepage: "https://deepseek.com",
    apiKeyUrl: "https://platform.deepseek.com/api_keys",
    endpoints: {
      openai: { baseUrl: "https://api.deepseek.com/v1", modelsPath: "/models" },
      anthropic: { baseUrl: "https://api.deepseek.com/anthropic" },
    },
  },
  zhipu: {
    id: "zhipu",
    labelKey: "providers.vendor.zhipu",
    homepage: "https://bigmodel.cn",
    apiKeyUrl: "https://bigmodel.cn/usercenter/proj-mgmt/apikeys",
    endpoints: {
      openai: { baseUrl: "https://open.bigmodel.cn/api/paas/v4", modelsPath: "/models" },
      anthropic: { baseUrl: "https://open.bigmodel.cn/api/anthropic" },
    },
  },
  minimax: {
    id: "minimax",
    labelKey: "providers.vendor.minimax",
    homepage: "https://minimaxi.com",
    apiKeyUrl: "https://platform.minimaxi.com/user-center/basic-information/interface-key",
    endpoints: { openai: { baseUrl: "https://api.minimaxi.com/v1", modelsPath: "/models" } },
  },
  stepfun: {
    id: "stepfun",
    labelKey: "providers.vendor.stepfun",
    homepage: "https://stepfun.com",
    apiKeyUrl: "https://platform.stepfun.com/interface-key",
    endpoints: { openai: { baseUrl: "https://api.stepfun.com/v1", modelsPath: "/models" } },
  },
  moonshot: {
    id: "moonshot",
    labelKey: "providers.vendor.moonshot",
    homepage: "https://moonshot.cn",
    apiKeyUrl: "https://platform.moonshot.cn/console/api-keys",
    endpoints: {
      openai: { baseUrl: "https://api.moonshot.cn/v1", modelsPath: "/models" },
      anthropic: { baseUrl: "https://api.moonshot.cn/anthropic" },
    },
  },
  siliconflow: {
    id: "siliconflow",
    labelKey: "providers.vendor.siliconflow",
    homepage: "https://siliconflow.cn",
    apiKeyUrl: "https://cloud.siliconflow.cn/account/ak",
    endpoints: { openai: { baseUrl: "https://api.siliconflow.cn/v1", modelsPath: "/models" } },
  },
  custom: {
    id: "custom",
    labelKey: "providers.vendor.custom",
    endpoints: {
      openai: { baseUrl: "" },
      anthropic: { baseUrl: "" },
      google: { baseUrl: "" },
    },
  },
};

export function supportedVendorIds(): VendorId[] {
  return Object.keys(VENDOR_CATALOG) as VendorId[];
}

export function getEndpoint(
  api: ProviderApi,
  vendor: VendorId,
): VendorEndpoint | undefined {
  return VENDOR_CATALOG[vendor].endpoints[api];
}
