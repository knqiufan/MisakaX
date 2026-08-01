import {
  Settings,
  Bot,
  Plug,
  Puzzle,
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
  contentWidth: "normal" | "wide";
  /** Preserves the established reading width of non-wide tabs. */
  normalMaxWidth?: "3xl" | "4xl";
}

export const SETTINGS_NAV: SettingsNavItem[] = [
  {
    id: "general",
    labelKey: "settings:general.title",
    icon: Settings,
    contentWidth: "normal",
    normalMaxWidth: "4xl",
  },
  {
    id: "models",
    labelKey: "settings:models.title",
    icon: Bot,
    contentWidth: "normal",
    normalMaxWidth: "4xl",
  },
  {
    id: "mcp",
    labelKey: "settings:mcp.title",
    icon: Plug,
    contentWidth: "normal",
    normalMaxWidth: "4xl",
  },
  {
    id: "skills",
    labelKey: "settings:skills.title",
    icon: Puzzle,
    contentWidth: "wide",
  },
  {
    id: "appearance",
    labelKey: "settings:appearance.title",
    icon: Palette,
    contentWidth: "normal",
    normalMaxWidth: "3xl",
  },
  {
    id: "about",
    labelKey: "settings:about.title",
    icon: Info,
    contentWidth: "normal",
    normalMaxWidth: "3xl",
  },
];

export function getSettingsNavItem(id: SettingsTab): SettingsNavItem {
  return SETTINGS_NAV.find((item) => item.id === id) ?? SETTINGS_NAV[0];
}
