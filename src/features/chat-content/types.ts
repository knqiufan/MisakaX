import type { ContentBlock } from "@/lib/ipc";

export interface ArtifactBlockPayload {
  artifact_id: string;
  display_name?: string;
  preview_hint?: string;
  alt?: string;
  width?: number;
  height?: number;
}

export interface MarkdownBlockPayload {
  text: string;
}

export interface NoticeBlockPayload {
  message_key: string;
  severity?: "info" | "warning" | "error";
  params?: Record<string, string>;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isString(value: unknown): value is string {
  return typeof value === "string";
}

export function readMarkdownPayload(block: ContentBlock): MarkdownBlockPayload | null {
  if (!isRecord(block.payload) || !isString(block.payload.text)) return null;
  return { text: block.payload.text };
}

export function readArtifactPayload(block: ContentBlock): ArtifactBlockPayload | null {
  if (!isRecord(block.payload) || !isString(block.payload.artifact_id)) return null;
  return {
    artifact_id: block.payload.artifact_id,
    display_name: isString(block.payload.display_name) ? block.payload.display_name : undefined,
    preview_hint: isString(block.payload.preview_hint) ? block.payload.preview_hint : undefined,
    alt: isString(block.payload.alt) ? block.payload.alt : undefined,
    width: typeof block.payload.width === "number" ? block.payload.width : undefined,
    height: typeof block.payload.height === "number" ? block.payload.height : undefined,
  };
}

export function readNoticePayload(block: ContentBlock): NoticeBlockPayload | null {
  if (!isRecord(block.payload) || !isString(block.payload.message_key)) return null;
  const severity = block.payload.severity;
  const params = isRecord(block.payload.params)
    ? Object.entries(block.payload.params).reduce<Record<string, string>>((result, [key, value]) => {
        if (isString(value)) result[key] = value;
        return result;
      }, {})
    : undefined;
  return {
    message_key: block.payload.message_key,
    severity: severity === "warning" || severity === "error" ? severity : "info",
    params,
  };
}
