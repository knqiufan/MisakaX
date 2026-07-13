import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Loader2 } from "lucide-react";
import { i18n } from "@/locales/i18n";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { SettingsCard, FieldRow, SettingsSectionHeader } from "@/components/settings";
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
    return (
      <div className="flex justify-center py-12">
        <Loader2 className="size-5 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SettingsSectionHeader title={t("general.title")} />

      <SettingsCard>
        <div className="space-y-4">
          <FieldRow
            label={t("general.language")}
            description={t("general.languageDesc")}
          >
            <Select
              value={config.language}
              onValueChange={(value) => {
                updateConfig({ language: value });
                i18n.changeLanguage(value);
              }}
            >
              <SelectTrigger className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="en">English</SelectItem>
                <SelectItem value="zh-CN">中文</SelectItem>
              </SelectContent>
            </Select>
          </FieldRow>

          <FieldRow
            label={t("general.logLevel")}
            description={t("general.logLevelDesc")}
            separator
          >
            <Select
              value={config.log_level}
              onValueChange={(value) => updateConfig({ log_level: value })}
            >
              <SelectTrigger className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="debug">Debug</SelectItem>
                <SelectItem value="info">Info</SelectItem>
                <SelectItem value="warn">Warn</SelectItem>
                <SelectItem value="error">Error</SelectItem>
              </SelectContent>
            </Select>
          </FieldRow>

          <FieldRow
            label={t("general.autoStartSidecar")}
            description={t("general.autoStartSidecarDesc")}
            separator
          >
            <Switch
              checked={config.auto_start_sidecar}
              onCheckedChange={(checked) =>
                updateConfig({ auto_start_sidecar: checked })
              }
            />
          </FieldRow>

          <FieldRow
            label={t("general.sidecarPort")}
            description={t("general.sidecarPortDesc")}
            separator
          >
            <Input
              id="sidecar-port"
              type="number"
              className="w-28"
              value={config.sidecar_port}
              min={1024}
              max={65535}
              aria-label={t("general.sidecarPort")}
              onChange={(e) => {
                const port = parseInt(e.target.value, 10);
                if (!isNaN(port) && port >= 1024 && port <= 65535) {
                  updateConfig({ sidecar_port: port });
                }
              }}
            />
          </FieldRow>
        </div>
      </SettingsCard>
    </div>
  );
}
