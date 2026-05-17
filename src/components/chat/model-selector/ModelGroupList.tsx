import { Check } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ModelGroup } from "./types";

interface ModelGroupListProps {
  groups: ModelGroup[];
  selectedModel: string | null;
  onSelect: (id: string) => void;
}

export function ModelGroupList({
  groups,
  selectedModel,
  onSelect,
}: ModelGroupListProps) {
  return (
    <div className="max-h-[360px] overflow-y-auto pr-1">
      {groups.map((group) => (
        <section key={group.vendorId} className="pb-1">
          <div className="sticky top-0 z-10 bg-[color:var(--surface-popover)] px-2 pb-1 pt-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/70">
            {group.vendorLabel}
          </div>
          {group.items.map((model) => (
            <button
              key={model.id}
              type="button"
              onClick={() => onSelect(model.id)}
              className={cn(
                "flex w-full items-center gap-2 rounded-[var(--radius-ui-sm)] px-2 py-2",
                "text-left text-xs text-muted-foreground transition-colors",
                "duration-[var(--ds-dur-fast)] hover:bg-[color:var(--surface-hover)] hover:text-foreground",
                selectedModel === model.id &&
                  "bg-[color:var(--surface-hover)] text-foreground",
              )}
            >
              <span className="min-w-0 flex-1">
                <span className="block truncate font-medium">{model.label}</span>
                <span className="block truncate text-[10px] text-muted-foreground/75">
                  {model.routerName}
                </span>
              </span>
              {selectedModel === model.id && (
                <Check className="size-3 shrink-0 text-primary" />
              )}
            </button>
          ))}
        </section>
      ))}
    </div>
  );
}
