export const WORKSPACE_PANEL_LAYOUT = {
  chatDefaultSize: "70%",
  chatMinSize: "45%",
  panelDefaultSize: 30,
  panelMinSize: 18,
  panelMaxSize: 55,
} as const;

export const WORKSPACE_PANEL_OVERLAY_BREAKPOINT = 960;

export type WorkspacePanelPresentation = "split" | "overlay";

export function resolveWorkspacePanelPresentation(
  viewportWidth: number,
): WorkspacePanelPresentation {
  return viewportWidth <= WORKSPACE_PANEL_OVERLAY_BREAKPOINT
    ? "overlay"
    : "split";
}
