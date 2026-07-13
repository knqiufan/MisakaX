import { create } from "zustand";

export type SettingsTab = "general" | "models" | "mcp" | "appearance" | "about";

export type Route =
  | { page: "chat"; sessionId?: string }
  | { page: "skills" }
  | { page: "knowledge" }
  | { page: "dashboard" }
  | { page: "notifications" }
  | { page: "settings"; tab?: SettingsTab };

export const SESSION_LIST_DEFAULT_WIDTH = 240;
export const SESSION_LIST_MIN_WIDTH = 180;
export const SESSION_LIST_MAX_WIDTH = 300;
export const SESSION_LIST_WIDTH_KEY = "misakax_chatlist_width";
export const LG_BREAKPOINT = 1024;

interface AppState {
  route: Route;
  sidebarCollapsed: boolean;
  sessionListOpen: boolean;
  sessionListWidth: number;
  globalLoading: boolean;

  navigate: (route: Route) => void;
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  toggleSessionList: () => void;
  setSessionListOpen: (open: boolean) => void;
  setSessionListWidth: (width: number) => void;
  setGlobalLoading: (loading: boolean) => void;
}

function loadSessionListWidth(): number {
  if (typeof window === "undefined") return SESSION_LIST_DEFAULT_WIDTH;
  const raw = localStorage.getItem(SESSION_LIST_WIDTH_KEY);
  if (!raw) return SESSION_LIST_DEFAULT_WIDTH;
  const parsed = Number.parseInt(raw, 10);
  if (Number.isNaN(parsed)) return SESSION_LIST_DEFAULT_WIDTH;
  return Math.min(
    SESSION_LIST_MAX_WIDTH,
    Math.max(SESSION_LIST_MIN_WIDTH, parsed)
  );
}

export const useAppStore = create<AppState>((set) => ({
  route: { page: "chat" },
  sidebarCollapsed: false,
  sessionListOpen: true,
  sessionListWidth: loadSessionListWidth(),
  globalLoading: false,

  navigate: (route) => set({ route }),
  toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
  toggleSessionList: () =>
    set((state) => ({ sessionListOpen: !state.sessionListOpen })),
  setSessionListOpen: (open) => set({ sessionListOpen: open }),
  setSessionListWidth: (width) => {
    const clamped = Math.min(
      SESSION_LIST_MAX_WIDTH,
      Math.max(SESSION_LIST_MIN_WIDTH, Math.round(width))
    );
    localStorage.setItem(SESSION_LIST_WIDTH_KEY, String(clamped));
    set({ sessionListWidth: clamped });
  },
  setGlobalLoading: (loading) => set({ globalLoading: loading }),
}));
