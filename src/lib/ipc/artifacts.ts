import { invoke } from "./invoke";
import type {
  ArtifactExportOutcome,
  ArtifactMetadata,
  ArtifactPreview,
  ArtifactRegisterRequest,
  ContentBlock,
} from "./types";

export const artifactsIpc = {
  register: (request: ArtifactRegisterRequest) =>
    invoke<ArtifactMetadata>("artifact_register", { request }),
  getMetadata: (sessionId: string, artifactId: string) =>
    invoke<ArtifactMetadata>("artifact_get_metadata", { sessionId, artifactId }),
  getPreview: (sessionId: string, artifactId: string) =>
    invoke<ArtifactPreview>("artifact_get_preview", { sessionId, artifactId }),
  readPreviewBase64: (sessionId: string, artifactId: string) =>
    invoke<string>("artifact_read_preview_base64", { sessionId, artifactId }),
  export: (sessionId: string, artifactId: string) =>
    invoke<ArtifactExportOutcome>("artifact_export", { sessionId, artifactId }),
  expire: (sessionId: string, artifactId: string) =>
    invoke<void>("artifact_delete_or_expire", { sessionId, artifactId }),
  appendBlock: (block: ContentBlock) =>
    invoke<void>("append_content_block", { block }),
  getMessageBlocks: (messageId: string) =>
    invoke<ContentBlock[]>("get_message_blocks", { messageId }),
};
