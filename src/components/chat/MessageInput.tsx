import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type ChangeEvent,
  type ClipboardEvent,
  type DragEvent,
  type KeyboardEvent,
} from "react";
import { useTranslation } from "react-i18next";
import { Send, Square } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { MessageAttachment } from "@/lib/ipc";
import { useChatStore } from "@/stores/chat-store";
import { useComposerStore, type PendingAttachment } from "@/stores/composer-store";
import { AttachButton } from "./composer/AttachButton";
import { AttachmentMenu } from "./composer/AttachmentMenu";
import { AttachmentPreview } from "./composer/AttachmentPreview";
import { ComposerFooter } from "./composer/ComposerFooter";
import {
  ACCEPTED_ATTACHMENT_TYPES,
  MAX_IMAGE_SIZE,
  MAX_TEXT_ATTACHMENT_SIZE,
  canSendComposerMessage,
  classifyAttachment,
  inferTextMime,
} from "./composer/attachment-utils";

const MIN_HEIGHT = 36;
const MAX_HEIGHT = 200;

interface MessageInputProps {
  onSend: (
    content: string,
    modelOverride?: string,
    attachments?: MessageAttachment[]
  ) => void;
  onStop: () => void;
  disabled?: boolean;
  onPickWorkspaceFile?: () => void;
}

export function MessageInput({
  onSend,
  onStop,
  disabled = false,
  onPickWorkspaceFile,
}: MessageInputProps) {
  const { t } = useTranslation("chat");
  const { isStreaming, selectedModel } = useChatStore();
  const { attachments, addAttachment, removeAttachment, clearAttachments } =
    useComposerStore();
  const [content, setContent] = useState("");
  const [isDragOver, setIsDragOver] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const imageInputRef = useRef<HTMLInputElement>(null);
  const textInputRef = useRef<HTMLInputElement>(null);

  const canSend = canSendComposerMessage({
    content,
    attachmentCount: attachments.length,
    disabled,
    isStreaming,
  });

  const adjustHeight = useCallback(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.min(Math.max(el.scrollHeight, MIN_HEIGHT), MAX_HEIGHT)}px`;
  }, []);

  useEffect(() => {
    adjustHeight();
  }, [content, adjustHeight]);

  const handleSend = useCallback(() => {
    const trimmed = content.trim();
    if (!canSend) return;
    onSend(trimmed, selectedModel ?? undefined, toMessageAttachments(attachments));
    setContent("");
    clearAttachments();
    requestAnimationFrame(() => {
      if (textareaRef.current) textareaRef.current.style.height = `${MIN_HEIGHT}px`;
    });
  }, [attachments, canSend, clearAttachments, content, onSend, selectedModel]);

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
      Array.from(files).forEach((file) => {
        const classification = classifyAttachment(file.type, file.name);
        if (!classification.supported) {
          console.warn(`Unsupported attachment type: ${file.name} (${file.type})`);
          return;
        }
        if (classification.kind === "image") {
          readImageAttachment(file, addAttachment);
          return;
        }
        if (classification.kind === "text") {
          readTextAttachment(file, addAttachment);
        }
      });
    },
    [addAttachment]
  );

  const handlePaste = useCallback(
    (e: ClipboardEvent<HTMLTextAreaElement>) => {
      const files = Array.from(e.clipboardData?.items ?? [])
        .filter((item) => item.type.startsWith("image/"))
        .map((item) => item.getAsFile())
        .filter((file): file is File => Boolean(file));
      if (files.length === 0) return;
      e.preventDefault();
      processFiles(files);
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
      if (e.dataTransfer?.files?.length) processFiles(e.dataTransfer.files);
    },
    [processFiles]
  );

  return (
    <div className="shrink-0 border-t border-[color:var(--border-muted)] bg-[color:var(--surface-topbar)] px-4 py-3">
      <div className="flex items-end gap-2">
        <AttachmentMenu
          trigger={
            <AttachButton
              label={t("composer.attach")}
              onClick={() => undefined}
              disabled={disabled}
            />
          }
          labels={{
            image: t("composer.attachImage"),
            text: t("composer.attachText"),
            workspace: t("composer.attachWorkspaceFile"),
            planned: t("composer.plannedDocuments"),
            comingSoon: t("composer.comingSoon"),
          }}
          onPickImages={() => imageInputRef.current?.click()}
          onPickText={() => textInputRef.current?.click()}
          onPickWorkspace={() => onPickWorkspaceFile?.()}
        />
        <div
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          className={cn(
            "flex min-w-0 flex-1 flex-col rounded-[var(--radius-ui-lg)]",
            "border bg-[color:var(--surface-card)]",
            "px-3 py-1.5 transition-colors duration-[var(--ds-dur-fast)]",
            isDragOver
              ? "border-primary/50 bg-primary/5"
              : "border-[color:var(--border-muted)] focus-within:border-[color:var(--border-strong)]"
          )}
        >
          <AttachmentPreview
            attachments={attachments}
            onRemove={removeAttachment}
          />
          <div className="flex items-center gap-2">
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
                "min-h-[36px] max-h-[200px] flex-1 resize-none bg-transparent",
                "py-2 text-sm leading-[20px] text-foreground outline-none",
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
      </div>
      <input
        ref={imageInputRef}
        type="file"
        accept="image/png,image/jpeg,image/gif,image/webp"
        multiple
        onChange={(e) => handleFileChange(e, processFiles)}
        className="hidden"
        aria-hidden
      />
      <input
        ref={textInputRef}
        type="file"
        accept={ACCEPTED_ATTACHMENT_TYPES}
        multiple
        onChange={(e) => handleFileChange(e, processFiles)}
        className="hidden"
        aria-hidden
      />
      <ComposerFooter t={t} />
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
              "bg-[color:rgba(255,107,107,0.12)] text-[#ff6b6b]",
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
            "bg-[color:var(--cm-surface-panel-strong)] text-foreground",
            "hover:bg-[color:var(--cm-surface-panel-solid)]",
            "disabled:cursor-not-allowed disabled:opacity-50"
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

function handleFileChange(
  e: ChangeEvent<HTMLInputElement>,
  processFiles: (files: FileList | File[]) => void
) {
  if (e.target.files?.length) processFiles(e.target.files);
  e.target.value = "";
}

function readImageAttachment(
  file: File,
  addAttachment: (attachment: PendingAttachment) => void
) {
  if (file.size > MAX_IMAGE_SIZE) {
    console.warn(`Image too large: ${file.name}`);
    return;
  }
  const reader = new FileReader();
  reader.onload = () => {
    const base64 = (reader.result as string).split(",")[1];
    if (!base64) return;
    addAttachment({
      id: crypto.randomUUID(),
      kind: "image",
      data: base64,
      media_type: file.type,
      file_name: file.name,
      size: file.size,
    });
  };
  reader.readAsDataURL(file);
}

function readTextAttachment(
  file: File,
  addAttachment: (attachment: PendingAttachment) => void
) {
  if (file.size > MAX_TEXT_ATTACHMENT_SIZE) {
    console.warn(`Text attachment too large: ${file.name}`);
    return;
  }
  const reader = new FileReader();
  reader.onload = () => {
    addAttachment({
      id: crypto.randomUUID(),
      kind: "text",
      extracted_text: String(reader.result ?? ""),
      mime: inferTextMime(file),
      file_name: file.name,
      size: file.size,
    });
  };
  reader.readAsText(file);
}

function toMessageAttachments(
  attachments: PendingAttachment[]
): MessageAttachment[] | undefined {
  if (attachments.length === 0) return undefined;
  return attachments.map((attachment) =>
    attachment.kind === "image"
      ? {
          kind: "image",
          data: attachment.data,
          media_type: attachment.media_type,
          file_name: attachment.file_name,
        }
      : {
          kind: "text",
          extracted_text: attachment.extracted_text,
          mime: attachment.mime,
          file_name: attachment.file_name,
          size: attachment.size,
        }
  );
}
