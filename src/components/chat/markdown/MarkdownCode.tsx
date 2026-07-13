import { useCallback, useState, type ComponentPropsWithoutRef } from "react";
import { Check, Copy } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ExtraProps } from "streamdown";
import { CodeBlock } from "../message/CodeBlock";

type CodeProps = ComponentPropsWithoutRef<"code"> & ExtraProps;

const TERMINAL_LANGS = new Set([
  "bash",
  "sh",
  "shell",
  "zsh",
  "console",
  "terminal",
]);

function isFenceBlock(
  className: string | undefined,
  children: React.ReactNode
): { language: string; code: string } | null {
  const match = /language-(\w+)/.exec(className ?? "");
  const code = String(children).replace(/\n$/, "");
  const multiline = code.includes("\n");
  if (match) {
    return { language: match[1], code };
  }
  if (multiline) {
    return { language: "text", code };
  }
  return null;
}

/** Inline pill vs fence block (docs/ui/06 §3). */
export function MarkdownCode({ className, children, ...props }: CodeProps) {
  const fence = isFenceBlock(className, children);

  if (fence) {
    const isTerminal = TERMINAL_LANGS.has(fence.language.toLowerCase());
    return (
      <div
        className={cn(
          "my-4 overflow-hidden rounded-xl bg-muted/20",
          isTerminal && "chat-terminal-fence"
        )}
        data-language={fence.language}
        data-misaka-chat-code-block=""
      >
        <CodeBlock
          code={fence.code}
          language={fence.language}
          className="my-0 border-0 bg-transparent"
          bare
        />
      </div>
    );
  }

  return (
    <code
      className={cn(
        "rounded bg-muted px-1.5 py-0.5 font-mono text-[0.875em] text-foreground",
        className
      )}
      {...props}
    >
      {children}
    </code>
  );
}

export function CopyIconButton({
  text,
  className,
  label = "Copy",
}: {
  text: string;
  className?: string;
  label?: string;
}) {
  const [copied, setCopied] = useState(false);

  const onCopy = useCallback(() => {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, [text]);

  return (
    <button
      type="button"
      onClick={onCopy}
      aria-label={label}
      title={label}
      className={cn(
        "inline-flex h-7 w-7 items-center justify-center rounded-md text-xs text-muted-foreground",
        "transition-colors duration-[var(--ds-dur-fast)] hover:bg-muted hover:text-foreground",
        className
      )}
    >
      {copied ? (
        <Check className="size-3.5 text-emerald-500" />
      ) : (
        <Copy className="size-3.5" />
      )}
    </button>
  );
}
