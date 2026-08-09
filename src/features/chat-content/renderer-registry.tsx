import type { ComponentType } from "react";
import type { ContentBlock } from "@/lib/ipc";

import { ArtifactBlockRenderer } from "./renderers/ArtifactBlockRenderer";
import { ImageBlockRenderer } from "./renderers/ImageBlockRenderer";
import { MarkdownBlockRenderer } from "./renderers/MarkdownBlockRenderer";
import { NoticeBlockRenderer } from "./renderers/NoticeBlockRenderer";

export interface BlockRendererProps {
  block: ContentBlock;
  sessionId: string;
  isStreaming: boolean;
}

const REGISTRY: Record<ContentBlock["kind"], ComponentType<BlockRendererProps>> = {
  markdown: MarkdownBlockRenderer,
  // Chart and map blocks are deliberately non-executing until their own
  // renderer phases complete; their persisted fallback remains readable.
  chart: NoticeBlockRenderer,
  map: NoticeBlockRenderer,
  artifact: ArtifactBlockRenderer,
  image: ImageBlockRenderer,
  notice: NoticeBlockRenderer,
  unknown: NoticeBlockRenderer,
};

export function resolveBlockRenderer(kind: ContentBlock["kind"]): ComponentType<BlockRendererProps> {
  return REGISTRY[kind] ?? NoticeBlockRenderer;
}
