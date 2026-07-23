import { useEffect, useState } from "react";
import { save as dialogSave } from "@tauri-apps/plugin-dialog";
import { Download, Power, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { SkillDetailPanel } from "@/components/skills/SkillDetailPanel";
import { ModelScopeImportDialog, RemoteInstallDialog, UninstallSkillDialog, UploadSkillDialog } from "@/components/skills/SkillInstallDialogs";
import { installedKey, remoteKey, SkillsList } from "@/components/skills/SkillsList";
import { SkillsToolbar, type SkillsView } from "@/components/skills/SkillsToolbar";
import { useSkillsInventory } from "@/components/skills/useSkillsInventory";
import { skillsIpc } from "@/lib/ipc";
import type { InstalledSkill, RemoteSkill, RemoteSkillDetail, SkillDetail } from "@/lib/ipc";
import { useTranslation } from "react-i18next";

type Selection =
  | { type: "installed"; skill: InstalledSkill }
  | { type: "remote"; skill: RemoteSkill }
  | null;

export function SkillsPage() {
  const { t } = useTranslation("skills");
  const inventory = useSkillsInventory();
  const [view, setView] = useState<SkillsView>("installed");
  const [query, setQuery] = useState("");
  const [source, setSource] = useState("all");
  const [remote, setRemote] = useState<RemoteSkill[]>([]);
  const [remoteLoading, setRemoteLoading] = useState(false);
  const [selection, setSelection] = useState<Selection>(null);
  const [detail, setDetail] = useState<SkillDetail | RemoteSkillDetail | null>(null);
  const [uploadOpen, setUploadOpen] = useState(false);
  const [modelScopeOpen, setModelScopeOpen] = useState(false);
  const [uninstallTarget, setUninstallTarget] = useState<InstalledSkill | null>(null);
  const [remoteInstallTarget, setRemoteInstallTarget] = useState<RemoteSkill | null>(null);

  useRemoteSearch(view, source, query, setRemote, setRemoteLoading);
  const selectedKey = selection ? selection.type === "installed" ? installedKey(selection.skill) : remoteKey(selection.skill) : null;
  const listLoading = view === "installed" ? inventory.loading : remoteLoading;
  const refresh = () => view === "installed" ? void inventory.refresh() : void searchNow(source, query, setRemote, setRemoteLoading);
  const selectInstalled = (skill: InstalledSkill) => void loadInstalledDetail(skill, setSelection, setDetail);
  const selectRemote = (skill: RemoteSkill) => void loadRemoteDetail(skill, setSelection, setDetail);
  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-y-auto">
      <div className="mx-auto flex w-full max-w-6xl min-h-0 flex-1 flex-col p-6 lg:p-10">
        <SkillsToolbar view={view} query={query} source={source} busy={listLoading} onViewChange={setView} onQueryChange={setQuery} onSourceChange={setSource} onUpload={() => setUploadOpen(true)} onModelScope={() => setModelScopeOpen(true)} onRefresh={refresh} />
        <SkillsContent view={view} query={query} selectedKey={selectedKey} detail={detail} installed={inventory.skills} remote={remote} loading={listLoading} onSelectInstalled={selectInstalled} onSelectRemote={selectRemote} onInstall={setRemoteInstallTarget} onBack={() => { setSelection(null); setDetail(null); }} actions={detail ? <DetailActions detail={detail} onRefresh={inventory.refresh} onUninstall={setUninstallTarget} onInstall={setRemoteInstallTarget} /> : null} />
        {inventory.error ? <p role="alert" className="mt-3 text-xs text-destructive">{t("loadFailed")}: {inventory.error}</p> : null}
      </div>
      <UploadSkillDialog open={uploadOpen} onOpenChange={setUploadOpen} onComplete={() => void inventory.refresh()} />
      <ModelScopeImportDialog open={modelScopeOpen} onOpenChange={setModelScopeOpen} onComplete={() => void inventory.refresh()} />
      <RemoteInstallDialog skill={remoteInstallTarget} open={Boolean(remoteInstallTarget)} onOpenChange={(open) => !open && setRemoteInstallTarget(null)} onComplete={() => { setView("installed"); void inventory.refresh(); }} />
      <UninstallSkillDialog skill={uninstallTarget} open={Boolean(uninstallTarget)} onOpenChange={(open) => !open && setUninstallTarget(null)} onComplete={() => { setSelection(null); setDetail(null); void inventory.refresh(); }} />
    </div>
  );
}

function SkillsContent({
  view, query, selectedKey, detail, installed, remote, loading, onSelectInstalled, onSelectRemote, onInstall, onBack, actions,
}: {
  view: SkillsView; query: string; selectedKey: string | null; detail: SkillDetail | RemoteSkillDetail | null;
  installed: InstalledSkill[]; remote: RemoteSkill[]; loading: boolean; onSelectInstalled: (skill: InstalledSkill) => void;
  onSelectRemote: (skill: RemoteSkill) => void; onInstall: (skill: RemoteSkill) => void; onBack: () => void; actions: React.ReactNode;
}) {
  return (
    <div className="mt-6 grid min-h-[420px] flex-1 gap-4 lg:grid-cols-[minmax(0,0.9fr)_minmax(360px,1.1fr)]">
      <section className="min-h-0"><SkillsList installed={installed} remote={remote} view={view} loading={loading} query={query} selectedKey={selectedKey} onSelectInstalled={onSelectInstalled} onSelectRemote={onSelectRemote} onInstall={onInstall} /></section>
      <aside className="min-h-[360px]"><SkillDetailPanel detail={detail} actions={actions} onBack={onBack} /></aside>
    </div>
  );
}

function DetailActions({ detail, onRefresh, onUninstall, onInstall }: { detail: SkillDetail | RemoteSkillDetail; onRefresh: () => Promise<void>; onUninstall: (skill: InstalledSkill) => void; onInstall: (skill: RemoteSkill) => void }) {
  const { t } = useTranslation("skills");
  if ("changelog" in detail) return <RemoteActions detail={detail} onInstall={onInstall} />;
  const setEnabled = async () => {
    try { await skillsIpc.setEnabled(detail.skill.slug, !detail.skill.enabled); await onRefresh(); } catch (error) { toast.error(String(error)); }
  };
  return <><Button size="xs" variant="outline" onClick={setEnabled}><Power className="size-3" />{t(detail.skill.enabled ? "disable" : "enable")}</Button><Button size="xs" variant="outline" onClick={() => void exportInstalled(detail.skill, t)}><Download className="size-3" />{t("export")}</Button><Button size="xs" variant="destructive" onClick={() => onUninstall(detail.skill)}><Trash2 className="size-3" />{t("uninstall")}</Button></>;
}

function RemoteActions({ detail, onInstall }: { detail: RemoteSkillDetail; onInstall: (skill: RemoteSkill) => void }) {
  const { t } = useTranslation("skills");
  return <><Button size="xs" onClick={() => onInstall(detail.skill)}>{t("install")}</Button><Button size="xs" variant="outline" onClick={() => void downloadRemote(detail.skill, t)}><Download className="size-3" />{t("download")}</Button></>;
}

function useRemoteSearch(view: SkillsView, source: string, query: string, setRemote: (skills: RemoteSkill[]) => void, setLoading: (loading: boolean) => void) {
  useEffect(() => {
    if (view !== "discover") return;
    const timer = window.setTimeout(() => void searchNow(source, query, setRemote, setLoading), 220);
    return () => window.clearTimeout(timer);
  }, [view, source, query, setRemote, setLoading]);
}

async function searchNow(source: string, query: string, setRemote: (skills: RemoteSkill[]) => void, setLoading: (loading: boolean) => void) {
  try { setLoading(true); setRemote((await skillsIpc.searchRemote(source, query, 30)).items); } catch (error) { toast.error(String(error)); } finally { setLoading(false); }
}

async function loadInstalledDetail(skill: InstalledSkill, setSelection: (value: Selection) => void, setDetail: (detail: SkillDetail | RemoteSkillDetail | null) => void) {
  setSelection({ type: "installed", skill }); setDetail(null);
  try { setDetail(await skillsIpc.getDetail(skill.slug)); } catch (error) { toast.error(String(error)); }
}

async function loadRemoteDetail(skill: RemoteSkill, setSelection: (value: Selection) => void, setDetail: (detail: SkillDetail | RemoteSkillDetail | null) => void) {
  setSelection({ type: "remote", skill }); setDetail(null);
  try { setDetail(await skillsIpc.getRemoteDetail(skill.provider, skill.slug)); } catch (error) { toast.error(String(error)); }
}

async function exportInstalled(skill: InstalledSkill, t: (key: string) => string) {
  const destination = await dialogSave({ defaultPath: `${skill.slug}.zip`, filters: [{ name: "ZIP", extensions: ["zip"] }] });
  if (!destination) return;
  await skillsIpc.exportInstalled(skill.slug, destination);
  toast.success(t("downloadSaved"));
}

async function downloadRemote(skill: RemoteSkill, t: (key: string) => string) {
  const destination = await dialogSave({ defaultPath: `${skill.slug}.zip`, filters: [{ name: "ZIP", extensions: ["zip"] }] });
  if (!destination) return;
  await skillsIpc.downloadRemote(skill.provider, skill.slug, destination, skill.version ?? undefined);
  toast.success(t("downloadSaved"));
}
