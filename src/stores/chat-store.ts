import { create } from "zustand";
import type { Message, Session } from "@/lib/ipc";
import { chatIpc, sessionsIpc } from "@/lib/ipc";

interface ChatState {
  sessions: Session[];
  activeSessionId: string | null;
  activeSession: Session | null;
  loading: boolean;
  showWorkspaceSelector: boolean;

  messages: Message[];
  isStreaming: boolean;
  streamingMessageId: string | null;
  selectedModel: string | null;

  setActiveSession: (id: string | null) => void;
  setActiveSessionData: (session: Session | null) => void;
  setSessions: (sessions: Session[]) => void;
  setLoading: (loading: boolean) => void;
  setShowWorkspaceSelector: (show: boolean) => void;
  updateActiveSessionWorkingDir: (dir: string | null) => void;
  refreshSessions: () => Promise<void>;
  requestAutoTitle: (sessionId: string, firstMessage: string) => void;

  setMessages: (messages: Message[]) => void;
  addMessage: (message: Message) => void;
  updateMessageContent: (messageId: string, delta: string) => void;
  setMessageStatus: (messageId: string, status: Message["status"]) => void;
  setStreaming: (streaming: boolean, messageId?: string | null) => void;
  setSelectedModel: (model: string | null) => void;
  clearMessages: () => void;
  loadMessages: (sessionId: string) => Promise<void>;
}

export const useChatStore = create<ChatState>((set, get) => ({
  sessions: [],
  activeSessionId: null,
  activeSession: null,
  loading: false,
  showWorkspaceSelector: false,

  messages: [],
  isStreaming: false,
  streamingMessageId: null,
  selectedModel: null,

  setActiveSession: (id) => set({ activeSessionId: id }),
  setActiveSessionData: (session) => {
    set({
      activeSession: session,
      activeSessionId: session?.id ?? null,
      messages: [],
      isStreaming: false,
      streamingMessageId: null,
    });
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

  setMessages: (messages) => set({ messages }),

  addMessage: (message) =>
    set((state) => ({ messages: [...state.messages, message] })),

  updateMessageContent: (messageId, delta) =>
    set((state) => ({
      messages: state.messages.map((m) =>
        m.id === messageId ? { ...m, content: m.content + delta } : m
      ),
    })),

  setMessageStatus: (messageId, status) =>
    set((state) => ({
      messages: state.messages.map((m) =>
        m.id === messageId ? { ...m, status } : m
      ),
    })),

  setStreaming: (streaming, messageId = null) =>
    set({ isStreaming: streaming, streamingMessageId: messageId }),

  setSelectedModel: (model) => set({ selectedModel: model }),

  clearMessages: () =>
    set({ messages: [], isStreaming: false, streamingMessageId: null }),

  loadMessages: async (sessionId: string) => {
    try {
      set({ loading: true });
      const messages = await chatIpc.getMessages(sessionId);
      set({ messages, loading: false });
    } catch (err) {
      console.error("Failed to load messages:", err);
      set({ loading: false });
    }
  },
}));
