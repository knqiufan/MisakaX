import { useTranslation } from "react-i18next";
import { MessageSquare, Sparkles, BookOpen, LayoutDashboard } from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/app-store";
import { SidebarFooter } from "./SidebarFooter";
import { NavItem } from "./NavItem";

export function Sidebar() {
  const collapsed = useAppStore((s) => s.sidebarCollapsed);
  const { t } = useTranslation("nav");

  return (
    <aside
      className={cn(
        "relative flex min-h-0 shrink-0 flex-col border-r border-sidebar-border bg-sidebar",
        "transition-[width] duration-[var(--ds-dur-slow)] ease-out",
        collapsed ? "w-14" : "w-[160px]"
      )}
    >
      <nav className="min-h-0 flex-1 space-y-0.5 overflow-y-auto p-2">
        <NavItem icon={MessageSquare} label={t("chat")} route={{ page: "chat" }} />
        <NavItem icon={Sparkles} label={t("skills")} route={{ page: "skills" }} />
        <NavItem icon={BookOpen} label={t("knowledge")} route={{ page: "knowledge" }} />
        <NavItem icon={LayoutDashboard} label={t("dashboard")} route={{ page: "dashboard" }} />
      </nav>

      <SidebarFooter />
    </aside>
  );
}
