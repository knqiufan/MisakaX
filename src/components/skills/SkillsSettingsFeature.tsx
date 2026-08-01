import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { save as dialogSave } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { Download, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { SkillDetailPanel } from "@/components/skills/SkillDetailPanel";
import {
  ModelScopeImportDialog,
  RemoteInstallDialog,
  UninstallSkillDialog,
  UploadSkillDialog,
} from "@/components/skills/SkillInstallDialogs";
import { installedKey, remoteKey, SkillsList } from "@/components/skills/SkillsList";
import { SkillsToolbar, type SkillsView } from "@/components/skills/SkillsToolbar";
import { useSkillsInventory } from "@/components/skills/useSkillsInventory";
import { skillsIpc } from "@/lib/ipc";
import { cn } from "@/lib/utils";
import type {
  InstalledSkill,
  RemoteSkill,
  RemoteSkillDetail,
  SkillFilePage,
  SkillMigrationStatus,
  SkillSummary,
} from "@/lib/ipc";
import { useTranslation } from "react-i18next";

type Selection =
  | { type: "installed"; skill: InstalledSkill }
  | { type: "remote"; skill: RemoteSkill }
  | null;

type Detail = SkillSummary | RemoteSkillDetail;

/** Settings-owned Skills domain UI. It has no dependency on the top-level route. */
export function SkillsSettingsFeature() {
  const { t } = useTranslation("skills");
  const inventory = useSkillsInventory();
  const [view, setView] = useState<SkillsView>("installed");
  const [query, setQuery] = useState("");
  const [source, setSource] = useState("all");
  const [remote, setRemote] = useState<RemoteSkill[]>([]);
  const [remoteLoading, setRemoteLoading] = useState(false);
  const [selection, setSelection] = useState<Selection>(null);
  const [detail, setDetail] = useState<Detail | null>(null);
  const [initialFiles, setInitialFiles] = useState<SkillFilePage | null>(null);
  const [uploadOpen, setUploadOpen] = useState(false);
  const [modelScopeOpen, setModelScopeOpen] = useState(false);
  const [uninstallTarget, setUninstallTarget] = useState<InstalledSkill | null>(null);
  const [remoteInstallTarget, setRemoteInstallTarget] = useState<RemoteSkill | null>(null);
  const [pendingSkillIds, setPendingSkillIds] = useState<Set<string>>(() => new Set());
  const detailRequestId = useRef(0);
  const migration = useSkillMigrationStatus(inventory.refresh);

  useRemoteSearch(view, source, query, setRemote, setRemoteLoading);
  const selectedKey = selection
    ? selection.type === "installed"
      ? installedKey(selection.skill)
      : remoteKey(selection.skill)
    : null;
  const listLoading = view === "installed" ? inventory.loading : remoteLoading;
  const refresh = () =>
    view === "installed"
      ? void inventory.refresh()
      : void searchNow(source, query, setRemote, setRemoteLoading);
  const clearDetail = useCallback(() => {
    detailRequestId.current += 1;
    setSelection(null);
    setDetail(null);
    setInitialFiles(null);
  }, []);
  const handleViewChange = useCallback(
    (nextView: SkillsView) => {
      if (nextView === view) return;
      clearDetail();
      setView(nextView);
    },
    [clearDetail, view],
  );
  const handleSourceChange = useCallback(
    (nextSource: string) => {
      if (nextSource === source) return;
      clearDetail();
      setSource(nextSource);
    },
    [clearDetail, source],
  );
  const selectInstalled = useCallback(
    (skill: InstalledSkill) => {
      void loadInstalledDetail(
        skill,
        detailRequestId,
        setSelection,
        setDetail,
        setInitialFiles,
      );
    },
    [],
  );
  const selectRemote = useCallback(
    (skill: RemoteSkill) => {
      void loadRemoteDetail(
        skill,
        detailRequestId,
        setSelection,
        setDetail,
        setInitialFiles,
      );
    },
    [],
  );
  const toggleInstalled = useCallback(
    async (skill: InstalledSkill, enabled: boolean) => {
      setPendingSkillIds((current) => new Set(current).add(skill.skill_id));
      try {
        await skillsIpc.setEnabled(skill.skill_id, enabled);
        await inventory.refresh();
        if (
          selection?.type === "installed" &&
          selection.skill.skill_id === skill.skill_id
        ) {
          await loadInstalledDetail(
            { ...skill, enabled },
            detailRequestId,
            setSelection,
            setDetail,
            setInitialFiles,
          );
        }
      } catch (error) {
        toast.error(`${t("operationFailed")}: ${String(error)}`);
      } finally {
        setPendingSkillIds((current) => {
          const next = new Set(current);
          next.delete(skill.skill_id);
          return next;
        });
      }
    },
    [inventory, selection, t],
  );

  return (
    <div className="flex h-full min-h-0 min-w-0 flex-1 flex-col overflow-y-auto lg:overflow-hidden">
      <SkillsToolbar
        view={view}
        query={query}
        source={source}
        busy={listLoading}
        onViewChange={handleViewChange}
        onQueryChange={setQuery}
        onSourceChange={handleSourceChange}
        onUpload={() => setUploadOpen(true)}
        onModelScope={() => setModelScopeOpen(true)}
        onRefresh={refresh}
      />
      <MigrationStatusBanner status={migration.status} onRetry={migration.retry} />
      <SkillsContent
        view={view}
        query={query}
        selectedKey={selectedKey}
        detail={detail}
        initialFiles={initialFiles}
        installed={inventory.skills}
        remote={remote}
        loading={listLoading}
        pendingSkillIds={pendingSkillIds}
        onSelectInstalled={selectInstalled}
        onSelectRemote={selectRemote}
        onToggleInstalled={toggleInstalled}
        onInstall={setRemoteInstallTarget}
        onBack={clearDetail}
        actions={
          detail ? (
            <DetailActions
              detail={detail}
              pending={
                "generation" in detail && pendingSkillIds.has(detail.skill.skill_id)
              }
              onToggle={toggleInstalled}
              onUninstall={setUninstallTarget}
              onInstall={setRemoteInstallTarget}
            />
          ) : null
        }
      />
      {inventory.error ? (
        <p role="alert" className="mt-3 text-xs text-destructive">
          {t("loadFailed")}: {inventory.error}
        </p>
      ) : null}
      <UploadSkillDialog
        open={uploadOpen}
        onOpenChange={setUploadOpen}
        onComplete={() => void inventory.refresh()}
      />
      <ModelScopeImportDialog
        open={modelScopeOpen}
        onOpenChange={setModelScopeOpen}
        onComplete={() => void inventory.refresh()}
      />
      <RemoteInstallDialog
        skill={remoteInstallTarget}
        open={Boolean(remoteInstallTarget)}
        onOpenChange={(open) => !open && setRemoteInstallTarget(null)}
        onComplete={() => {
          clearDetail();
          setView("installed");
          void inventory.refresh();
        }}
      />
      <UninstallSkillDialog
        skill={uninstallTarget}
        open={Boolean(uninstallTarget)}
        onOpenChange={(open) => !open && setUninstallTarget(null)}
        onComplete={() => {
          clearDetail();
          void inventory.refresh();
        }}
      />
    </div>
  );
}

function SkillsContent({
  view,
  query,
  selectedKey,
  detail,
  initialFiles,
  installed,
  remote,
  loading,
  pendingSkillIds,
  onSelectInstalled,
  onSelectRemote,
  onToggleInstalled,
  onInstall,
  onBack,
  actions,
}: {
  view: SkillsView;
  query: string;
  selectedKey: string | null;
  detail: Detail | null;
  initialFiles: SkillFilePage | null;
  installed: InstalledSkill[];
  remote: RemoteSkill[];
  loading: boolean;
  pendingSkillIds: Set<string>;
  onSelectInstalled: (skill: InstalledSkill) => void;
  onSelectRemote: (skill: RemoteSkill) => void;
  onToggleInstalled: (skill: InstalledSkill, enabled: boolean) => Promise<void>;
  onInstall: (skill: RemoteSkill) => void;
  onBack: () => void;
  actions: ReactNode;
}) {
  return (
    <div className="mt-5 grid min-h-0 min-w-0 flex-1 gap-4 lg:grid-cols-[minmax(17rem,0.8fr)_minmax(32rem,1.2fr)]">
      <section
        className={cn(
          "min-h-[280px] min-w-0 lg:h-full lg:min-h-0",
          selectedKey && "max-lg:hidden",
        )}
      >
        <SkillsList
          installed={installed}
          remote={remote}
          view={view}
          loading={loading}
          query={query}
          selectedKey={selectedKey}
          pendingSkillIds={pendingSkillIds}
          onSelectInstalled={onSelectInstalled}
          onSelectRemote={onSelectRemote}
          onToggleInstalled={onToggleInstalled}
          onInstall={onInstall}
        />
      </section>
      <aside
        className={cn(
          "min-h-[340px] min-w-0 lg:h-full lg:min-h-0",
          !selectedKey && "max-lg:hidden",
        )}
      >
        <SkillDetailPanel
          detail={detail}
          initialFiles={initialFiles}
          actions={actions}
          onBack={onBack}
        />
      </aside>
    </div>
  );
}

function DetailActions({
  detail,
  pending,
  onToggle,
  onUninstall,
  onInstall,
}: {
  detail: Detail;
  pending: boolean;
  onToggle: (skill: InstalledSkill, enabled: boolean) => Promise<void>;
  onUninstall: (skill: InstalledSkill) => void;
  onInstall: (skill: RemoteSkill) => void;
}) {
  const { t } = useTranslation("skills");
  if (!("generation" in detail)) {
    return <RemoteActions detail={detail} onInstall={onInstall} />;
  }
  const skill = detail.skill;
  const canEnable =
    skill.health === "healthy" &&
    ["passed", "warnings", "approved"].includes(skill.security_state);
  return (
    <>
      <label className="flex min-h-8 cursor-pointer items-center gap-2 rounded-lg px-1 text-xs text-muted-foreground">
        <span>{t(skill.enabled ? "enabled" : "disabled")}</span>
        <Switch
          checked={skill.enabled}
          disabled={pending || !canEnable}
          aria-label={t("enableSkill", { name: skill.name })}
          onCheckedChange={(checked) => void onToggle(skill, checked)}
        />
      </label>
      {!skill.is_external ? (
        <>
          <Button size="xs" variant="outline" onClick={() => void exportInstalled(skill, t)}>
            <Download className="size-3" />
            {t("export")}
          </Button>
          <Button size="xs" variant="destructive" onClick={() => onUninstall(skill)}>
            <Trash2 className="size-3" />
            {t("uninstall")}
          </Button>
        </>
      ) : null}
    </>
  );
}

function useSkillMigrationStatus(refreshInventory: () => Promise<void>) {
  const [status, setStatus] = useState<SkillMigrationStatus | null>(null);
  useEffect(() => {
    void skillsIpc
      .getMigrationStatus()
      .then((value) => {
        if (value) setStatus(value);
      })
      .catch(() => undefined);
    const unlisten = listen<SkillMigrationStatus>("skills:migration-progress", (event) => {
      setStatus(event.payload);
      void refreshInventory();
    });
    return () => {
      void unlisten.then((dispose) => dispose());
    };
  }, [refreshInventory]);

  const retry = useCallback(async () => {
    try {
      const next = await skillsIpc.retryMigrationScan();
      setStatus(next);
    } catch (error) {
      toast.error(String(error));
    }
  }, []);
  return { status, retry };
}

function MigrationStatusBanner({
  status,
  onRetry,
}: {
  status: SkillMigrationStatus | null;
  onRetry: () => Promise<void>;
}) {
  const { t } = useTranslation("skills");
  if (!status || status.state === "completed") return null;
  const progress = status.total === 0 ? 100 : Math.round((status.completed / status.total) * 100);
  return (
    <div
      className="mx-3 mt-3 flex min-h-10 items-center gap-3 rounded-lg border border-border/70 bg-muted/35 px-3 py-2 text-xs"
      role="status"
      aria-live="polite"
    >
      <div className="min-w-0 flex-1">
        <p className="font-medium text-foreground">
          {status.state === "completed_with_errors"
            ? t("migrationCompletedWithErrors", { count: status.failed })
            : t("migrationScanning", { completed: status.completed, total: status.total })}
        </p>
        <div className="mt-1 h-1 overflow-hidden rounded-full bg-border" aria-hidden="true">
          <div className="h-full bg-primary transition-[width] duration-150" style={{ width: `${progress}%` }} />
        </div>
      </div>
      {status.state === "completed_with_errors" ? (
        <Button size="xs" variant="outline" onClick={() => void onRetry()}>
          {t("retryFailedMigrationScans")}
        </Button>
      ) : null}
    </div>
  );
}

function RemoteActions({
  detail,
  onInstall,
}: {
  detail: RemoteSkillDetail;
  onInstall: (skill: RemoteSkill) => void;
}) {
  const { t } = useTranslation("skills");
  return (
    <>
      <Button size="xs" onClick={() => onInstall(detail.skill)}>
        {t("install")}
      </Button>
      <Button
        size="xs"
        variant="outline"
        onClick={() => void downloadRemote(detail.skill, t)}
      >
        <Download className="size-3" />
        {t("download")}
      </Button>
    </>
  );
}

function useRemoteSearch(
  view: SkillsView,
  source: string,
  query: string,
  setRemote: (skills: RemoteSkill[]) => void,
  setLoading: (loading: boolean) => void,
) {
  useEffect(() => {
    if (view !== "discover") return;
    const timer = window.setTimeout(
      () => void searchNow(source, query, setRemote, setLoading),
      220,
    );
    return () => window.clearTimeout(timer);
  }, [view, source, query, setRemote, setLoading]);
}

async function searchNow(
  source: string,
  query: string,
  setRemote: (skills: RemoteSkill[]) => void,
  setLoading: (loading: boolean) => void,
) {
  try {
    setLoading(true);
    setRemote([]);
    setRemote((await skillsIpc.searchRemote(source, query, 30)).items);
  } catch (error) {
    toast.error(String(error));
  } finally {
    setLoading(false);
  }
}

async function loadInstalledDetail(
  skill: InstalledSkill,
  requestIdRef: { current: number },
  setSelection: (value: Selection) => void,
  setDetail: (detail: Detail | null) => void,
  setInitialFiles: (page: SkillFilePage | null) => void,
) {
  const requestId = ++requestIdRef.current;
  setSelection({ type: "installed", skill });
  setDetail(null);
  setInitialFiles(null);
  try {
    for (let attempt = 0; attempt < 2; attempt += 1) {
      const [summary, files] = await Promise.all([
        skillsIpc.getSummary(skill.skill_id),
        skillsIpc.listFiles(skill.skill_id, undefined, undefined, 500),
      ]);
      if (requestId !== requestIdRef.current) return;
      if (
        summary.generation === files.generation &&
        summary.skill.skill_id === files.skill_id
      ) {
        setDetail(summary);
        setInitialFiles(files);
        return;
      }
    }
    throw new Error("Skill inventory changed while details were loading");
  } catch (error) {
    if (requestId === requestIdRef.current) toast.error(String(error));
  }
}

async function loadRemoteDetail(
  skill: RemoteSkill,
  requestIdRef: { current: number },
  setSelection: (value: Selection) => void,
  setDetail: (detail: Detail | null) => void,
  setInitialFiles: (page: SkillFilePage | null) => void,
) {
  const requestId = ++requestIdRef.current;
  setSelection({ type: "remote", skill });
  setDetail(null);
  setInitialFiles(null);
  try {
    const loaded = await skillsIpc.getRemoteDetail(skill.provider, skill.slug);
    if (requestId === requestIdRef.current) setDetail(loaded);
  } catch (error) {
    if (requestId === requestIdRef.current) toast.error(String(error));
  }
}

async function exportInstalled(skill: InstalledSkill, t: (key: string) => string) {
  const destination = await dialogSave({
    defaultPath: `${skill.slug}.zip`,
    filters: [{ name: "ZIP", extensions: ["zip"] }],
  });
  if (!destination) return;
  await skillsIpc.exportInstalled(skill.skill_id, destination);
  toast.success(t("downloadSaved"));
}

async function downloadRemote(skill: RemoteSkill, t: (key: string) => string) {
  const destination = await dialogSave({
    defaultPath: `${skill.slug}.zip`,
    filters: [{ name: "ZIP", extensions: ["zip"] }],
  });
  if (!destination) return;
  await skillsIpc.downloadRemote(
    skill.provider,
    skill.slug,
    destination,
    skill.version ?? undefined,
  );
  toast.success(t("downloadSaved"));
}
