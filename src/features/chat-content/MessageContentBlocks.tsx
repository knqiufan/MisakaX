import { Component, type ErrorInfo, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { ContentBlock } from "@/lib/ipc";
import { resolveBlockRenderer } from "./renderer-registry";

interface MessageContentBlocksProps {
  blocks: ContentBlock[];
  sessionId: string;
  isStreaming: boolean;
}

export function MessageContentBlocks({ blocks, sessionId, isStreaming }: MessageContentBlocksProps) {
  return (
    <div className="min-w-0">
      {[...blocks]
        .sort((left, right) => left.position - right.position || left.id.localeCompare(right.id))
        .map((block) => {
          const Renderer = resolveBlockRenderer(block.kind);
          return (
            <BlockErrorBoundary key={block.id}>
              <Renderer block={block} sessionId={sessionId} isStreaming={isStreaming} />
            </BlockErrorBoundary>
          );
        })}
    </div>
  );
}

class BlockErrorBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  public state = { failed: false };

  public static getDerivedStateFromError() {
    return { failed: true };
  }

  public componentDidCatch(_error: Error, _info: ErrorInfo) {
    // A block failure must not prevent adjacent Markdown/tool blocks from rendering.
  }

  public render() {
    if (this.state.failed) return <BlockRenderFailure />;
    return this.props.children;
  }
}

function BlockRenderFailure() {
  const { t } = useTranslation("chat");
  return (
    <p className="my-3 rounded-lg border border-destructive/35 bg-destructive/8 px-3 py-2 text-sm text-destructive" role="alert">
      {t("richContent.blockUnavailable")}
    </p>
  );
}
