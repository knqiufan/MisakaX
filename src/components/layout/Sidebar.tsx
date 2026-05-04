import { useTranslation } from "react-i18next";
import { MessageSquare, Sparkles, BookOpen, LayoutDashboard } from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores";
import { SidebarHeader } from "./SidebarHeader";
import { SidebarFooter } from "./SidebarFooter";
import { NavItem } from "./NavItem";

export function Sidebar() {
  const collapsed = useAppStore((s) => s.sidebarCollapsed);
  const { t } = useTranslation("nav");

  return (
    <aside
      className={cn(
        "flex flex-col border-r border-border bg-sidebar transition-all duration-300 ease-out",
        collapsed ? "w-14" : "w-60"
      )}
    >
      <SidebarHeader />

      <nav className="flex-1 space-y-1 px-2 py-2">
        <NavItem icon={MessageSquare} label={t("chat")} route={{ page: "chat" }} />
        <NavItem icon={Sparkles} label={t("skills")} route={{ page: "skills" }} />
        <NavItem icon={BookOpen} label={t("knowledge")} route={{ page: "knowledge" }} />
        <NavItem icon={LayoutDashboard} label={t("dashboard")} route={{ page: "dashboard" }} />
      </nav>

      <SidebarFooter />
    </aside>
  );
}
