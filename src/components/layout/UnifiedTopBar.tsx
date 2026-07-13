import { useTranslation } from "react-i18next";
import { PanelLeftClose, PanelLeftOpen, ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useAppStore } from "@/stores/app-store";
import { cn } from "@/lib/utils";

const PAGE_TITLE_KEYS: Record<string, string> = {
  chat: "nav:chat",
  skills: "nav:skills",
  knowledge: "nav:knowledge",
  dashboard: "nav:dashboard",
  notifications: "nav:notifications",
  settings: "nav:settings",
};

export function UnifiedTopBar() {
  const { t } = useTranslation();
  const route = useAppStore((s) => s.route);
  const sessionListOpen = useAppStore((s) => s.sessionListOpen);
  const toggleSessionList = useAppStore((s) => s.toggleSessionList);
  const navigate = useAppStore((s) => s.navigate);

  const isChat = route.page === "chat";
  const isSettings = route.page === "settings";
  const titleKey = PAGE_TITLE_KEYS[route.page] ?? "nav:chat";

  return (
    <header
      className={cn(
        "flex h-10 shrink-0 items-center gap-2 border-b border-border bg-[color:var(--surface-topbar)] pl-3 pr-3"
      )}
    >
      <div className="flex min-w-0 items-center gap-2">
        {isChat ? (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                onClick={toggleSessionList}
                aria-label={
                  sessionListOpen
                    ? t("common:collapseSessionList", { defaultValue: "收起会话列表" })
                    : t("common:expandSessionList", { defaultValue: "展开会话列表" })
                }
                className="size-8 text-muted-foreground"
              >
                {sessionListOpen ? (
                  <PanelLeftClose className="size-4" />
                ) : (
                  <PanelLeftOpen className="size-4" />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              {sessionListOpen ? t("common:collapseSessionList") : t("common:expandSessionList")}
            </TooltipContent>
          </Tooltip>
        ) : null}

        {isSettings ? (
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
        ) : null}

        {!isChat ? (
          <h1 className="truncate text-sm font-medium text-foreground">
            {t(titleKey)}
          </h1>
        ) : null}
      </div>

      <div className="flex-1" />
    </header>
  );
}
