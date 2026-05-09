import { useState, useCallback, useEffect, useMemo, useRef } from "react";
import { Plus, Search, MessageSquare } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { SessionItem } from "./SessionItem";
import { sessionsIpc } from "@/lib/ipc";
import type { Session } from "@/lib/ipc";
import { useChatStore } from "@/stores/chat-store";

interface SessionPanelProps {
  collapsed: boolean;
  onNewSession: () => void;
}

export function SessionPanel({ collapsed, onNewSession }: SessionPanelProps) {
  const {
    sessions,
    activeSessionId,
    setSessions,
    setActiveSession,
    setActiveSessionData,
  } = useChatStore();

  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<Session[] | null>(null);
  const searchTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const loadSessions = useCallback(async () => {
    try {
      const list = await sessionsIpc.list();
      setSessions(list);
    } catch (err) {
      console.error("Failed to load sessions:", err);
    }
  }, [setSessions]);

  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  const handleSearch = useCallback(
    (query: string) => {
      setSearchQuery(query);
      if (searchTimerRef.current) clearTimeout(searchTimerRef.current);

      if (!query.trim()) {
        setSearchResults(null);
        return;
      }

      searchTimerRef.current = setTimeout(async () => {
        try {
          const results = await sessionsIpc.search(query.trim());
          setSearchResults(results);
        } catch (err) {
          console.error("Session search failed:", err);
        }
      }, 300);
    },
    []
  );

  const handleSelect = useCallback(
    async (id: string) => {
      try {
        const session = await sessionsIpc.get(id);
        setActiveSessionData(session);
      } catch (err) {
        console.error("Failed to load session:", err);
      }
    },
    [setActiveSessionData]
  );

  const handleRename = useCallback(
    async (id: string, title: string) => {
      try {
        await sessionsIpc.update({ id, title });
        await loadSessions();
      } catch (err) {
        console.error("Failed to rename session:", err);
      }
    },
    [loadSessions]
  );

  const handleDelete = useCallback(
    async (id: string) => {
      try {
        await sessionsIpc.delete(id);
        if (activeSessionId === id) {
          setActiveSession(null);
          setActiveSessionData(null);
        }
        await loadSessions();
      } catch (err) {
        console.error("Failed to delete session:", err);
      }
    },
    [activeSessionId, setActiveSession, setActiveSessionData, loadSessions]
  );

  const handleArchive = useCallback(
    async (id: string) => {
      try {
        await sessionsIpc.update({ id, status: "archived" });
        if (activeSessionId === id) {
          setActiveSession(null);
          setActiveSessionData(null);
        }
        await loadSessions();
      } catch (err) {
        console.error("Failed to archive session:", err);
      }
    },
    [activeSessionId, setActiveSession, setActiveSessionData, loadSessions]
  );

  const handleTogglePin = useCallback(
    async (id: string, pinned: boolean) => {
      try {
        await sessionsIpc.update({ id, pinned });
        await loadSessions();
      } catch (err) {
        console.error("Failed to toggle pin:", err);
      }
    },
    [loadSessions]
  );

  const displaySessions = useMemo(
    () => searchResults ?? sessions,
    [searchResults, sessions]
  );

  if (collapsed) return null;

  return (
    <div className="flex h-full w-[220px] shrink-0 flex-col border-r border-[color:var(--border-muted)] bg-sidebar">
      <SessionPanelHeader
        searchQuery={searchQuery}
        onSearchChange={handleSearch}
        onNewSession={onNewSession}
      />

      <ScrollArea className="min-h-0 flex-1">
        <div className="space-y-0.5 px-2 py-1">
          {displaySessions.length > 0 ? (
            displaySessions.map((session) => (
              <SessionItem
                key={session.id}
                session={session}
                isActive={activeSessionId === session.id}
                onSelect={handleSelect}
                onRename={handleRename}
                onDelete={handleDelete}
                onArchive={handleArchive}
                onTogglePin={handleTogglePin}
              />
            ))
          ) : (
            <EmptySessionList hasSearch={!!searchQuery.trim()} />
          )}
        </div>
      </ScrollArea>
    </div>
  );
}

function SessionPanelHeader({
  searchQuery,
  onSearchChange,
  onNewSession,
}: {
  searchQuery: string;
  onSearchChange: (q: string) => void;
  onNewSession: () => void;
}) {
  return (
    <div className="shrink-0 space-y-2 px-2 pb-1 pt-2">
      <div className="flex items-center gap-1.5">
        <div className="relative min-w-0 flex-1">
          <Search className="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground/50" />
          <Input
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="搜索会话…"
            className="h-7 pl-8 text-xs"
          />
        </div>
        <Button
          variant="ghost"
          size="icon"
          onClick={onNewSession}
          className={cn(
            "h-7 w-7 shrink-0",
            "text-muted-foreground hover:text-foreground"
          )}
        >
          <Plus className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}

function EmptySessionList({ hasSearch }: { hasSearch: boolean }) {
  return (
    <div className="flex flex-col items-center gap-2 px-4 py-8 text-center">
      <MessageSquare className="h-8 w-8 text-muted-foreground/30" strokeWidth={1.15} />
      <p className="text-xs text-muted-foreground/70">
        {hasSearch ? "未找到匹配的会话" : "暂无会话"}
      </p>
    </div>
  );
}
