import { useTranslation } from "react-i18next";
import { Sun, Moon, Monitor } from "lucide-react";
import { cn } from "@/lib/utils";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import {
  SettingsCard,
  FieldRow,
  SettingsSectionHeader,
} from "@/components/settings";
import { useThemeStore } from "@/stores/theme-store";
import type { ThemeMode } from "@/lib/theme";

const THEME_OPTIONS: {
  mode: ThemeMode;
  labelKey: string;
  icon: typeof Sun;
}[] = [
  { mode: "light", labelKey: "appearance.themeLight", icon: Sun },
  { mode: "dark", labelKey: "appearance.themeDark", icon: Moon },
  { mode: "system", labelKey: "appearance.themeSystem", icon: Monitor },
];

export function AppearanceSettings() {
  const { t } = useTranslation("settings");
  const themeMode = useThemeStore((s) => s.mode);
  const setMode = useThemeStore((s) => s.setMode);
  const reducedTransparency = useThemeStore((s) => s.reducedTransparency);
  const setReducedTransparency = useThemeStore((s) => s.setReducedTransparency);
  const uiFontSize = useThemeStore((s) => s.uiFontSize);
  const setUiFontSize = useThemeStore((s) => s.setUiFontSize);

  return (
    <div className="space-y-6">
      <SettingsSectionHeader title={t("appearance.title")} />

      <SettingsCard
        title={t("appearance.theme")}
        description={t("appearance.themeDesc")}
      >
        <div
          className="inline-flex rounded-md bg-muted p-0.5"
          role="tablist"
          aria-label={t("appearance.theme")}
        >
          {THEME_OPTIONS.map(({ mode, labelKey, icon: Icon }) => {
            const active = themeMode === mode;
            return (
              <button
                key={mode}
                type="button"
                role="tab"
                aria-selected={active}
                onClick={() => setMode(mode)}
                className={cn(
                  "inline-flex items-center gap-1.5 rounded-sm px-2.5 py-1 text-xs transition-colors duration-[var(--ds-dur-fast)] ease-out",
                  active
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                <Icon className="size-3.5 shrink-0" aria-hidden />
                <span>{t(labelKey)}</span>
              </button>
            );
          })}
        </div>
      </SettingsCard>

      <SettingsCard>
        <div className="space-y-4">
          <FieldRow
            label={t("appearance.reducedTransparency")}
            description={t("appearance.reducedTransparencyDesc")}
          >
            <Switch
              checked={reducedTransparency}
              onCheckedChange={setReducedTransparency}
              aria-label={
                reducedTransparency
                  ? t("appearance.reducedTransparencyOn")
                  : t("appearance.reducedTransparencyOff")
              }
            />
          </FieldRow>

          <FieldRow
            label={t("appearance.fontSize")}
            description={t("appearance.fontSizeDesc", { size: uiFontSize })}
            separator
          >
            <div className="flex items-center gap-3">
              <span className="text-[10px] text-muted-foreground">12</span>
              <Slider
                value={[uiFontSize]}
                onValueChange={(v) => setUiFontSize(v[0] ?? 14)}
                min={12}
                max={18}
                step={1}
                className="w-36"
                aria-label={t("appearance.fontSize")}
              />
              <span className="text-[10px] text-muted-foreground">18</span>
            </div>
          </FieldRow>
        </div>
      </SettingsCard>
    </div>
  );
}
