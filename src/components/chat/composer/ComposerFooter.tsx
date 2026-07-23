import { useEffect, useMemo, useState, type ReactNode } from "react";
import { Brain, ChevronDown, Telescope } from "lucide-react";
import { modelsIpc } from "@/lib/ipc";
import type { ProviderModels } from "@/lib/ipc";
import { useChatStore } from "@/stores/chat-store";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
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
  const {
    modelsVersion,
    selectedModel,
    setSelectedModel,
    thinkingEnabled,
    setThinkingEnabled,
    researchEnabled,
    setResearchEnabled,
  } = useChatStore();
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

  const thinkingLabel = thinkingEnabled
    ? t("composer.thinkingOn")
    : t("composer.thinkingOff");
  const researchLabel = researchEnabled
    ? t("composer.researchOn")
    : t("composer.researchOff");

  return (
    <div className="mt-2 flex items-center gap-2 pl-10">
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
              "bg-[color:var(--surface-card-strong)] text-[11px] text-muted-foreground",
              "transition-colors duration-[var(--ds-dur-fast)]",
              "hover:bg-[color:var(--surface-card-strong)] hover:text-foreground"
            )}
          >
            <span className="max-w-[120px] truncate">{selectedLabel}</span>
            <ChevronDown className="size-3 opacity-70" />
          </button>
        }
      />
      <ComposerToggleButton
        pressed={thinkingEnabled}
        label={thinkingLabel}
        onToggle={() => setThinkingEnabled(!thinkingEnabled)}
      >
        <Brain className="size-3.5" />
      </ComposerToggleButton>
      <ComposerToggleButton
        pressed={researchEnabled}
        label={researchLabel}
        onToggle={() => setResearchEnabled(!researchEnabled)}
      >
        <Telescope className="size-3.5" />
      </ComposerToggleButton>
      <McpStatusPopover
        label={t("composer.mcp")}
        emptyLabel={t("composer.mcpEmpty")}
      />
      <SkillSelectorPopover
        label={t("composer.skill")}
      />
    </div>
  );
}

function ComposerToggleButton({
  pressed,
  label,
  onToggle,
  children,
}: {
  pressed: boolean;
  label: string;
  onToggle: () => void;
  children: ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          type="button"
          variant={pressed ? "secondary" : "ghost"}
          size="icon-sm"
          aria-label={label}
          aria-pressed={pressed}
          onClick={onToggle}
          className={cn(
            "shrink-0 rounded-full",
            pressed &&
              "bg-primary/15 text-primary hover:bg-primary/20 hover:text-primary"
          )}
        >
          {children}
        </Button>
      </TooltipTrigger>
      <TooltipContent side="top" className="text-xs">
        {label}
      </TooltipContent>
    </Tooltip>
  );
}
