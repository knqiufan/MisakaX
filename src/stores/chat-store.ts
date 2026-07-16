import { create } from "zustand";
import type { Message, MessageAttachment, Session, ToolCall } from "@/lib/ipc";
import { chatIpc, sessionsIpc } from "@/lib/ipc";

const SELECTED_MODEL_KEY = "misakax:selectedModel";
const THINKING_ENABLED_KEY = "misakax:thinkingEnabled";
const MESSAGE_INITIAL_LIMIT = 50;
const MESSAGE_EARLIER_LIMIT = 100;

export type WorkspaceSelectorIntent = "new-session" | "change-session";

export interface PendingOutbound {
  content: string;
  modelOverride?: string;
  attachments?: MessageAttachment[];
}

function readCachedModel(): string | null {
  try {
    return localStorage.getItem(SELECTED_MODEL_KEY);
  } catch {
    return null;
  }
}

function writeCachedModel(model: string | null) {
  try {
    if (model) {
      localStorage.setItem(SELECTED_MODEL_KEY, model);
    } else {
      localStorage.removeItem(SELECTED_MODEL_KEY);
    }
  } catch { /* noop */ }
}

function readThinkingEnabled(): boolean {
  try {
    const raw = localStorage.getItem(THINKING_ENABLED_KEY);
    if (raw === null) return true;
    return raw === "true";
  } catch {
    return true;
  }
}

function writeThinkingEnabled(enabled: boolean) {
  try {
    localStorage.setItem(THINKING_ENABLED_KEY, String(enabled));
  } catch { /* noop */ }
}

interface ChatState {
  sessions: Session[];
  activeSessionId: string | null;
  activeSession: Session | null;
  loading: boolean;
  showWorkspaceSelector: boolean;
  workspaceSelectorIntent: WorkspaceSelectorIntent | null;
  workspaceSelectorTargetSessionId: string | null;
  thinkingEnabled: boolean;

  messages: Message[];
  isStreaming: boolean;
  streamingMessageId: string | null;
  isThinkingStreaming: boolean;
  selectedModel: string | null;
  modelsVersion: number;
  sessionsReloadToken: number;
  pendingOutbound: PendingOutbound | null;
  hasMoreEarlier: boolean;
  loadingEarlier: boolean;
  scrollToMessageId: string | null;

  setActiveSession: (id: string | null) => void;
  setActiveSessionData: (session: Session | null) => void;
  upsertSession: (session: Session) => void;
  setPendingOutbound: (pending: PendingOutbound | null) => void;
  consumePendingOutbound: () => PendingOutbound | null;
  setSessions: (sessions: Session[]) => void;
  setLoading: (loading: boolean) => void;
  setShowWorkspaceSelector: (show: boolean) => void;
  openWorkspaceSelector: (
    intent: WorkspaceSelectorIntent,
    targetSessionId?: string | null
  ) => void;
  closeWorkspaceSelector: () => void;
  setThinkingEnabled: (enabled: boolean) => void;
  updateActiveSessionWorkingDir: (
    dir: string | null,
    workspaceKind?: string
  ) => void;
  refreshSessions: () => Promise<void>;
  bumpSessionsReload: () => void;
  requestAutoTitle: (sessionId: string, firstMessage: string) => void;

  setMessages: (messages: Message[]) => void;
  addMessage: (message: Message) => void;
  removeMessagesFrom: (messageId: string) => void;
  updateMessageContent: (messageId: string, delta: string) => void;
  updateMessageError: (messageId: string, error: string) => void;
  setMessageStatus: (messageId: string, status: Message["status"]) => void;
  setStreaming: (streaming: boolean, messageId?: string | null) => void;
  setThinkingStreaming: (streaming: boolean) => void;
  setSelectedModel: (model: string | null) => void;
  bumpModels: () => void;
  clearMessages: () => void;
  loadMessages: (sessionId: string) => Promise<void>;
  loadEarlierMessages: (sessionId: string) => Promise<string | null>;
  setScrollToMessageId: (id: string | null) => void;
  remapMessageId: (oldId: string, newId: string) => void;
  addToolCall: (messageId: string, toolCall: ToolCall) => void;
  updateToolCall: (messageId: string, toolCallId: string, patch: Partial<ToolCall>) => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  sessions: [],
  activeSessionId: null,
  activeSession: null,
  loading: false,
  showWorkspaceSelector: false,
  workspaceSelectorIntent: null,
  workspaceSelectorTargetSessionId: null,
  thinkingEnabled: readThinkingEnabled(),

  messages: [],
  isStreaming: false,
  streamingMessageId: null,
  isThinkingStreaming: false,
  selectedModel: readCachedModel(),
  modelsVersion: 0,
  sessionsReloadToken: 0,
  pendingOutbound: null,
  hasMoreEarlier: false,
  loadingEarlier: false,
  scrollToMessageId: null,

  setActiveSession: (id) => set({ activeSessionId: id }),
  setActiveSessionData: (session) => {
    set({
      activeSession: session,
      activeSessionId: session?.id ?? null,
      messages: [],
      isStreaming: false,
      streamingMessageId: null,
      isThinkingStreaming: false,
      hasMoreEarlier: false,
      loadingEarlier: false,
      scrollToMessageId: null,
    });
    get().refreshSessions();
  },
  upsertSession: (session) => {
    set((state) => {
      const idx = state.sessions.findIndex((s) => s.id === session.id);
      const sessions =
        idx >= 0
          ? state.sessions.map((s, i) => (i === idx ? session : s))
          : [session, ...state.sessions];
      const sameId = state.activeSessionId === session.id;
      return {
        sessions,
        activeSession: session,
        activeSessionId: session.id,
        ...(sameId
          ? {}
          : {
              messages: [],
              isStreaming: false,
              streamingMessageId: null,
              isThinkingStreaming: false,
              hasMoreEarlier: false,
              loadingEarlier: false,
              scrollToMessageId: null,
            }),
      };
    });
  },
  setPendingOutbound: (pending) => set({ pendingOutbound: pending }),
  consumePendingOutbound: () => {
    const pending = get().pendingOutbound;
    if (pending) set({ pendingOutbound: null });
    return pending;
  },
  setSessions: (sessions) => set({ sessions }),
  setLoading: (loading) => set({ loading }),
  setShowWorkspaceSelector: (show) =>
    set(
      show
        ? { showWorkspaceSelector: true }
        : {
            showWorkspaceSelector: false,
            workspaceSelectorIntent: null,
            workspaceSelectorTargetSessionId: null,
          }
    ),
  openWorkspaceSelector: (intent, targetSessionId = null) =>
    set({
      showWorkspaceSelector: true,
      workspaceSelectorIntent: intent,
      workspaceSelectorTargetSessionId: targetSessionId ?? null,
    }),
  closeWorkspaceSelector: () =>
    set({
      showWorkspaceSelector: false,
      workspaceSelectorIntent: null,
      workspaceSelectorTargetSessionId: null,
    }),
  setThinkingEnabled: (enabled) => {
    writeThinkingEnabled(enabled);
    set({ thinkingEnabled: enabled });
  },
  updateActiveSessionWorkingDir: (dir, workspaceKind) =>
    set((state) => ({
      activeSession: state.activeSession
        ? {
            ...state.activeSession,
            working_directory: dir,
            ...(workspaceKind !== undefined
              ? { workspace_kind: workspaceKind }
              : {}),
          }
        : null,
      sessions: state.sessions.map((s) =>
        s.id === state.activeSessionId
          ? {
              ...s,
              working_directory: dir,
              ...(workspaceKind !== undefined
                ? { workspace_kind: workspaceKind }
                : {}),
            }
          : s
      ),
    })),

  refreshSessions: async () => {
    try {
      const list = await sessionsIpc.list();
      set({ sessions: list });
    } catch (err) {
      console.error("Failed to refresh sessions:", err);
    }
  },

  bumpSessionsReload: () =>
    set((state) => ({ sessionsReloadToken: state.sessionsReloadToken + 1 })),

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

  removeMessagesFrom: (messageId) =>
    set((state) => {
      const idx = state.messages.findIndex((m) => m.id === messageId);
      if (idx === -1) return state;
      return { messages: state.messages.slice(0, idx) };
    }),

  updateMessageContent: (messageId, delta) =>
    set((state) => ({
      messages: state.messages.map((m) =>
        m.id === messageId ? { ...m, content: m.content + delta } : m
      ),
    })),

  updateMessageError: (messageId, error) =>
    set((state) => ({
      messages: state.messages.map((m) =>
        m.id === messageId ? { ...m, status: "error", content: error } : m
      ),
    })),

  setMessageStatus: (messageId, status) =>
    set((state) => ({
      messages: state.messages.map((m) =>
        m.id === messageId ? { ...m, status } : m
      ),
    })),

  setStreaming: (streaming, messageId = null) =>
    set({
      isStreaming: streaming,
      streamingMessageId: messageId,
      ...(streaming ? {} : { isThinkingStreaming: false }),
    }),

  setThinkingStreaming: (streaming) => set({ isThinkingStreaming: streaming }),

  setSelectedModel: (model) => {
    writeCachedModel(model);
    set({ selectedModel: model });
  },

  bumpModels: () => set((state) => ({ modelsVersion: state.modelsVersion + 1 })),

  clearMessages: () =>
    set({
      messages: [],
      isStreaming: false,
      streamingMessageId: null,
      isThinkingStreaming: false,
      hasMoreEarlier: false,
      loadingEarlier: false,
      scrollToMessageId: null,
    }),

  loadMessages: async (sessionId: string) => {
    try {
      set({ loading: true, hasMoreEarlier: false, loadingEarlier: false });
      const messages = await chatIpc.getMessages(
        sessionId,
        MESSAGE_INITIAL_LIMIT
      );
      set({
        messages,
        loading: false,
        hasMoreEarlier: messages.length >= MESSAGE_INITIAL_LIMIT,
      });
    } catch (err) {
      console.error("Failed to load messages:", err);
      set({ loading: false });
    }
  },

  loadEarlierMessages: async (sessionId: string) => {
    const { messages, loadingEarlier, hasMoreEarlier } = get();
    if (loadingEarlier || !hasMoreEarlier || messages.length === 0) {
      return null;
    }
    const anchorId = messages[0].id;
    try {
      set({ loadingEarlier: true });
      const earlier = await chatIpc.getMessages(
        sessionId,
        MESSAGE_EARLIER_LIMIT,
        anchorId
      );
      set((state) => ({
        messages: [...earlier, ...state.messages],
        loadingEarlier: false,
        hasMoreEarlier: earlier.length >= MESSAGE_EARLIER_LIMIT,
      }));
      return anchorId;
    } catch (err) {
      console.error("Failed to load earlier messages:", err);
      set({ loadingEarlier: false });
      return null;
    }
  },

  setScrollToMessageId: (id) => set({ scrollToMessageId: id }),

  remapMessageId: (oldId, newId) =>
    set((state) => ({
      messages: state.messages.map((m) =>
        m.id === oldId ? { ...m, id: newId } : m
      ),
      streamingMessageId:
        state.streamingMessageId === oldId ? newId : state.streamingMessageId,
    })),

  addToolCall: (messageId, toolCall) =>
    set((state) => ({
      messages: state.messages.map((m) => {
        if (m.id !== messageId) return m;
        const existing = m.tool_calls ?? [];
        const idx = existing.findIndex((tc) => tc.id === toolCall.id);
        if (idx >= 0) {
          const next = existing.slice();
          next[idx] = { ...next[idx], ...toolCall };
          return { ...m, tool_calls: next };
        }
        return { ...m, tool_calls: [...existing, toolCall] };
      }),
    })),

  updateToolCall: (messageId, toolCallId, patch) =>
    set((state) => ({
      messages: state.messages.map((m) =>
        m.id === messageId
          ? {
              ...m,
              tool_calls: (m.tool_calls ?? []).map((tc) =>
                tc.id === toolCallId ? { ...tc, ...patch } : tc
              ),
            }
          : m
      ),
    })),
}));
