import { useTranslation } from "react-i18next";
import { FolderOpen, FolderTree, FolderX, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

interface WorkspaceBarProps {
  workingDir: string | null;
  onChangeDir: () => void;
  onToggleExplorer?: () => void;
  explorerOpen?: boolean;
}

const barShellClass = cn(
  "flex min-h-[52px] shrink-0 items-center gap-4 border-b border-[color:var(--border-muted)]",
  "bg-[color:var(--surface-topbar)] px-5 py-3 backdrop-blur-[2px]"
);

export function WorkspaceBar({
  workingDir,
  onChangeDir,
  onToggleExplorer,
  explorerOpen = false,
}: WorkspaceBarProps) {
  const { t } = useTranslation("workspace");

  if (!workingDir) {
    return <WorkspaceBarUnset t={t} onSelectDir={onChangeDir} />;
  }

  return (
    <WorkspaceBarSet
      t={t}
      workingDir={workingDir}
      onChangeDir={onChangeDir}
      onToggleExplorer={onToggleExplorer}
      explorerOpen={explorerOpen}
    />
  );
}

function WorkspaceBarUnset({
  t,
  onSelectDir,
}: {
  t: (k: string) => string;
  onSelectDir: () => void;
}) {
  return (
    <div className={barShellClass}>
      <div
        className={cn(
          "flex size-11 shrink-0 items-center justify-center rounded-[var(--radius-ui-md)]",
          "border border-[color:var(--border-muted)] bg-[color:var(--surface-card-strong)]",
          "text-muted-foreground"
        )}
      >
        <FolderX className="h-5 w-5" aria-hidden />
      </div>
      <div className="min-w-0 flex-1 space-y-1">
        <p className="text-sm font-semibold leading-tight text-foreground">{t("noWorkingDir")}</p>
        <p className="line-clamp-2 text-xs leading-relaxed text-muted-foreground">{t("description")}</p>
      </div>
      <Button type="button" size="sm" className="shrink-0 px-4" onClick={onSelectDir}>
        {t("selectDir")}
      </Button>
    </div>
  );
}

function WorkspaceBarSet({
  t,
  workingDir,
  onChangeDir,
  onToggleExplorer,
  explorerOpen,
}: {
  t: (k: string) => string;
  workingDir: string;
  onChangeDir: () => void;
  onToggleExplorer?: () => void;
  explorerOpen: boolean;
}) {
  const dirName = extractDirName(workingDir);

  return (
    <div className={cn(barShellClass, "gap-3")}>
      <div
        className={cn(
          "flex size-11 shrink-0 items-center justify-center rounded-[var(--radius-ui-md)]",
          "border border-[color:var(--border-muted)]/80 bg-[color:var(--surface-card)]",
          "text-primary/85"
        )}
      >
        <FolderOpen className="h-5 w-5" aria-hidden />
      </div>
      <div className="min-w-0 flex-1 space-y-0.5">
        <p className="text-sm font-semibold leading-tight text-foreground">{dirName}</p>
        <Tooltip>
          <TooltipTrigger asChild>
            <p className="max-w-full cursor-default truncate text-xs text-muted-foreground">
              {workingDir}
            </p>
          </TooltipTrigger>
          <TooltipContent side="bottom" className="max-w-md text-xs">
            {workingDir}
          </TooltipContent>
        </Tooltip>
      </div>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            type="button"
            variant="outline"
            size="icon"
            onClick={onChangeDir}
            className="size-9 shrink-0 rounded-[var(--radius-ui-md)] border-[color:var(--border-muted)]"
            aria-label={t("switchDir")}
          >
            <RefreshCw className="h-4 w-4 text-muted-foreground" />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom" className="text-xs">
          {t("switchDir")}
        </TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            type="button"
            variant="outline"
            size="icon"
            onClick={onToggleExplorer}
            disabled={!onToggleExplorer || explorerOpen}
            className="size-9 shrink-0 rounded-[var(--radius-ui-md)] border-[color:var(--border-muted)]"
            aria-label={t("openExplorer")}
          >
            <FolderTree className="h-4 w-4 text-muted-foreground" />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom" className="text-xs">
          {t("openExplorer")}
        </TooltipContent>
      </Tooltip>
    </div>
  );
}

export function extractDirName(path: string): string {
  const normalized = path.replace(/\\/g, "/").replace(/\/+$/, "");
  const parts = normalized.split("/");
  return parts[parts.length - 1] || path;
}
