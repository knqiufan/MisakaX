import { useState, useCallback, useEffect, useMemo, useRef, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import {
  Plus,
  Search,
  ChevronDown,
  ChevronRight,
  Archive,
  FolderOpen,
  Pin,
  Trash2,
} from "lucide-react";
import { save as dialogSave } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { SessionItem } from "./SessionItem";
import { SessionPanelFooter } from "./SessionPanelFooter";
import { MessageSearchResults } from "../message/MessageSearchResults";
import {
  collapseKeyForGroup,
  groupSessionsByWorkspace,
  type WorkspaceSessionGroup,
} from "./groupSessionsByWorkspace";
import { fsIpc, sessionsIpc, workspaceIpc } from "@/lib/ipc";
import type { Session, MessageSearchResult, WorkspacePreference } from "@/lib/ipc";
import { useChatStore } from "@/stores/chat-store";
import { extractDirName } from "@/components/chat/workspace/WorkspaceBar";

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
  const sessionsReloadToken = useChatStore((s) => s.sessionsReloadToken);

  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<Session[] | null>(null);
  const [messageResults, setMessageResults] = useState<MessageSearchResult[] | null>(null);
  const [groups, setGroups] = useState<string[]>([]);
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set());
  const [showArchived, setShowArchived] = useState(false);
  const [archivedSessions, setArchivedSessions] = useState<Session[]>([]);
  const [archivedCount, setArchivedCount] = useState(0);
  const [workspacePreferences, setWorkspacePreferences] = useState<WorkspacePreference[]>([]);
  const [pendingDelete, setPendingDelete] = useState<Session | null>(null);
  const [deleting, setDeleting] = useState(false);
  const searchTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const loadSessions = useCallback(async () => {
    try {
      const [list, groupList, archived, preferences] = await Promise.all([
        sessionsIpc.list(),
        sessionsIpc.listGroups(),
        sessionsIpc.list("archived"),
        workspaceIpc.listPreferences(),
      ]);
      setSessions(list);
      setGroups(groupList);
      setWorkspacePreferences(preferences);
      setArchivedCount(
        groupSessionsByWorkspace(archived, preferences).workspaces.reduce(
          (count, workspace) => count + workspace.sessions.length,
          0
        )
      );
      if (showArchived) setArchivedSessions(archived);
    } catch (err) {
      console.error("Failed to load sessions:", err);
    }
  }, [setSessions, showArchived]);

  const loadArchivedSessions = useCallback(async () => {
    try {
      const list = await sessionsIpc.list("archived");
      setArchivedSessions(list);
      setArchivedCount(
        groupSessionsByWorkspace(list, workspacePreferences).workspaces.reduce(
          (count, workspace) => count + workspace.sessions.length,
          0
        )
      );
    } catch (err) {
      console.error("Failed to load archived sessions:", err);
    }
  }, [workspacePreferences]);

  useEffect(() => {
    loadSessions();
  }, [loadSessions, sessionsReloadToken]);

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
      setDeleting(true);
      try {
        await sessionsIpc.delete(id);
        if (activeSessionId === id) {
          setActiveSession(null);
          setActiveSessionData(null);
        }
        await loadSessions();
        if (showArchived) await loadArchivedSessions();
        setPendingDelete(null);
      } catch (err) {
        console.error("Failed to delete session:", err);
        toast.error(t("common:error"));
      } finally {
        setDeleting(false);
      }
    },
    [activeSessionId, setActiveSession, setActiveSessionData, loadSessions, showArchived, loadArchivedSessions, t]
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
        title: t("chat:session.export"),
        defaultPath: "task-export.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!filePath) return;
      await sessionsIpc.exportSessions([id], filePath);
      toast.success(t("common:success"));
    } catch (err) {
      console.error("Export failed:", err);
      toast.error(t("common:error"));
    }
  }, [t]);

  const handleNewTaskForWorkspace = useCallback(
    async (path: string) => {
      try {
        const session = await sessionsIpc.create({ workingDirectory: path });
        setActiveSessionData(session);
        await loadSessions();
      } catch (err) {
        console.error("Failed to create task for workspace:", err);
        toast.error(t("session.projectActionFailed"));
      }
    },
    [loadSessions, setActiveSessionData, t]
  );

  const handleUpdateWorkspacePreference = useCallback(
    async (
      workspaceKey: string,
      preference: { pinned?: boolean; hidden?: boolean }
    ) => {
      try {
        await workspaceIpc.updatePreference(workspaceKey, preference);
        await loadSessions();
      } catch (err) {
        console.error("Failed to update workspace preference:", err);
        toast.error(t("session.projectActionFailed"));
      }
    },
    [loadSessions, t]
  );

  const handleRevealWorkspace = useCallback(
    async (path: string) => {
      try {
        await fsIpc.revealInExplorer(path, path);
      } catch (err) {
        console.error("Failed to reveal workspace in explorer:", err);
        toast.error(t("workspace:explorer.openInExplorerFailed"));
      }
    },
    [t]
  );

  const handleMessageResultClick = useCallback(
    (sessionId: string) => {
      handleSelect(sessionId);
      setSearchQuery("");
      setSearchResults(null);
      setMessageResults(null);
    },
    [handleSelect]
  );

  const toggleGroupCollapse = useCallback((key: string) => {
    setCollapsedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }, []);

  const isSearching = !!searchQuery.trim();
  const displaySessions = searchResults ?? sessions;
  const grouped = useMemo(
    () => groupSessionsByWorkspace(displaySessions, workspacePreferences),
    [displaySessions, workspacePreferences]
  );
  const archivedGrouped = useMemo(
    () => groupSessionsByWorkspace(archivedSessions, workspacePreferences),
    [archivedSessions, workspacePreferences]
  );

  const renderSessionItem = (session: Session) => (
    <SessionItem
      key={session.id}
      session={session}
      isActive={activeSessionId === session.id}
      groups={groups}
      onSelect={handleSelect}
      onRename={handleRename}
      onDelete={() => setPendingDelete(session)}
      onArchive={handleArchive}
      onTogglePin={handleTogglePin}
      onSetGroup={handleSetGroup}
      onExport={handleExport}
    />
  );

  const totalDisplay = grouped.workspaces.length;

  return (
    <div className="flex h-full min-w-0 w-full flex-col bg-sidebar">
      <SessionPanelHeader
        searchQuery={searchQuery}
        onSearchChange={handleSearch}
        onNewSession={onNewSession}
        newSessionLabel={t("common:newSession")}
      />

      <ScrollArea className="min-h-0 flex-1">
        <div className="space-y-0.5 px-2 py-2">
          {isSearching && messageResults && messageResults.length > 0 && (
            <MessageSearchResults
              results={messageResults}
              onResultClick={handleMessageResultClick}
            />
          )}

          {totalDisplay > 0 ? (
            <WorkspaceGroupedList
              workspaces={grouped.workspaces}
              collapsedGroups={collapsedGroups}
              onToggleCollapse={toggleGroupCollapse}
              renderSessionItem={renderSessionItem}
              defaultWorkspaceLabel={t("workspace:defaultWorkspaceName")}
              onNewTask={handleNewTaskForWorkspace}
              onUpdatePreference={handleUpdateWorkspacePreference}
              onRevealWorkspace={handleRevealWorkspace}
            />
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

          {showArchived && archivedGrouped.workspaces.length > 0 && (
            <WorkspaceGroupedList
              workspaces={archivedGrouped.workspaces}
              collapsedGroups={collapsedGroups}
              onToggleCollapse={toggleGroupCollapse}
              renderSessionItem={renderSessionItem}
              defaultWorkspaceLabel={t("workspace:defaultWorkspaceName")}
              onNewTask={handleNewTaskForWorkspace}
              onUpdatePreference={handleUpdateWorkspacePreference}
              onRevealWorkspace={handleRevealWorkspace}
            />
          )}
        </div>
      </ScrollArea>

      <SessionPanelFooter />
      <Dialog
        open={pendingDelete !== null}
        onOpenChange={(open) => {
          if (!open && !deleting) setPendingDelete(null);
        }}
      >
        <DialogContent showCloseButton={!deleting}>
          <DialogHeader>
            <DialogTitle>{t("session.deleteConfirmTitle")}</DialogTitle>
            <DialogDescription>
              {t("session.deleteConfirm", {
                title: pendingDelete?.title || t("session.newTask"),
              })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              disabled={deleting}
              onClick={() => setPendingDelete(null)}
            >
              {t("common:cancel")}
            </Button>
            <Button
              type="button"
              variant="destructive"
              disabled={deleting || !pendingDelete}
              onClick={() => pendingDelete && void handleDelete(pendingDelete.id)}
            >
              {deleting ? t("common:loading") : t("session.delete")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

function WorkspaceGroupedList({
  workspaces,
  collapsedGroups,
  onToggleCollapse,
  renderSessionItem,
  defaultWorkspaceLabel,
  onNewTask,
  onUpdatePreference,
  onRevealWorkspace,
}: {
  workspaces: WorkspaceSessionGroup[];
  collapsedGroups: Set<string>;
  onToggleCollapse: (key: string) => void;
  renderSessionItem: (session: Session) => ReactNode;
  defaultWorkspaceLabel: string;
  onNewTask: (path: string) => void;
  onUpdatePreference: (
    workspaceKey: string,
    preference: { pinned?: boolean; hidden?: boolean }
  ) => void;
  onRevealWorkspace: (path: string) => void;
}) {
  return (
    <>
      {workspaces.map((workspace) => (
        <WorkspaceBlock
          key={workspace.key}
          workspace={workspace}
          collapsedGroups={collapsedGroups}
          onToggleCollapse={onToggleCollapse}
          renderSessionItem={renderSessionItem}
          defaultWorkspaceLabel={defaultWorkspaceLabel}
          onNewTask={onNewTask}
          onUpdatePreference={onUpdatePreference}
          onRevealWorkspace={onRevealWorkspace}
        />
      ))}
    </>
  );
}

function workspaceDisplayName(
  workspace: WorkspaceSessionGroup,
  defaultWorkspaceLabel: string
): string {
  if (workspace.workspaceKind === "default") return defaultWorkspaceLabel;
  if (workspace.projectName) return workspace.projectName;
  if (workspace.path) return extractDirName(workspace.path);
  return defaultWorkspaceLabel;
}

function WorkspaceBlock({
  workspace,
  collapsedGroups,
  onToggleCollapse,
  renderSessionItem,
  defaultWorkspaceLabel,
  onNewTask,
  onUpdatePreference,
  onRevealWorkspace,
}: {
  workspace: WorkspaceSessionGroup;
  collapsedGroups: Set<string>;
  onToggleCollapse: (key: string) => void;
  renderSessionItem: (session: Session) => ReactNode;
  defaultWorkspaceLabel: string;
  onNewTask: (path: string) => void;
  onUpdatePreference: (
    workspaceKey: string,
    preference: { pinned?: boolean; hidden?: boolean }
  ) => void;
  onRevealWorkspace: (path: string) => void;
}) {
  const workspaceCollapseKey = `ws::${workspace.key}`;
  const workspaceCollapsed = collapsedGroups.has(workspaceCollapseKey);
  const label = workspaceDisplayName(workspace, defaultWorkspaceLabel);
  const { t } = useTranslation("chat");
  const { t: workspaceT } = useTranslation("workspace");

  return (
    <div className="mb-1">
      <GroupHeader
        label={label}
        count={workspace.sessions.length}
        collapsed={workspaceCollapsed}
        onToggle={() => onToggleCollapse(workspaceCollapseKey)}
        icon={<FolderOpen className="size-3 text-muted-foreground/80" />}
        contextMenu={
          <ContextMenuContent>
            <ContextMenuItem
              onSelect={() =>
                onUpdatePreference(workspace.key, { pinned: !workspace.pinned })
              }
            >
              <Pin />
              {workspace.pinned ? t("session.unpinProject") : t("session.pinProject")}
            </ContextMenuItem>
            <ContextMenuItem
              disabled={!workspace.path}
              onSelect={() => workspace.path && onNewTask(workspace.path)}
            >
              <Plus />
              {t("session.newTask")}
            </ContextMenuItem>
            <ContextMenuItem
              disabled={!workspace.path}
              onSelect={() => workspace.path && onRevealWorkspace(workspace.path)}
            >
              <FolderOpen />
              {workspaceT("explorer.openInExplorer")}
            </ContextMenuItem>
            <ContextMenuSeparator />
            <ContextMenuItem
              variant="destructive"
              onSelect={() => onUpdatePreference(workspace.key, { hidden: true })}
            >
              <Trash2 />
              {t("session.removeProject")}
            </ContextMenuItem>
          </ContextMenuContent>
        }
      />
      {!workspaceCollapsed && (
        <>
          {workspace.ungrouped.map(renderSessionItem)}
          {Array.from(workspace.manualGroups.entries()).map(
            ([groupName, items]) => {
              const key = collapseKeyForGroup(workspace.key, groupName);
              return (
                <div key={key}>
                  <GroupHeader
                    label={groupName}
                    count={items.length}
                    collapsed={collapsedGroups.has(key)}
                    onToggle={() => onToggleCollapse(key)}
                  />
                  {!collapsedGroups.has(key) && items.map(renderSessionItem)}
                </div>
              );
            }
          )}
        </>
      )}
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
    <div className="flex shrink-0 flex-col gap-0.5 p-2">
      <Button
        type="button"
        variant="ghost"
        onClick={onNewSession}
        className="h-9 w-full justify-start gap-2 rounded-xl px-3 text-[13px] font-normal text-sidebar-foreground"
      >
        <Plus className="size-4 shrink-0" />
        {newSessionLabel}
      </Button>
      <div className="relative">
        <Search className="pointer-events-none absolute left-3 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground/55" />
        <Input
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder={t("session.searchPlaceholder")}
          className="h-9 rounded-xl border-transparent bg-transparent pl-9 pr-3 text-[13px] shadow-none placeholder:text-muted-foreground/55 hover:bg-sidebar-accent focus-visible:bg-sidebar-accent"
        />
      </div>
    </div>
  );
}

function GroupHeader({
  label,
  count,
  collapsed,
  onToggle,
  icon,
  contextMenu,
}: {
  label: string;
  count: number;
  collapsed: boolean;
  onToggle: () => void;
  icon?: ReactNode;
  contextMenu?: ReactNode;
}) {
  const button = (
    <button
      type="button"
      onClick={onToggle}
      className="flex h-7 w-full items-center gap-1 rounded-xl px-3 text-[13px] font-semibold text-sidebar-foreground/55 transition-colors hover:bg-sidebar-accent/60"
    >
      {collapsed ? (
        <ChevronRight className="size-3 text-muted-foreground/80" />
      ) : (
        <ChevronDown className="size-3 text-muted-foreground/80" />
      )}
      {icon}
      <span className="truncate">{label}</span>
      <span className="ml-auto text-[11px] tabular-nums text-muted-foreground/50">
        {count}
      </span>
    </button>
  );

  if (!contextMenu) return button;

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>{button}</ContextMenuTrigger>
      {contextMenu}
    </ContextMenu>
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
    <div className="px-2.5 py-3 text-[11px] text-muted-foreground/60">
      {hasSearch ? t("session.noMatchingSessions") : t("session.noSessions")}
    </div>
  );
}
