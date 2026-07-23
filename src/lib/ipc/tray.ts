import { invoke } from "./invoke";

export type CloseRequestAction = "minimize_to_tray" | "quit";

export interface TrayContext {
  workingDirectory: string | null;
  projectName: string | null;
  workspaceKind: string | null;
}

export const trayIpc = {
  updateContext: ({
    workingDirectory,
    projectName,
    workspaceKind,
  }: TrayContext) =>
    invoke<void>("update_tray_context", {
      workingDirectory,
      projectName,
      workspaceKind,
    }),

  resolveCloseRequest: (action: CloseRequestAction, remember: boolean) =>
    invoke<void>("resolve_close_request", { action, remember }),
};
