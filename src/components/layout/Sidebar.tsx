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
        "relative flex min-h-0 shrink-0 flex-col border-r border-sidebar-border",
        "border-[color:var(--border-muted)] bg-sidebar",
        "backdrop-blur-xl backdrop-saturate-150 [transition:width_var(--ds-dur-slow)_var(--ds-ease-out)]",
        collapsed ? "w-14" : "w-[160px]"
      )}
    >
      {/* <SidebarHeader /> */}

      <nav className="min-h-0 flex-1 space-y-1 overflow-y-auto px-2 py-2">
        <NavItem icon={MessageSquare} label={t("chat")} route={{ page: "chat" }} />
        <NavItem icon={Sparkles} label={t("skills")} route={{ page: "skills" }} />
        <NavItem icon={BookOpen} label={t("knowledge")} route={{ page: "knowledge" }} />
        <NavItem icon={LayoutDashboard} label={t("dashboard")} route={{ page: "dashboard" }} />
      </nav>

      <SidebarFooter />
    </aside>
  );
}
