import { create } from "zustand";
import type { Session } from "@/lib/ipc";
import { sessionsIpc } from "@/lib/ipc";

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
  refreshSessions: () => Promise<void>;
  requestAutoTitle: (sessionId: string, firstMessage: string) => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  sessions: [],
  activeSessionId: null,
  activeSession: null,
  loading: false,
  showWorkspaceSelector: false,

  setActiveSession: (id) => set({ activeSessionId: id }),
  setActiveSessionData: (session) => {
    set({ activeSession: session, activeSessionId: session?.id ?? null });
    get().refreshSessions();
  },
  setSessions: (sessions) => set({ sessions }),
  setLoading: (loading) => set({ loading }),
  setShowWorkspaceSelector: (show) => set({ showWorkspaceSelector: show }),
  updateActiveSessionWorkingDir: (dir) =>
    set((state) => ({
      activeSession: state.activeSession
        ? { ...state.activeSession, working_directory: dir }
        : null,
    })),

  refreshSessions: async () => {
    try {
      const list = await sessionsIpc.list();
      set({ sessions: list });
    } catch (err) {
      console.error("Failed to refresh sessions:", err);
    }
  },

  requestAutoTitle: (sessionId: string, firstMessage: string) => {
    const truncated = firstMessage.slice(0, 100);
    const title =
      truncated.length <= 20
        ? truncated
        : truncated.slice(0, 20).replace(/\s+\S*$/, "…");

    sessionsIpc
      .update({ id: sessionId, title })
      .then(() => get().refreshSessions())
      .catch((err) => console.error("Auto-title failed:", err));
  },
}));
