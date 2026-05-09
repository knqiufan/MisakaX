import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent,
} from "react";
import { useTranslation } from "react-i18next";
import { Send, Square, ChevronDown } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { modelsIpc } from "@/lib/ipc";
import type { ProviderModels } from "@/lib/ipc";
import { useChatStore } from "@/stores/chat-store";
import { ModelSelector } from "./ModelSelector";

const MIN_HEIGHT = 40;
const MAX_HEIGHT = 200;

interface MessageInputProps {
  onSend: (content: string, modelOverride?: string) => void;
  onStop: () => void;
  disabled?: boolean;
}

export function MessageInput({ onSend, onStop, disabled }: MessageInputProps) {
  const { t } = useTranslation("chat");
  const { isStreaming, selectedModel } = useChatStore();
  const [content, setContent] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const canSend = content.trim().length > 0 && !disabled && !isStreaming;

  const adjustHeight = useCallback(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = "auto";
    const scrollH = el.scrollHeight;
    el.style.height = `${Math.min(Math.max(scrollH, MIN_HEIGHT), MAX_HEIGHT)}px`;
  }, []);

  useEffect(() => {
    adjustHeight();
  }, [content, adjustHeight]);

  const handleSend = useCallback(() => {
    const trimmed = content.trim();
    if (!trimmed || isStreaming) return;
    onSend(trimmed, selectedModel ?? undefined);
    setContent("");
    requestAnimationFrame(() => {
      if (textareaRef.current) {
        textareaRef.current.style.height = `${MIN_HEIGHT}px`;
      }
    });
  }, [content, isStreaming, onSend, selectedModel]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend]
  );

  const handleStopClick = useCallback(() => {
    onStop();
  }, [onStop]);

  return (
    <div className="shrink-0 border-t border-[color:var(--border-muted)] bg-[color:var(--surface-topbar)] px-4 py-3">
      <div
        className={cn(
          "flex items-end gap-2 rounded-[var(--radius-ui-lg)]",
          "border border-[color:var(--border-muted)] bg-[color:var(--surface-card)]",
          "px-3 py-2 transition-colors duration-[var(--ds-dur-fast)]",
          "focus-within:border-[color:var(--border-strong)]"
        )}
      >
        <textarea
          ref={textareaRef}
          value={content}
          onChange={(e) => setContent(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={t("inputPlaceholder")}
          disabled={disabled}
          rows={1}
          className={cn(
            "min-h-[40px] max-h-[200px] flex-1 resize-none bg-transparent",
            "text-sm leading-relaxed text-foreground outline-none",
            "placeholder:text-muted-foreground/55",
            "disabled:cursor-not-allowed disabled:opacity-50"
          )}
          style={{ height: `${MIN_HEIGHT}px` }}
        />

        <InputToolbar
          isStreaming={isStreaming}
          canSend={canSend}
          onSend={handleSend}
          onStop={handleStopClick}
          t={t}
        />
      </div>

      <InputFooter t={t} />
    </div>
  );
}

function InputToolbar({
  isStreaming,
  canSend,
  onSend,
  onStop,
  t,
}: {
  isStreaming: boolean;
  canSend: boolean;
  onSend: () => void;
  onStop: () => void;
  t: (key: string) => string;
}) {
  if (isStreaming) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            onClick={onStop}
            aria-label={t("stop")}
            className={cn(
              "size-8 shrink-0 rounded-full",
              "border border-[color:rgba(255,107,107,0.6)]",
              "bg-[color:rgba(255,107,107,0.12)]",
              "text-[#ff6b6b]",
              "hover:bg-[color:rgba(255,107,107,0.2)] hover:text-[#ff6b6b]"
            )}
          >
            <Square className="size-3 fill-current" />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="top" className="text-xs">
          {t("stop")}
        </TooltipContent>
      </Tooltip>
    );
  }

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          onClick={onSend}
          disabled={!canSend}
          aria-label={t("send")}
          className={cn(
            "size-8 shrink-0 rounded-full",
            "border border-[color:var(--cm-border-emphasis)]",
            "bg-[color:var(--cm-surface-panel-strong)]",
            "text-foreground",
            "hover:bg-[color:var(--cm-surface-panel-solid)]",
            "disabled:opacity-50 disabled:cursor-not-allowed"
          )}
        >
          <Send className="size-3.5" />
        </Button>
      </TooltipTrigger>
      <TooltipContent side="top" className="text-xs">
        {t("send")}
      </TooltipContent>
    </Tooltip>
  );
}

function InputFooter({ t }: { t: (key: string) => string }) {
  const { selectedModel, setSelectedModel } = useChatStore();
  const [providerModels, setProviderModels] = useState<ProviderModels[]>([]);
  const [modelListOpen, setModelListOpen] = useState(false);

  useEffect(() => {
    modelsIpc.listAvailable().then(setProviderModels).catch(console.error);
  }, []);

  const flatModels = useMemo(() => {
    return providerModels.flatMap((pm) =>
      pm.models.map((m) => ({
        id: `${pm.provider.id}:${m.id}`,
        label: m.name,
        provider: pm.provider.name,
      }))
    );
  }, [providerModels]);

  const selectedLabel = useMemo(() => {
    if (!selectedModel) return t("selectModel");
    const found = flatModels.find((m) => m.id === selectedModel);
    return found?.label ?? selectedModel.split(":").pop() ?? t("selectModel");
  }, [selectedModel, flatModels, t]);

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
              "bg-[color:var(--cm-surface-panel-strong)]",
              "text-[11px] text-muted-foreground",
              "transition-colors duration-[var(--ds-dur-fast)]",
              "hover:bg-[color:var(--cm-surface-panel-solid)] hover:text-foreground"
            )}
          >
            <span className="max-w-[120px] truncate">{selectedLabel}</span>
            <ChevronDown className="size-3 opacity-70" />
          </button>
        }
      />
    </div>
  );
}
