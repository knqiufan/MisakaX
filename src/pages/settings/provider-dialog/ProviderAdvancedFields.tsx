import { ChevronDown } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { AdvancedConfig } from "@/lib/ipc";
import { cn } from "@/lib/utils";
import { FormField } from "./FormField";

interface ProviderAdvancedFieldsProps {
  value: AdvancedConfig;
  onChange: (value: AdvancedConfig) => void;
}

export function ProviderAdvancedFields({
  value,
  onChange,
}: ProviderAdvancedFieldsProps) {
  const { t } = useTranslation("settings");
  const [open, setOpen] = useState(false);

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <CollapsibleTrigger className="flex w-full items-center justify-between rounded-[var(--radius-ui-md)] border border-[color:var(--border-strong)] bg-[color:var(--surface-card)] px-3 py-2 text-sm font-medium transition-colors hover:bg-[color:var(--surface-hover)]">
        <span>{t("providers.advanced.title", "Advanced configuration")}</span>
        <ChevronDown className={cn("size-4 transition-transform", open && "rotate-180")} />
      </CollapsibleTrigger>
      <CollapsibleContent className="pt-3">
        <TooltipProvider>
          <div className="grid gap-4 rounded-[var(--radius-ui-lg)] border border-[color:var(--border-subtle)] bg-background/35 p-3 md:grid-cols-2">
          <FormField
            label={t("providers.advanced.temperature", "Temperature")}
            hint={
              <FieldHint text={t("providers.advanced.temperatureHint", "Creativity")} />
            }
          >
            <div className="flex items-center gap-3">
              <Slider
                max={2}
                min={0}
                step={0.1}
                value={[value.temperature]}
                onValueChange={([temperature]) =>
                  onChange({ ...value, temperature: temperature ?? 0.7 })
                }
              />
              <Input
                className="w-20"
                max={2}
                min={0}
                step={0.1}
                type="number"
                value={value.temperature}
                onChange={(event) =>
                  onChange({ ...value, temperature: Number(event.target.value) })
                }
              />
            </div>
          </FormField>

          <FormField
            label={t("providers.advanced.maxTokens", "Max tokens")}
            hint={
              <FieldHint text={t("providers.advanced.maxTokensHint", "Optional")} />
            }
          >
            <Input
              min={1}
              type="number"
              value={value.max_tokens ?? ""}
              onChange={(event) =>
                onChange({
                  ...value,
                  max_tokens: event.target.value ? Number(event.target.value) : null,
                })
              }
              placeholder="4096"
            />
          </FormField>
          </div>
        </TooltipProvider>
      </CollapsibleContent>
    </Collapsible>
  );
}

function FieldHint({ text }: { text: string }) {
  return (
    <Tooltip>
      <TooltipTrigger className="rounded-full border border-[color:var(--border-strong)] px-1.5 text-[10px]">
        ?
      </TooltipTrigger>
      <TooltipContent>{text}</TooltipContent>
    </Tooltip>
  );
}
