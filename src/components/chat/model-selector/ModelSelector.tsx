import { type ReactNode, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "@/stores/app-store";
import { usePresence } from "@/hooks/usePresence";
import { cn } from "@/lib/utils";
import { ModelEmptyState } from "./ModelEmptyState";
import { ModelGroupList } from "./ModelGroupList";
import { ModelSearchInput } from "./ModelSearchInput";
import type { FlatModel, ModelGroup } from "./types";
import { useFilteredModels } from "./useFilteredModels";

interface ModelSelectorProps {
  models: FlatModel[];
  selectedModel: string | null;
  onSelect: (modelId: string | null) => void;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  trigger: ReactNode;
}

export function ModelSelector({
  models,
  selectedModel,
  onSelect,
  open,
  onOpenChange,
  trigger,
}: ModelSelectorProps) {
  const { t } = useTranslation("chat");
  const navigate = useAppStore((state) => state.navigate);
  const [query, setQuery] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);
  const groups = useFilteredModels(models, query);
  const { mounted, exiting, onExitComplete } = usePresence(open);

  useEffect(() => {
    if (open) return;
    setQuery("");
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const handleClickOutside = (event: MouseEvent) => {
      if (!containerRef.current?.contains(event.target as Node)) {
        onOpenChange(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open, onOpenChange]);

  return (
    <div ref={containerRef} className="relative">
      {trigger}
      {mounted ? (
        <ModelSelectorPanel
          exiting={exiting}
          onExitComplete={onExitComplete}
          query={query}
          onQueryChange={setQuery}
          searchPlaceholder={t("modelSearch")}
          clearLabel={t("clearModelSearch")}
          groups={groups}
          hasModels={models.length > 0}
          selectedModel={selectedModel}
          noModelsText={t("noModels")}
          noResultsText={t("modelSearchEmpty")}
          goToSettingsText={t("goToSettings")}
          onGoToSettings={() => {
            onOpenChange(false);
            navigate({ page: "settings", tab: "models" });
          }}
          onSelect={(id) => {
            onSelect(id);
            onOpenChange(false);
          }}
        />
      ) : null}
    </div>
  );
}

interface ModelSelectorPanelProps {
  exiting: boolean;
  onExitComplete: () => void;
  query: string;
  onQueryChange: (value: string) => void;
  searchPlaceholder: string;
  clearLabel: string;
  groups: ModelGroup[];
  hasModels: boolean;
  selectedModel: string | null;
  noModelsText: string;
  noResultsText: string;
  goToSettingsText: string;
  onGoToSettings: () => void;
  onSelect: (id: string) => void;
}

function ModelSelectorPanel({
  exiting,
  onExitComplete,
  query,
  onQueryChange,
  searchPlaceholder,
  clearLabel,
  groups,
  hasModels,
  selectedModel,
  noModelsText,
  noResultsText,
  goToSettingsText,
  onGoToSettings,
  onSelect,
}: ModelSelectorPanelProps) {
  return (
    <div
      data-state={exiting ? "closed" : "open"}
      onAnimationEnd={(event) => {
        if (event.target !== event.currentTarget) return;
        if (exiting) onExitComplete();
      }}
      className={cn(
        "absolute bottom-full left-0 z-50 mb-2 min-w-[280px] max-w-[360px]",
        "rounded-[10px] border border-[color:var(--border-muted)]",
        "bg-[color:var(--surface-popover)] p-2",
        "shadow-[0_14px_34px_rgba(0,0,0,0.3)]",
        "duration-150 ease-out",
        "data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:slide-in-from-bottom-1",
        "data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:slide-out-to-bottom-1",
      )}
    >
      <ModelSearchInput
        value={query}
        onChange={onQueryChange}
        placeholder={searchPlaceholder}
        clearLabel={clearLabel}
      />
      {groups.length === 0 ? (
        <ModelEmptyState
          hasModels={hasModels}
          noModelsText={noModelsText}
          noResultsText={noResultsText}
          goToSettingsText={goToSettingsText}
          onGoToSettings={onGoToSettings}
        />
      ) : (
        <ModelGroupList
          groups={groups}
          selectedModel={selectedModel}
          onSelect={onSelect}
        />
      )}
    </div>
  );
}

export type { FlatModel } from "./types";
