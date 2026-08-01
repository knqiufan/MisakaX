import { create } from "zustand";
import { persist } from "zustand/middleware";

export type WorkspacePanelMode = "explorer" | "terminal";

export const WORKSPACE_PANEL_STORAGE_KEY = "misakax:workspace-panel";
export const LEGACY_WORKSPACE_EXPLORER_STORAGE_KEY =
  "misakax:workspace-explorer";
export const WORKSPACE_PANEL_STORAGE_VERSION = 1;
export const WORKSPACE_PANEL_DEFAULT_SIZE = 30;
export const WORKSPACE_PANEL_MIN_SIZE = 18;
export const WORKSPACE_PANEL_MAX_SIZE = 55;

interface PersistedWorkspacePanelState {
  open: boolean;
  mode: WorkspacePanelMode;
  size: number;
}

export interface WorkspacePanelState extends PersistedWorkspacePanelState {
  sessionId: string | null;
  workspaceGeneration: number;
  bindSession: (sessionId: string | null, workspaceGeneration: number) => void;
  toggle: (mode: WorkspacePanelMode) => void;
  close: () => void;
  setSize: (size: number) => void;
}

function clampSize(size: number): number {
  if (!Number.isFinite(size)) return WORKSPACE_PANEL_DEFAULT_SIZE;
  return Math.min(
    WORKSPACE_PANEL_MAX_SIZE,
    Math.max(WORKSPACE_PANEL_MIN_SIZE, Math.round(size * 100) / 100),
  );
}

function isPanelMode(value: unknown): value is WorkspacePanelMode {
  return value === "explorer" || value === "terminal";
}

export function normalizeWorkspacePanelState(
  value: unknown,
): PersistedWorkspacePanelState {
  const candidate =
    value && typeof value === "object"
      ? (value as Partial<PersistedWorkspacePanelState>)
      : {};
  return {
    open: candidate.open === true,
    mode: isPanelMode(candidate.mode) ? candidate.mode : "explorer",
    size: clampSize(
      typeof candidate.size === "number"
        ? candidate.size
        : WORKSPACE_PANEL_DEFAULT_SIZE,
    ),
  };
}

/** One-time W2 migration from the Explorer-owned `open` preference. */
export function migrateLegacyWorkspacePanelStorage(
  storage: Pick<Storage, "getItem" | "setItem" | "removeItem">,
): boolean {
  if (storage.getItem(WORKSPACE_PANEL_STORAGE_KEY) !== null) return false;
  const legacyRaw = storage.getItem(LEGACY_WORKSPACE_EXPLORER_STORAGE_KEY);
  if (!legacyRaw) return false;

  try {
    const legacy = JSON.parse(legacyRaw) as { state?: { open?: unknown } };
    const state = normalizeWorkspacePanelState({
      open: legacy.state?.open === true,
      mode: "explorer",
      size: WORKSPACE_PANEL_DEFAULT_SIZE,
    });
    storage.setItem(
      WORKSPACE_PANEL_STORAGE_KEY,
      JSON.stringify({ state, version: WORKSPACE_PANEL_STORAGE_VERSION }),
    );
    storage.removeItem(LEGACY_WORKSPACE_EXPLORER_STORAGE_KEY);
    return true;
  } catch {
    return false;
  }
}

if (typeof window !== "undefined") {
  migrateLegacyWorkspacePanelStorage(window.localStorage);
}

export const useWorkspacePanelStore = create<WorkspacePanelState>()(
  persist(
    (set) => ({
      open: false,
      mode: "explorer",
      size: WORKSPACE_PANEL_DEFAULT_SIZE,
      sessionId: null,
      workspaceGeneration: 0,
      bindSession: (sessionId, workspaceGeneration) =>
        set((state) => {
          if (
            state.sessionId === sessionId &&
            workspaceGeneration !== 0 &&
            workspaceGeneration < state.workspaceGeneration
          ) {
            return state;
          }
          return {
            sessionId,
            workspaceGeneration: Math.max(0, workspaceGeneration),
          };
        }),
      toggle: (mode) =>
        set((state) => ({
          open: state.mode === mode ? !state.open : true,
          mode,
        })),
      close: () => set({ open: false }),
      setSize: (size) => set({ size: clampSize(size) }),
    }),
    {
      name: WORKSPACE_PANEL_STORAGE_KEY,
      version: WORKSPACE_PANEL_STORAGE_VERSION,
      partialize: ({ open, mode, size }) => ({ open, mode, size }),
      migrate: (persisted) => normalizeWorkspacePanelState(persisted),
      merge: (persisted, current) => ({
        ...current,
        ...normalizeWorkspacePanelState(persisted),
      }),
    },
  ),
);
