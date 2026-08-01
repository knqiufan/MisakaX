import type { ReactNode } from "react";
import { GitBranch, HardDrive } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useWorkspaceContext } from "@/hooks/use-workspace-context";
import { cn } from "@/lib/utils";

export interface WorkspaceContextAction {
  id: string;
  label: string;
  icon?: ReactNode;
  onInvoke: () => void;
}

interface WorkspaceContextBadgeProps {
  chatSessionId?: string | null;
  workingDirectory?: string | null;
  actions?: WorkspaceContextAction[];
}

export function WorkspaceContextBadge({
  chatSessionId,
  workingDirectory,
}: WorkspaceContextBadgeProps) {
  const { t } = useTranslation("workspace");
  const { context, status, errorCode } = useWorkspaceContext(
    chatSessionId,
    workingDirectory,
  );

  if (!chatSessionId || !workingDirectory) return null;
  if (status === "loading" && !context) {
    return (
      <span
        data-testid="workspace-context-badge"
        aria-label={t("context.loading")}
        className="inline-flex h-5 w-24 animate-pulse rounded-full bg-muted/70"
      />
    );
  }

  const isGit = context?.kind === "git";
  const fullLabel = isGit
    ? context.branch ?? `detached:${context.detached_head ?? "—"}`
    : t("context.localProject");
  const visibleLabel = isGit ? middleEllipsis(fullLabel, 28) : fullLabel;
  const diagnostic = context?.diagnostic;
  const tooltip = diagnostic || errorCode
    ? t("context.gitUnavailable", {
        id: diagnostic?.correlation_id.slice(0, 8) ?? errorCode ?? "unknown",
      })
    : isGit
      ? fullLabel
      : t("context.localHint");

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span
          data-testid="workspace-context-badge"
          className={cn(
            "inline-flex h-6 min-w-0 max-w-44 items-center gap-1.5 rounded-full px-2",
            "bg-muted/45 text-[11px] text-muted-foreground",
            "transition-colors duration-[var(--ds-dur-fast)]",
            status === "stale" && "opacity-70",
          )}
          aria-label={t("context.ariaLabel", { value: fullLabel })}
        >
          {isGit ? (
            <GitBranch className="size-3 shrink-0" aria-hidden />
          ) : (
            <HardDrive className="size-3 shrink-0" aria-hidden />
          )}
          <span className="truncate max-sm:hidden">{visibleLabel}</span>
        </span>
      </TooltipTrigger>
      <TooltipContent side="top" className="max-w-72 text-xs">
        {tooltip}
      </TooltipContent>
    </Tooltip>
  );
}

export function middleEllipsis(value: string, maxLength: number): string {
  if (value.length <= maxLength) return value;
  const visible = Math.max(2, maxLength - 1);
  const start = Math.ceil(visible / 2);
  const end = Math.floor(visible / 2);
  return `${value.slice(0, start)}…${value.slice(value.length - end)}`;
}
