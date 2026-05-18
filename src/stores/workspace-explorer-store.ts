import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface OpenTab {
  path: string;
  originalContent: string;
  currentContent: string;
  dirty: boolean;
}

interface WorkspaceExplorerState {
  open: boolean;
  tabs: OpenTab[];
  activePath: string | null;
  setOpen: (open: boolean) => void;
  openTab: (path: string, content: string) => void;
  updateContent: (path: string, content: string) => void;
  markSaved: (path: string) => void;
  closeTab: (path: string) => void;
  setActive: (path: string | null) => void;
}

export const useWorkspaceExplorerStore = create<WorkspaceExplorerState>()(
  persist(
    (set) => ({
      open: false,
      tabs: [],
      activePath: null,
      setOpen: (open) => set({ open }),
      openTab: (path, content) =>
        set((state) => {
          if (state.tabs.some((tab) => tab.path === path)) {
            return { activePath: path };
          }
          return {
            tabs: [
              ...state.tabs,
              { path, originalContent: content, currentContent: content, dirty: false },
            ],
            activePath: path,
          };
        }),
      updateContent: (path, content) =>
        set((state) => ({
          tabs: state.tabs.map((tab) =>
            tab.path === path
              ? { ...tab, currentContent: content, dirty: content !== tab.originalContent }
              : tab
          ),
        })),
      markSaved: (path) =>
        set((state) => ({
          tabs: state.tabs.map((tab) =>
            tab.path === path
              ? { ...tab, originalContent: tab.currentContent, dirty: false }
              : tab
          ),
        })),
      closeTab: (path) =>
        set((state) => {
          const tabs = state.tabs.filter((tab) => tab.path !== path);
          const activePath =
            state.activePath === path ? tabs[tabs.length - 1]?.path ?? null : state.activePath;
          return { tabs, activePath };
        }),
      setActive: (path) => set({ activePath: path }),
    }),
    {
      name: "misakax:workspace-explorer",
      partialize: (state) => ({ open: state.open }),
    }
  )
);
