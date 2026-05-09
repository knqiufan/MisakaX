import { useCallback, useEffect, useRef, useState } from "react";
import { Check, Copy } from "lucide-react";
import { cn } from "@/lib/utils";
import { codeToHtml } from "shiki";

interface CodeBlockProps {
  code: string;
  language?: string;
  className?: string;
}

export function CodeBlock({ code, language, className }: CodeBlockProps) {
  const [copied, setCopied] = useState(false);
  const [html, setHtml] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    let cancelled = false;

    codeToHtml(code, {
      lang: language ?? "text",
      theme: "github-dark-default",
    })
      .then((result) => {
        if (!cancelled) setHtml(result);
      })
      .catch(() => {
        if (!cancelled) setHtml(null);
      });

    return () => {
      cancelled = true;
    };
  }, [code, language]);

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(code).then(() => {
      setCopied(true);
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => setCopied(false), 2000);
    });
  }, [code]);

  return (
    <div
      className={cn(
        "group/code relative my-3 overflow-hidden rounded-[var(--radius-ui-md)]",
        "border border-[color:var(--border-muted)] bg-[#0d1117]",
        className
      )}
    >
      <div className="flex items-center justify-between border-b border-[color:var(--border-muted)] bg-[color:rgba(255,255,255,0.03)] px-3 py-1.5">
        <span className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground/60">
          {language ?? "text"}
        </span>
        <button
          type="button"
          onClick={handleCopy}
          className={cn(
            "inline-flex items-center gap-1 rounded-md px-1.5 py-0.5",
            "text-[10px] text-muted-foreground/60",
            "transition-colors duration-[var(--ds-dur-fast)]",
            "opacity-0 group-hover/code:opacity-100",
            "hover:bg-[color:var(--surface-hover)] hover:text-foreground"
          )}
          aria-label="Copy code"
        >
          {copied ? (
            <Check className="size-3 text-emerald-400" />
          ) : (
            <Copy className="size-3" />
          )}
        </button>
      </div>
      <div className="overflow-x-auto p-3 text-[13px] leading-relaxed">
        {html ? (
          <div
            className="[&_pre]:!bg-transparent [&_pre]:!p-0 [&_code]:!bg-transparent"
            dangerouslySetInnerHTML={{ __html: html }}
          />
        ) : (
          <pre className="whitespace-pre-wrap text-muted-foreground">
            <code>{code}</code>
          </pre>
        )}
      </div>
    </div>
  );
}
