import { type ReactNode, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "@/stores/app-store";
import { cn } from "@/lib/utils";
import { ModelEmptyState } from "./ModelEmptyState";
import { ModelGroupList } from "./ModelGroupList";
import { ModelSearchInput } from "./ModelSearchInput";
import { useFilteredModels } from "./useFilteredModels";
import type { FlatModel } from "./types";

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

  useEffect(() => {
    if (open) return;
    setQuery("");
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const handleClickOutside = (event: MouseEvent) => {
      if (!containerRef.current?.contains(event.target as Node)) onOpenChange(false);
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open, onOpenChange]);

  return (
    <div ref={containerRef} className="relative">
      {trigger}
      {open && (
        <div
          className={cn(
            "absolute bottom-full left-0 z-50 mb-2 min-w-[280px] max-w-[360px]",
            "rounded-[10px] border border-[color:var(--border-muted)]",
            "bg-[color:var(--surface-popover)] p-2",
            "shadow-[0_14px_34px_rgba(0,0,0,0.3)]",
            "animate-in fade-in-0 slide-in-from-bottom-1 duration-[120ms]",
          )}
        >
          <ModelSearchInput
            value={query}
            onChange={setQuery}
            placeholder={t("modelSearch")}
            clearLabel={t("clearModelSearch")}
          />
          {groups.length === 0 ? (
            <ModelEmptyState
              hasModels={models.length > 0}
              noModelsText={t("noModels")}
              noResultsText={t("modelSearchEmpty")}
              goToSettingsText={t("goToSettings")}
              onGoToSettings={() => {
                onOpenChange(false);
                navigate({ page: "settings", tab: "models" });
              }}
            />
          ) : (
            <ModelGroupList
              groups={groups}
              selectedModel={selectedModel}
              onSelect={(id) => {
                onSelect(id);
                onOpenChange(false);
              }}
            />
          )}
        </div>
      )}
    </div>
  );
}

export type { FlatModel } from "./types";
