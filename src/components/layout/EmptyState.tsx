import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";

interface EmptyStateProps {
  icon: LucideIcon;
  title: string;
  description?: string;
  className?: string;
}

export function EmptyState({
  icon: Icon,
  title,
  description,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        "flex h-full min-h-[min(420px,70vh)] flex-col items-center justify-center gap-4 px-8 py-12 text-center",
        className
      )}
    >
      <div className="flex size-16 items-center justify-center rounded-[var(--radius-ui-xl)] border border-[color:var(--border-subtle)] bg-[color:var(--surface-card)] shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]">
        <Icon className="size-8 text-muted-foreground/60" strokeWidth={1.15} />
      </div>
      <div className="max-w-md space-y-2">
        <h2 className="text-[1rem] font-semibold tracking-tight text-foreground">
          {title}
        </h2>
        {description ? (
          <p className="text-[0.8125rem] leading-relaxed text-muted-foreground">
            {description}
          </p>
        ) : null}
      </div>
    </div>
  );
}
