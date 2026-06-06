import { create } from "zustand";
import type { ToolCall } from "@/lib/ipc";

interface ToolLogsState {
  toolCalls: ToolCall[];
  addToolCall: (tc: ToolCall) => void;
  clearToolCalls: () => void;
}

export const useToolLogsStore = create<ToolLogsState>((set) => ({
  toolCalls: [],
  addToolCall: (tc) =>
    set((s) => ({ toolCalls: [...s.toolCalls, tc] })),
  clearToolCalls: () => set({ toolCalls: [] }),
}));
