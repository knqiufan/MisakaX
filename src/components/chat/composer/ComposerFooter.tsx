import { useEffect, useMemo, useState } from "react";
import { ChevronDown } from "lucide-react";
import { modelsIpc } from "@/lib/ipc";
import type { ProviderModels } from "@/lib/ipc";
import { useChatStore } from "@/stores/chat-store";
import { cn } from "@/lib/utils";
import { ModelSelector, type FlatModel } from "../model-selector/ModelSelector";
import {
  isSelectedModelValid,
  selectedModelLabel,
  toFlatModels,
} from "../model-selector/modelData";
import { McpStatusPopover } from "./McpStatusPopover";
import { SkillSelectorPopover } from "./SkillSelectorPopover";

interface ComposerFooterProps {
  t: (key: string) => string;
}

export function ComposerFooter({ t }: ComposerFooterProps) {
  const { modelsVersion, selectedModel, setSelectedModel } = useChatStore();
  const [providerModels, setProviderModels] = useState<ProviderModels[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [modelListOpen, setModelListOpen] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setLoaded(false);
    modelsIpc
      .listAvailable()
      .then((models) => {
        if (!cancelled) setProviderModels(models);
      })
      .catch(console.error)
      .finally(() => {
        if (!cancelled) setLoaded(true);
      });
    return () => {
      cancelled = true;
    };
  }, [modelsVersion]);

  const flatModels: FlatModel[] = useMemo(
    () => toFlatModels(providerModels, t("selectModel")),
    [providerModels, t]
  );

  useEffect(() => {
    if (!loaded || !selectedModel) return;
    if (!isSelectedModelValid(selectedModel, flatModels)) {
      setSelectedModel(null);
    }
  }, [flatModels, loaded, selectedModel, setSelectedModel]);

  const selectedLabel = useMemo(
    () => selectedModelLabel(selectedModel, flatModels, t("selectModel")),
    [selectedModel, flatModels, t]
  );

  return (
    <div className="mt-1.5 flex items-center gap-2 px-1">
      <ModelSelector
        models={flatModels}
        selectedModel={selectedModel}
        onSelect={setSelectedModel}
        open={modelListOpen}
        onOpenChange={setModelListOpen}
        trigger={
          <button
            type="button"
            onClick={() => setModelListOpen(!modelListOpen)}
            className={cn(
              "inline-flex items-center gap-1 rounded-full px-2.5 py-1",
              "bg-[color:var(--cm-surface-panel-strong)] text-[11px] text-muted-foreground",
              "transition-colors duration-[var(--ds-dur-fast)]",
              "hover:bg-[color:var(--cm-surface-panel-solid)] hover:text-foreground"
            )}
          >
            <span className="max-w-[120px] truncate">{selectedLabel}</span>
            <ChevronDown className="size-3 opacity-70" />
          </button>
        }
      />
      <McpStatusPopover
        label={t("composer.mcp")}
        emptyLabel={t("composer.mcpEmpty")}
      />
      <SkillSelectorPopover
        label={t("composer.skill")}
        comingSoonLabel={t("composer.skillComingSoon")}
      />
    </div>
  );
}
