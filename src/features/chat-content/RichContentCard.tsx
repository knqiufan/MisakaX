import type { ReactNode } from "react";
import { LoaderCircle } from "lucide-react";
import { cn } from "@/lib/utils";

interface RichContentCardProps {
  title: string;
  meta?: ReactNode;
  icon: ReactNode;
  status?: "pending" | "failed" | "unsupported" | "ready";
  actions?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  className?: string;
}

export function RichContentCard({
  title,
  meta,
  icon,
  status = "ready",
  actions,
  children,
  footer,
  className,
}: RichContentCardProps) {
  return (
    <section
      className={cn(
        "my-4 min-w-0 overflow-hidden rounded-xl border border-border/40 bg-muted/20",
        "text-sm text-foreground",
        className
      )}
      aria-busy={status === "pending" || undefined}
    >
      <header className="flex min-h-10 items-center gap-2 border-b border-border/35 px-3 py-2">
        <span className="text-muted-foreground" aria-hidden>
          {icon}
        </span>
        <h3 className="min-w-0 flex-1 truncate text-sm font-medium">{title}</h3>
        {meta ? <span className="max-w-40 truncate text-xs text-muted-foreground">{meta}</span> : null}
        {status === "pending" ? (
          <LoaderCircle className="size-3.5 animate-spin text-muted-foreground" aria-hidden />
        ) : null}
        {actions ? <div className="flex shrink-0 items-center gap-1">{actions}</div> : null}
      </header>
      <div className={cn("min-w-0", status === "pending" && "min-h-40")}>{children}</div>
      {footer ? (
        <footer className="border-t border-border/35 px-3 py-2 text-xs text-muted-foreground">{footer}</footer>
      ) : null}
    </section>
  );
}
