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
    set((s) => {
      const idx = s.toolCalls.findIndex((item) => item.id === tc.id);
      if (idx >= 0) {
        const next = s.toolCalls.slice();
        next[idx] = { ...next[idx], ...tc };
        return { toolCalls: next };
      }
      return { toolCalls: [...s.toolCalls, tc] };
    }),
  clearToolCalls: () => set({ toolCalls: [] }),
}));
