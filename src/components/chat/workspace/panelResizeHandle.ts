/** Shared Chat↔Explorer / tree↔editor resize handle chrome (shell-spec §1.3 / §5). */
export const PANEL_RESIZE_HANDLE_CLASS =
  "group relative w-2 shrink-0 bg-transparent " +
  "transition-colors duration-[var(--ds-dur-fast)]";

export const PANEL_RESIZE_HANDLE_LINE_CLASS =
  "absolute inset-y-0 left-1/2 w-px -translate-x-1/2 bg-transparent " +
  "transition-colors duration-[var(--ds-dur-fast)] " +
  "group-hover:bg-border group-active:bg-border";
