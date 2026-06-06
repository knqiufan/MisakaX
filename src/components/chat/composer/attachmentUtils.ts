export const MAX_IMAGE_SIZE = 10 * 1024 * 1024;
export const MAX_TEXT_ATTACHMENT_SIZE = 1024 * 1024;

const IMAGE_TYPES = ["image/png", "image/jpeg", "image/gif", "image/webp"];
const TEXT_EXTENSIONS = [".md", ".markdown", ".txt"];
const TEXT_MIME_TYPES = ["text/plain", "text/markdown"];
const PLANNED_EXTENSIONS = [".pdf", ".doc", ".docx", ".xls", ".xlsx"];

/**
 * 图片文件 picker 的 `accept`：仅严格列出受支持的图片 MIME，
 * 避免出现 PDF/Word 等被误选后又被前端拒绝的反直觉体验。
 */
export const ACCEPTED_IMAGE_TYPES = IMAGE_TYPES.join(",");

/**
 * Markdown / 文本 picker 的 `accept`：仅 .md/.markdown/.txt 与对应 MIME。
 * 不包含 PLANNED_EXTENSIONS，PDF/Office 文档支持将来通过独立入口接入。
 */
export const ACCEPTED_TEXT_TYPES = [...TEXT_EXTENSIONS, ...TEXT_MIME_TYPES].join(",");

/** 通用入口（拖拽/粘贴时使用，可接受 Image+Text，不含规划中的 PDF/Office）。 */
export const ACCEPTED_ATTACHMENT_TYPES = [...IMAGE_TYPES, ...TEXT_EXTENSIONS].join(",");

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

/**
 * 把绝对路径 `absPath` 转成相对于 `workingDir` 的 POSIX 风格相对路径。
 * 兼容 Windows 大小写不敏感的盘符 + 反斜杠。如果 absPath 不在 workingDir 下，
 * 返回 absPath 自身（POSIX 化），便于上层做兜底。
 */
export function toWorkspaceRelativePath(workingDir: string, absPath: string): string {
  const normRoot = workingDir.replace(/\\/g, "/").replace(/\/+$/, "");
  const normTarget = absPath.replace(/\\/g, "/");
  const lowerRoot = normRoot.toLowerCase();
  const lowerTarget = normTarget.toLowerCase();

  if (lowerTarget === lowerRoot) return ".";
  const prefix = lowerRoot + "/";
  if (lowerTarget.startsWith(prefix)) {
    return normTarget.slice(prefix.length);
  }
  return normTarget;
}

/** 取路径的文件名（不依赖 OS API）。 */
export function basenameOf(path: string): string {
  const normalized = path.replace(/\\/g, "/").replace(/\/+$/, "");
  const idx = normalized.lastIndexOf("/");
  return idx === -1 ? normalized : normalized.slice(idx + 1);
}
