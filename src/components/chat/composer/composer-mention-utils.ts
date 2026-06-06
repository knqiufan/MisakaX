import type { PendingAttachment } from "@/stores/composer-store";
import {
  buildOutgoingFromSegments,
  collectMentions,
  countSendableFromSegments,
  type ComposerSegment,
} from "./composer-segment";

export function mentionsToWorkspaceAttachments(
  segments: ComposerSegment[]
): PendingAttachment[] {
  return collectMentions(segments)
    .filter((m) => m.status === "ready" && m.extractedText != null)
    .map((m) => ({
      id: m.id,
      kind: "text" as const,
      extracted_text: m.extractedText!,
      mime: m.mime ?? "text/plain",
      file_name: m.relPath,
      size: m.size ?? m.extractedText!.length,
    }));
}

export function buildOutgoingContent(segments: ComposerSegment[]): string {
  return buildOutgoingFromSegments(segments);
}

export function countSendableMentions(segments: ComposerSegment[]): number {
  return countSendableFromSegments(segments);
}
