/**
 * Shared enter/exit classes for floating overlays (menus, selects, tooltips).
 * Fade + ≤4px slide only — never zoom (see frontend-ui-guidelines §2).
 */
export const OVERLAY_MOTION =
  "duration-150 ease-out data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=closed]:animate-out data-[state=closed]:fade-out-0";

/** Side-aware micro-slide (Radix `data-side`). Use with OVERLAY_MOTION. */
export const OVERLAY_SIDE_SLIDE =
  "data-[side=bottom]:slide-in-from-top-1 data-[side=left]:slide-in-from-right-1 data-[side=right]:slide-in-from-left-1 data-[side=top]:slide-in-from-bottom-1";

/** Dialog overlay / card — slightly longer than menus. */
export const DIALOG_MOTION =
  "duration-[var(--ds-dur-entrance)] ease-[var(--ds-ease-out)] data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=closed]:animate-out data-[state=closed]:fade-out-0";
