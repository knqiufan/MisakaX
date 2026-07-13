import { cn } from "@/lib/utils";

export type StatusPillTone =
  | "available"
  | "needs-config"
  | "error"
  | "unknown";

const TONE_CLASS: Record<StatusPillTone, { shell: string; dot: string }> = {
  available: {
    shell: "bg-emerald-500/10 text-emerald-700 dark:text-emerald-400",
    dot: "bg-emerald-600 dark:bg-emerald-400",
  },
  "needs-config": {
    shell: "bg-amber-500/10 text-amber-800 dark:text-amber-300",
    dot: "bg-amber-600 dark:bg-amber-400",
  },
  error: {
    shell: "bg-destructive/10 text-destructive",
    dot: "bg-destructive",
  },
  unknown: {
    shell: "bg-muted text-muted-foreground",
    dot: "bg-muted-foreground",
  },
};

interface StatusPillProps {
  label: string;
  tone: StatusPillTone;
  className?: string;
}

/** Provider runtime status pill (docs/ui/04-settings.md §5.1). */
export function StatusPill({ label, tone, className }: StatusPillProps) {
  const colors = TONE_CLASS[tone];
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium",
        colors.shell,
        className
      )}
    >
      <span className={cn("size-1.5 shrink-0 rounded-full", colors.dot)} />
      {label}
    </span>
  );
}
