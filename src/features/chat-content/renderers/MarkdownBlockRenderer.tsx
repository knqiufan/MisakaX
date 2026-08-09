import { MessageResponse } from "@/components/chat/markdown/MessageResponse";
import type { BlockRendererProps } from "../renderer-registry";
import { readMarkdownPayload } from "../types";
import { NoticeBlockRenderer } from "./NoticeBlockRenderer";

export function MarkdownBlockRenderer({ block, isStreaming, sessionId }: BlockRendererProps) {
  const payload = readMarkdownPayload(block);
  if (!payload) return <NoticeBlockRenderer block={block} isStreaming={isStreaming} sessionId={sessionId} />;
  return <MessageResponse content={payload.text} isStreaming={isStreaming && block.status === "pending"} />;
}
