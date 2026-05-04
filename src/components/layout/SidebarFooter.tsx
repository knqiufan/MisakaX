import { useTranslation } from "react-i18next";
import { Bell, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores";
import { NavItem } from "./NavItem";
import { UserMenu } from "./UserMenu";
import { Separator } from "@/components/ui/separator";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

export function SidebarFooter() {
  const collapsed = useAppStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useAppStore((s) => s.toggleSidebar);
  const { t } = useTranslation("nav");

  const collapseButton = (
    <button
      type="button"
      onClick={toggleSidebar}
      className={cn(
        "flex w-full items-center gap-3 rounded-[var(--radius-button)] px-3 py-2 text-sm font-medium text-muted-foreground transition-[background,color] duration-[var(--ds-dur-fast)] ease-out hover:bg-[color:var(--surface-hover)] hover:text-foreground",
        collapsed && "justify-center px-0"
      )}
    >
      {collapsed ? (
        <PanelLeftOpen className="h-5 w-5 shrink-0" />
      ) : (
        <PanelLeftClose className="h-5 w-5 shrink-0" />
      )}
      {!collapsed && (
        <span className="truncate">
          {collapsed ? t("expand") : t("collapse")}
        </span>
      )}
    </button>
  );

  return (
    <div className="space-y-1 px-2 py-2">
      <Separator className="mb-2" />

      <NavItem
        icon={Bell}
        label={t("notifications")}
        route={{ page: "notifications" }}
        badge={3}
      />

      <UserMenu collapsed={collapsed} />

      {collapsed ? (
        <Tooltip>
          <TooltipTrigger asChild>{collapseButton}</TooltipTrigger>
          <TooltipContent side="right" sideOffset={8}>
            <p>{t("expand")}</p>
          </TooltipContent>
        </Tooltip>
      ) : (
        collapseButton
      )}
    </div>
  );
}
