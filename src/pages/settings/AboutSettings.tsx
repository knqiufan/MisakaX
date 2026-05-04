import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ExternalLink } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { settingsIpc } from "@/lib/ipc";
import type { SystemInfo } from "@/lib/ipc";

export function AboutSettings() {
  const { t } = useTranslation("settings");
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);

  useEffect(() => {
    settingsIpc
      .getSystemInfo()
      .then(setSystemInfo)
      .catch((err) => {
        if (import.meta.env.DEV) {
          console.warn("[About] Failed to load system info:", err);
        }
      });
  }, []);

  return (
    <div className="max-w-2xl space-y-6">
      <h2 className="text-lg font-semibold text-foreground">
        {t("about.title")}
      </h2>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <CardTitle className="text-base">MisakaX</CardTitle>
            <Badge variant="secondary">
              v{systemInfo?.app_version ?? "0.1.0"}
            </Badge>
          </div>
        </CardHeader>
        <CardContent className="space-y-3">
          <Button variant="outline" size="sm" disabled>
            {t("about.checkUpdate")}
          </Button>
          <p className="text-xs text-muted-foreground">
            {t("about.checkUpdateDesc")}
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t("about.systemInfo")}</CardTitle>
        </CardHeader>
        <CardContent>
          <dl className="space-y-2 text-sm">
            <InfoRow label={t("about.os")} value={systemInfo?.os ?? "—"} />
            <InfoRow
              label={t("about.architecture")}
              value={systemInfo?.arch ?? "—"}
            />
            <InfoRow
              label={t("about.dataDirectory")}
              value={systemInfo?.data_dir ?? "—"}
            />
            <InfoRow
              label={t("about.dbSize")}
              value={formatBytes(systemInfo?.db_size_bytes ?? 0)}
            />
          </dl>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t("about.techStack")}</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            <Badge variant="outline">Tauri 2.x</Badge>
            <Badge variant="outline">React 19</Badge>
            <Badge variant="outline">Rust</Badge>
            <Badge variant="outline">TypeScript</Badge>
            <Badge variant="outline">SQLite</Badge>
            <Badge variant="outline">Python (FastAPI)</Badge>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className="pt-6">
          <div className="flex items-center gap-4">
            <Button variant="outline" size="sm" asChild>
              <a
                href="https://github.com"
                target="_blank"
                rel="noopener noreferrer"
              >
                <ExternalLink className="mr-1.5 h-3.5 w-3.5" />
                GitHub
              </a>
            </Button>
            <span className="text-xs text-muted-foreground">
              MIT {t("about.license")}
            </span>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between">
      <dt className="text-muted-foreground">{label}</dt>
      <dd className="font-mono text-foreground">{value}</dd>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}
