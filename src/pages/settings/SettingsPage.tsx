import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/app-store";
import type { SettingsTab } from "@/stores/app-store";
import {
  SETTINGS_NAV,
  getSettingsNavItem,
  type SettingsNavItem,
} from "@/components/settings";
import { GeneralSettings } from "./GeneralSettings";
import { ModelSettings } from "./ModelSettings";
import { McpSettings } from "./McpSettings";
import { AppearanceSettings } from "./AppearanceSettings";
import { AboutSettings } from "./AboutSettings";

export function SettingsPage() {
  const route = useAppStore((s) => s.route);
  const navigate = useAppStore((s) => s.navigate);
  const { t } = useTranslation();

  const activeTab: SettingsTab =
    route.page === "settings" && route.tab ? route.tab : "general";
  const activeNav = getSettingsNavItem(activeTab);

  useEffect(() => {
    if (route.page === "settings" && !route.tab) {
      navigate({ page: "settings", tab: "general" });
    }
  }, [route, navigate]);

  const goToTab = (id: SettingsTab) => {
    navigate({ page: "settings", tab: id });
  };

  return (
    <div className="flex h-full min-h-0 flex-col lg:flex-row">
      <nav
        className={cn(
          "hidden w-52 shrink-0 flex-col border-r border-border/50 bg-sidebar p-2 lg:flex"
        )}
        aria-label={t("nav:settings", "Settings")}
      >
        <div className="flex flex-col gap-0.5">
          {SETTINGS_NAV.map((tab) => (
            <DesktopNavButton
              key={tab.id}
              tab={tab}
              active={activeTab === tab.id}
              onClick={() => goToTab(tab.id)}
              label={t(tab.labelKey)}
            />
          ))}
        </div>
      </nav>

      <nav
        className="flex gap-1 overflow-x-auto border-b border-border/50 px-3 py-2 lg:hidden"
        aria-label={t("nav:settings", "Settings")}
      >
        {SETTINGS_NAV.map((tab) => (
          <MobileNavPill
            key={tab.id}
            tab={tab}
            active={activeTab === tab.id}
            onClick={() => goToTab(tab.id)}
            label={t(tab.labelKey)}
          />
        ))}
      </nav>

      <div className="min-h-0 min-w-0 flex-1 overflow-auto p-6 lg:p-10">
        <div
          className={cn(
            "mx-auto w-full",
            activeNav.contentWidth === "3xl"
              ? "max-w-3xl space-y-6"
              : "max-w-4xl space-y-10"
          )}
        >
          <SettingsContent tab={activeTab} />
        </div>
      </div>
    </div>
  );
}

function DesktopNavButton({
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

function MobileNavPill({
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
        "inline-flex shrink-0 items-center gap-2 rounded-full px-3 py-1.5 text-sm font-medium transition-colors duration-[var(--ds-dur-fast)] ease-out",
        active
          ? "bg-sidebar-accent text-sidebar-accent-foreground"
          : "text-muted-foreground hover:bg-sidebar-accent hover:text-foreground"
      )}
    >
      <Icon className="size-3.5 shrink-0" aria-hidden />
      <span>{label}</span>
    </button>
  );
}

function SettingsContent({ tab }: { tab: SettingsTab }) {
  switch (tab) {
    case "general":
      return <GeneralSettings />;
    case "models":
      return <ModelSettings />;
    case "mcp":
      return <McpSettings />;
    case "appearance":
      return <AppearanceSettings />;
    case "about":
      return <AboutSettings />;
  }
}
