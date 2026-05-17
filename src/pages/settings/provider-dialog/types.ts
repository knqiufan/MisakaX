import type { Dispatch, SetStateAction } from "react";
import type { FetchModelsResult, RouterConfigView } from "@/lib/ipc";
import type { ProviderFormSubmitResult, useProviderForm } from "./hooks";

export interface ProviderDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  editingProvider?: RouterConfigView | null;
  onSubmit: (data: ProviderFormSubmitResult) => Promise<void>;
}

export interface ProviderDialogController {
  baseUrlDirty: boolean;
  editingProvider?: RouterConfigView | null;
  fetchDialogOpen: boolean;
  fetchLoading: boolean;
  fetchResult: FetchModelsResult | null;
  form: ReturnType<typeof useProviderForm>;
  open: boolean;
  revealing: boolean;
  setBaseUrlDirty: Dispatch<SetStateAction<boolean>>;
  setFetchDialogOpen: Dispatch<SetStateAction<boolean>>;
  setShowKey: Dispatch<SetStateAction<boolean>>;
  showKey: boolean;
  testingModelId: string | null;
  onFetchModels: () => void;
  onOpenChange: (open: boolean) => void;
  onRevealKey: (routerConfigId?: string) => void;
  onSetModels: (models: ReturnType<typeof useProviderForm>["values"]["models"]) => void;
  onTestModel: (modelId: string) => void;
  onUpdateProvider: (provider: ReturnType<typeof useProviderForm>["values"]["provider"]) => void;
  onUpdateVendor: (vendor: ReturnType<typeof useProviderForm>["values"]["vendor"]) => void;
}
