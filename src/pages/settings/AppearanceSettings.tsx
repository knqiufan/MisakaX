import { useTranslation } from "react-i18next";
import { Sun, Moon, MoonStar, Monitor } from "lucide-react";
import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { useThemeStore, ACCENT_COLORS } from "@/stores/theme-store";
import { useAppStore } from "@/stores/app-store";
import type { ThemeMode } from "@/lib/theme";

const THEME_OPTIONS: { mode: ThemeMode; labelKey: string; icon: typeof Sun }[] = [
  { mode: "light", labelKey: "settings:appearance.themeLight", icon: Sun },
  { mode: "dark", labelKey: "settings:appearance.themeDark", icon: Moon },
  { mode: "dim", labelKey: "settings:appearance.themeDim", icon: MoonStar },
  { mode: "system", labelKey: "settings:appearance.themeSystem", icon: Monitor },
];

export function AppearanceSettings() {
  const { t } = useTranslation("settings");
  const themeMode = useThemeStore((s) => s.mode);
  const setMode = useThemeStore((s) => s.setMode);
  const accentColor = useThemeStore((s) => s.accentColor);
  const setAccentColor = useThemeStore((s) => s.setAccentColor);
  const reducedTransparency = useThemeStore((s) => s.reducedTransparency);
  const setReducedTransparency = useThemeStore((s) => s.setReducedTransparency);
  const uiFontSize = useThemeStore((s) => s.uiFontSize);
  const setUiFontSize = useThemeStore((s) => s.setUiFontSize);
  const sidebarCollapsed = useAppStore((s) => s.sidebarCollapsed);
  const setSidebarCollapsed = useAppStore((s) => s.setSidebarCollapsed);

  return (
    <div className="w-full min-w-0 space-y-6">

      <Card>
        <CardHeader>
          <CardTitle>{t("appearance.theme")}</CardTitle>
          <p className="text-sm text-muted-foreground">{t("appearance.themeDesc")}</p>
        </CardHeader>
        <CardContent>
          <div
            className="inline-flex w-full max-w-lg flex-wrap gap-1 rounded-full border border-[color:var(--border-subtle)] bg-[color:var(--surface-control)] p-1 shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]"
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
                    "flex min-h-9 flex-1 items-center justify-center gap-1.5 rounded-full px-2 py-1.5 text-[0.75rem] font-semibold transition-colors duration-[var(--ds-dur-fast)] ease-out sm:text-[0.8125rem]",
                    active
                      ? "border border-[color:var(--border-subtle)] bg-[color:var(--surface-control-hover)] text-foreground shadow-[inset_0_1px_0_rgba(255,255,255,0.06)] dark:shadow-none"
                      : "text-muted-foreground hover:bg-[color:var(--surface-hover)] hover:text-foreground"
                  )}
                >
                  <Icon className="size-3.5 shrink-0 sm:size-4" />
                  <span className="hidden sm:inline">{t(labelKey)}</span>
                </button>
              );
            })}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t("appearance.reducedTransparency")}</CardTitle>
          <p className="text-sm text-muted-foreground">
            {t("appearance.reducedTransparencyDesc")}
          </p>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-3">
            <Switch
              checked={reducedTransparency}
              onCheckedChange={setReducedTransparency}
            />
            <Label className="text-sm text-muted-foreground">
              {reducedTransparency ? t("appearance.reducedTransparencyOn") : t("appearance.reducedTransparencyOff")}
            </Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t("appearance.accentColor")}</CardTitle>
          <p className="text-sm text-muted-foreground">{t("appearance.accentColorDesc")}</p>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-3">
            {ACCENT_COLORS.map((color) => (
              <button
                key={color.value}
                type="button"
                onClick={() => setAccentColor(color.value)}
                title={color.name}
                className={cn(
                  "h-8 w-8 rounded-full transition-[transform] duration-[var(--ds-dur-fast)] ease-out hover:brightness-105",
                  accentColor === color.value
                    ? "scale-110 shadow-[0_0_0_2px_color-mix(in_srgb,var(--background)_60%,transparent),0_0_0_4px_var(--ring)]"
                    : "hover:scale-105"
                )}
                style={{ backgroundColor: color.value }}
              />
            ))}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t("appearance.fontSize")}</CardTitle>
          <p className="text-sm text-muted-foreground">
            {t("appearance.fontSizeDesc", { size: uiFontSize })}
          </p>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-4">
            <span className="text-meta text-muted-foreground">12</span>
            <Slider
              value={[uiFontSize]}
              onValueChange={(v) => setUiFontSize(v[0] ?? 14)}
              min={12}
              max={18}
              step={1}
              className="w-48"
            />
            <span className="text-meta text-muted-foreground">18</span>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t("appearance.sidebarDefault")}</CardTitle>
          <p className="text-sm text-muted-foreground">{t("appearance.sidebarDefaultDesc")}</p>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-3">
            <Switch
              checked={!sidebarCollapsed}
              onCheckedChange={(checked) => setSidebarCollapsed(!checked)}
            />
            <Label className="text-sm text-muted-foreground">
              {sidebarCollapsed ? t("appearance.sidebarCollapsed") : t("appearance.sidebarExpanded")}
            </Label>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
