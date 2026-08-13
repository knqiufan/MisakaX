import { cn } from "@/lib/utils";
import type { TokenUsage } from "@/lib/ipc";

interface TokenBadgeProps {
  usage: TokenUsage;
  className?: string;
}

export function TokenBadge({ usage, className }: TokenBadgeProps) {
  const resolvedTotal =
    usage.total_tokens ??
    (usage.input_tokens != null && usage.output_tokens != null
      ? usage.input_tokens + usage.output_tokens
      : null);
  const isEstimated =
    usage.measurement_source === "tokenizer_estimated" ||
    usage.measurement_source === "heuristic_estimated";
  const label =
    resolvedTotal == null
      ? "Usage unavailable"
      : `${isEstimated ? "~" : ""}${formatTokenCount(resolvedTotal)} tokens`;
  const title =
    resolvedTotal == null
      ? "Token usage was not reported"
      : `Input: ${usage.input_tokens ?? "unknown"} | Output: ${usage.output_tokens ?? "unknown"} | Total: ${resolvedTotal}${isEstimated ? " (estimated)" : ""}`;

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-full px-2 py-0.5",
        "bg-[color:var(--surface-card)] text-[10px] text-muted-foreground/70",
        "border border-[color:var(--border-muted)]",
        className
      )}
      title={title}
      aria-label={title}
    >
      <span>{label}</span>
    </span>
  );
}

function formatTokenCount(count: number): string {
  if (count >= 1_000_000) {
    return `${(count / 1_000_000).toFixed(1)}M`;
  }
  if (count >= 1_000) {
    return `${(count / 1_000).toFixed(1)}K`;
  }
  return String(count);
}
