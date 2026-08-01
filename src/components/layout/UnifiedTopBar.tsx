import { useTranslation } from "react-i18next";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useAppStore } from "@/stores/app-store";
import { cn } from "@/lib/utils";

const PAGE_TITLE_KEYS: Record<string, string> = {
  chat: "nav:chat",
  knowledge: "nav:knowledge",
  dashboard: "nav:dashboard",
  notifications: "nav:notifications",
  settings: "nav:settings",
};

/** Shell chrome for non-chat routes only. Chat uses WorkspaceBar / hero — no empty top strip. */
export function UnifiedTopBar() {
  const { t } = useTranslation();
  const route = useAppStore((s) => s.route);
  const navigate = useAppStore((s) => s.navigate);

  if (route.page === "chat") {
    return null;
  }

  const titleKey = PAGE_TITLE_KEYS[route.page] ?? "nav:chat";

  return (
    <header
      className={cn(
        "flex h-10 shrink-0 items-center gap-2 border-b border-border bg-[color:var(--surface-topbar)] pl-3 pr-3"
      )}
    >
      <div className="flex min-w-0 items-center gap-2">
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={() => navigate({ page: "chat" })}
          className="h-7 gap-1 px-2 text-xs"
        >
          <ArrowLeft className="size-3.5" />
          {t("common:back", { defaultValue: "返回" })}
        </Button>
        <h1 className="truncate text-sm font-medium text-foreground">
          {t(titleKey)}
        </h1>
      </div>
      <div className="flex-1" />
    </header>
  );
}
