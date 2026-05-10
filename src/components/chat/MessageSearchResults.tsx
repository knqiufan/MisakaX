import { useMemo } from "react";
import { MessageSquare } from "lucide-react";
import { cn } from "@/lib/utils";
import type { MessageSearchResult } from "@/lib/ipc";

interface MessageSearchResultsProps {
  results: MessageSearchResult[];
  onResultClick: (sessionId: string, messageId?: string) => void;
}

export function MessageSearchResults({
  results,
  onResultClick,
}: MessageSearchResultsProps) {
  const grouped = useMemo(() => {
    const map = new Map<string, { title: string | null; items: MessageSearchResult[] }>();
    for (const r of results) {
      const entry = map.get(r.session_id) ?? { title: r.session_title, items: [] };
      entry.items.push(r);
      map.set(r.session_id, entry);
    }
    return map;
  }, [results]);

  if (results.length === 0) return null;

  return (
    <div className="mb-2 rounded-[var(--radius-ui-md)] border border-[color:var(--border-muted)] bg-[color:var(--surface-card)]">
      <div className="flex items-center gap-1.5 border-b border-[color:var(--border-muted)] px-3 py-1.5 text-[0.6875rem] font-semibold text-muted-foreground/70">
        <MessageSquare className="h-3 w-3" />
        消息搜索结果
      </div>
      <div className="max-h-48 overflow-y-auto">
        {Array.from(grouped.entries()).map(([sessionId, { title, items }]) => (
          <div key={sessionId}>
            <div className="px-3 pt-2 pb-0.5 text-[0.625rem] font-medium text-muted-foreground/55 truncate">
              {title || "New Chat"}
            </div>
            {items.map((item) => (
              <SearchResultItem
                key={item.id}
                item={item}
                onClick={() => onResultClick(sessionId, item.id)}
              />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}

function SearchResultItem({
  item,
  onClick,
}: {
  item: MessageSearchResult;
  onClick: () => void;
}) {
  const roleLabel = item.role === "user" ? "我" : "AI";

  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "flex w-full items-start gap-2 px-3 py-1.5 text-left text-[0.75rem]",
        "transition-colors hover:bg-[color:var(--surface-hover)]"
      )}
    >
      <span className="mt-0.5 shrink-0 rounded bg-muted px-1 py-0.5 text-[0.5625rem] font-medium text-muted-foreground">
        {roleLabel}
      </span>
      <span
        className="min-w-0 flex-1 text-muted-foreground line-clamp-2"
        dangerouslySetInnerHTML={{ __html: item.snippet }}
      />
    </button>
  );
}
