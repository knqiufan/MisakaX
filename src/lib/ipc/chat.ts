import { invoke } from "./invoke";
import type {
  Message,
  SendMessageRequest,
  SendMessageResult,
} from "./types";

export const chatIpc = {
  sendMessage: (request: SendMessageRequest) =>
    invoke<SendMessageResult>("send_message", { request }),

  stopGeneration: (sessionId: string) =>
    invoke<boolean>("stop_generation", { sessionId }),

  regenerateMessage: (sessionId: string, messageId: string) =>
    invoke<SendMessageResult>("regenerate_message", { sessionId, messageId }),

  getMessages: (sessionId: string, limit?: number, beforeId?: string) =>
    invoke<Message[]>("get_messages", { sessionId, limit, beforeId }),
};
