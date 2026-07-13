import { useCallback, useEffect, useRef, useState } from "react";
import { Check, Copy } from "lucide-react";
import { cn } from "@/lib/utils";
import { codeToHtml } from "shiki";
import { useThemeStore } from "@/stores/theme-store";

interface CodeBlockProps {
  code: string;
  language?: string;
  className?: string;
  /** When true, omit outer chrome (used inside Markdown fence wrapper). */
  bare?: boolean;
}

export function CodeBlock({
  code,
  language,
  className,
  bare = false,
}: CodeBlockProps) {
  const [copied, setCopied] = useState(false);
  const [html, setHtml] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const resolvedTheme = useThemeStore((s) => s.resolvedTheme);

  useEffect(() => {
    let cancelled = false;
    const theme =
      resolvedTheme === "dark" ? "github-dark" : "github-light";

    codeToHtml(code, {
      lang: language ?? "text",
      theme,
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
  }, [code, language, resolvedTheme]);

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(code).then(() => {
      setCopied(true);
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => setCopied(false), 2000);
    });
  }, [code]);

  const body = (
    <>
      <div
        className={cn(
          "flex items-center justify-between px-3 py-1",
          bare ? "bg-muted/30" : "border-b border-border/40 bg-muted/30"
        )}
      >
        <span className="text-[10px] font-mono uppercase tracking-wider text-muted-foreground/70">
          {language ?? "text"}
        </span>
        <button
          type="button"
          onClick={handleCopy}
          className={cn(
            "inline-flex h-7 w-7 items-center justify-center rounded-md",
            "text-muted-foreground transition-colors duration-[var(--ds-dur-fast)]",
            "hover:bg-muted hover:text-foreground"
          )}
          aria-label="Copy code"
        >
          {copied ? (
            <Check className="size-3.5 text-emerald-500" />
          ) : (
            <Copy className="size-3.5" />
          )}
        </button>
      </div>
      <div className="overflow-x-auto p-3 text-[13px] leading-relaxed">
        {html ? (
          <div
            className="[&_pre]:!bg-transparent [&_pre]:!m-0 [&_pre]:!p-0 [&_code]:!bg-transparent"
            dangerouslySetInnerHTML={{ __html: html }}
          />
        ) : (
          <pre className="whitespace-pre-wrap text-muted-foreground">
            <code>{code}</code>
          </pre>
        )}
      </div>
    </>
  );

  if (bare) {
    return <div className={cn("overflow-hidden", className)}>{body}</div>;
  }

  return (
    <div
      className={cn(
        "group/code relative my-3 overflow-hidden rounded-xl bg-muted/20",
        className
      )}
    >
      {body}
    </div>
  );
}
