import { cn } from "@/lib/utils";
import type { PendingFileMention } from "@/stores/composer-store";
import { ChipStatusIcon, getFileTypeColorClass } from "./file-type-icon";

interface MentionPillsProps {
  mentions: PendingFileMention[];
}

/**
 * @deprecated 已由 ComposerInlineField 中的行内 chip 替代。
 */
export function MentionPills({ mentions }: MentionPillsProps) {
  if (mentions.length === 0) return null;

  return (
    <div className="flex flex-wrap gap-1.5 px-1 pb-1.5">
      {mentions.map((mention) => (
        <InlineMentionChip key={mention.id} mention={mention} />
      ))}
    </div>
  );
}

interface InlineMentionChipProps {
  mention: PendingFileMention;
}

/**
 * 行内文件引用 chip，对齐 Cursor 暗色块样式。
 * 无删除按钮，通过 Backspace / Delete 键盘操作删除。
 */
export function InlineMentionChip({ mention }: InlineMentionChipProps) {
  const isLoading = mention.status === "loading";
  const isError = mention.status === "error";
  const colorClass = isError ? "text-destructive" : getFileTypeColorClass(mention.name);

  return (
    <span
      className={cn(
        "inline-flex h-6 max-w-[240px] items-center gap-1.5 rounded-md px-2 text-xs",
        "bg-[color:var(--surface-control)] transition-colors duration-[var(--ds-dur-fast)]",
        isLoading && "opacity-60",
        isError && "bg-destructive/8"
      )}
      title={isError ? `${mention.relPath} (read failed)` : mention.relPath}
    >
      <ChipStatusIcon
        isLoading={isLoading}
        isError={isError}
        fileName={mention.name}
      />
      <span className={cn("truncate font-medium", colorClass)}>{mention.name}</span>
    </span>
  );
}
