import { invoke } from "./invoke";
import type {
  Message,
  SendMessageRequest,
  SendMessageResult,
  ToolCall,
} from "./types";

/// 后端 `get_messages` 返回的原始行：`tool_calls` 为 JSON 字符串（或 null）
type RawMessage = Omit<Message, "tool_calls"> & {
  tool_calls?: string | ToolCall[] | null;
};

/**
 * 将后端存储的 `tool_calls`（JSON 字符串）反序列化为 `ToolCall[]`。
 *
 * 兼容三种形态：已是数组、JSON 字符串、null/空串；非法 JSON 返回 `undefined` 而非抛出。
 */
export function parseToolCalls(raw: unknown): ToolCall[] | undefined {
  if (raw == null) return undefined;
  if (Array.isArray(raw)) return raw as ToolCall[];
  if (typeof raw === "string") {
    if (raw.trim() === "") return undefined;
    try {
      const parsed = JSON.parse(raw);
      return Array.isArray(parsed) ? (parsed as ToolCall[]) : undefined;
    } catch {
      return undefined;
    }
  }
  return undefined;
}

/// 将后端原始消息映射为前端 `Message`（解析 tool_calls）
export function mapMessage(raw: RawMessage): Message {
  return { ...raw, tool_calls: parseToolCalls(raw.tool_calls) };
}

export const chatIpc = {
  sendMessage: (request: SendMessageRequest) =>
    invoke<SendMessageResult>("send_message", { request }),

  stopGeneration: (sessionId: string) =>
    invoke<boolean>("stop_generation", { sessionId }),

  regenerateMessage: (sessionId: string, messageId: string) =>
    invoke<SendMessageResult>("regenerate_message", { sessionId, messageId }),

  getMessages: (sessionId: string, limit?: number, beforeId?: string) =>
    invoke<RawMessage[]>("get_messages", { sessionId, limit, beforeId }).then(
      (rows) => rows.map(mapMessage)
    ),
};
