import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Check, Copy, RefreshCw, AlertTriangle } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Message, TokenUsage } from "@/lib/ipc";
import { CodeBlock } from "./CodeBlock";
import { ThinkingBlock } from "./ThinkingBlock";
import { ToolCallBlock } from "./ToolCallBlock";
import { TokenBadge } from "./TokenBadge";
import { StreamingIndicator } from "./StreamingIndicator";

interface MessageItemProps {
  message: Message;
  isStreaming?: boolean;
  isThinkingStreaming?: boolean;
  onRegenerate?: (messageId: string) => void;
  prevRole?: string | null;
}

export function MessageItem({
  message,
  isStreaming,
  isThinkingStreaming,
  onRegenerate,
  prevRole,
}: MessageItemProps) {
  const isUser = message.role === "user";
  const hasThinking = !isUser && (!!message.thinking_content || isThinkingStreaming);
  const hasToolCalls = !isUser && message.tool_calls && message.tool_calls.length > 0;

  return (
    <div
      className={cn(
        "group/msg px-4",
        prevRole !== message.role ? "pt-4 pb-1" : "pt-1 pb-1",
        isUser ? "flex justify-end" : "w-full"
      )}
    >
      {isUser ? (
        <div className="flex max-w-[80%] flex-col items-end gap-1">
          <MessageBubble message={message} isUser={isUser} />
          <UserMessageActions message={message} onRegenerate={onRegenerate} />
        </div>
      ) : (
        <div className="mx-auto w-full max-w-[min(100%,48rem)] flex flex-col gap-1">
          {hasThinking && (
            <ThinkingBlock
              content={message.thinking_content ?? ""}
              isStreaming={isThinkingStreaming}
            />
          )}
          {hasToolCalls && (
            <div className="flex w-full flex-col gap-0.5">
              {message.tool_calls!.map((tc) => (
                <ToolCallBlock key={tc.id} toolCall={tc} />
              ))}
            </div>
          )}
          <MessageBubble message={message} isUser={isUser} />
          {isStreaming && message.status === "streaming" && (
            <StreamingIndicator className="ml-1" />
          )}
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

  const handleRegenerate = useCallback(() => {
    onRegenerate?.(message.id);
  }, [message.id, onRegenerate]);

  return (
    <div className="flex items-center gap-1 opacity-0 transition-opacity duration-[var(--ds-dur-fast)] group-hover/msg:opacity-100">
      <ActionButton
        onClick={handleCopy}
        label={copied ? t("copied") : t("copyMessage")}
      >
        {copied ? (
          <Check className="size-3 text-emerald-400" />
        ) : (
          <Copy className="size-3" />
        )}
      </ActionButton>
      {onRegenerate && (
        <ActionButton onClick={handleRegenerate} label={t("regenerate")}>
          <RefreshCw className="size-3" />
        </ActionButton>
      )}
    </div>
  );
}

function MessageBubble({
  message,
  isUser,
}: {
  message: Message;
  isUser: boolean;
}) {
  if (!isUser && message.status === "error") {
    return (
      <div
        className={cn(
          "rounded-[var(--radius-ui-lg)] px-3.5 py-2.5 text-sm leading-relaxed",
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
      <div
        className={cn(
          "rounded-[var(--radius-ui-lg)] px-3.5 py-2.5 text-sm leading-relaxed",
          "bg-primary text-primary-foreground"
        )}
      >
        {attachedImages.length > 0 && (
          <div className="mb-2 flex flex-wrap gap-2">
            {attachedImages.map((img, i) => (
              <img
                key={i}
                src={`data:${img.mime_type};base64,${img.data}`}
                alt=""
                className="max-h-48 max-w-[200px] rounded-[var(--radius-ui-md)] object-contain"
              />
            ))}
          </div>
        )}
        <p className="whitespace-pre-wrap">{message.content}</p>
      </div>
    );
  }

  return (
    <div className="w-full rounded-[var(--radius-ui-lg)] bg-[color:var(--surface-card)]/50 px-4 py-3 text-sm leading-relaxed text-foreground">
      {attachedImages.length > 0 && (
        <div className="mb-2 flex flex-wrap gap-2">
          {attachedImages.map((img, i) => (
            <img
              key={i}
              src={`data:${img.mime_type};base64,${img.data}`}
              alt=""
              className="max-h-48 max-w-[200px] rounded-[var(--radius-ui-md)] object-contain"
            />
          ))}
        </div>
      )}
      <MarkdownContent content={message.content} />
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

function MarkdownContent({ content }: { content: string }) {
  return (
    <Markdown
      remarkPlugins={[remarkGfm]}
      components={{
        code({ className, children, ...props }) {
          const match = /language-(\w+)/.exec(className ?? "");
          const codeStr = String(children).replace(/\n$/, "");

          if (match) {
            return <CodeBlock code={codeStr} language={match[1]} />;
          }

          return (
            <code
              className={cn(
                "rounded-[4px] bg-[color:rgba(255,255,255,0.08)] px-1.5 py-0.5",
                "text-[0.8125em] font-mono text-foreground/90",
                className
              )}
              {...props}
            >
              {children}
            </code>
          );
        },
        pre({ children }) {
          return <>{children}</>;
        },
        table({ children }) {
          return (
            <div className="my-3 overflow-x-auto rounded-[var(--radius-ui-md)] border border-[color:var(--border-muted)]">
              <table className="w-full text-xs">{children}</table>
            </div>
          );
        },
        th({ children }) {
          return (
            <th className="border-b border-[color:var(--border-muted)] bg-[color:var(--surface-card)] px-3 py-2 text-left font-semibold">
              {children}
            </th>
          );
        },
        td({ children }) {
          return (
            <td className="border-b border-[color:var(--border-muted)] px-3 py-2">
              {children}
            </td>
          );
        },
        a({ href, children }) {
          return (
            <a
              href={href}
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary underline underline-offset-2 hover:text-primary/80"
            >
              {children}
            </a>
          );
        },
        ul({ children }) {
          return <ul className="my-2 ml-4 list-disc space-y-1">{children}</ul>;
        },
        ol({ children }) {
          return (
            <ol className="my-2 ml-4 list-decimal space-y-1">{children}</ol>
          );
        },
        blockquote({ children }) {
          return (
            <blockquote className="my-2 border-l-2 border-[color:var(--border-strong)] pl-3 text-muted-foreground">
              {children}
            </blockquote>
          );
        },
        h1({ children }) {
          return (
            <h1 className="mb-2 mt-4 text-lg font-bold">{children}</h1>
          );
        },
        h2({ children }) {
          return (
            <h2 className="mb-2 mt-3 text-base font-bold">{children}</h2>
          );
        },
        h3({ children }) {
          return (
            <h3 className="mb-1.5 mt-2.5 text-sm font-bold">{children}</h3>
          );
        },
        p({ children }) {
          return <p className="my-1.5 leading-relaxed">{children}</p>;
        },
        hr() {
          return (
            <hr className="my-4 border-[color:var(--border-muted)]" />
          );
        },
      }}
    >
      {content}
    </Markdown>
  );
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

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(message.content).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, [message.content]);

  const handleRegenerate = useCallback(() => {
    onRegenerate?.(message.id);
  }, [message.id, onRegenerate]);

  if (!showActions && !parsedUsage) return null;

  return (
    <div className="flex items-center gap-2 px-1 pt-0.5 opacity-0 transition-opacity duration-[var(--ds-dur-fast)] group-hover/msg:opacity-100">
      {showActions && (
        <>
          <ActionButton
            onClick={handleCopy}
            label={copied ? t("copied") : t("copyMessage")}
          >
            {copied ? (
              <Check className="size-3 text-emerald-400" />
            ) : (
              <Copy className="size-3" />
            )}
          </ActionButton>
          {onRegenerate && (
            <ActionButton onClick={handleRegenerate} label={t("regenerate")}>
              <RefreshCw className="size-3" />
            </ActionButton>
          )}
        </>
      )}
      {parsedUsage && <TokenBadge usage={parsedUsage} />}
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
        "inline-flex size-6 items-center justify-center rounded-md",
        "text-muted-foreground/60",
        "transition-colors duration-[var(--ds-dur-fast)]",
        "hover:bg-[color:var(--surface-hover)] hover:text-foreground"
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
