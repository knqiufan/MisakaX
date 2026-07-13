import { useTranslation } from "react-i18next";
import {
  Activity,
  AlertTriangle,
  Loader2,
  Power,
  RefreshCw,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useSidecarStatus } from "@/hooks/use-sidecar-status";
import type { SidecarStatus } from "@/lib/ipc/sidecar";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

const STATUS_CONFIG: Record<
  SidecarStatus,
  {
    colorClass: string;
    dotClass: string;
    Icon: typeof Activity;
    spinning?: boolean;
  }
> = {
  ready: {
    colorClass: "text-emerald-400",
    dotClass: "bg-emerald-400",
    Icon: Activity,
  },
  starting: {
    colorClass: "text-amber-400",
    dotClass: "bg-amber-400",
    Icon: Loader2,
    spinning: true,
  },
  restarting: {
    colorClass: "text-amber-400",
    dotClass: "bg-amber-400",
    Icon: RefreshCw,
    spinning: true,
  },
  error: {
    colorClass: "text-destructive",
    dotClass: "bg-destructive",
    Icon: AlertTriangle,
  },
  stopped: {
    colorClass: "text-muted-foreground/50",
    dotClass: "bg-muted-foreground/50",
    Icon: Power,
  },
};

interface SidecarStatusBadgeProps {
  className?: string;
  /** Show status text beside the indicator (e.g. About system info). */
  showLabel?: boolean;
}

export function SidecarStatusBadge({
  className,
  showLabel = false,
}: SidecarStatusBadgeProps) {
  const { status, message, restartSidecar } = useSidecarStatus();
  const { t } = useTranslation("common");

  const cfg = STATUS_CONFIG[status];
  const label = t(`sidecar.status.${status}`);
  const canRestart = status === "error";

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type="button"
          onClick={canRestart ? restartSidecar : undefined}
          className={cn(
            "inline-flex items-center gap-1.5 rounded-[var(--radius-ui-sm)] px-1.5 py-1",
            "text-xs transition-[background,color] duration-[var(--ds-dur-fast)] ease-out",
            "hover:bg-[color:var(--surface-hover)]",
            canRestart ? "cursor-pointer" : "cursor-default",
            className
          )}
          aria-label={label}
        >
          <span
            className={cn(
              "inline-block size-1.5 shrink-0 rounded-full",
              cfg.dotClass
            )}
          />
          <cfg.Icon
            className={cn(
              "size-3 shrink-0",
              cfg.colorClass,
              cfg.spinning && "animate-spin"
            )}
          />
          {showLabel ? (
            <span className="max-w-[12rem] truncate text-[11px] text-muted-foreground">
              {label}
            </span>
          ) : null}
        </button>
      </TooltipTrigger>
      <TooltipContent side="bottom" sideOffset={8}>
        <p className="font-medium">{label}</p>
        {message ? (
          <p className="mt-0.5 text-muted-foreground">{message}</p>
        ) : null}
        {canRestart ? (
          <p className="mt-1 text-xs text-muted-foreground">
            {t("sidecar.clickToRestart")}
          </p>
        ) : null}
      </TooltipContent>
    </Tooltip>
  );
}
