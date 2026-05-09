import { useTranslation } from "react-i18next";
import { MessageSquare } from "lucide-react";

export function ChatEmptyState() {
  const { t } = useTranslation("chat");

  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 px-8 py-12 text-center">
      <div className="flex size-16 items-center justify-center rounded-[var(--radius-ui-xl)] border border-[color:var(--border-subtle)] bg-[color:var(--surface-card)] shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]">
        <MessageSquare
          className="size-8 text-muted-foreground/60"
          strokeWidth={1.15}
        />
      </div>
      <div className="max-w-md space-y-2">
        <h2 className="text-[1rem] font-semibold tracking-tight text-foreground">
          {t("emptyTitle")}
        </h2>
        <p className="text-[0.8125rem] leading-relaxed text-muted-foreground">
          {t("emptyDescription")}
        </p>
      </div>
    </div>
  );
}
