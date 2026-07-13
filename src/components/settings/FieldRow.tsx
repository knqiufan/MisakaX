import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface FieldRowProps {
  label: string;
  description?: string;
  children: ReactNode;
  className?: string;
  /** Top separator when stacking rows inside one card */
  separator?: boolean;
}

/** Label + description left, control right (docs/ui/04-settings.md §4). */
export function FieldRow({
  label,
  description,
  children,
  className,
  separator = false,
}: FieldRowProps) {
  return (
    <div
      className={cn(
        "flex items-center justify-between gap-4",
        separator && "border-t border-border/50 pt-4",
        className
      )}
    >
      <div className="min-w-0 flex-1 space-y-0.5">
        <div className="text-sm font-medium text-foreground">{label}</div>
        {description ? (
          <div className="text-xs text-muted-foreground">{description}</div>
        ) : null}
      </div>
      <div className="flex shrink-0 items-center">{children}</div>
    </div>
  );
}
