import { create } from "zustand";
import type { Session } from "@/lib/ipc";

interface ChatState {
  sessions: Session[];
  activeSessionId: string | null;
  activeSession: Session | null;
  loading: boolean;
  showWorkspaceSelector: boolean;

  setActiveSession: (id: string | null) => void;
  setActiveSessionData: (session: Session | null) => void;
  setSessions: (sessions: Session[]) => void;
  setLoading: (loading: boolean) => void;
  setShowWorkspaceSelector: (show: boolean) => void;
  updateActiveSessionWorkingDir: (dir: string | null) => void;
}

export const useChatStore = create<ChatState>((set) => ({
  sessions: [],
  activeSessionId: null,
  activeSession: null,
  loading: false,
  showWorkspaceSelector: false,

  setActiveSession: (id) => set({ activeSessionId: id }),
  setActiveSessionData: (session) =>
    set({ activeSession: session, activeSessionId: session?.id ?? null }),
  setSessions: (sessions) => set({ sessions }),
  setLoading: (loading) => set({ loading }),
  setShowWorkspaceSelector: (show) => set({ showWorkspaceSelector: show }),
  updateActiveSessionWorkingDir: (dir) =>
    set((state) => ({
      activeSession: state.activeSession
        ? { ...state.activeSession, working_directory: dir }
        : null,
    })),
}));
