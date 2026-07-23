import { cn } from "@/lib/utils";
import type { PendingFileMention, PendingSkill } from "@/stores/composer-store";
import { ChipStatusIcon, getFileTypeColorClass } from "./FileTypeIcon";
import { Sparkles } from "lucide-react";

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
 * 行内文件引用 chip，primary 低彩底。
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
          "border border-primary/25 bg-primary/8 transition-colors duration-[var(--ds-dur-fast)]",
          isLoading && "opacity-60",
          isError && "border-destructive/40 bg-destructive/8 text-destructive"
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

/**
 * 行内 Skill chip 遵循文件引用的低彩 primary 语义；
 * 键盘 Backspace / Delete 是唯一的删除操作。
 */
export function InlineSkillChip({ skill }: { skill: PendingSkill }) {
  return (
    <span
      className={cn(
        "inline-flex h-6 max-w-[240px] items-center gap-1.5 rounded-md px-2 text-xs",
        "border border-primary/25 bg-primary/8 text-primary",
        "transition-colors duration-[var(--ds-dur-fast)]"
      )}
      title={skill.description}
    >
      <Sparkles className="size-3 shrink-0" />
      <span className="truncate font-medium">{skill.name}</span>
    </span>
  );
}
