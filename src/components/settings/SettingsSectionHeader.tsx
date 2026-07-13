import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface SettingsSectionHeaderProps {
  title: string;
  description?: string;
  action?: ReactNode;
  className?: string;
}

/** Page title rhythm: text-sm font-medium (docs/ui/04-settings.md §2.3). */
export function SettingsSectionHeader({
  title,
  description,
  action,
  className,
}: SettingsSectionHeaderProps) {
  return (
    <div
      className={cn(
        "flex items-start justify-between gap-4",
        className
      )}
    >
      <div className="min-w-0 space-y-1">
        <h2 className="text-sm font-medium text-foreground">{title}</h2>
        {description ? (
          <p className="text-xs text-muted-foreground">{description}</p>
        ) : null}
      </div>
      {action ? <div className="shrink-0">{action}</div> : null}
    </div>
  );
}

interface SettingsPartitionLabelProps {
  children: ReactNode;
  className?: string;
}

/** Uppercase partition label inside a page. */
export function SettingsPartitionLabel({
  children,
  className,
}: SettingsPartitionLabelProps) {
  return (
    <h3
      className={cn(
        "text-[11px] font-medium uppercase tracking-wider text-muted-foreground",
        className
      )}
    >
      {children}
    </h3>
  );
}
