import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { modelsIpc, type FetchModelsResult, type RouterConfigView } from "@/lib/ipc";
import { getEndpoint, VENDOR_CATALOG, type ProviderApi, type VendorId } from "@/lib/providers/catalog";
import { VENDOR_FALLBACK_LABELS } from "./catalog-ui";
import { useSettingsStore } from "@/stores/settings-store";
import { customModelToCreate } from "./model-utils";
import {
  useFetchModels,
  useModelTest,
  useProviderForm,
  useVendorAutoFill,
} from "./hooks";
import type { ProviderDialogProps } from "./types";

export function useProviderDialogController({
  open,
  onOpenChange,
  editingProvider,
  onSubmit,
}: ProviderDialogProps) {
  const { t } = useTranslation("settings");
  const [showKey, setShowKey] = useState(false);
  const [revealing, setRevealing] = useState(false);
  const [baseUrlDirty, setBaseUrlDirty] = useState(false);
  const [resetKey, setResetKey] = useState(0);
  const [fetchDialogOpen, setFetchDialogOpen] = useState(false);
  const [fetchResult, setFetchResult] = useState<FetchModelsResult | null>(null);
  const [testingModelId, setTestingModelId] = useState<string | null>(null);
  const revealApiKey = useSettingsStore((state) => state.revealApiKey);
  const fetch = useFetchModels();
  const tester = useModelTest();
  const modelsDirtyRef = useRef(false);
  const form = useProviderForm({
    editingProvider,
    resetKey,
    onSubmit: async (result) => {
      await onSubmit(result);
      onOpenChange(false);
    },
  });

  useEffect(() => {
    if (!open) return;
    setResetKey((current) => current + 1);
    setShowKey(false);
    setFetchResult(null);
    setBaseUrlDirty(hasCustomBaseUrl(editingProvider));
  }, [open, editingProvider]);

  useEffect(() => {
    modelsDirtyRef.current = form.values.modelsDirty;
  }, [form.values.modelsDirty]);

  useEffect(() => {
    if (!open || !editingProvider) return;
    let cancelled = false;
    modelsIpc
      .listCustom(editingProvider.id)
      .then((models) => {
        if (!cancelled && !modelsDirtyRef.current) {
          form.setModels(models.map(customModelToCreate), false);
        }
      })
      .catch((error) => toast.error(t("common:error"), { description: String(error) }));
    return () => {
      cancelled = true;
    };
  }, [open, editingProvider?.id]);

  useVendorAutoFill({
    provider: form.values.provider,
    vendor: form.values.vendor,
    baseUrl: form.values.baseUrl,
    baseUrlDirty,
    onBaseUrlChange: (value) => form.update("baseUrl", value),
  });

  function onUpdateProvider(provider: ProviderApi) {
    form.update("provider", provider);
    if (!getEndpoint(provider, form.values.vendor)) form.update("vendor", "custom");
    setBaseUrlDirty(false);
  }

  function onUpdateVendor(vendor: VendorId) {
    form.update("vendor", vendor);
    form.update("name", t(VENDOR_CATALOG[vendor].labelKey, VENDOR_FALLBACK_LABELS[vendor]));
    setBaseUrlDirty(false);
  }

  function onSetModels(models: typeof form.values.models) {
    form.setModels(models);
    form.update("model", models[0]?.model_id ?? "");
  }

  return {
    baseUrlDirty,
    editingProvider,
    fetchDialogOpen,
    fetchLoading: fetch.loading,
    fetchResult,
    form,
    open,
    revealing,
    setBaseUrlDirty,
    setFetchDialogOpen,
    setShowKey,
    showKey,
    testingModelId,
    onFetchModels: () => fetchModels(),
    onOpenChange,
    onRevealKey: (id?: string) => revealKey(id),
    onSetModels,
    onTestModel: (modelId: string) => testModel(modelId),
    onUpdateProvider,
    onUpdateVendor,
  };

  async function revealKey(routerConfigId?: string) {
    if (!routerConfigId) return;
    setRevealing(true);
    try {
      form.update("apiKey", await revealApiKey(routerConfigId));
      setShowKey(true);
    } catch (error) {
      toast.error(t("providers.apiKey.revealFailed", "Failed to reveal API key"), {
        description: String(error),
      });
    } finally {
      setRevealing(false);
    }
  }

  async function apiKeyForNetwork() {
    if (form.values.apiKey) return form.values.apiKey;
    if (editingProvider) return revealApiKey(editingProvider.id);
    return "";
  }

  async function fetchModels() {
    try {
      const apiKey = await apiKeyForNetwork();
      if (!apiKey) return toast.error(t("models.apiKeyPlaceholder"));
      const result = await fetch.fetchModels({
        api: form.values.provider,
        vendor: form.values.vendor,
        base_url: form.values.baseUrl || undefined,
        api_key: apiKey,
      });
      setFetchResult(result);
      setFetchDialogOpen(true);
    } catch (error) {
      toast.error(t("providers.models.fetchFailed", "Failed to fetch models"), {
        description: String(error),
      });
    }
  }

  async function testModel(modelId: string) {
    setTestingModelId(modelId);
    try {
      const apiKey = await apiKeyForNetwork();
      if (!apiKey) return toast.error(t("models.apiKeyPlaceholder"));
      const result = await tester.testModel({
        api: form.values.provider,
        vendor: form.values.vendor,
        base_url: form.values.baseUrl || undefined,
        api_key: apiKey,
        model_id: modelId,
        advanced: form.values.advanced,
      });
      toast[result.success ? "success" : "error"](result.message);
    } catch (error) {
      toast.error(t("providers.models.testFailed", "Model test failed"), {
        description: String(error),
      });
    } finally {
      setTestingModelId(null);
    }
  }
}

function hasCustomBaseUrl(config?: RouterConfigView | null): boolean {
  if (!config?.base_url) return false;
  const vendor = config.vendor ?? "custom";
  return config.base_url !== getEndpoint(config.provider, vendor)?.baseUrl;
}
