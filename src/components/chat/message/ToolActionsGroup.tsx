import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  CheckCircle2,
  ChevronDown,
  Loader2,
  Server,
  Wrench,
  XCircle,
} from "lucide-react";
import { cn } from "@/lib/utils";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import type { ToolCall } from "@/lib/ipc";
import { CodeBlock } from "./CodeBlock";

const AUTO_CLOSE_DELAY_MS = 1000;

interface ToolActionsGroupProps {
  toolCalls: ToolCall[];
  /** Force open (e.g. Tool Logs drawer). Otherwise auto: open while running, close when done. */
  defaultOpen?: boolean;
}

/** Compact left-rail tool list (docs/ui/06 §7.2). */
export function ToolActionsGroup({
  toolCalls,
  defaultOpen,
}: ToolActionsGroupProps) {
  const { t } = useTranslation("chat");
  const hasRunning = useMemo(
    () =>
      toolCalls.some(
        (tc) => tc.status === "running" || tc.status === "pending"
      ),
    [toolCalls]
  );

  const [open, setOpen] = useState(
    defaultOpen !== undefined ? defaultOpen : hasRunning
  );
  const userToggledRef = useRef(false);
  const autoClosedRef = useRef(false);

  useEffect(() => {
    if (defaultOpen !== undefined) {
      setOpen(defaultOpen);
      return;
    }
    if (userToggledRef.current) return;

    if (hasRunning) {
      autoClosedRef.current = false;
      setOpen(true);
      return;
    }

    if (autoClosedRef.current || toolCalls.length === 0) return;

    // History / already-complete groups mount collapsed — no delayed close needed.
    if (!open) {
      autoClosedRef.current = true;
      return;
    }

    const timer = window.setTimeout(() => {
      setOpen(false);
      autoClosedRef.current = true;
    }, AUTO_CLOSE_DELAY_MS);

    return () => window.clearTimeout(timer);
  }, [hasRunning, defaultOpen, toolCalls.length, open]);

  const summary = useMemo(() => {
    const running = toolCalls.filter(
      (tc) => tc.status === "running" || tc.status === "pending"
    ).length;
    if (running > 0) {
      return t("toolGroup.running", { count: running });
    }
    const failed = toolCalls.filter((tc) => tc.status === "error").length;
    if (failed > 0) {
      return t("toolGroup.failed", { count: failed });
    }
    return t("toolGroup.done", { count: toolCalls.length });
  }, [toolCalls, t]);

  if (toolCalls.length === 0) return null;

  return (
    <div className="mb-2 ml-1.5 max-w-[min(100%,48rem)] border-l-2 border-border/50 pl-2">
      <Collapsible
        open={open}
        onOpenChange={(next) => {
          userToggledRef.current = true;
          setOpen(next);
        }}
      >
        <CollapsibleTrigger
          className={cn(
            "flex w-full items-center gap-2 rounded-md px-2 py-1 text-xs",
            "text-muted-foreground/60 transition-colors duration-[var(--ds-dur-fast)]",
            "hover:bg-muted/30 hover:text-muted-foreground"
          )}
        >
          <span className="rounded bg-muted/80 px-1.5 py-0.5 text-[10px] tabular-nums">
            {toolCalls.length}
          </span>
          <span className="min-w-0 flex-1 truncate text-left">{summary}</span>
          <ChevronDown
            className={cn(
              "size-3.5 shrink-0 transition-transform duration-[var(--ds-dur-fast)]",
              open && "rotate-180"
            )}
          />
        </CollapsibleTrigger>
        <CollapsibleContent>
          <div className="mt-0.5 space-y-0.5">
            {toolCalls.map((tc) => (
              <ToolActionRow key={tc.id} toolCall={tc} />
            ))}
          </div>
        </CollapsibleContent>
      </Collapsible>
    </div>
  );
}

interface ToolActionRowProps {
  toolCall: ToolCall;
}

export function ToolActionRow({ toolCall }: ToolActionRowProps) {
  const { t } = useTranslation("chat");
  const [expanded, setExpanded] = useState(false);
  const argSummary = summarizeArgs(toolCall.arguments);

  return (
    <Collapsible open={expanded} onOpenChange={setExpanded}>
      <CollapsibleTrigger
        className={cn(
          "flex min-h-7 w-full items-center gap-2 rounded-sm px-2 py-1 text-xs",
          "transition-colors duration-[var(--ds-dur-fast)] hover:bg-muted/30"
        )}
      >
        <Wrench className="size-3.5 shrink-0 text-muted-foreground" />
        <span className="shrink-0 font-medium text-muted-foreground">
          {toolCall.tool_name}
        </span>
        {argSummary ? (
          <span className="min-w-0 flex-1 truncate font-mono text-muted-foreground/60">
            {argSummary}
          </span>
        ) : (
          <span className="flex-1" />
        )}
        <span className="flex max-w-[200px] shrink-0 items-center gap-1 font-mono text-[11px] text-muted-foreground/40">
          <Server className="size-2.5" />
          <span className="truncate">{toolCall.server_name}</span>
        </span>
        <StatusIcon status={toolCall.status} />
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div className="ml-6 mt-1 space-y-2 border-l-2 border-border/30 py-1 pl-2">
          <DetailBlock
            label={t("toolCall.arguments")}
            body={
              <CodeBlock
                code={formatJson(toolCall.arguments)}
                language="json"
                className="my-0"
              />
            }
          />
          {toolCall.result !== null ? (
            <DetailBlock
              label={t("toolCall.result")}
              body={
                <pre className="max-h-[140px] overflow-auto rounded-lg bg-muted/40 px-2.5 py-2 font-mono text-[11px] text-muted-foreground/80">
                  {formatResult(toolCall.result)}
                </pre>
              }
            />
          ) : null}
          {toolCall.error ? (
            <p className="text-[11px] text-destructive">{toolCall.error}</p>
          ) : null}
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
}

function DetailBlock({
  label,
  body,
}: {
  label: string;
  body: React.ReactNode;
}) {
  return (
    <div className="space-y-1">
      <p className="text-[10px] font-medium uppercase tracking-wider text-muted-foreground/60">
        {label}
      </p>
      {body}
    </div>
  );
}

function StatusIcon({ status }: { status: ToolCall["status"] }) {
  switch (status) {
    case "pending":
      return (
        <span
          className="size-2 shrink-0 rounded-full bg-muted-foreground/40"
          aria-hidden
        />
      );
    case "running":
      return (
        <Loader2 className="size-3.5 shrink-0 animate-spin text-muted-foreground" />
      );
    case "complete":
      return <CheckCircle2 className="size-3.5 shrink-0 text-emerald-500" />;
    case "error":
      return <XCircle className="size-3.5 shrink-0 text-destructive" />;
    default:
      return null;
  }
}

function summarizeArgs(args: Record<string, unknown>): string {
  const keys = Object.keys(args);
  if (keys.length === 0) return "";
  const first = keys[0];
  const value = args[first];
  const text =
    typeof value === "string"
      ? value
      : value === null || value === undefined
        ? ""
        : JSON.stringify(value);
  return text.length > 80 ? `${text.slice(0, 80)}…` : text;
}

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function formatResult(result: unknown): string {
  if (typeof result === "string") return result;
  return formatJson(result);
}
