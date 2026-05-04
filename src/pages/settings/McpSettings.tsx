import { useTranslation } from "react-i18next";
import { Plug } from "lucide-react";

export function McpSettings() {
  const { t } = useTranslation("settings");

  return (
    <div className="max-w-2xl space-y-6">
      <h2 className="text-lg font-semibold text-foreground">
        {t("mcp.title")}
      </h2>
      <div className="flex h-64 flex-col items-center justify-center gap-3 rounded-lg border border-dashed border-border">
        <Plug className="h-10 w-10 text-muted-foreground/50" />
        <p className="text-sm text-muted-foreground">{t("mcp.comingSoon")}</p>
      </div>
    </div>
  );
}
