import { type ReactNode, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Check } from "lucide-react";
import { cn } from "@/lib/utils";

export interface FlatModel {
  id: string;
  label: string;
  provider: string;
}

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
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        onOpenChange(false);
      }
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
            "absolute bottom-full left-0 z-50 mb-2",
            "min-w-[200px] max-w-[280px]",
            "rounded-[10px] border border-[color:var(--border-muted)]",
            "bg-[color:var(--surface-popover)]",
            "p-1.5 shadow-[0_14px_34px_rgba(0,0,0,0.3)]",
            "animate-in fade-in-0 slide-in-from-bottom-1 duration-[120ms]"
          )}
        >
          {models.length === 0 ? (
            <div className="px-3 py-4 text-center">
              <p className="text-xs text-muted-foreground">{t("noModels")}</p>
              <p className="mt-1 text-[10px] text-muted-foreground/60">
                {t("noModelsHint")}
              </p>
            </div>
          ) : (
            <ModelList
              models={models}
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

function ModelList({
  models,
  selectedModel,
  onSelect,
}: {
  models: FlatModel[];
  selectedModel: string | null;
  onSelect: (id: string) => void;
}) {
  const grouped = groupByProvider(models);

  return (
    <div className="max-h-[240px] overflow-y-auto">
      {grouped.map(({ provider, items }) => (
        <div key={provider}>
          <p className="px-2 pb-1 pt-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/60">
            {provider}
          </p>
          {items.map((model) => (
            <button
              key={model.id}
              type="button"
              onClick={() => onSelect(model.id)}
              className={cn(
                "flex w-full items-center gap-2 rounded-[8px] px-2 py-1.5",
                "text-left text-xs text-muted-foreground",
                "transition-colors duration-[var(--ds-dur-fast)]",
                "hover:bg-[color:var(--surface-hover)] hover:text-foreground",
                selectedModel === model.id &&
                  "bg-[color:var(--surface-hover)] text-foreground"
              )}
            >
              <span className="min-w-0 flex-1 truncate">{model.label}</span>
              {selectedModel === model.id && (
                <Check className="size-3 shrink-0 text-primary" />
              )}
            </button>
          ))}
        </div>
      ))}
    </div>
  );
}

function groupByProvider(
  models: FlatModel[]
): { provider: string; items: FlatModel[] }[] {
  const map = new Map<string, FlatModel[]>();
  for (const m of models) {
    const list = map.get(m.provider) ?? [];
    list.push(m);
    map.set(m.provider, list);
  }
  return Array.from(map.entries()).map(([provider, items]) => ({
    provider,
    items,
  }));
}
