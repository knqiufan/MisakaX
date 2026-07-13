import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface SettingsCardProps {
  title?: string;
  description?: string;
  children: ReactNode;
  className?: string;
  /** Use inset divide rows instead of padded body */
  divided?: boolean;
  /** Compact vertical padding for strip cards */
  compact?: boolean;
  onClick?: () => void;
  interactive?: boolean;
}

export function SettingsCard({
  title,
  description,
  children,
  className,
  divided = false,
  compact = false,
  onClick,
  interactive = false,
}: SettingsCardProps) {
  const clickable = interactive || Boolean(onClick);

  return (
    <div
      role={clickable ? "button" : undefined}
      tabIndex={clickable ? 0 : undefined}
      onClick={onClick}
      onKeyDown={
        clickable
          ? (event) => {
              if (event.key !== "Enter" && event.key !== " ") return;
              event.preventDefault();
              onClick?.();
            }
          : undefined
      }
      className={cn(
        "rounded-lg border border-border/50 bg-card",
        !divided && (compact ? "px-5 py-4" : "p-5"),
        (title || description) && !divided && "space-y-4",
        clickable &&
          "cursor-pointer transition-colors duration-[var(--ds-dur-fast)] ease-out hover:bg-muted/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        className
      )}
    >
      {(title || description) && (
        <div className={cn(divided && "px-5 pt-5", "space-y-1")}>
          {title ? (
            <h3 className="text-sm font-medium text-foreground">{title}</h3>
          ) : null}
          {description ? (
            <p className="text-xs text-muted-foreground">{description}</p>
          ) : null}
        </div>
      )}
      {divided ? (
        <div className="px-5 divide-y divide-border/50">{children}</div>
      ) : (
        children
      )}
    </div>
  );
}
