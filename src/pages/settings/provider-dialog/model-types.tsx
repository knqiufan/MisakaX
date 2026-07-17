import {
  AudioLines,
  FileText,
  Image,
  Layers3,
  ListFilter,
  ScanText,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { ModelType } from "@/lib/ipc";
import { cn } from "@/lib/utils";

export const MODEL_TYPES = [
  "text",
  "multimodal",
  "speech",
  "embedding",
  "rerank",
  "image",
] as const satisfies readonly ModelType[];

const MODEL_TYPE_ICONS = {
  text: FileText,
  multimodal: Layers3,
  speech: AudioLines,
  embedding: ScanText,
  rerank: ListFilter,
  image: Image,
} as const;

export function inferManualModelTypes(modelId: string): ModelType[] {
  const id = modelId.toLowerCase();
  if (id.includes("embed")) return ["embedding"];
  if (id.includes("rerank")) return ["rerank"];
  if (/(tts|speech|transcribe|whisper|audio)/.test(id)) return ["speech"];
  if (/(image|dall-e|sora)/.test(id)) return ["image"];
  if (/(vision|vl|omni)/.test(id)) return ["multimodal"];
  return ["text"];
}

export function ModelTypeBadges({
  types,
  className,
}: {
  types?: ModelType[];
  className?: string;
}) {
  const { t } = useTranslation("settings");
  const resolvedTypes: ModelType[] = types?.length ? types : ["text"];

  return (
    <div className={cn("flex items-center gap-1", className)}>
      {resolvedTypes.map((type) => {
        const Icon = MODEL_TYPE_ICONS[type];
        return (
          <Tooltip key={type}>
            <TooltipTrigger asChild>
              <span
                aria-label={t(`providers.models.types.${type}`)}
                className="inline-flex size-5 items-center justify-center rounded-full border border-[color:var(--border-subtle)] bg-muted/45 text-muted-foreground"
              >
                <Icon className="size-3" aria-hidden />
              </span>
            </TooltipTrigger>
            <TooltipContent>{t(`providers.models.types.${type}`)}</TooltipContent>
          </Tooltip>
        );
      })}
    </div>
  );
}
