import { useTranslation } from "react-i18next";
import { Bell, Settings } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/app-store";
import { UserMenu } from "@/components/layout/UserMenu";

export function SessionPanelFooter() {
  const { t } = useTranslation("nav");
  const route = useAppStore((s) => s.route);
  const navigate = useAppStore((s) => s.navigate);

  return (
    <div className="shrink-0 space-y-0.5 border-t border-sidebar-border p-2">
      <FooterNavRow
        icon={Bell}
        label={t("notifications")}
        active={route.page === "notifications"}
        onClick={() => navigate({ page: "notifications" })}
        badge={3}
      />

      <UserMenu />

      <FooterNavRow
        icon={Settings}
        label={t("settings")}
        active={route.page === "settings"}
        onClick={() => navigate({ page: "settings" })}
      />
    </div>
  );
}

function FooterNavRow({
  icon: Icon,
  label,
  active,
  onClick,
  badge,
}: {
  icon: LucideIcon;
  label: string;
  active: boolean;
  onClick: () => void;
  badge?: number;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-current={active ? "page" : undefined}
      className={cn(
        "relative flex h-9 w-full items-center gap-2 rounded-xl px-3 text-[13px] font-normal outline-none transition-colors duration-150",
        "focus-visible:ring-2 focus-visible:ring-ring/35",
        active
          ? "bg-sidebar-accent font-medium text-sidebar-accent-foreground"
          : "text-sidebar-foreground hover:bg-sidebar-accent/60"
      )}
    >
      <Icon className="size-4 shrink-0" strokeWidth={active ? 2 : 1.75} />
      <span className="truncate">{label}</span>
      {badge !== undefined && badge > 0 ? (
        <span className="bg-destructive text-destructive-foreground ml-auto flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-[10px]">
          {badge > 99 ? "99+" : badge}
        </span>
      ) : null}
    </button>
  );
}
