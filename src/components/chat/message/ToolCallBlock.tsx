import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Wrench,
  ChevronRight,
  Loader2,
  CheckCircle2,
  XCircle,
  Clock,
  Server,
} from "lucide-react";
import { cn } from "@/lib/utils";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import type { ToolCall } from "@/lib/ipc";
import { CodeBlock } from "./CodeBlock";

interface ToolCallBlockProps {
  toolCall: ToolCall;
}

export function ToolCallBlock({ toolCall }: ToolCallBlockProps) {
  const { t } = useTranslation("chat");
  const [expanded, setExpanded] = useState(false);

  const elapsed = computeElapsed(toolCall);

  return (
    <Collapsible open={expanded} onOpenChange={setExpanded} className="my-1">
      <CollapsibleTrigger
        className={cn(
          "flex w-full items-center gap-2 rounded-[var(--radius-ui-md)] px-2.5 py-1.5",
          "text-xs",
          "transition-colors duration-[var(--ds-dur-fast)]",
          "hover:bg-[color:var(--surface-hover)]",
          "focus-visible:ring-2 focus-visible:ring-ring/35 focus-visible:ring-offset-1 focus-visible:outline-none",
          statusContainerStyle(toolCall.status)
        )}
      >
        <StatusIcon status={toolCall.status} />
        <Wrench className="size-3 text-muted-foreground/70" />
        <span className="min-w-0 truncate font-medium text-foreground/90">
          {toolCall.tool_name}
        </span>
        <span className="text-muted-foreground/60">·</span>
        <span className="flex min-w-0 shrink items-center gap-1 text-muted-foreground/60">
          <Server className="size-2.5" />
          <span className="truncate">{toolCall.server_name}</span>
        </span>
        {elapsed !== null && (
          <span className="ml-auto flex items-center gap-1 text-muted-foreground/50">
            <Clock className="size-2.5" />
            {formatDuration(elapsed)}
          </span>
        )}
        <ChevronRight
          className={cn(
            "size-3 shrink-0 text-muted-foreground/50 transition-transform duration-[var(--ds-dur-fast)]",
            expanded && "rotate-90",
            elapsed === null ? "ml-auto" : "ml-1"
          )}
        />
      </CollapsibleTrigger>
      <CollapsibleContent className="animate-in fade-in-0 slide-in-from-top-1 ease-out duration-[var(--ds-dur-fast)] data-[state=closed]:animate-out data-[state=closed]:fade-out-0">
        <div
          className={cn(
            "mt-1 space-y-2 rounded-[var(--radius-ui-md)] px-3 py-2.5",
            "border border-[color:var(--border-muted)] bg-[color:var(--surface-card)]"
          )}
        >
          <DetailSection label={t("toolCall.arguments")}>
            <CodeBlock
              code={formatJson(toolCall.arguments)}
              language="json"
            />
          </DetailSection>
          {toolCall.result !== null && (
            <DetailSection label={t("toolCall.result")}>
              <CodeBlock
                code={formatResult(toolCall.result)}
                language="json"
              />
            </DetailSection>
          )}
          {toolCall.error && (
            <DetailSection label={t("toolCall.error")}>
              <p className="text-xs text-destructive">{toolCall.error}</p>
            </DetailSection>
          )}
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
}

function StatusIcon({ status }: { status: ToolCall["status"] }) {
  switch (status) {
    case "pending":
      return <span className="size-2 shrink-0 rounded-full bg-muted-foreground/40" />;
    case "running":
      return (
        <Loader2 className="size-3.5 animate-spin text-primary" />
      );
    case "complete":
      return <CheckCircle2 className="size-3.5 text-emerald-500" />;
    case "error":
      return <XCircle className="size-3.5 text-destructive" />;
  }
}

function DetailSection({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <p className="mb-1 text-[0.6875rem] font-medium uppercase tracking-wide text-muted-foreground/60">
        {label}
      </p>
      {children}
    </div>
  );
}

function statusContainerStyle(status: ToolCall["status"]): string {
  switch (status) {
    case "pending":
      return "";
    case "running":
      return "border border-primary/20 bg-primary/5";
    case "complete":
      return "border border-emerald-500/20 bg-emerald-500/5";
    case "error":
      return "border border-destructive/20 bg-destructive/5";
  }
}

function computeElapsed(toolCall: ToolCall): number | null {
  if (toolCall.started_at == null) return null;
  if (toolCall.completed_at != null) {
    return toolCall.completed_at - toolCall.started_at;
  }
  if (toolCall.status === "running") {
    return Date.now() - toolCall.started_at;
  }
  return null;
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function formatResult(value: unknown): string {
  if (typeof value === "string") return value;
  return formatJson(value);
}
