import { create } from "zustand";

export type SettingsTab = "general" | "models" | "mcp" | "appearance" | "about";

export type Route =
  | { page: "chat"; sessionId?: string }
  | { page: "skills" }
  | { page: "knowledge" }
  | { page: "dashboard" }
  | { page: "notifications" }
  | { page: "settings"; tab?: SettingsTab };

interface AppState {
  route: Route;
  sidebarCollapsed: boolean;
  globalLoading: boolean;

  navigate: (route: Route) => void;
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setGlobalLoading: (loading: boolean) => void;
}

export const useAppStore = create<AppState>((set) => ({
  route: { page: "chat" },
  sidebarCollapsed: false,
  globalLoading: false,

  navigate: (route) => set({ route }),
  toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
  setGlobalLoading: (loading) => set({ globalLoading: loading }),
}));
