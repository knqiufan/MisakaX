import { useEffect, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { useSettingsStore } from "@/stores";
import { routerConfigsIpc } from "@/lib/ipc";
import type {
  CreateRouterConfig,
  RouterConfigView,
  UpdateRouterConfig,
} from "@/lib/ipc";
import { ProviderCard } from "./ProviderCard";
import { ProviderDialog } from "./ProviderDialog";
import type { ProviderFormSubmitResult } from "./provider-dialog/hooks";

export function ModelSettings() {
  const { t } = useTranslation("settings");
  const providers = useSettingsStore((s) => s.providers);
  const loadProviders = useSettingsStore((s) => s.loadProviders);
  const addProviderWithModels = useSettingsStore((s) => s.addProviderWithModels);
  const updateProvider = useSettingsStore((s) => s.updateProvider);
  const deleteProvider = useSettingsStore((s) => s.deleteProvider);
  const replaceModels = useSettingsStore((s) => s.replaceModels);

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingProvider, setEditingProvider] =
    useState<RouterConfigView | null>(null);
  const [testingId, setTestingId] = useState<string | null>(null);

  useEffect(() => {
    loadProviders();
  }, [loadProviders]);

  const handleAdd = useCallback(() => {
    setEditingProvider(null);
    setDialogOpen(true);
  }, []);

  const handleEdit = useCallback((provider: RouterConfigView) => {
    setEditingProvider(provider);
    setDialogOpen(true);
  }, []);

  const handleDelete = useCallback(
    async (id: string) => {
      if (!window.confirm(t("models.deleteConfirm"))) return;
      try {
        await deleteProvider(id);
        toast.success(t("common:success"));
      } catch (err) {
        toast.error(t("common:error"), { description: String(err) });
      }
    },
    [deleteProvider, t]
  );

  const handleTest = useCallback(
    async (id: string) => {
      setTestingId(id);
      try {
        const result = await routerConfigsIpc.testConnection(id);
        if (result.success) {
          toast.success(t("models.connectionSuccess"));
        } else {
          toast.error(t("models.connectionFailed"), {
            description: result.message,
          });
        }
      } catch (err) {
        toast.error(t("models.connectionFailed"), {
          description: String(err),
        });
      } finally {
        setTestingId(null);
      }
    },
    [t]
  );

  const handleSubmit = useCallback(
    async (data: ProviderFormSubmitResult) => {
      let handledModelError = false;
      try {
        if (editingProvider) {
          await updateProvider(editingProvider.id, data.config as UpdateRouterConfig);
          if (data.models) {
            try {
              await replaceModels(editingProvider.id, data.models);
            } catch (err) {
              handledModelError = true;
              await loadProviders();
              toast.error(t("models.modelSaveFailed", "Provider saved, but models failed"), {
                description: String(err),
              });
              throw err;
            }
          }
        } else {
          await addProviderWithModels({
            config: data.config as CreateRouterConfig,
            models: data.models ?? [],
          });
        }
        toast.success(t("common:success"));
      } catch (err) {
        if (!handledModelError) {
          toast.error(t("common:error"), { description: String(err) });
        }
        throw err;
      }
    },
    [editingProvider, updateProvider, replaceModels, addProviderWithModels, loadProviders, t]
  );

  return (
    <div className="w-full min-w-0 space-y-6">
      <div className="flex items-center justify-between">
        <Button size="sm" onClick={handleAdd}>
          <Plus className="mr-1.5 h-4 w-4" />
          {t("models.addProvider")}
        </Button>
      </div>

      {providers.length === 0 ? (
        <ProvidersEmptyState />
      ) : (
        <div className="space-y-3">
          {providers.map((p) => (
            <ProviderCard
              key={p.id}
              config={p}
              onEdit={() => handleEdit(p)}
              onDelete={() => handleDelete(p.id)}
              onTest={() => handleTest(p.id)}
              testing={testingId === p.id}
            />
          ))}
        </div>
      )}

      <ProviderDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        editingProvider={editingProvider}
        onSubmit={handleSubmit}
      />
    </div>
  );
}

function ProvidersEmptyState() {
  const { t } = useTranslation("settings");

  return (
    <div className="flex h-64 flex-col items-center justify-center gap-3 rounded-lg border border-dashed border-border">
      <p className="text-sm font-medium text-muted-foreground">
        {t("models.noProviders")}
      </p>
      <p className="text-xs text-muted-foreground">
        {t("models.noProvidersDesc")}
      </p>
    </div>
  );
}
