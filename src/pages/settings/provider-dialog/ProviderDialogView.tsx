import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { mergeModels } from "./model-utils";
import { FetchModelsDialog } from "./FetchModelsDialog";
import { ProviderAdvancedFields } from "./ProviderAdvancedFields";
import { ProviderBasicFields } from "./ProviderBasicFields";
import { ProviderCredentialFields } from "./ProviderCredentialFields";
import { ProviderEndpointFields } from "./ProviderEndpointFields";
import { ProviderModelSection } from "./ProviderModelSection";
import type { ProviderDialogController } from "./types";

export function ProviderDialogView({ controller }: { controller: ProviderDialogController }) {
  const { t } = useTranslation("settings");
  const form = controller.form;

  return (
    <Dialog open={controller.open} onOpenChange={controller.onOpenChange}>
      <DialogContent className="grid max-h-[85vh] w-[min(60rem,calc(100vw-2rem))] max-w-3xl sm:max-w-[96rem] grid-rows-[auto_minmax(0,1fr)_auto] overflow-hidden p-0">
        <DialogHeader>
          <DialogTitle className="px-6 pt-6">
            {form.isEditing ? t("models.editProvider") : t("models.addProvider")}
          </DialogTitle>
        </DialogHeader>

        <div className="min-h-0 space-y-5 overflow-y-auto px-6 py-2">
          <ProviderBasicFields
            name={form.values.name}
            provider={form.values.provider}
            vendor={form.values.vendor}
            onNameChange={(value) => form.update("name", value)}
            onProviderChange={controller.onUpdateProvider}
            onVendorChange={controller.onUpdateVendor}
          />
          <ProviderCredentialFields
            apiKey={form.values.apiKey}
            isEditing={form.isEditing}
            revealing={controller.revealing}
            showKey={controller.showKey}
            vendor={form.values.vendor}
            onApiKeyChange={(value) => form.update("apiKey", value)}
            onReveal={() => controller.onRevealKey(controller.editingProvider?.id)}
            onShowKeyChange={controller.setShowKey}
          />
          <ProviderEndpointFields
            baseUrl={form.values.baseUrl}
            provider={form.values.provider}
            vendor={form.values.vendor}
            onBaseUrlChange={(value) => {
              form.update("baseUrl", value);
              controller.setBaseUrlDirty(true);
            }}
          />
          <ProviderModelSection
            fetching={controller.fetchLoading}
            models={form.values.models}
            testingModelId={controller.testingModelId}
            onFetchModels={controller.onFetchModels}
            onModelsChange={controller.onSetModels}
            onTestModel={controller.onTestModel}
          />
          <ProviderAdvancedFields
            value={form.values.advanced}
            onChange={(value) => form.update("advanced", value)}
          />
          <DefaultProviderSwitch controller={controller} />
        </div>

        <DialogFooter className="border-t border-[color:var(--border-subtle)] px-6 py-4">
          <Button variant="outline" onClick={() => controller.onOpenChange(false)}>
            {t("common:cancel")}
          </Button>
          <Button onClick={form.submit} disabled={!form.canSubmit || form.submitting}>
            {form.submitting ? t("common:loading") : t("common:save")}
          </Button>
        </DialogFooter>

        <FetchModelsDialog
          open={controller.fetchDialogOpen}
          result={controller.fetchResult}
          selectedModels={form.values.models}
          onApply={(models) => {
            controller.onSetModels(mergeModels(form.values.models, models));
            controller.setFetchDialogOpen(false);
          }}
          onOpenChange={controller.setFetchDialogOpen}
        />
      </DialogContent>
    </Dialog>
  );
}

function DefaultProviderSwitch({
  controller,
}: {
  controller: ProviderDialogController;
}) {
  const { t } = useTranslation("settings");
  return (
    <div className="flex items-center justify-between">
      <Label>{t("models.setDefault")}</Label>
      <Switch
        checked={controller.form.values.isActive}
        onCheckedChange={(value) => controller.form.update("isActive", value)}
      />
    </div>
  );
}
