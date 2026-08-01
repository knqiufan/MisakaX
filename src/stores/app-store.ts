import { create } from "zustand";

export type SettingsTab =
  | "general"
  | "models"
  | "mcp"
  | "skills"
  | "appearance"
  | "about";

export type Route =
  | { page: "chat"; sessionId?: string }
  | { page: "skills" }
  | { page: "knowledge" }
  | { page: "dashboard" }
  | { page: "notifications" }
  | { page: "settings"; tab?: SettingsTab };

export function normalizeRoute(route: Route): Route {
  return route.page === "skills"
    ? { page: "settings", tab: "skills" }
    : route;
}

export const SESSION_LIST_DEFAULT_WIDTH = 240;
export const SESSION_LIST_MIN_WIDTH = 180;
export const SESSION_LIST_MAX_WIDTH = 300;
export const SESSION_LIST_WIDTH_KEY = "misakax_chatlist_width";
export const LG_BREAKPOINT = 1024;

interface AppState {
  route: Route;
  sessionListWidth: number;
  globalLoading: boolean;

  navigate: (route: Route) => void;
  setSessionListWidth: (width: number) => void;
  setGlobalLoading: (loading: boolean) => void;
}

function clampSessionListWidth(width: number): number {
  return Math.min(
    SESSION_LIST_MAX_WIDTH,
    Math.max(SESSION_LIST_MIN_WIDTH, Math.round(width))
  );
}

function loadSessionListWidth(): number {
  if (typeof window === "undefined") return SESSION_LIST_DEFAULT_WIDTH;
  const raw = localStorage.getItem(SESSION_LIST_WIDTH_KEY);
  if (!raw) return SESSION_LIST_DEFAULT_WIDTH;
  const parsed = Number.parseInt(raw, 10);
  if (Number.isNaN(parsed)) return SESSION_LIST_DEFAULT_WIDTH;
  return clampSessionListWidth(parsed);
}

/** Display width: on narrow viewports shrink toward min without rewriting preference. */
export function resolveLeftColumnWidth(
  preferredWidth: number,
  viewportWidth: number
): number {
  if (viewportWidth >= LG_BREAKPOINT) return preferredWidth;
  return Math.min(preferredWidth, SESSION_LIST_MIN_WIDTH);
}

export const useAppStore = create<AppState>((set) => ({
  route: { page: "chat" },
  sessionListWidth: loadSessionListWidth(),
  globalLoading: false,

  navigate: (route) => set({ route: normalizeRoute(route) }),
  setSessionListWidth: (width) => {
    const clamped = clampSessionListWidth(width);
    localStorage.setItem(SESSION_LIST_WIDTH_KEY, String(clamped));
    set({ sessionListWidth: clamped });
  },
  setGlobalLoading: (loading) => set({ globalLoading: loading }),
}));
