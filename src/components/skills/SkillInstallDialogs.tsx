import { useEffect, useState } from "react";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { skillsIpc } from "@/lib/ipc";
import type { InstalledSkill, RemoteSkill, SkillArchiveInspection } from "@/lib/ipc";

interface DialogControl {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete: (skill: InstalledSkill) => void;
}

export function UploadSkillDialog(control: DialogControl) {
  const { t } = useTranslation("skills");
  return (
    <Dialog open={control.open} onOpenChange={control.onOpenChange}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{t("uploadDialogTitle")}</DialogTitle>
          <DialogDescription>{t("uploadDialogDescription")}</DialogDescription>
        </DialogHeader>
        <UploadSkillForm {...control} />
      </DialogContent>
    </Dialog>
  );
}

function UploadSkillForm({ onOpenChange, onComplete }: DialogControl) {
  const { t } = useTranslation("skills");
  const [path, setPath] = useState<string | null>(null);
  const [inspection, setInspection] = useState<SkillArchiveInspection | null>(null);
  const [busy, setBusy] = useState(false);
  const chooseArchive = async () => {
    const selected = await dialogOpen({ multiple: false, filters: [{ name: "ZIP", extensions: ["zip"] }] });
    if (typeof selected === "string") {
      setPath(selected);
      setInspection(null);
    }
  };
  const inspect = async () => {
    if (!path) return toast.error(t("archiveNotSelected"));
    await runAction(setBusy, async () => setInspection(await skillsIpc.inspectArchive(path)), t);
  };
  const install = async () => {
    if (!path || !inspection) return;
    await runAction(setBusy, async () => {
      const result = await skillsIpc.installArchive(path);
      onComplete(result.skill);
      onOpenChange(false);
    }, t);
  };
  return (
    <>
      <ArchivePicker path={path} inspection={inspection} busy={busy} onChoose={chooseArchive} onInspect={inspect} />
      <p className="text-xs text-muted-foreground">{t("replaceNotice")}</p>
      <DialogFooter>
        <Button variant="outline" onClick={() => onOpenChange(false)} disabled={busy}>{t("cancel")}</Button>
        <Button onClick={install} disabled={!inspection || busy}>{t(busy ? "installing" : "confirmInstall")}</Button>
      </DialogFooter>
    </>
  );
}

function ArchivePicker({
  path,
  inspection,
  busy,
  onChoose,
  onInspect,
}: {
  path: string | null;
  inspection: SkillArchiveInspection | null;
  busy: boolean;
  onChoose: () => void;
  onInspect: () => void;
}) {
  const { t } = useTranslation("skills");
  return (
    <div className="rounded-lg border border-border/70 bg-muted/25 p-3">
      <div className="flex flex-wrap items-center gap-2">
        <Button size="sm" variant="outline" onClick={onChoose} disabled={busy}>{t("chooseArchive")}</Button>
        {path ? <span className="min-w-0 truncate text-xs text-muted-foreground">{t("selectedArchive", { name: fileName(path) })}</span> : null}
        <Button size="sm" onClick={onInspect} disabled={!path || busy}>{t("inspect")}</Button>
      </div>
      {inspection ? <InspectionSummary inspection={inspection} /> : null}
    </div>
  );
}

function InspectionSummary({ inspection }: { inspection: SkillArchiveInspection }) {
  const { t } = useTranslation("skills");
  return (
    <div className="mt-3 border-t border-border/60 pt-3 text-xs">
      <p className="font-medium text-primary">{t("archiveReady")}</p>
      <p className="mt-1 text-muted-foreground">{inspection.manifest.name} · {t("archiveFiles", { count: inspection.files.length })}</p>
    </div>
  );
}

export function ModelScopeImportDialog(control: DialogControl) {
  const { t } = useTranslation("skills");
  return (
    <Dialog open={control.open} onOpenChange={control.onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader><DialogTitle>{t("modelscopeTitle")}</DialogTitle><DialogDescription>{t("modelscopeDescription")}</DialogDescription></DialogHeader>
        <ModelScopeImportForm {...control} />
      </DialogContent>
    </Dialog>
  );
}

function ModelScopeImportForm({ onOpenChange, onComplete }: DialogControl) {
  const { t } = useTranslation("skills");
  const [reference, setReference] = useState("");
  const [busy, setBusy] = useState(false);
  const importSkill = async () => {
    if (!reference.trim()) return;
    await runAction(setBusy, async () => {
      const result = await skillsIpc.importModelScope(reference);
      onComplete(result.skill);
      onOpenChange(false);
    }, t);
  };
  return (
    <>
      <Input value={reference} onChange={(event) => setReference(event.target.value)} placeholder={t("modelscopePlaceholder")} />
      <DialogFooter>
        <Button variant="outline" onClick={() => onOpenChange(false)} disabled={busy}>{t("cancel")}</Button>
        <Button onClick={importSkill} disabled={!reference.trim() || busy}>{t(busy ? "installing" : "import")}</Button>
      </DialogFooter>
    </>
  );
}

export function UninstallSkillDialog({
  skill,
  open,
  onOpenChange,
  onComplete,
}: {
  skill: InstalledSkill | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete: () => void;
}) {
  const { t } = useTranslation("skills");
  const [busy, setBusy] = useState(false);
  useEffect(() => setBusy(false), [skill, open]);
  const uninstall = async () => {
    if (!skill) return;
    await runAction(setBusy, async () => {
      await skillsIpc.uninstall(skill.slug);
      onComplete();
      onOpenChange(false);
    }, t);
  };
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader><DialogTitle>{t("uninstallTitle", { name: skill?.name ?? "" })}</DialogTitle><DialogDescription>{t("uninstallDescription")}</DialogDescription></DialogHeader>
        <DialogFooter><Button variant="outline" onClick={() => onOpenChange(false)} disabled={busy}>{t("cancel")}</Button><Button variant="destructive" onClick={uninstall} disabled={busy}>{t("confirmUninstall")}</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export function RemoteInstallDialog({
  skill,
  open,
  onOpenChange,
  onComplete,
}: {
  skill: RemoteSkill | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete: (skill: InstalledSkill) => void;
}) {
  const { t } = useTranslation("skills");
  const [busy, setBusy] = useState(false);
  useEffect(() => setBusy(false), [skill, open]);
  const install = async () => {
    if (!skill) return;
    await runAction(setBusy, async () => {
      const result = await skillsIpc.installRemote(
        skill.provider,
        skill.slug,
        skill.version ?? undefined
      );
      onComplete(result.skill);
      onOpenChange(false);
    }, t);
  };
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader><DialogTitle>{t("installTitle", { name: skill?.display_name ?? "" })}</DialogTitle><DialogDescription>{t("installDescription")}</DialogDescription></DialogHeader>
        <DialogFooter><Button variant="outline" onClick={() => onOpenChange(false)} disabled={busy}>{t("cancel")}</Button><Button onClick={install} disabled={busy}>{t(busy ? "installing" : "confirmInstall")}</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

async function runAction(
  setBusy: (busy: boolean) => void,
  action: () => Promise<void>,
  t: (key: string) => string
) {
  try {
    setBusy(true);
    await action();
    toast.success(t("operationSuccess"));
  } catch (error) {
    toast.error(`${t("operationFailed")}: ${error instanceof Error ? error.message : String(error)}`);
  } finally {
    setBusy(false);
  }
}

function fileName(path: string) {
  return path.split(/[\\/]/).pop() ?? path;
}
