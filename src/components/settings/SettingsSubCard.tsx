import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface SettingsSubCardProps {
  children: ReactNode;
  className?: string;
}

/** Nested inset card with padded divide rows (docs/ui/04-settings.md §3.2). */
export function SettingsSubCard({ children, className }: SettingsSubCardProps) {
  return (
    <div className={cn("rounded-md bg-muted/40", className)}>
      <div className="divide-y divide-border/50 px-3.5">{children}</div>
    </div>
  );
}

interface SettingsSubRowProps {
  children: ReactNode;
  className?: string;
}

export function SettingsSubRow({ children, className }: SettingsSubRowProps) {
  return (
    <div
      className={cn(
        "flex items-center justify-between gap-4 py-2.5",
        className
      )}
    >
      {children}
    </div>
  );
}
