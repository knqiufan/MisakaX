import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Brain, ChevronRight, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";

interface ThinkingBlockProps {
  content: string;
  isStreaming?: boolean;
}

export function ThinkingBlock({ content, isStreaming }: ThinkingBlockProps) {
  const { t } = useTranslation("chat");
  const [expanded, setExpanded] = useState(false);

  if (!content && !isStreaming) return null;

  return (
    <Collapsible open={expanded} onOpenChange={setExpanded} className="mb-1">
      <CollapsibleTrigger
        className={cn(
          "flex items-center gap-1.5 rounded-md px-2 py-1",
          "text-xs text-muted-foreground/70",
          "transition-colors duration-[var(--ds-dur-fast)]",
          "hover:bg-[color:var(--surface-hover)] hover:text-muted-foreground"
        )}
      >
        {isStreaming ? (
          <Loader2 className="size-3 animate-spin" />
        ) : (
          <Brain className="size-3" />
        )}
        <span>
          {isStreaming ? t("thinkingStreaming") : t("thinkingDone")}
        </span>
        <ChevronRight
          className={cn(
            "size-3 transition-transform duration-[var(--ds-dur-fast)]",
            expanded && "rotate-90"
          )}
        />
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div
          className={cn(
            "mt-1 rounded-[var(--radius-ui-md)] px-3 py-2",
            "bg-[color:var(--surface-card)] border border-[color:var(--border-muted)]",
            "text-xs leading-relaxed text-muted-foreground/80",
            "whitespace-pre-wrap"
          )}
        >
          {content}
          {isStreaming && (
            <span className="ml-0.5 inline-block size-1.5 animate-pulse rounded-full bg-muted-foreground/40" />
          )}
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
}
