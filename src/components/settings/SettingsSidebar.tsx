import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/app-store";
import type { SettingsTab } from "@/stores/app-store";
import { SETTINGS_NAV, type SettingsNavItem } from "./nav-config";

export function SettingsSidebar() {
  const route = useAppStore((s) => s.route);
  const navigate = useAppStore((s) => s.navigate);
  const { t } = useTranslation();

  const activeTab: SettingsTab =
    route.page === "settings" && route.tab ? route.tab : "general";

  return (
    <nav
      className="flex h-full min-w-0 w-full flex-col bg-sidebar p-2"
      aria-label={t("nav:settings", "Settings")}
    >
      <div className="flex flex-col gap-0.5">
        {SETTINGS_NAV.map((tab) => (
          <SettingsNavButton
            key={tab.id}
            tab={tab}
            active={activeTab === tab.id}
            onClick={() => navigate({ page: "settings", tab: tab.id })}
            label={t(tab.labelKey)}
          />
        ))}
      </div>
    </nav>
  );
}

function SettingsNavButton({
  tab,
  active,
  onClick,
  label,
}: {
  tab: SettingsNavItem;
  active: boolean;
  onClick: () => void;
  label: string;
}) {
  const Icon = tab.icon;
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "flex h-9 w-full items-center gap-2 rounded-xl px-3 text-[13px] transition-colors duration-[var(--ds-dur-fast)] ease-out",
        active
          ? "bg-sidebar-accent font-medium text-sidebar-accent-foreground"
          : "font-normal text-sidebar-foreground hover:bg-sidebar-accent/60"
      )}
    >
      <Icon
        className="size-4 shrink-0"
        strokeWidth={active ? 2 : 1.75}
        aria-hidden
      />
      <span className="truncate">{label}</span>
    </button>
  );
}
