import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { i18n } from "@/locales/i18n";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useSettingsStore } from "@/stores/settings-store";

export function GeneralSettings() {
  const { t } = useTranslation("settings");
  const config = useSettingsStore((s) => s.config);
  const loadConfig = useSettingsStore((s) => s.loadConfig);
  const updateConfig = useSettingsStore((s) => s.updateConfig);

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  if (!config) {
    return <SettingsLoading />;
  }

  return (
    <div className="w-full min-w-0 space-y-6">

      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t("general.language")}</CardTitle>
          <p className="text-sm text-muted-foreground">
            {t("general.languageDesc")}
          </p>
        </CardHeader>
        <CardContent>
          <Select
            value={config.language}
            onValueChange={(value) => {
              updateConfig({ language: value });
              i18n.changeLanguage(value);
            }}
          >
            <SelectTrigger className="w-48">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="en">English</SelectItem>
              <SelectItem value="zh-CN">中文</SelectItem>
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t("general.logLevel")}</CardTitle>
          <p className="text-sm text-muted-foreground">
            {t("general.logLevelDesc")}
          </p>
        </CardHeader>
        <CardContent>
          <Select
            value={config.log_level}
            onValueChange={(value) => updateConfig({ log_level: value })}
          >
            <SelectTrigger className="w-48">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="debug">Debug</SelectItem>
              <SelectItem value="info">Info</SelectItem>
              <SelectItem value="warn">Warn</SelectItem>
              <SelectItem value="error">Error</SelectItem>
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            {t("general.autoStartSidecar")}
          </CardTitle>
          <p className="text-sm text-muted-foreground">
            {t("general.autoStartSidecarDesc")}
          </p>
        </CardHeader>
        <CardContent>
          <Switch
            checked={config.auto_start_sidecar}
            onCheckedChange={(checked) =>
              updateConfig({ auto_start_sidecar: checked })
            }
          />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">
            {t("general.sidecarPort")}
          </CardTitle>
          <p className="text-sm text-muted-foreground">
            {t("general.sidecarPortDesc")}
          </p>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Label htmlFor="sidecar-port" className="sr-only">
              {t("general.sidecarPort")}
            </Label>
            <Input
              id="sidecar-port"
              type="number"
              className="w-32"
              value={config.sidecar_port}
              min={1024}
              max={65535}
              onChange={(e) => {
                const port = parseInt(e.target.value, 10);
                if (!isNaN(port) && port >= 1024 && port <= 65535) {
                  updateConfig({ sidecar_port: port });
                }
              }}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function SettingsLoading() {
  return (
    <div className="flex h-32 items-center justify-center">
      <p className="text-sm text-muted-foreground">Loading...</p>
    </div>
  );
}
