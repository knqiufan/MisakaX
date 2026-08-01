import { useEffect, useState, type ReactNode } from "react";
import type { WorkspacePanelMode } from "@/stores/workspace-panel-store";
import { TerminalPanel } from "./TerminalPanel";
import { WorkspaceExplorer } from "./WorkspaceExplorer";

interface WorkspacePanelProps {
  mode: WorkspacePanelMode;
  chatSessionId: string;
  workspaceGeneration: number;
  workingDir: string;
  terminalContent?: ReactNode;
  onClose: () => void;
}

/** Mode shell only: Explorer/Terminal domain state remains in their own stores. */
export function WorkspacePanel({
  mode,
  chatSessionId,
  workspaceGeneration,
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
          {terminalContent ?? (
            <TerminalPanel
              active={mode === "terminal"}
              chatSessionId={chatSessionId}
              workspaceGeneration={workspaceGeneration}
              workingDir={workingDir}
              onClose={onClose}
            />
          )}
        </div>
      ) : null}
    </div>
  );
}
