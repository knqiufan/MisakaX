import { Plus, RefreshCw, Trash2, Wifi } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { CreateCustomModel, ModelType } from "@/lib/ipc";
import { manualModelToCreate } from "./model-utils";
import { MODEL_TYPES, ModelTypeBadges } from "./model-types";

const CLEAR_THINKING_OFF = "__none__";

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

  function setThinkingOffModel(modelId: string, thinkingOffId: string | null) {
    props.onModelsChange(
      props.models.map((model) =>
        model.model_id === modelId
          ? { ...model, thinking_off_model_id: thinkingOffId }
          : model,
      ),
    );
  }

  function setModelType(modelId: string, modelType: ModelType) {
    props.onModelsChange(
      props.models.map((model) =>
        model.model_id === modelId
          ? {
              ...model,
              model_types: [modelType],
              supports_vision: modelType === "multimodal",
            }
          : model,
      ),
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
              allModels={props.models}
              testing={props.testingModelId === model.model_id}
              thinkingLabel={t("providers.models.thinking")}
              thinkingOffLabel={t("providers.models.thinkingOffModel")}
              thinkingOffNoneLabel={t("providers.models.thinkingOffNone")}
              thinkingOffEmptyHint={t("providers.models.thinkingOffEmpty")}
              removeLabel={t("providers.models.remove", "Remove model")}
              testLabel={t("providers.models.test", "Test model")}
              visionLabel={t("providers.models.vision")}
              modelTypeLabel={t("providers.models.type", "Model type")}
              modelTypes={model.model_types}
              onRemove={() => removeModel(model.model_id)}
              onTest={() => props.onTestModel(model.model_id)}
              onThinkingOffChange={(id) =>
                setThinkingOffModel(model.model_id, id)
              }
              onModelTypeChange={(modelType) => setModelType(model.model_id, modelType)}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function nonThinkingTargets(
  models: CreateCustomModel[],
  excludeModelId: string,
): CreateCustomModel[] {
  return models.filter(
    (m) =>
      m.model_id !== excludeModelId &&
      !m.supports_thinking &&
      m.enabled !== false,
  );
}

function ModelRow({
  model,
  allModels,
  removeLabel,
  testLabel,
  thinkingLabel,
  thinkingOffLabel,
  thinkingOffNoneLabel,
  thinkingOffEmptyHint,
  testing,
  visionLabel,
  modelTypeLabel,
  modelTypes,
  onRemove,
  onTest,
  onThinkingOffChange,
  onModelTypeChange,
}: {
  model: CreateCustomModel;
  allModels: CreateCustomModel[];
  removeLabel: string;
  testLabel: string;
  thinkingLabel: string;
  thinkingOffLabel: string;
  thinkingOffNoneLabel: string;
  thinkingOffEmptyHint: string;
  testing: boolean;
  visionLabel: string;
  modelTypeLabel: string;
  modelTypes?: ModelType[];
  onRemove: () => void;
  onTest: () => void;
  onThinkingOffChange: (id: string | null) => void;
  onModelTypeChange: (type: ModelType) => void;
}) {
  const { t } = useTranslation("settings");
  const targets = nonThinkingTargets(allModels, model.model_id);
  const selectedOff = model.thinking_off_model_id ?? CLEAR_THINKING_OFF;

  return (
    <div className="space-y-2 rounded-[var(--radius-ui-md)] border border-[color:var(--border-subtle)] bg-background/35 px-3 py-2">
      <div className="flex items-center gap-2">
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-medium">{model.display_name}</div>
          <div className="truncate font-mono text-xs text-muted-foreground">
            {model.model_id}
          </div>
        </div>
        <ModelTypeBadges types={modelTypes} />
        {model.supports_vision && <Badge variant="secondary">{visionLabel}</Badge>}
        {model.supports_thinking && (
          <Badge variant="secondary">{thinkingLabel}</Badge>
        )}
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

      {model.supports_thinking ? (
        <div className="space-y-1 pl-0.5">
          <p className="text-[11px] text-muted-foreground">{thinkingOffLabel}</p>
          {targets.length === 0 ? (
            <p className="text-[11px] text-muted-foreground/80">
              {thinkingOffEmptyHint}
            </p>
          ) : (
            <Select
              value={selectedOff}
              onValueChange={(value) => {
                onThinkingOffChange(
                  value === CLEAR_THINKING_OFF ? null : value,
                );
              }}
            >
              <SelectTrigger size="sm" className="w-full max-w-full">
                <SelectValue placeholder={thinkingOffNoneLabel} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={CLEAR_THINKING_OFF}>
                  {thinkingOffNoneLabel}
                </SelectItem>
                {targets.map((target) => (
                  <SelectItem key={target.model_id} value={target.model_id}>
                    {target.display_name || target.model_id}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
        </div>
      ) : null}
      <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
        <span>{modelTypeLabel}</span>
        <Select
          value={modelTypes?.[0] ?? "text"}
          onValueChange={(value) =>
            onModelTypeChange(value as ModelType)
          }
        >
          <SelectTrigger className="h-7 min-w-32 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {MODEL_TYPES.map((type) => (
              <SelectItem key={type} value={type}>
                {t(`providers.models.types.${type}`)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}
