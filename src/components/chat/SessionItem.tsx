import { useState, useCallback, useRef, useEffect } from "react";
import { MoreHorizontal, Pin, Pencil, Archive, Trash2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { Input } from "@/components/ui/input";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { Session } from "@/lib/ipc";

interface SessionItemProps {
  session: Session;
  isActive: boolean;
  onSelect: (id: string) => void;
  onRename: (id: string, title: string) => void;
  onDelete: (id: string) => void;
  onArchive: (id: string) => void;
  onTogglePin: (id: string, pinned: boolean) => void;
}

export function SessionItem({
  session,
  isActive,
  onSelect,
  onRename,
  onDelete,
  onArchive,
  onTogglePin,
}: SessionItemProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  const displayTitle = session.title || "New Chat";
  const timeLabel = formatRelativeTime(session.last_message_at ?? session.updated_at);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

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

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={() => !isEditing && onSelect(session.id)}
      onKeyDown={(e) => {
        if (e.key === "Enter" && !isEditing) onSelect(session.id);
      }}
      className={cn(
        "group relative flex w-full cursor-pointer items-center gap-2 rounded-[var(--radius-button)] px-2.5 py-2 text-left text-sm outline-none",
        "transition-[background-color,color] duration-[var(--ds-dur-fast)] ease-out",
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
            "flex h-6 w-6 shrink-0 items-center justify-center rounded-[var(--radius-button)] text-muted-foreground/60 opacity-0 transition-opacity duration-[var(--ds-dur-fast)] hover:bg-[color:var(--surface-hover)] hover:text-foreground",
            "group-hover:opacity-100",
            isActive && "opacity-100"
          )}
        >
          <MoreHorizontal className="h-3.5 w-3.5" />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" sideOffset={4} className="w-40">
          <DropdownMenuItem onClick={handleStartRename}>
            <Pencil className="mr-2 h-3.5 w-3.5" />
            重命名
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => onTogglePin(session.id, !session.pinned)}>
            <Pin className="mr-2 h-3.5 w-3.5" />
            {session.pinned ? "取消置顶" : "置顶"}
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => onArchive(session.id)}>
            <Archive className="mr-2 h-3.5 w-3.5" />
            归档
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            onClick={() => onDelete(session.id)}
            className="text-destructive focus:text-destructive"
          >
            <Trash2 className="mr-2 h-3.5 w-3.5" />
            删除
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}

function formatRelativeTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60_000);

  if (diffMin < 1) return "刚刚";
  if (diffMin < 60) return `${diffMin}分钟前`;

  const diffHour = Math.floor(diffMin / 60);
  if (diffHour < 24) return `${diffHour}小时前`;

  const diffDay = Math.floor(diffHour / 24);
  if (diffDay < 7) return `${diffDay}天前`;

  return date.toLocaleDateString("zh-CN", { month: "short", day: "numeric" });
}
