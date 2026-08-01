import { useEffect, useState, type ReactNode } from "react";
import { Terminal, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import type { WorkspacePanelMode } from "@/stores/workspace-panel-store";
import { WorkspaceExplorer } from "./WorkspaceExplorer";

interface WorkspacePanelProps {
  mode: WorkspacePanelMode;
  workingDir: string;
  terminalContent?: ReactNode;
  onClose: () => void;
}

/** Mode shell only: Explorer/Terminal domain state remains in their own stores. */
export function WorkspacePanel({
  mode,
  workingDir,
  terminalContent,
  onClose,
}: WorkspacePanelProps) {
  const [terminalMounted, setTerminalMounted] = useState(mode === "terminal");

  useEffect(() => {
    if (mode === "terminal") setTerminalMounted(true);
  }, [mode]);

  return (
    <div
      className="h-full min-h-0 min-w-0 bg-background"
      data-testid="workspace-panel"
      data-mode={mode}
    >
      <div className="h-full" hidden={mode !== "explorer"}>
        <WorkspaceExplorer workingDir={workingDir} />
      </div>
      {terminalMounted ? (
        <div className="h-full" hidden={mode !== "terminal"}>
          {terminalContent ?? <TerminalPreparationPanel onClose={onClose} />}
        </div>
      ) : null}
    </div>
  );
}

function TerminalPreparationPanel({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation("workspace");
  return (
    <aside className="flex h-full flex-col border-l border-border/40 bg-background">
      <div className="flex h-10 shrink-0 items-center justify-between px-3">
        <h2 className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
          <Terminal className="size-3.5" aria-hidden />
          {t("terminal.title")}
        </h2>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          onClick={onClose}
          className="size-7"
          aria-label={t("terminal.collapse")}
        >
          <X className="size-3.5" />
        </Button>
      </div>
      <div className="flex min-h-0 flex-1 items-center justify-center px-6 text-center">
        <p className="max-w-xs text-sm text-muted-foreground">
          {t("terminal.preparing")}
        </p>
      </div>
    </aside>
  );
}
