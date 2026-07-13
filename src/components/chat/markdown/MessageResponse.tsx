import { Streamdown } from "streamdown";
import { code } from "@streamdown/code";
import { math } from "@streamdown/math";
import { mermaid } from "@streamdown/mermaid";
import { cjk } from "@streamdown/cjk";
import { cn } from "@/lib/utils";
import { CHAT_MARKDOWN_COMPONENTS } from "./markdown-components";

const PLUGINS = { code, math, mermaid, cjk } as const;

interface MessageResponseProps {
  content: string;
  isStreaming?: boolean;
  className?: string;
}

/** Streamdown wrapper for assistant markdown (docs/ui/06 §1). */
export function MessageResponse({
  content,
  isStreaming = false,
  className,
}: MessageResponseProps) {
  if (!content) return null;

  return (
    <div className={cn("misaka-chat-md text-sm text-foreground", className)}>
      <Streamdown
        mode={isStreaming ? "streaming" : "static"}
        isAnimating={isStreaming}
        plugins={PLUGINS}
        components={CHAT_MARKDOWN_COMPONENTS}
        parseIncompleteMarkdown={isStreaming}
        controls={{
          code: { copy: true, download: false },
          table: { copy: true, download: false, fullscreen: false },
          mermaid: { copy: true, download: false, fullscreen: false, panZoom: false },
        }}
        shikiTheme={["github-light", "github-dark"]}
        lineNumbers={false}
      >
        {content}
      </Streamdown>
    </div>
  );
}
