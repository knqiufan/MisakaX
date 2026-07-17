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
  const [menuOpen, setMenuOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const newGroupRef = useRef<HTMLInputElement>(null);

  const isArchived = session.status === "archived";
  const displayTitle = session.title || t("session.newTask");
  const timeLabel = formatCompactTime(
    session.last_message_at ?? session.updated_at,
    i18n.language
  );

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
        "group relative flex h-8 w-full cursor-pointer items-center gap-2 rounded-xl px-3 text-left outline-none transition-all duration-150",
        "focus-visible:ring-2 focus-visible:ring-ring/35",
        isArchived && "opacity-50",
        isActive
          ? "bg-sidebar-accent text-sidebar-accent-foreground"
          : "text-sidebar-foreground hover:bg-sidebar-accent"
      )}
    >
      {session.pinned ? (
        <Pin className="size-3 shrink-0 text-muted-foreground/70" />
      ) : null}

      <div className="min-w-0 flex-1">
        {isEditing ? (
          <Input
            ref={inputRef}
            value={editValue}
            onChange={(e) => setEditValue(e.target.value)}
            onBlur={handleCommitRename}
            onKeyDown={handleKeyDown}
            className="h-6 rounded-md px-1 py-0 text-[13px]"
            onClick={(e) => e.stopPropagation()}
          />
        ) : (
          <p className="line-clamp-1 text-[13px] font-normal leading-tight">
            {displayTitle}
          </p>
        )}
      </div>

      <div className="relative h-5 w-[38px] shrink-0">
        <span
          className={cn(
            "absolute inset-0 flex items-center justify-end text-[11px] text-muted-foreground/40 transition-opacity duration-150",
            menuOpen || isActive ? "opacity-0" : "group-hover:opacity-0"
          )}
        >
          {timeLabel}
        </span>

        <DropdownMenu open={menuOpen} onOpenChange={setMenuOpen}>
          <DropdownMenuTrigger
            type="button"
            aria-label={t("session.moreActions")}
            onClick={(e) => e.stopPropagation()}
            className={cn(
              "absolute inset-y-0 right-0 flex h-5 w-5 items-center justify-center rounded-md text-muted-foreground/60 opacity-0 transition-opacity duration-150 hover:bg-muted hover:text-foreground",
              "group-hover:opacity-100",
              (menuOpen || isActive) && "opacity-100"
            )}
          >
            <MoreHorizontal className="size-3.5" />
          </DropdownMenuTrigger>
        <DropdownMenuContent align="end" sideOffset={4} className="min-w-[160px]">
          <DropdownMenuItem onClick={handleStartRename}>
            <Pencil className="mr-2 size-3.5" />
            {t("session.rename")}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => onTogglePin(session.id, !session.pinned)}>
            <Pin className="mr-2 size-3.5" />
            {session.pinned ? t("session.unpin") : t("session.pin")}
          </DropdownMenuItem>

          <DropdownMenuSub>
            <DropdownMenuSubTrigger>
              <FolderOpen className="mr-2 size-3.5" />
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
                  <Plus className="mr-2 size-3.5" />
                  {t("session.newGroup")}
                </DropdownMenuItem>
              )}
            </DropdownMenuSubContent>
          </DropdownMenuSub>

          <DropdownMenuItem onClick={() => onExport(session.id)}>
            <Download className="mr-2 size-3.5" />
            {t("session.export")}
          </DropdownMenuItem>

          <DropdownMenuItem onClick={() => onArchive(session.id, !isArchived)}>
            {isArchived ? (
              <>
                <ArchiveRestore className="mr-2 size-3.5" />
                {t("session.unarchive")}
              </>
            ) : (
              <>
                <Archive className="mr-2 size-3.5" />
                {t("session.archive")}
              </>
            )}
          </DropdownMenuItem>

          <DropdownMenuSeparator />
          <DropdownMenuItem
            onClick={() => onDelete(session.id)}
            className="text-destructive focus:text-destructive"
          >
            <Trash2 className="mr-2 size-3.5" />
            {t("session.delete")}
          </DropdownMenuItem>
        </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
}

function formatCompactTime(dateStr: string, locale = "en"): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60_000);

  if (diffMin < 1) return locale.startsWith("zh") ? "刚刚" : "now";
  if (diffMin < 60) return `${diffMin}m`;

  const diffHour = Math.floor(diffMin / 60);
  if (diffHour < 24) return `${diffHour}h`;

  const diffDay = Math.floor(diffHour / 24);
  if (diffDay < 7) return `${diffDay}d`;

  return date.toLocaleDateString(locale, { month: "short", day: "numeric" });
}
