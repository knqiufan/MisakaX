import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { Check, Copy, RefreshCw, AlertTriangle } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Message, TokenUsage } from "@/lib/ipc";
import { MessageResponse } from "../markdown/MessageResponse";
import { ThinkingBlock } from "./ThinkingBlock";
import { ToolActionsGroup } from "./ToolActionsGroup";
import { TokenBadge } from "./TokenBadge";
import { StreamingIndicator } from "./StreamingIndicator";

interface MessageItemProps {
  message: Message;
  isStreaming?: boolean;
  isThinkingStreaming?: boolean;
  onRegenerate?: (messageId: string) => void;
  prevRole?: string | null;
  highlight?: boolean;
  onOpenToolLogs?: () => void;
}

export function MessageItem({
  message,
  isStreaming,
  isThinkingStreaming,
  onRegenerate,
  highlight = false,
  onOpenToolLogs,
}: MessageItemProps) {
  const isUser = message.role === "user";
  const hasThinking =
    !isUser && (!!message.thinking_content || isThinkingStreaming);
  const hasToolCalls =
    !isUser && message.tool_calls && message.tool_calls.length > 0;

  return (
    <div
      id={`msg-${message.id}`}
      className={cn(
        "group/msg pb-6",
        isUser ? "flex justify-end" : "w-full",
        highlight && "search-highlight-flash"
      )}
    >
      {isUser ? (
        <div className="flex w-full max-w-[95%] flex-col items-end gap-2">
          <MessageBubble
            message={message}
            isUser
            isStreaming={false}
          />
          <UserMessageActions message={message} onRegenerate={onRegenerate} />
        </div>
      ) : (
        <div className="flex w-full flex-col gap-2">
          {hasThinking ? (
            <ThinkingBlock
              content={message.thinking_content ?? ""}
              isStreaming={isThinkingStreaming}
            />
          ) : null}
          {hasToolCalls ? (
            <ToolActionsGroup
              toolCalls={message.tool_calls!}
              onOpenToolLogs={onOpenToolLogs}
            />
          ) : null}
          <MessageBubble
            message={message}
            isUser={false}
            isStreaming={Boolean(isStreaming && message.status === "streaming")}
          />
          {isStreaming && message.status === "streaming" ? (
            <StreamingIndicator className="ml-1" />
          ) : null}
          <MessageFooter message={message} onRegenerate={onRegenerate} />
        </div>
      )}
    </div>
  );
}

function UserMessageActions({
  message,
  onRegenerate,
}: {
  message: Message;
  onRegenerate?: (id: string) => void;
}) {
  const { t } = useTranslation("chat");
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(message.content).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, [message.content]);

  return (
    <div className="flex items-center gap-1 opacity-0 transition-opacity duration-[var(--ds-dur-fast)] group-hover/msg:opacity-100 focus-within:opacity-100">
      <ActionButton
        onClick={handleCopy}
        label={copied ? t("copied") : t("copyMessage")}
      >
        {copied ? (
          <Check className="size-3 text-emerald-500" />
        ) : (
          <Copy className="size-3" />
        )}
      </ActionButton>
      {onRegenerate ? (
        <ActionButton
          onClick={() => onRegenerate(message.id)}
          label={t("regenerate")}
        >
          <RefreshCw className="size-3" />
        </ActionButton>
      ) : null}
    </div>
  );
}

function MessageBubble({
  message,
  isUser,
  isStreaming,
}: {
  message: Message;
  isUser: boolean;
  isStreaming: boolean;
}) {
  if (!isUser && message.status === "error") {
    return (
      <div
        className={cn(
          "rounded-xl px-4 py-3 text-sm leading-7",
          "border border-destructive/45 bg-destructive/8 text-destructive"
        )}
        role="alert"
      >
        <div className="flex gap-2.5">
          <AlertTriangle className="size-4 shrink-0 opacity-90" aria-hidden />
          <p className="whitespace-pre-wrap break-words">
            {message.content || "—"}
          </p>
        </div>
      </div>
    );
  }

  if (!message.content && message.status === "streaming") {
    return null;
  }

  const attachedImages = parseAttachments(message.attachments);

  if (isUser) {
    return (
      <div className="ml-auto w-fit min-w-0 max-w-full overflow-hidden break-words rounded-2xl bg-muted px-4 py-3 text-sm text-foreground">
        {attachedImages.length > 0 ? (
          <div className="mb-2 flex flex-wrap gap-2">
            {attachedImages.map((img, i) => (
              <img
                key={i}
                src={`data:${img.mime_type};base64,${img.data}`}
                alt=""
                className="max-h-48 max-w-[200px] rounded-lg border border-border/40 object-contain"
              />
            ))}
          </div>
        ) : null}
        <p className="whitespace-pre-wrap">{message.content}</p>
      </div>
    );
  }

  return (
    <div className="w-full min-w-0 text-sm text-foreground">
      {attachedImages.length > 0 ? (
        <div className="mb-2 flex flex-wrap gap-2">
          {attachedImages.map((img, i) => (
            <img
              key={i}
              src={`data:${img.mime_type};base64,${img.data}`}
              alt=""
              className="max-h-48 max-w-[200px] rounded-lg border border-border/40 object-contain"
            />
          ))}
        </div>
      ) : null}
      <MessageResponse content={message.content} isStreaming={isStreaming} />
    </div>
  );
}

interface AttachmentImage {
  data: string;
  mime_type: string;
}

function parseAttachments(raw: string | null): AttachmentImage[] {
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw) as AttachmentImage[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function MessageFooter({
  message,
  onRegenerate,
}: {
  message: Message;
  onRegenerate?: (id: string) => void;
}) {
  const { t } = useTranslation("chat");
  const [copied, setCopied] = useState(false);
  const parsedUsage = parseTokenUsage(message.token_usage);
  const showActions = message.status === "complete";

  if (!showActions && !parsedUsage) return null;

  return (
    <div className="flex items-center gap-1 opacity-0 transition-opacity duration-[var(--ds-dur-fast)] group-hover/msg:opacity-100 focus-within:opacity-100">
      {showActions ? (
        <>
          <ActionButton
            onClick={() => {
              navigator.clipboard.writeText(message.content).then(() => {
                setCopied(true);
                setTimeout(() => setCopied(false), 2000);
              });
            }}
            label={copied ? t("copied") : t("copyMessage")}
          >
            {copied ? (
              <Check className="size-3 text-emerald-500" />
            ) : (
              <Copy className="size-3" />
            )}
          </ActionButton>
          {onRegenerate ? (
            <ActionButton
              onClick={() => onRegenerate(message.id)}
              label={t("regenerate")}
            >
              <RefreshCw className="size-3" />
            </ActionButton>
          ) : null}
        </>
      ) : null}
      {parsedUsage ? <TokenBadge usage={parsedUsage} /> : null}
    </div>
  );
}

function ActionButton({
  onClick,
  label,
  children,
}: {
  onClick: () => void;
  label: string;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      title={label}
      aria-label={label}
      className={cn(
        "inline-flex size-7 items-center justify-center rounded-md",
        "text-muted-foreground/60",
        "transition-colors duration-[var(--ds-dur-fast)]",
        "hover:bg-muted/50 hover:text-foreground"
      )}
    >
      {children}
    </button>
  );
}

function parseTokenUsage(raw: string | null): TokenUsage | null {
  if (!raw) return null;
  try {
    return JSON.parse(raw) as TokenUsage;
  } catch {
    return null;
  }
}
