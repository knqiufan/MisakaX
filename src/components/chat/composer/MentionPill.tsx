import { AtSign, X } from "lucide-react";
import { cn } from "@/lib/utils";
import type { PendingFileMention } from "@/stores/composer-store";

interface MentionPillsProps {
  mentions: PendingFileMention[];
  onRemove: (id: string) => void;
  removeLabel: (name: string) => string;
}

/**
 * 在 Composer 顶部以「@xxx.tsx」chip 形式展示已引用的工作区文件。
 * 设计基线与 Cursor Agent 一致：
 * - 圆角药丸（pill），紧凑内边距；
 * - 前置 `@` 图标 + 文件名，悬停或聚焦时显示 X 关闭；
 * - 颜色使用 Agent 强调蓝色 token，避免与普通文本附件混淆。
 */
export function MentionPills({ mentions, onRemove, removeLabel }: MentionPillsProps) {
  if (mentions.length === 0) return null;

  return (
    <div className="flex flex-wrap gap-1.5 px-1 pb-1.5">
      {mentions.map((mention) => (
        <MentionPill
          key={mention.id}
          mention={mention}
          onRemove={() => onRemove(mention.id)}
          removeLabel={removeLabel(mention.name)}
        />
      ))}
    </div>
  );
}

interface MentionPillProps {
  mention: PendingFileMention;
  onRemove: () => void;
  removeLabel: string;
}

function MentionPill({ mention, onRemove, removeLabel }: MentionPillProps) {
  return (
    <span
      className={cn(
        "group/pill inline-flex h-6 max-w-[220px] items-center gap-1 rounded-full",
        "border border-[color:var(--border-muted)] bg-[color:var(--surface-card-strong)]",
        "pl-1.5 pr-1 text-xs transition-colors duration-[var(--ds-dur-fast)]",
        "hover:border-[color:var(--border-strong)]"
      )}
      title={mention.relPath}
    >
      <AtSign className="size-3 text-primary/80" />
      <span className="truncate font-medium text-foreground">{mention.name}</span>
      <button
        type="button"
        onClick={onRemove}
        aria-label={removeLabel}
        className={cn(
          "flex size-4 shrink-0 items-center justify-center rounded-full",
          "text-muted-foreground/80 transition-colors duration-[var(--ds-dur-fast)]",
          "hover:bg-[color:var(--surface-control-hover)] hover:text-foreground"
        )}
      >
        <X className="size-2.5" />
      </button>
    </span>
  );
}
