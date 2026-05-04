import { useTranslation } from "react-i18next";
import { Plug } from "lucide-react";

export function McpSettings() {
  const { t } = useTranslation("settings");

  return (
    <div className="w-full min-w-0 space-y-6">
      <div
        className="flex min-h-64 w-full flex-col items-center justify-center gap-3 rounded-[var(--radius-ui-lg)] border border-dashed border-[color:var(--border-strong)] bg-[color:var(--surface-card)]/40 px-4 py-8"
      >
        <Plug className="h-10 w-10 text-muted-foreground/50" />
        <p className="max-w-lg text-center text-sm text-muted-foreground">
          {t("mcp.comingSoon")}
        </p>
      </div>
    </div>
  );
}
