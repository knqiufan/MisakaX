import type { WorkspacePanelMode } from "@/stores/workspace-panel-store";
import type { WorkspacePanelPresentation } from "./workspacePanelLayout";

export type WorkspacePanelShortcut = WorkspacePanelMode | "close" | null;

interface ShortcutEvent {
  key: string;
  code: string;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
  defaultPrevented: boolean;
}

export function resolveWorkspacePanelShortcut(
  event: ShortcutEvent,
  options: {
    terminalEnabled: boolean;
    panelOpen: boolean;
    presentation: WorkspacePanelPresentation;
  },
): WorkspacePanelShortcut {
  if (event.defaultPrevented) return null;
  const command = event.ctrlKey || event.metaKey;
  if (command && event.shiftKey && event.key.toLowerCase() === "e") {
    return "explorer";
  }
  if (
    command &&
    event.code === "Backquote" &&
    options.terminalEnabled
  ) {
    return "terminal";
  }
  if (
    event.key === "Escape" &&
    options.panelOpen &&
    options.presentation === "overlay"
  ) {
    return "close";
  }
  return null;
}
