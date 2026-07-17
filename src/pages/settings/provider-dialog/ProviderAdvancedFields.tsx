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

const MAX_TOKEN_PRESETS = [200_000, 220_000, 273_000, 300_000, 1_000_000] as const;

function maxTokensMode(value: number | null): string {
  if (value === null || !MAX_TOKEN_PRESETS.includes(value as (typeof MAX_TOKEN_PRESETS)[number])) {
    return "custom";
  }
  return String(value);
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
            <div className="space-y-2">
              <div
                className="flex flex-wrap gap-1.5"
                role="radiogroup"
                aria-label={t("providers.advanced.maxTokens", "Max tokens")}
              >
                {[...MAX_TOKEN_PRESETS.map(String), "custom"].map((mode) => {
                  const checked = maxTokensMode(value.max_tokens) === mode;
                  return (
                    <label
                      key={mode}
                      className={cn(
                        "cursor-pointer rounded-full border px-2.5 py-1 text-xs font-medium transition-colors duration-[var(--ds-dur-fast)]",
                        "focus-within:outline-none focus-within:ring-2 focus-within:ring-ring/45",
                        checked
                          ? "border-primary/55 bg-primary/10 text-primary"
                          : "border-[color:var(--border-subtle)] bg-muted/35 text-muted-foreground hover:bg-[color:var(--surface-hover)] hover:text-foreground",
                      )}
                    >
                      <input
                        className="sr-only"
                        type="radio"
                        name="max-tokens-mode"
                        value={mode}
                        checked={checked}
                        onChange={() =>
                          onChange({
                            ...value,
                            max_tokens: mode === "custom" ? null : Number(mode),
                          })
                        }
                      />
                      {mode === "custom"
                        ? t("providers.advanced.maxTokensCustom", "Custom")
                        : formatTokenPreset(Number(mode))}
                    </label>
                  );
                })}
              </div>
              {maxTokensMode(value.max_tokens) === "custom" ? (
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
              ) : null}
            </div>
          </FormField>
          </div>
        </TooltipProvider>
      </CollapsibleContent>
    </Collapsible>
  );
}

function formatTokenPreset(value: number): string {
  return value === 1_000_000 ? "1M" : `${value / 1_000}K`;
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
