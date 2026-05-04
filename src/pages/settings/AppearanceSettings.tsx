import { useTranslation } from "react-i18next";
import { Sun, Moon, Monitor } from "lucide-react";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { useThemeStore, ACCENT_COLORS } from "@/stores/theme-store";
import { useAppStore } from "@/stores";
import type { ThemeMode } from "@/lib/theme";

const THEME_OPTIONS: { mode: ThemeMode; labelKey: string; icon: typeof Sun }[] = [
  { mode: "light", labelKey: "settings:appearance.themeLight", icon: Sun },
  { mode: "dark", labelKey: "settings:appearance.themeDark", icon: Moon },
  { mode: "system", labelKey: "settings:appearance.themeSystem", icon: Monitor },
];

export function AppearanceSettings() {
  const { t } = useTranslation("settings");
  const themeMode = useThemeStore((s) => s.mode);
  const setMode = useThemeStore((s) => s.setMode);
  const accentColor = useThemeStore((s) => s.accentColor);
  const setAccentColor = useThemeStore((s) => s.setAccentColor);
  const sidebarCollapsed = useAppStore((s) => s.sidebarCollapsed);
  const setSidebarCollapsed = useAppStore((s) => s.setSidebarCollapsed);

  return (
    <div className="max-w-2xl space-y-6">
      <h2 className="text-lg font-semibold text-foreground">
        {t("appearance.title")}
      </h2>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t("appearance.theme")}</CardTitle>
          <p className="text-sm text-muted-foreground">
            {t("appearance.themeDesc")}
          </p>
        </CardHeader>
        <CardContent>
          <div className="flex gap-3">
            {THEME_OPTIONS.map(({ mode, labelKey, icon: Icon }) => (
              <button
                key={mode}
                onClick={() => setMode(mode)}
                className={cn(
                  "flex items-center gap-2 rounded-md border px-4 py-2 text-sm transition-colors",
                  themeMode === mode
                    ? "border-primary bg-primary/10 text-primary"
                    : "border-border text-muted-foreground hover:border-primary/50"
                )}
              >
                <Icon className="h-4 w-4" />
                {t(labelKey)}
              </button>
            ))}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            {t("appearance.accentColor")}
          </CardTitle>
          <p className="text-sm text-muted-foreground">
            {t("appearance.accentColorDesc")}
          </p>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-3">
            {ACCENT_COLORS.map((color) => (
              <button
                key={color.value}
                onClick={() => setAccentColor(color.value)}
                title={color.name}
                className={cn(
                  "h-8 w-8 rounded-full transition-transform",
                  accentColor === color.value
                    ? "scale-110 ring-2 ring-offset-2 ring-offset-background"
                    : "hover:scale-105"
                )}
                style={
                  {
                    backgroundColor: color.value,
                    "--tw-ring-color": accentColor === color.value ? color.value : undefined,
                  } as React.CSSProperties
                }
              />
            ))}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            {t("appearance.fontSize")}
          </CardTitle>
          <p className="text-sm text-muted-foreground">
            {t("appearance.fontSizeDesc", { size: 14 })}
          </p>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-4">
            <span className="text-xs text-muted-foreground">12</span>
            <Slider
              defaultValue={[14]}
              min={12}
              max={18}
              step={1}
              className="w-48"
            />
            <span className="text-xs text-muted-foreground">18</span>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            {t("appearance.sidebarDefault")}
          </CardTitle>
          <p className="text-sm text-muted-foreground">
            {t("appearance.sidebarDefaultDesc")}
          </p>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-3">
            <Switch
              checked={!sidebarCollapsed}
              onCheckedChange={(checked) => setSidebarCollapsed(!checked)}
            />
            <Label className="text-sm text-muted-foreground">
              {sidebarCollapsed ? "Collapsed" : "Expanded"}
            </Label>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
