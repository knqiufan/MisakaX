import { useTranslation } from "react-i18next";
import { FolderOpen, FolderTree, FolderX, RefreshCw, Terminal } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import type { WorkspacePanelMode } from "@/stores/workspace-panel-store";

interface WorkspaceBarProps {
  workingDir: string | null;
  workspaceKind?: string | null;
  onChangeDir: () => void;
  onTogglePanel?: (mode: WorkspacePanelMode) => void;
  panelOpen?: boolean;
  panelMode?: WorkspacePanelMode;
  terminalEnabled?: boolean;
}

const barShellClass = cn(
  "flex min-h-10 shrink-0 items-center gap-3 border-b border-border/40",
  "bg-background px-3 py-1.5"
);

export function WorkspaceBar({
  workingDir,
  workspaceKind,
  onChangeDir,
  onTogglePanel,
  panelOpen = false,
  panelMode = "explorer",
  terminalEnabled = false,
}: WorkspaceBarProps) {
  const { t } = useTranslation("workspace");

  if (!workingDir) {
    return <WorkspaceBarUnset t={t} onSelectDir={onChangeDir} />;
  }

  return (
    <WorkspaceBarSet
      t={t}
      workingDir={workingDir}
      workspaceKind={workspaceKind}
      onChangeDir={onChangeDir}
      onTogglePanel={onTogglePanel}
      panelOpen={panelOpen}
      panelMode={panelMode}
      terminalEnabled={terminalEnabled}
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
      <FolderX
        className="size-4 shrink-0 text-muted-foreground"
        aria-hidden
      />
      <div className="min-w-0 flex-1 space-y-0.5">
        <p className="text-sm font-medium leading-tight text-foreground">
          {t("noWorkingDir")}
        </p>
        <p className="truncate text-xs text-muted-foreground">
          {t("description")}
        </p>
      </div>
      <Button
        type="button"
        size="sm"
        className="h-7 shrink-0 px-3 text-xs"
        onClick={onSelectDir}
      >
        {t("selectDir")}
      </Button>
    </div>
  );
}

function WorkspaceBarSet({
  t,
  workingDir,
  workspaceKind,
  onChangeDir,
  onTogglePanel,
  panelOpen,
  panelMode,
  terminalEnabled,
}: {
  t: (k: string) => string;
  workingDir: string;
  workspaceKind?: string | null;
  onChangeDir: () => void;
  onTogglePanel?: (mode: WorkspacePanelMode) => void;
  panelOpen: boolean;
  panelMode: WorkspacePanelMode;
  terminalEnabled: boolean;
}) {
  const isDefault = workspaceKind === "default";
  const displayName = isDefault
    ? t("defaultWorkspaceName")
    : extractDirName(workingDir);

  return (
    <div className={barShellClass}>
      <FolderOpen
        className="size-4 shrink-0 text-muted-foreground"
        aria-hidden
      />
      <div className="min-w-0 flex-1 space-y-0">
        <p className="truncate text-sm font-medium leading-tight text-foreground">
          {displayName}
        </p>
        <Tooltip>
          <TooltipTrigger asChild>
            <p className="max-w-full cursor-default truncate font-mono text-[10px] text-muted-foreground/60">
              {workingDir}
            </p>
          </TooltipTrigger>
          <TooltipContent side="bottom" className="max-w-md text-xs">
            {workingDir}
          </TooltipContent>
        </Tooltip>
      </div>
      <BarIconButton
        label={t("switchDir")}
        onClick={onChangeDir}
        icon={RefreshCw}
      />
      <BarIconButton
        label={t("openExplorer")}
        tooltip={t("openExplorerShortcut")}
        onClick={() => onTogglePanel?.("explorer")}
        disabled={!onTogglePanel}
        active={panelOpen && panelMode === "explorer"}
        icon={FolderTree}
        strokeWidth={panelOpen && panelMode === "explorer" ? 2 : undefined}
        ariaKeyShortcuts="Control+Shift+E Meta+Shift+E"
      />
      {terminalEnabled ? (
        <BarIconButton
          label={t("openTerminal")}
          tooltip={t("openTerminalShortcut")}
          onClick={() => onTogglePanel?.("terminal")}
          disabled={!onTogglePanel}
          active={panelOpen && panelMode === "terminal"}
          icon={Terminal}
          strokeWidth={panelOpen && panelMode === "terminal" ? 2 : undefined}
          ariaKeyShortcuts="Control+Backquote Meta+Backquote"
        />
      ) : null}
    </div>
  );
}

function BarIconButton({
  label,
  tooltip = label,
  onClick,
  disabled,
  active,
  icon: Icon,
  strokeWidth,
  ariaKeyShortcuts,
}: {
  label: string;
  tooltip?: string;
  onClick?: () => void;
  disabled?: boolean;
  active?: boolean;
  icon: typeof FolderTree;
  strokeWidth?: number;
  ariaKeyShortcuts?: string;
}) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          type="button"
          variant={active ? "secondary" : "ghost"}
          size="icon"
          onClick={onClick}
          disabled={disabled}
          className="size-7 shrink-0"
          aria-label={label}
          aria-pressed={active}
          aria-keyshortcuts={ariaKeyShortcuts}
        >
          <Icon className="size-3.5" strokeWidth={strokeWidth} />
        </Button>
      </TooltipTrigger>
      <TooltipContent side="bottom" className="text-xs">
        {tooltip}
      </TooltipContent>
    </Tooltip>
  );
}

export function extractDirName(path: string): string {
  const normalized = path.replace(/\\/g, "/").replace(/\/+$/, "");
  const parts = normalized.split("/");
  return parts[parts.length - 1] || path;
}
