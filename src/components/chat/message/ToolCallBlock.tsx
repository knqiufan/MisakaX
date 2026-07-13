import type { ToolCall } from "@/lib/ipc";
import { ToolActionRow } from "./ToolActionsGroup";

interface ToolCallBlockProps {
  toolCall: ToolCall;
}

/** @deprecated Prefer ToolActionsGroup / ToolActionRow; kept for ToolLogsPanel. */
export function ToolCallBlock({ toolCall }: ToolCallBlockProps) {
  return <ToolActionRow toolCall={toolCall} />;
}
