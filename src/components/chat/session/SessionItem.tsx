import { useState, useCallback, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  MoreHorizontal,
  Pin,
  Pencil,
  Archive,
  ArchiveRestore,
  Trash2,
  FolderOpen,
  Download,
  Plus,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Input } from "@/components/ui/input";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { Session } from "@/lib/ipc";

interface SessionItemProps {
  session: Session;
  isActive: boolean;
  groups: string[];
  onSelect: (id: string) => void;
  onRename: (id: string, title: string) => void;
  onDelete: (id: string) => void;
  onArchive: (id: string, archived: boolean) => void;
  onTogglePin: (id: string, pinned: boolean) => void;
  onSetGroup: (id: string, group: string | null) => void;
  onExport: (id: string) => void;
}

export function SessionItem({
  session,
  isActive,
  groups,
  onSelect,
  onRename,
  onDelete,
  onArchive,
  onTogglePin,
  onSetGroup,
  onExport,
}: SessionItemProps) {
  const { t, i18n } = useTranslation("chat");
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState("");
  const [newGroupInput, setNewGroupInput] = useState("");
  const [isCreatingGroup, setIsCreatingGroup] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const newGroupRef = useRef<HTMLInputElement>(null);

  const isArchived = session.status === "archived";
  const displayTitle = session.title || "New Chat";
  const timeLabel = formatRelativeTime(session.last_message_at ?? session.updated_at, i18n.language);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  useEffect(() => {
    if (isCreatingGroup && newGroupRef.current) {
      newGroupRef.current.focus();
    }
  }, [isCreatingGroup]);

  const handleStartRename = useCallback(() => {
    setEditValue(displayTitle);
    setIsEditing(true);
  }, [displayTitle]);

  const handleCommitRename = useCallback(() => {
    const trimmed = editValue.trim();
    if (trimmed && trimmed !== displayTitle) {
      onRename(session.id, trimmed);
    }
    setIsEditing(false);
  }, [editValue, displayTitle, session.id, onRename]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") {
        handleCommitRename();
      } else if (e.key === "Escape") {
        setIsEditing(false);
      }
    },
    [handleCommitRename]
  );

  const handleNewGroupCommit = useCallback(() => {
    const trimmed = newGroupInput.trim();
    if (trimmed) {
      onSetGroup(session.id, trimmed);
    }
    setNewGroupInput("");
    setIsCreatingGroup(false);
  }, [newGroupInput, session.id, onSetGroup]);

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={() => !isEditing && onSelect(session.id)}
      onKeyDown={(e) => {
        if (e.key === "Enter" && !isEditing) onSelect(session.id);
      }}
      className={cn(
        "group relative flex w-full cursor-pointer items-center gap-2 rounded-[var(--radius-button)] px-2.5 py-2 text-left text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/35 focus-visible:ring-offset-1",
        "transition-[background-color,color] duration-[var(--ds-dur-fast)] ease-out",
        isArchived && "opacity-50",
        isActive
          ? "bg-[color:var(--surface-active)] text-foreground"
          : "text-muted-foreground hover:bg-[color:var(--surface-hover)] hover:text-foreground"
      )}
    >
      {session.pinned && (
        <Pin className="h-3 w-3 shrink-0 text-primary/60" />
      )}

      <div className="min-w-0 flex-1">
        {isEditing ? (
          <Input
            ref={inputRef}
            value={editValue}
            onChange={(e) => setEditValue(e.target.value)}
            onBlur={handleCommitRename}
            onKeyDown={handleKeyDown}
            className="h-6 px-1 py-0 text-sm"
          />
        ) : (
          <p className="truncate text-[0.8125rem] font-medium leading-snug">
            {displayTitle}
          </p>
        )}
        <p className="mt-0.5 truncate text-[0.6875rem] text-muted-foreground/70">
          {session.group_name && (
            <span className="mr-1.5 inline-flex items-center gap-0.5">
              <FolderOpen className="inline h-2.5 w-2.5" />
              {session.group_name}
            </span>
          )}
          {session.project_name && (
            <span className="mr-1.5">{session.project_name}</span>
          )}
          <span>{timeLabel}</span>
        </p>
      </div>

      <DropdownMenu>
        <DropdownMenuTrigger
          onClick={(e) => e.stopPropagation()}
          className={cn(
            "flex h-6 w-6 shrink-0 items-center justify-center rounded-[var(--radius-button)] text-muted-foreground/60 opacity-40 transition-opacity duration-[var(--ds-dur-fast)] hover:bg-[color:var(--surface-hover)] hover:text-foreground",
            "group-hover:opacity-100",
            isActive && "opacity-100"
          )}
        >
          <MoreHorizontal className="h-3.5 w-3.5" />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" sideOffset={4} className="w-44">
          <DropdownMenuItem onClick={handleStartRename}>
            <Pencil className="mr-2 h-3.5 w-3.5" />
            {t("session.rename")}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => onTogglePin(session.id, !session.pinned)}>
            <Pin className="mr-2 h-3.5 w-3.5" />
            {session.pinned ? t("session.unpin") : t("session.pin")}
          </DropdownMenuItem>

          <DropdownMenuSub>
            <DropdownMenuSubTrigger>
              <FolderOpen className="mr-2 h-3.5 w-3.5" />
              {t("session.setGroup")}
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent className="w-40">
              {session.group_name && (
                <DropdownMenuItem onClick={() => onSetGroup(session.id, null)}>
                  {t("session.removeGroup")}
                </DropdownMenuItem>
              )}
              {groups.map((g) => (
                <DropdownMenuItem
                  key={g}
                  onClick={() => onSetGroup(session.id, g)}
                  className={cn(session.group_name === g && "font-semibold")}
                >
                  {g}
                </DropdownMenuItem>
              ))}
              {groups.length > 0 && <DropdownMenuSeparator />}
              {isCreatingGroup ? (
                <div className="px-2 py-1.5" onClick={(e) => e.stopPropagation()}>
                  <Input
                    ref={newGroupRef}
                    value={newGroupInput}
                    onChange={(e) => setNewGroupInput(e.target.value)}
                    onBlur={handleNewGroupCommit}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") handleNewGroupCommit();
                      if (e.key === "Escape") setIsCreatingGroup(false);
                    }}
                    placeholder={t("session.newGroup")}
                    className="h-6 px-1 py-0 text-sm"
                  />
                </div>
              ) : (
                <DropdownMenuItem onClick={() => setIsCreatingGroup(true)}>
                  <Plus className="mr-2 h-3.5 w-3.5" />
                  {t("session.newGroup")}
                </DropdownMenuItem>
              )}
            </DropdownMenuSubContent>
          </DropdownMenuSub>

          <DropdownMenuItem onClick={() => onExport(session.id)}>
            <Download className="mr-2 h-3.5 w-3.5" />
            {t("session.export")}
          </DropdownMenuItem>

          <DropdownMenuItem onClick={() => onArchive(session.id, !isArchived)}>
            {isArchived ? (
              <>
                <ArchiveRestore className="mr-2 h-3.5 w-3.5" />
                {t("session.unarchive")}
              </>
            ) : (
              <>
                <Archive className="mr-2 h-3.5 w-3.5" />
                {t("session.archive")}
              </>
            )}
          </DropdownMenuItem>

          <DropdownMenuSeparator />
          <DropdownMenuItem
            onClick={() => onDelete(session.id)}
            className="text-destructive focus:text-destructive"
          >
            <Trash2 className="mr-2 h-3.5 w-3.5" />
            {t("session.delete")}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}

function formatRelativeTime(dateStr: string, locale = "en"): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60_000);

  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });

  if (diffMin < 1) return rtf.format(0, "second");
  if (diffMin < 60) return rtf.format(-diffMin, "minute");

  const diffHour = Math.floor(diffMin / 60);
  if (diffHour < 24) return rtf.format(-diffHour, "hour");

  const diffDay = Math.floor(diffHour / 24);
  if (diffDay < 7) return rtf.format(-diffDay, "day");

  return date.toLocaleDateString(locale, { month: "short", day: "numeric" });
}
