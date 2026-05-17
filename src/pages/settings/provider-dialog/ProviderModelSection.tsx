import { Plus, RefreshCw, Trash2, Wifi } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { CreateCustomModel } from "@/lib/ipc";
import { manualModelToCreate } from "./model-utils";

interface ProviderModelSectionProps {
  fetching: boolean;
  models: CreateCustomModel[];
  testingModelId: string | null;
  onFetchModels: () => void;
  onModelsChange: (models: CreateCustomModel[]) => void;
  onTestModel: (modelId: string) => void;
}

export function ProviderModelSection(props: ProviderModelSectionProps) {
  const { t } = useTranslation("settings");
  const [manualModel, setManualModel] = useState("");

  function addManualModel() {
    const modelId = manualModel.trim();
    if (!modelId || props.models.some((model) => model.model_id === modelId)) {
      return;
    }
    props.onModelsChange([
      ...props.models,
      manualModelToCreate(modelId, props.models.length),
    ]);
    setManualModel("");
  }

  function removeModel(modelId: string) {
    props.onModelsChange(
      props.models.filter((model) => model.model_id !== modelId),
    );
  }

  return (
    <section className="space-y-3 rounded-[var(--radius-ui-lg)] border border-[color:var(--border-strong)] bg-[color:var(--surface-card)] p-3">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h3 className="text-sm font-semibold">
            {t("providers.models.available", "Available models")}
          </h3>
          <p className="text-xs text-muted-foreground">
            {t("providers.models.onlySelected", "Only selected models appear in chat.")}
          </p>
        </div>
        <Button
          disabled={props.fetching}
          onClick={props.onFetchModels}
          size="sm"
          variant="outline"
        >
          <RefreshCw className={props.fetching ? "size-3.5 animate-spin" : "size-3.5"} />
          {t("providers.models.fetch", "Fetch models")}
        </Button>
      </div>

      <div className="flex gap-2">
        <Input
          value={manualModel}
          onChange={(event) => setManualModel(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") addManualModel();
          }}
          placeholder={t("models.modelPlaceholder")}
        />
        <Button onClick={addManualModel} variant="outline">
          <Plus className="size-4" />
          {t("common:add", "Add")}
        </Button>
      </div>

      {props.models.length === 0 ? (
        <div className="rounded-[var(--radius-ui-md)] border border-dashed border-[color:var(--border-strong)] px-3 py-6 text-center text-sm text-muted-foreground">
          {t("providers.models.empty", "No models selected yet.")}
        </div>
      ) : (
        <div className="space-y-2">
          {props.models.map((model) => (
            <ModelRow
              key={model.model_id}
              model={model}
              testing={props.testingModelId === model.model_id}
              thinkingLabel={t("providers.models.thinking")}
              removeLabel={t("providers.models.remove", "Remove model")}
              testLabel={t("providers.models.test", "Test model")}
              visionLabel={t("providers.models.vision")}
              onRemove={() => removeModel(model.model_id)}
              onTest={() => props.onTestModel(model.model_id)}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function ModelRow({
  model,
  removeLabel,
  testLabel,
  thinkingLabel,
  testing,
  visionLabel,
  onRemove,
  onTest,
}: {
  model: CreateCustomModel;
  removeLabel: string;
  testLabel: string;
  thinkingLabel: string;
  testing: boolean;
  visionLabel: string;
  onRemove: () => void;
  onTest: () => void;
}) {
  return (
    <div className="flex items-center gap-2 rounded-[var(--radius-ui-md)] border border-[color:var(--border-subtle)] bg-background/35 px-3 py-2">
      <div className="min-w-0 flex-1">
        <div className="truncate text-sm font-medium">{model.display_name}</div>
        <div className="truncate font-mono text-xs text-muted-foreground">
          {model.model_id}
        </div>
      </div>
      {model.supports_vision && <Badge variant="secondary">{visionLabel}</Badge>}
      {model.supports_thinking && <Badge variant="secondary">{thinkingLabel}</Badge>}
      <Button
        aria-label={testLabel}
        disabled={testing}
        onClick={onTest}
        size="icon"
        variant="ghost"
      >
        <Wifi className={testing ? "size-4 animate-pulse" : "size-4"} />
      </Button>
      <Button
        aria-label={removeLabel}
        onClick={onRemove}
        size="icon"
        variant="ghost"
      >
        <Trash2 className="size-4 text-destructive" />
      </Button>
    </div>
  );
}
