import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  Settings,
  Bot,
  Plug,
  Palette,
  Info,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores";
import type { SettingsTab } from "@/stores/app-store";
import { GeneralSettings } from "./GeneralSettings";
import { ModelSettings } from "./ModelSettings";
import { McpSettings } from "./McpSettings";
import { AppearanceSettings } from "./AppearanceSettings";
import { AboutSettings } from "./AboutSettings";

interface TabDef {
  id: SettingsTab;
  labelKey: string;
  icon: typeof Settings;
}

const TABS: TabDef[] = [
  { id: "general", labelKey: "settings:general.title", icon: Settings },
  { id: "models", labelKey: "settings:models.title", icon: Bot },
  { id: "mcp", labelKey: "settings:mcp.title", icon: Plug },
  { id: "appearance", labelKey: "settings:appearance.title", icon: Palette },
  { id: "about", labelKey: "settings:about.title", icon: Info },
];

export function SettingsPage() {
  const route = useAppStore((s) => s.route);
  const navigate = useAppStore((s) => s.navigate);
  const { t } = useTranslation();

  const activeTab: SettingsTab =
    route.page === "settings" && route.tab ? route.tab : "general";

  useEffect(() => {
    if (route.page === "settings" && !route.tab) {
      navigate({ page: "settings", tab: "general" });
    }
  }, [route, navigate]);

  return (
    <div className="flex h-full">
      <nav className="w-48 shrink-0 space-y-1 border-r border-border p-4">
        {TABS.map((tab) => (
          <TabButton
            key={tab.id}
            tab={tab}
            active={activeTab === tab.id}
            onClick={() => navigate({ page: "settings", tab: tab.id })}
            label={t(tab.labelKey)}
          />
        ))}
      </nav>

      <div className="flex-1 overflow-auto p-6">
        <SettingsContent tab={activeTab} />
      </div>
    </div>
  );
}

function TabButton({
  tab,
  active,
  onClick,
  label,
}: {
  tab: TabDef;
  active: boolean;
  onClick: () => void;
  label: string;
}) {
  const Icon = tab.icon;
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors duration-150",
        active
          ? "bg-primary/10 text-primary"
          : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
      )}
    >
      <Icon className="h-4 w-4 shrink-0" />
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
