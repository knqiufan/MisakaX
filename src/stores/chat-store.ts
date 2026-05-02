import { create } from "zustand";
import type { Session } from "@/lib/ipc";

interface ChatState {
  sessions: Session[];
  activeSessionId: string | null;
  loading: boolean;

  setActiveSession: (id: string | null) => void;
  setSessions: (sessions: Session[]) => void;
  setLoading: (loading: boolean) => void;
}

export const useChatStore = create<ChatState>((set) => ({
  sessions: [],
  activeSessionId: null,
  loading: false,

  setActiveSession: (id) => set({ activeSessionId: id }),
  setSessions: (sessions) => set({ sessions }),
  setLoading: (loading) => set({ loading }),
}));
