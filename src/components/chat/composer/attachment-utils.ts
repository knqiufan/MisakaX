export const MAX_IMAGE_SIZE = 10 * 1024 * 1024;
export const MAX_TEXT_ATTACHMENT_SIZE = 1024 * 1024;

const IMAGE_TYPES = ["image/png", "image/jpeg", "image/gif", "image/webp"];
const TEXT_EXTENSIONS = [".md", ".markdown", ".txt"];
const PLANNED_EXTENSIONS = [".pdf", ".doc", ".docx", ".xls", ".xlsx"];

export const ACCEPTED_ATTACHMENT_TYPES = [
  ...IMAGE_TYPES,
  ...TEXT_EXTENSIONS,
  ...PLANNED_EXTENSIONS,
].join(",");

export type AttachmentKind = "image" | "text" | "planned" | "unsupported";

export interface AttachmentClassification {
  kind: AttachmentKind;
  supported: boolean;
}

export interface ComposerSendState {
  content: string;
  attachmentCount: number;
  disabled: boolean;
  isStreaming: boolean;
}

export function classifyAttachment(
  mimeType: string,
  fileName: string
): AttachmentClassification {
  const lowerName = fileName.toLowerCase();
  const mime = mimeType.toLowerCase();

  if (IMAGE_TYPES.includes(mime)) {
    return { kind: "image", supported: true };
  }
  if (mime === "text/plain" || mime === "text/markdown" || hasAnyExt(lowerName, TEXT_EXTENSIONS)) {
    return { kind: "text", supported: true };
  }
  if (
    mime === "application/pdf" ||
    mime.includes("wordprocessingml") ||
    mime.includes("spreadsheetml") ||
    hasAnyExt(lowerName, PLANNED_EXTENSIONS)
  ) {
    return { kind: "planned", supported: false };
  }
  return { kind: "unsupported", supported: false };
}

export function canSendComposerMessage(state: ComposerSendState): boolean {
  const hasPayload = state.content.trim().length > 0 || state.attachmentCount > 0;
  return hasPayload && !state.disabled && !state.isStreaming;
}

export function formatFileSize(size: number): string {
  if (size >= 1024 * 1024) return `${(size / 1024 / 1024).toFixed(1)} MB`;
  if (size >= 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${size} B`;
}

export function inferTextMime(file: File): string {
  if (file.type) return file.type;
  return file.name.toLowerCase().endsWith(".md") ? "text/markdown" : "text/plain";
}

function hasAnyExt(fileName: string, extensions: string[]): boolean {
  return extensions.some((ext) => fileName.endsWith(ext));
}
