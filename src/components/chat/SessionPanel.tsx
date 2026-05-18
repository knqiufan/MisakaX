import { useState, useCallback, useEffect, useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import {
  Plus,
  Search,
  MessageSquare,
  ChevronDown,
  ChevronRight,
  Archive,
} from "lucide-react";
import { save as dialogSave } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { SessionItem } from "./SessionItem";
import { MessageSearchResults } from "./MessageSearchResults";
import { sessionsIpc } from "@/lib/ipc";
import type { Session, MessageSearchResult } from "@/lib/ipc";
import { useChatStore } from "@/stores/chat-store";

interface SessionPanelProps {
  onNewSession: () => void;
}

export function SessionPanel({ onNewSession }: SessionPanelProps) {
  const { t } = useTranslation();
  const {
    sessions,
    activeSessionId,
    setSessions,
    setActiveSession,
    setActiveSessionData,
  } = useChatStore();

  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<Session[] | null>(null);
  const [messageResults, setMessageResults] = useState<MessageSearchResult[] | null>(null);
  const [groups, setGroups] = useState<string[]>([]);
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());
  const [showArchived, setShowArchived] = useState(false);
  const [archivedSessions, setArchivedSessions] = useState<Session[]>([]);
  const [archivedCount, setArchivedCount] = useState(0);
  const searchTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const loadSessions = useCallback(async () => {
    try {
      const [list, groupList, archived] = await Promise.all([
        sessionsIpc.list(),
        sessionsIpc.listGroups(),
        sessionsIpc.list("archived"),
      ]);
      setSessions(list);
      setGroups(groupList);
      setArchivedCount(archived.length);
      if (showArchived) setArchivedSessions(archived);
    } catch (err) {
      console.error("Failed to load sessions:", err);
    }
  }, [setSessions, showArchived]);

  const loadArchivedSessions = useCallback(async () => {
    try {
      const list = await sessionsIpc.list("archived");
      setArchivedSessions(list);
      setArchivedCount(list.length);
    } catch (err) {
      console.error("Failed to load archived sessions:", err);
    }
  }, []);

  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  useEffect(() => {
    if (showArchived) loadArchivedSessions();
  }, [showArchived, loadArchivedSessions]);

  const handleSearch = useCallback((query: string) => {
    setSearchQuery(query);
    if (searchTimerRef.current) clearTimeout(searchTimerRef.current);

    if (!query.trim()) {
      setSearchResults(null);
      setMessageResults(null);
      return;
    }

    searchTimerRef.current = setTimeout(async () => {
      try {
        const [sessionHits, msgHits] = await Promise.all([
          sessionsIpc.search(query.trim()),
          sessionsIpc.searchMessages(query.trim(), undefined, 20),
        ]);
        setSearchResults(sessionHits);
        setMessageResults(msgHits);
      } catch (err) {
        console.error("Search failed:", err);
      }
    }, 300);
  }, []);

  const handleSelect = useCallback(
    async (id: string) => {
      if (id === activeSessionId) return;
      try {
        const session = await sessionsIpc.get(id);
        setActiveSessionData(session);
      } catch (err) {
        console.error("Failed to load session:", err);
      }
    },
    [activeSessionId, setActiveSessionData]
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
        if (showArchived) await loadArchivedSessions();
      } catch (err) {
        console.error("Failed to delete session:", err);
      }
    },
    [activeSessionId, setActiveSession, setActiveSessionData, loadSessions, showArchived, loadArchivedSessions]
  );

  const handleArchive = useCallback(
    async (id: string, archived: boolean) => {
      try {
        await sessionsIpc.archive(id, archived);
        if (archived && activeSessionId === id) {
          setActiveSession(null);
          setActiveSessionData(null);
        }
        await loadSessions();
        if (showArchived) await loadArchivedSessions();
      } catch (err) {
        console.error("Failed to toggle archive:", err);
      }
    },
    [activeSessionId, setActiveSession, setActiveSessionData, loadSessions, showArchived, loadArchivedSessions]
  );

  const handleTogglePin = useCallback(
    async (id: string, pinned: boolean) => {
      try {
        await sessionsIpc.pin(id, pinned);
        await loadSessions();
      } catch (err) {
        console.error("Failed to toggle pin:", err);
      }
    },
    [loadSessions]
  );

  const handleSetGroup = useCallback(
    async (id: string, group: string | null) => {
      try {
        await sessionsIpc.setGroup(id, group);
        await loadSessions();
      } catch (err) {
        console.error("Failed to set group:", err);
      }
    },
    [loadSessions]
  );

  const handleExport = useCallback(async (id: string) => {
    try {
      const filePath = await dialogSave({
        title: "导出会话",
        defaultPath: "session-export.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!filePath) return;
      await sessionsIpc.exportSessions([id], filePath);
      toast.success("导出成功");
    } catch (err) {
      console.error("Export failed:", err);
      toast.error("导出失败");
    }
  }, []);

  const handleMessageResultClick = useCallback(
    (sessionId: string) => {
      handleSelect(sessionId);
      setSearchQuery("");
      setSearchResults(null);
      setMessageResults(null);
    },
    [handleSelect]
  );

  const toggleGroupCollapse = useCallback((group: string) => {
    setCollapsedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(group)) next.delete(group);
      else next.add(group);
      return next;
    });
  }, []);

  const isSearching = !!searchQuery.trim();

  const { pinnedSessions, groupedSessions, ungroupedSessions } = useMemo(() => {
    const display = searchResults ?? sessions;
    const pinned = display.filter((s) => s.pinned);
    const grouped = new Map<string, Session[]>();
    const ungrouped: Session[] = [];

    for (const s of display) {
      if (s.pinned) continue;
      if (s.group_name) {
        const list = grouped.get(s.group_name) ?? [];
        list.push(s);
        grouped.set(s.group_name, list);
      } else {
        ungrouped.push(s);
      }
    }

    return { pinnedSessions: pinned, groupedSessions: grouped, ungroupedSessions: ungrouped };
  }, [searchResults, sessions]);

  const renderSessionItem = (session: Session) => (
    <SessionItem
      key={session.id}
      session={session}
      isActive={activeSessionId === session.id}
      groups={groups}
      onSelect={handleSelect}
      onRename={handleRename}
      onDelete={handleDelete}
      onArchive={handleArchive}
      onTogglePin={handleTogglePin}
      onSetGroup={handleSetGroup}
      onExport={handleExport}
    />
  );

  const totalDisplay =
    pinnedSessions.length +
    Array.from(groupedSessions.values()).reduce((acc, g) => acc + g.length, 0) +
    ungroupedSessions.length;

  return (
    <div className="flex h-full min-w-0 w-full flex-col border-r border-[color:var(--border-muted)] bg-sidebar">
      <SessionPanelHeader
        searchQuery={searchQuery}
        onSearchChange={handleSearch}
        onNewSession={onNewSession}
        newSessionLabel={t("common:newSession")}
      />

      <ScrollArea className="min-h-0 flex-1">
        <div className="space-y-0.5 px-3 py-2">
          {isSearching && messageResults && messageResults.length > 0 && (
            <MessageSearchResults
              results={messageResults}
              onResultClick={handleMessageResultClick}
            />
          )}

          {totalDisplay > 0 ? (
            <>
              {pinnedSessions.length > 0 && (
                <SectionHeader label={t("chat:session.pinned")} />
              )}
              {pinnedSessions.map(renderSessionItem)}

              {Array.from(groupedSessions.entries()).map(([groupName, items]) => (
                <div key={groupName}>
                  <GroupHeader
                    label={groupName}
                    count={items.length}
                    collapsed={collapsedGroups.has(groupName)}
                    onToggle={() => toggleGroupCollapse(groupName)}
                  />
                  {!collapsedGroups.has(groupName) && items.map(renderSessionItem)}
                </div>
              ))}

              {ungroupedSessions.length > 0 && groupedSessions.size > 0 && (
                <SectionHeader label={t("chat:session.ungrouped")} />
              )}
              {ungroupedSessions.map(renderSessionItem)}
            </>
          ) : (
            <EmptySessionList hasSearch={isSearching} />
          )}

          {!isSearching && archivedCount > 0 && (
            <ArchivedToggle
              count={archivedCount}
              show={showArchived}
              onToggle={() => setShowArchived((v) => !v)}
              onLoadArchived={loadArchivedSessions}
            />
          )}

          {showArchived && archivedSessions.map(renderSessionItem)}
        </div>
      </ScrollArea>
    </div>
  );
}

function SessionPanelHeader({
  searchQuery,
  onSearchChange,
  onNewSession,
  newSessionLabel,
}: {
  searchQuery: string;
  onSearchChange: (q: string) => void;
  onNewSession: () => void;
  newSessionLabel: string;
}) {
  const { t } = useTranslation("chat");
  return (
    <div className="shrink-0 border-b border-[color:var(--border-muted)] bg-[color:var(--surface-sidebar)] px-3 pb-3 pt-3">
      <div className="flex w-full items-stretch gap-2">
        <div className="relative min-w-0 flex-1">
          <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground/55" />
          <Input
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder={t("session.searchPlaceholder")}
            className={cn(
              "h-10 w-full rounded-[var(--radius-ui-md)] border-[color:var(--border-muted)]",
              "bg-[color:var(--surface-card)] pl-10 pr-3 text-sm shadow-none",
              "placeholder:text-muted-foreground/55"
            )}
          />
        </div>
        <Button
          type="button"
          variant="outline"
          size="icon"
          onClick={onNewSession}
          title={newSessionLabel}
          aria-label={newSessionLabel}
          className={cn(
            "h-10 w-10 shrink-0 rounded-[var(--radius-ui-md)]",
            "border-[color:var(--border-muted)] bg-[color:var(--surface-card)]",
            "text-muted-foreground hover:bg-[color:var(--surface-control-hover)] hover:text-foreground"
          )}
        >
          <Plus className="h-5 w-5" />
        </Button>
      </div>
    </div>
  );
}

function SectionHeader({ label }: { label: string }) {
  return (
    <div className="px-1 pb-0.5 pt-3 text-[0.625rem] font-semibold uppercase tracking-wider text-muted-foreground/50">
      {label}
    </div>
  );
}

function GroupHeader({
  label,
  count,
  collapsed,
  onToggle,
}: {
  label: string;
  count: number;
  collapsed: boolean;
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onToggle}
      className="flex w-full items-center gap-1 px-1 pb-0.5 pt-3 text-[0.625rem] font-semibold uppercase tracking-wider text-muted-foreground/50 hover:text-muted-foreground transition-colors"
    >
      {collapsed ? (
        <ChevronRight className="h-3 w-3" />
      ) : (
        <ChevronDown className="h-3 w-3" />
      )}
      <span>{label}</span>
      <span className="ml-auto text-[0.5625rem] tabular-nums">{count}</span>
    </button>
  );
}

function ArchivedToggle({
  count,
  show,
  onToggle,
  onLoadArchived,
}: {
  count: number;
  show: boolean;
  onToggle: () => void;
  onLoadArchived: () => void;
}) {
  const { t } = useTranslation("chat");
  const handleClick = () => {
    if (!show) onLoadArchived();
    onToggle();
  };

  return (
    <button
      type="button"
      onClick={handleClick}
      className="mt-2 flex w-full items-center justify-center gap-1.5 rounded-[var(--radius-button)] px-2 py-1.5 text-[0.6875rem] text-muted-foreground/60 transition-colors hover:bg-[color:var(--surface-hover)] hover:text-muted-foreground"
    >
      <Archive className="h-3 w-3" />
      {show ? t("session.hideArchived") : t("session.showArchived", { count })}
    </button>
  );
}

function EmptySessionList({ hasSearch }: { hasSearch: boolean }) {
  const { t } = useTranslation("chat");
  return (
    <div className="flex flex-col items-center gap-2 px-4 py-8 text-center">
      <MessageSquare className="h-8 w-8 text-muted-foreground/30" strokeWidth={1.15} />
      <p className="text-xs text-muted-foreground/70">
        {hasSearch ? t("session.noMatchingSessions") : t("session.noSessions")}
      </p>
    </div>
  );
}
