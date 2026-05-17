import { useEffect, useMemo, useState } from "react";
import type {
  AdvancedConfig,
  CreateCustomModel,
  CreateRouterConfig,
  RouterConfigView,
  UpdateRouterConfig,
} from "@/lib/ipc";
import type { ProviderApi, VendorId } from "@/lib/providers/catalog";

export interface ProviderFormValues {
  name: string;
  provider: ProviderApi;
  vendor: VendorId;
  apiKey: string;
  baseUrl: string;
  model: string;
  isActive: boolean;
  advanced: AdvancedConfig;
  models: CreateCustomModel[];
  modelsDirty: boolean;
}

export interface ProviderFormSubmitResult {
  config: CreateRouterConfig | UpdateRouterConfig;
  models?: CreateCustomModel[];
}

interface UseProviderFormOptions {
  editingProvider?: RouterConfigView | null;
  resetKey?: unknown;
  onSubmit: (result: ProviderFormSubmitResult) => Promise<void>;
}

const DEFAULT_ADVANCED: AdvancedConfig = {
  temperature: 0.7,
  max_tokens: null,
  proxy: null,
};

export function createInitialProviderForm(
  editingProvider?: RouterConfigView | null,
): ProviderFormValues {
  return {
    name: editingProvider?.name ?? "",
    provider: editingProvider?.provider ?? "openai",
    vendor: editingProvider?.vendor ?? "custom",
    apiKey: "",
    baseUrl: editingProvider?.base_url ?? "",
    model: editingProvider?.model ?? "",
    isActive: editingProvider?.is_active ?? false,
    advanced: editingProvider?.advanced ?? DEFAULT_ADVANCED,
    models: [],
    modelsDirty: false,
  };
}

export function toProviderPayload(
  values: ProviderFormValues,
  isEditing: boolean,
): CreateRouterConfig | UpdateRouterConfig {
  const payload: CreateRouterConfig | UpdateRouterConfig = {
    name: values.name.trim(),
    provider: values.provider,
    vendor: values.vendor,
    model: values.model.trim() || undefined,
    base_url: values.baseUrl.trim() || undefined,
    advanced: values.advanced,
    is_active: values.isActive,
    api_compat: values.provider,
  };

  if (!isEditing || values.apiKey.trim()) {
    payload.api_key = values.apiKey.trim();
  }

  return payload;
}

export function validateProviderForm(values: ProviderFormValues, isEditing: boolean): string[] {
  const errors: string[] = [];
  if (!values.name.trim()) errors.push("name");
  if (!isEditing && !values.apiKey.trim()) errors.push("apiKey");
  return errors;
}

export function useProviderForm({ editingProvider, onSubmit, resetKey }: UseProviderFormOptions) {
  const isEditing = !!editingProvider;
  const [values, setValues] = useState(() => createInitialProviderForm(editingProvider));
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    setValues(createInitialProviderForm(editingProvider));
  }, [editingProvider, resetKey]);

  const errors = useMemo(() => validateProviderForm(values, isEditing), [isEditing, values]);
  const canSubmit = errors.length === 0 && !submitting;

  function update<K extends keyof ProviderFormValues>(key: K, value: ProviderFormValues[K]) {
    setValues((current) => ({ ...current, [key]: value }));
  }

  function setModels(models: CreateCustomModel[], dirty = true) {
    setValues((current) => ({ ...current, models, modelsDirty: dirty }));
  }

  function reset() {
    setValues(createInitialProviderForm(editingProvider));
  }

  async function submit() {
    if (!canSubmit) return;
    setSubmitting(true);
    try {
      await onSubmit({
        config: toProviderPayload(values, isEditing),
        models: !isEditing || values.modelsDirty ? values.models : undefined,
      });
    } finally {
      setSubmitting(false);
    }
  }

  return {
    values,
    setValues,
    update,
    setModels,
    reset,
    submit,
    canSubmit,
    errors,
    submitting,
    isEditing,
  };
}
