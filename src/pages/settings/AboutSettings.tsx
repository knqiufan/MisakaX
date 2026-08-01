import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ExternalLink, Download, Upload } from "lucide-react";
import { save as dialogSave, open as dialogOpen } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { handleExternalLinkClick } from "@/lib/external-links";
import { Badge } from "@/components/ui/badge";
import { MisakaLogo } from "@/components/brand/MisakaLogo";
import {
  SettingsCard,
  SettingsSectionHeader,
  SettingsSubCard,
  SettingsSubRow,
} from "@/components/settings";
import { SidecarStatusBadge } from "@/components/layout/SidecarStatusBadge";
import { settingsIpc, sessionsIpc } from "@/lib/ipc";
import type { SystemInfo } from "@/lib/ipc";
import { useChatStore } from "@/stores/chat-store";

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
    <div className="space-y-6">
      <SettingsSectionHeader title={t("about.title")} />

      <SettingsCard>
        <div className="flex flex-wrap items-center gap-3">
          <MisakaLogo className="size-10 text-foreground" title="MisakaX" />
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <h3 className="text-sm font-medium text-foreground">MisakaX</h3>
              <Badge variant="secondary" className="text-[10px]">
                v{systemInfo?.app_version ?? "0.1.0"}
              </Badge>
            </div>
            <p className="mt-0.5 text-[11px] text-muted-foreground">
              {t("about.tagline")}
            </p>
          </div>
        </div>
        <div className="mt-4 space-y-2">
          <Button variant="outline" size="sm" disabled>
            {t("about.checkUpdate")}
          </Button>
          <p className="text-[10px] text-muted-foreground">
            {t("about.checkUpdateDesc")}
          </p>
        </div>
      </SettingsCard>

      <SettingsCard title={t("about.systemInfo")}>
        <SettingsSubCard>
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
          <SettingsSubRow>
            <dt className="text-xs text-muted-foreground">
              {t("about.agentStatus")}
            </dt>
            <dd className="shrink-0">
              <SidecarStatusBadge showLabel />
            </dd>
          </SettingsSubRow>
        </SettingsSubCard>
      </SettingsCard>

      <SettingsCard title={t("about.techStack")}>
        <div className="flex flex-wrap gap-2">
          <Badge variant="outline" className="text-[10px]">
            Tauri 2.x
          </Badge>
          <Badge variant="outline" className="text-[10px]">
            React 19
          </Badge>
          <Badge variant="outline" className="text-[10px]">
            Rust
          </Badge>
          <Badge variant="outline" className="text-[10px]">
            TypeScript
          </Badge>
          <Badge variant="outline" className="text-[10px]">
            SQLite
          </Badge>
          <Badge variant="outline" className="text-[10px]">
            Python (FastAPI)
          </Badge>
        </div>
      </SettingsCard>

      <DataManagementCard />

      <SettingsCard>
        <div className="flex items-center gap-4">
          <Button variant="outline" size="sm" asChild>
            <a
              href="https://github.com"
              target="_blank"
              rel="noopener noreferrer"
              onClick={(event) =>
                handleExternalLinkClick(event, "https://github.com")
              }
            >
              <ExternalLink className="mr-1.5 size-3.5" />
              GitHub
            </a>
          </Button>
          <span className="text-[10px] text-muted-foreground">
            MIT {t("about.license")}
          </span>
        </div>
      </SettingsCard>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <SettingsSubRow>
      <dt className="text-xs text-muted-foreground">{label}</dt>
      <dd className="max-w-[60%] truncate font-mono text-[11px] text-foreground">
        {value}
      </dd>
    </SettingsSubRow>
  );
}

function DataManagementCard() {
  const { t } = useTranslation("settings");
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);

  const handleExportAll = async () => {
    try {
      setExporting(true);
      const allSessions = await sessionsIpc.list();
      if (allSessions.length === 0) {
        toast.info(t("about.noSessionsToExport"));
        return;
      }

      const filePath = await dialogSave({
        title: t("about.exportAllSessions"),
        defaultPath: `misakax-export-${new Date().toISOString().slice(0, 10)}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!filePath) return;

      const ids = allSessions.map((s) => s.id);
      await sessionsIpc.exportSessions(ids, filePath);
      toast.success(t("about.exportSuccess"));
    } catch (err) {
      console.error("Export all failed:", err);
      toast.error(t("about.exportFailed"));
    } finally {
      setExporting(false);
    }
  };

  const handleImport = async () => {
    try {
      setImporting(true);
      const selected = await dialogOpen({
        title: t("about.importSessions"),
        filters: [{ name: "JSON", extensions: ["json"] }],
        multiple: false,
      });
      if (!selected) return;

      const filePath = typeof selected === "string" ? selected : selected;
      const result = await sessionsIpc.importSessions(filePath);

      if (result.errors.length > 0) {
        toast.warning(
          `${t("about.importPartial")}: ${result.imported_count} ${t("about.imported")}, ${result.skipped_count} ${t("about.skipped")}`
        );
      } else {
        toast.success(
          `${t("about.importSuccess")}: ${result.imported_count} ${t("about.imported")}, ${result.skipped_count} ${t("about.skipped")}`
        );
      }

      if (result.imported_count > 0) {
        const store = useChatStore.getState();
        await store.refreshSessions();
        store.bumpSessionsReload();
      }
    } catch (err) {
      console.error("Import failed:", err);
      toast.error(t("about.importFailed"));
    } finally {
      setImporting(false);
    }
  };

  return (
    <SettingsCard
      title={t("about.dataManagement")}
      description={t("about.dataManagementDesc")}
    >
      <div className="flex flex-wrap items-center gap-3">
        <Button
          variant="outline"
          size="sm"
          onClick={handleExportAll}
          disabled={exporting}
        >
          <Download className="mr-1.5 size-3.5" />
          {exporting ? t("about.exporting") : t("about.exportAllSessions")}
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={handleImport}
          disabled={importing}
        >
          <Upload className="mr-1.5 size-3.5" />
          {importing ? t("about.importing") : t("about.importSessions")}
        </Button>
      </div>
    </SettingsCard>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}
