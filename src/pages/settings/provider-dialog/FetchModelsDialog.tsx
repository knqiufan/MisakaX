import { Search } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { CreateCustomModel, FetchModelsResult } from "@/lib/ipc";
import { modelInfoToCreate } from "./model-utils";
import { ModelTypeBadges } from "./model-types";

interface FetchModelsDialogProps {
  open: boolean;
  result: FetchModelsResult | null;
  selectedModels: CreateCustomModel[];
  onApply: (models: CreateCustomModel[]) => void;
  onOpenChange: (open: boolean) => void;
}

export function FetchModelsDialog(props: FetchModelsDialogProps) {
  const { t } = useTranslation("settings");
  const [query, setQuery] = useState("");
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const models = props.result?.models ?? [];
  const filteredModels = useMemo(
    () => models.filter((model) => matchesQuery(model.model_id, query)),
    [models, query],
  );

  useEffect(() => {
    if (props.open) {
      setSelectedIds(new Set());
      setQuery("");
    }
  }, [props.open, props.result]);

  function applySelection() {
    const existingCount = props.selectedModels.length;
    const selectedModels = models
      .filter((model) => selectedIds.has(model.model_id))
      .map((model, index) => modelInfoToCreate(model, existingCount + index));
    props.onApply(selectedModels);
  }

  return (
    <Dialog open={props.open} onOpenChange={props.onOpenChange}>
      <DialogContent className="max-h-[80vh] max-w-2xl">
        <DialogHeader>
          <DialogTitle>
            {t("providers.models.fetchDialogTitle", "Select models")}
          </DialogTitle>
        </DialogHeader>

        {props.result?.warning && (
          <div className="rounded-[var(--radius-ui-md)] border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs text-amber-200">
            {props.result.warning}
          </div>
        )}

        <div className="relative">
          <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            className="pl-9"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t("providers.models.search", "Search models")}
          />
        </div>

        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span>
            {t("providers.models.fetchCount", "{{count}} models", {
              count: filteredModels.length,
            })}
          </span>
          <Button
            onClick={() =>
              setSelectedIds(new Set(filteredModels.map((model) => model.model_id)))
            }
            size="sm"
            variant="outline"
          >
            {t("providers.models.selectAll", "Select all")}
          </Button>
        </div>

        <ScrollArea className="h-[360px] rounded-[var(--radius-ui-md)] border border-[color:var(--border-strong)]">
          <div className="space-y-1 p-2">
            {filteredModels.map((model) => (
              <label
                key={model.model_id}
                className="flex cursor-pointer items-center gap-3 rounded-[var(--radius-ui-sm)] px-3 py-2 hover:bg-[color:var(--surface-hover)]"
              >
                <input
                  checked={selectedIds.has(model.model_id)}
                  onChange={() => {
                    setSelectedIds((current) => toggleSet(current, model.model_id));
                  }}
                  type="checkbox"
                />
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-sm font-medium">
                    {model.display_name}
                  </span>
                  <span className="block truncate font-mono text-xs text-muted-foreground">
                    {model.model_id}
                  </span>
                </span>
                <ModelTypeBadges types={model.model_types} />
              </label>
            ))}
          </div>
        </ScrollArea>

        <DialogFooter>
          <Button variant="outline" onClick={() => props.onOpenChange(false)}>
            {t("common:cancel")}
          </Button>
          <Button disabled={selectedIds.size === 0} onClick={applySelection}>
            {t("common:add", "Add")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function matchesQuery(value: string, query: string): boolean {
  return value.toLowerCase().includes(query.trim().toLowerCase());
}

function toggleSet(current: Set<string>, value: string): Set<string> {
  const next = new Set(current);
  if (next.has(value)) {
    next.delete(value);
  } else {
    next.add(value);
  }
  return next;
}
