import { useEffect } from "react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/app-store";
import type { SettingsTab } from "@/stores/app-store";
import { getSettingsNavItem } from "@/components/settings";
import { GeneralSettings } from "./GeneralSettings";
import { ModelSettings } from "./ModelSettings";
import { McpSettings } from "./McpSettings";
import { AppearanceSettings } from "./AppearanceSettings";
import { AboutSettings } from "./AboutSettings";
import { SkillsSettingsFeature } from "@/components/skills/SkillsSettingsFeature";

export function SettingsPage() {
  const route = useAppStore((s) => s.route);
  const navigate = useAppStore((s) => s.navigate);

  const activeTab: SettingsTab =
    route.page === "settings" && route.tab ? route.tab : "general";
  const activeNav = getSettingsNavItem(activeTab);

  useEffect(() => {
    if (route.page === "settings" && !route.tab) {
      navigate({ page: "settings", tab: "general" });
    }
  }, [route, navigate]);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div
        className={cn(
          "min-h-0 min-w-0 flex-1 p-6 lg:p-10",
          activeNav.contentWidth === "wide"
            ? "overflow-auto lg:overflow-hidden"
            : "overflow-auto",
        )}
      >
        <div
          className={cn(
            "mx-auto w-full",
            activeNav.contentWidth === "wide"
              ? "h-full min-h-0 max-w-6xl"
              : activeNav.normalMaxWidth === "3xl"
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

function SettingsContent({ tab }: { tab: SettingsTab }) {
  switch (tab) {
    case "general":
      return <GeneralSettings />;
    case "models":
      return <ModelSettings />;
    case "mcp":
      return <McpSettings />;
    case "skills":
      return <SkillsSettingsFeature />;
    case "appearance":
      return <AppearanceSettings />;
    case "about":
      return <AboutSettings />;
  }
}
