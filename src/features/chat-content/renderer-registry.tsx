import type { ComponentType } from "react";
import type { ContentBlock } from "@/lib/ipc";

import { ArtifactBlockRenderer } from "./renderers/ArtifactBlockRenderer";
import { ChartBlockRenderer } from "./renderers/ChartBlockRenderer";
import { ImageBlockRenderer } from "./renderers/ImageBlockRenderer";
import { MapBlockRenderer } from "./renderers/MapBlockRenderer";
import { MarkdownBlockRenderer } from "./renderers/MarkdownBlockRenderer";
import { NoticeBlockRenderer } from "./renderers/NoticeBlockRenderer";

export interface BlockRendererProps {
  block: ContentBlock;
  sessionId: string;
  isStreaming: boolean;
}

const REGISTRY: Record<ContentBlock["kind"], ComponentType<BlockRendererProps>> = {
  markdown: MarkdownBlockRenderer,
  chart: ChartBlockRenderer,
  map: MapBlockRenderer,
  artifact: ArtifactBlockRenderer,
  image: ImageBlockRenderer,
  notice: NoticeBlockRenderer,
  unknown: NoticeBlockRenderer,
};

export function resolveBlockRenderer(kind: ContentBlock["kind"]): ComponentType<BlockRendererProps> {
  return REGISTRY[kind] ?? NoticeBlockRenderer;
}
