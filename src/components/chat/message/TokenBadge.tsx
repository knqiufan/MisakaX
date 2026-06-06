import { cn } from "@/lib/utils";
import type { TokenUsage } from "@/lib/ipc";

interface TokenBadgeProps {
  usage: TokenUsage;
  className?: string;
}

export function TokenBadge({ usage, className }: TokenBadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-full px-2 py-0.5",
        "bg-[color:var(--surface-card)] text-[10px] text-muted-foreground/70",
        "border border-[color:var(--border-muted)]",
        className
      )}
      title={`Input: ${usage.input_tokens} | Output: ${usage.output_tokens} | Total: ${usage.total_tokens}`}
    >
      <span>{formatTokenCount(usage.total_tokens)} tokens</span>
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
