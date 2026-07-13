import {
  Settings,
  Bot,
  Plug,
  Palette,
  Info,
  type LucideIcon,
} from "lucide-react";
import type { SettingsTab } from "@/stores/app-store";

export interface SettingsNavItem {
  id: SettingsTab;
  labelKey: string;
  icon: LucideIcon;
  /** Page content width ladder from docs/ui/04-settings.md */
  contentWidth: "4xl" | "3xl";
}

export const SETTINGS_NAV: SettingsNavItem[] = [
  {
    id: "general",
    labelKey: "settings:general.title",
    icon: Settings,
    contentWidth: "4xl",
  },
  {
    id: "models",
    labelKey: "settings:models.title",
    icon: Bot,
    contentWidth: "4xl",
  },
  {
    id: "mcp",
    labelKey: "settings:mcp.title",
    icon: Plug,
    contentWidth: "4xl",
  },
  {
    id: "appearance",
    labelKey: "settings:appearance.title",
    icon: Palette,
    contentWidth: "3xl",
  },
  {
    id: "about",
    labelKey: "settings:about.title",
    icon: Info,
    contentWidth: "3xl",
  },
];

export function getSettingsNavItem(id: SettingsTab): SettingsNavItem {
  return SETTINGS_NAV.find((item) => item.id === id) ?? SETTINGS_NAV[0];
}
