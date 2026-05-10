import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent,
  type ClipboardEvent,
  type DragEvent,
} from "react";
import { useTranslation } from "react-i18next";
import { Send, Square, ChevronDown, Paperclip } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { modelsIpc } from "@/lib/ipc";
import type { ImageAttachment, ProviderModels } from "@/lib/ipc";
import { useChatStore } from "@/stores/chat-store";
import { ModelSelector } from "./ModelSelector";
import { ImagePreview, type PendingImage } from "./ImagePreview";

const MIN_HEIGHT = 40;
const MAX_HEIGHT = 200;
const MAX_IMAGE_SIZE = 10 * 1024 * 1024;
const ALLOWED_IMAGE_TYPES = [
  "image/png",
  "image/jpeg",
  "image/gif",
  "image/webp",
];

interface MessageInputProps {
  onSend: (
    content: string,
    modelOverride?: string,
    images?: ImageAttachment[]
  ) => void;
  onStop: () => void;
  disabled?: boolean;
}

export function MessageInput({ onSend, onStop, disabled }: MessageInputProps) {
  const { t } = useTranslation("chat");
  const { isStreaming, selectedModel } = useChatStore();
  const [content, setContent] = useState("");
  const [pendingImages, setPendingImages] = useState<PendingImage[]>([]);
  const [isDragOver, setIsDragOver] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const canSend =
    (content.trim().length > 0 || pendingImages.length > 0) &&
    !disabled &&
    !isStreaming;

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
    if ((!trimmed && pendingImages.length === 0) || isStreaming) return;

    const images: ImageAttachment[] | undefined =
      pendingImages.length > 0
        ? pendingImages.map((img) => ({
            type: "base64",
            data: img.data,
            mime_type: img.mime_type,
          }))
        : undefined;

    onSend(trimmed, selectedModel ?? undefined, images);
    setContent("");
    setPendingImages([]);
    requestAnimationFrame(() => {
      if (textareaRef.current) {
        textareaRef.current.style.height = `${MIN_HEIGHT}px`;
      }
    });
  }, [content, isStreaming, onSend, selectedModel, pendingImages]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend]
  );

  const processFiles = useCallback(
    (files: FileList | File[]) => {
      const fileArray = Array.from(files);
      for (const file of fileArray) {
        if (!ALLOWED_IMAGE_TYPES.includes(file.type)) {
          console.warn(`Unsupported image type: ${file.type}`);
          continue;
        }
        if (file.size > MAX_IMAGE_SIZE) {
          console.warn(
            `Image too large: ${file.name} (${(file.size / 1024 / 1024).toFixed(1)}MB > 10MB)`
          );
          continue;
        }

        const reader = new FileReader();
        reader.onload = () => {
          const base64 = (reader.result as string).split(",")[1];
          if (!base64) return;

          setPendingImages((prev) => [
            ...prev,
            {
              id: crypto.randomUUID(),
              data: base64,
              mime_type: file.type,
              name: file.name,
              size: file.size,
            },
          ]);
        };
        reader.readAsDataURL(file);
      }
    },
    []
  );

  const handlePaste = useCallback(
    (e: ClipboardEvent<HTMLTextAreaElement>) => {
      const items = e.clipboardData?.items;
      if (!items) return;

      const imageFiles: File[] = [];
      for (const item of Array.from(items)) {
        if (item.type.startsWith("image/")) {
          const file = item.getAsFile();
          if (file) imageFiles.push(file);
        }
      }

      if (imageFiles.length > 0) {
        e.preventDefault();
        processFiles(imageFiles);
      }
    },
    [processFiles]
  );

  const handleDragOver = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragOver(false);

      const files = e.dataTransfer?.files;
      if (files && files.length > 0) {
        processFiles(files);
      }
    },
    [processFiles]
  );

  const handleAttachClick = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleFileChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const files = e.target.files;
      if (files && files.length > 0) {
        processFiles(files);
      }
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    },
    [processFiles]
  );

  const handleRemoveImage = useCallback((id: string) => {
    setPendingImages((prev) => prev.filter((img) => img.id !== id));
  }, []);

  return (
    <div className="shrink-0 border-t border-[color:var(--border-muted)] bg-[color:var(--surface-topbar)] px-4 py-3">
      <div
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        className={cn(
          "flex flex-col rounded-[var(--radius-ui-lg)]",
          "border bg-[color:var(--surface-card)]",
          "px-3 py-2 transition-colors duration-[var(--ds-dur-fast)]",
          isDragOver
            ? "border-primary/50 bg-primary/5"
            : "border-[color:var(--border-muted)] focus-within:border-[color:var(--border-strong)]"
        )}
      >
        <ImagePreview images={pendingImages} onRemove={handleRemoveImage} />

        <div className="flex items-end gap-2">
          <AttachButton onClick={handleAttachClick} t={t} />

          <textarea
            ref={textareaRef}
            value={content}
            onChange={(e) => setContent(e.target.value)}
            onKeyDown={handleKeyDown}
            onPaste={handlePaste}
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
            onStop={onStop}
            t={t}
          />
        </div>
      </div>

      <input
        ref={fileInputRef}
        type="file"
        accept={ALLOWED_IMAGE_TYPES.join(",")}
        multiple
        onChange={handleFileChange}
        className="hidden"
        aria-hidden
      />

      <InputFooter t={t} />
    </div>
  );
}

function AttachButton({
  onClick,
  t,
}: {
  onClick: () => void;
  t: (key: string) => string;
}) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type="button"
          onClick={onClick}
          aria-label={t("attach")}
          className={cn(
            "mb-0.5 inline-flex size-7 shrink-0 items-center justify-center rounded-full",
            "border border-[color:var(--cm-border-strong)]",
            "bg-[color:var(--cm-surface-panel-solid)]",
            "text-muted-foreground",
            "transition-colors duration-[var(--ds-dur-fast)]",
            "hover:text-foreground"
          )}
        >
          <Paperclip className="size-3.5" />
        </button>
      </TooltipTrigger>
      <TooltipContent side="top" className="text-xs">
        {t("attach")}
      </TooltipContent>
    </Tooltip>
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
