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
import type {
  InstalledSkill,
  RemoteSkill,
  SkillArchiveInspection,
  SkillFinding,
  SkillScanOperation,
} from "@/lib/ipc";

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
  const [pendingScan, setPendingScan] = useState<SkillScanOperation | null>(null);
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
      finishOrReview(result, setPendingScan, onComplete, onOpenChange);
    }, t);
  };
  return (
    <>
      <ArchivePicker path={path} inspection={inspection} busy={busy} onChoose={chooseArchive} onInspect={inspect} />
      {pendingScan ? (
        <PendingScanReview
          operation={pendingScan}
          onApproved={(skill) => {
            onComplete(skill);
            onOpenChange(false);
          }}
          onClose={() => onOpenChange(false)}
        />
      ) : null}
      <p className="text-xs text-muted-foreground">{t("replaceNotice")}</p>
      <DialogFooter>
        <Button variant="outline" onClick={() => onOpenChange(false)} disabled={busy}>{t("cancel")}</Button>
        <Button onClick={install} disabled={!inspection || busy || Boolean(pendingScan)}>{t(busy ? "installing" : "confirmInstall")}</Button>
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
  const [pendingScan, setPendingScan] = useState<SkillScanOperation | null>(null);
  const importSkill = async () => {
    if (!reference.trim()) return;
    await runAction(setBusy, async () => {
      const result = await skillsIpc.importModelScope(reference);
      finishOrReview(result, setPendingScan, onComplete, onOpenChange);
    }, t);
  };
  return (
    <>
      <Input value={reference} onChange={(event) => setReference(event.target.value)} placeholder={t("modelscopePlaceholder")} />
      {pendingScan ? (
        <PendingScanReview
          operation={pendingScan}
          onApproved={(skill) => {
            onComplete(skill);
            onOpenChange(false);
          }}
          onClose={() => onOpenChange(false)}
        />
      ) : null}
      <DialogFooter>
        <Button variant="outline" onClick={() => onOpenChange(false)} disabled={busy}>{t("cancel")}</Button>
        <Button onClick={importSkill} disabled={!reference.trim() || busy || Boolean(pendingScan)}>{t(busy ? "installing" : "import")}</Button>
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
      await skillsIpc.uninstall(skill.skill_id);
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
  const [pendingScan, setPendingScan] = useState<SkillScanOperation | null>(null);
  useEffect(() => {
    setBusy(false);
    setPendingScan(null);
  }, [skill, open]);
  const install = async () => {
    if (!skill) return;
    await runAction(setBusy, async () => {
      const result = await skillsIpc.installRemote(
        skill.provider,
        skill.slug,
        skill.version ?? undefined
      );
      finishOrReview(result, setPendingScan, onComplete, onOpenChange);
    }, t);
  };
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader><DialogTitle>{t("installTitle", { name: skill?.display_name ?? "" })}</DialogTitle><DialogDescription>{t("installDescription")}</DialogDescription></DialogHeader>
        {pendingScan ? (
          <PendingScanReview
            operation={pendingScan}
            onApproved={(installed) => {
              onComplete(installed);
              onOpenChange(false);
            }}
            onClose={() => onOpenChange(false)}
          />
        ) : null}
        <DialogFooter><Button variant="outline" onClick={() => onOpenChange(false)} disabled={busy}>{t("cancel")}</Button><Button onClick={install} disabled={busy || Boolean(pendingScan)}>{t(busy ? "installing" : "confirmInstall")}</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function PendingScanReview({
  operation,
  onApproved,
  onClose,
}: {
  operation: SkillScanOperation;
  onApproved: (skill: InstalledSkill) => void;
  onClose: () => void;
}) {
  const { t } = useTranslation("skills");
  const [findings, setFindings] = useState<SkillFinding[]>([]);
  const [reason, setReason] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    let active = true;
    void skillsIpc
      .listFindings(operation.scan_id, undefined, undefined, 20)
      .then((page) => {
        if (active) setFindings(page.items);
      })
      .catch((error) => {
        if (active) toast.error(String(error));
      });
    return () => {
      active = false;
    };
  }, [operation.scan_id]);
  const approve = async () => {
    await runAction(setBusy, async () => {
      const result = await skillsIpc.approveScan(
        operation.scan_id,
        "local-user",
        reason.trim(),
      );
      if (result.operation.installed_skill) onApproved(result.operation.installed_skill);
    }, t);
  };
  const reject = async () => {
    await runAction(setBusy, async () => {
      await skillsIpc.rejectScan(operation.scan_id, "local-user", reason.trim());
      onClose();
    }, t);
  };
  return (
    <section className="rounded-lg border border-border/70 bg-muted/25 p-3" aria-live="polite">
      <p className="text-sm font-medium text-foreground">
        {t(`scanState.${operation.state}`)}
      </p>
      <p className="mt-1 text-xs leading-5 text-muted-foreground">
        {operation.state === "review_required" ? t("reviewRequiredHelp") : t("scanBlockedHelp")}
      </p>
      {findings.length > 0 ? (
        <ul className="mt-3 max-h-32 space-y-1 overflow-auto text-xs text-muted-foreground">
          {findings.map((finding) => (
            <li key={finding.finding_id}>
              <span className="font-medium uppercase text-foreground">{finding.severity}</span>
              {" · "}{finding.title}
            </li>
          ))}
        </ul>
      ) : null}
      {operation.state === "review_required" ? (
        <>
          <Input
            className="mt-3"
            value={reason}
            onChange={(event) => setReason(event.target.value)}
            placeholder={t("approvalReasonPlaceholder")}
            aria-label={t("approvalReason")}
          />
          <div className="mt-3 flex justify-end gap-2">
            <Button size="sm" variant="outline" disabled={busy || reason.trim().length < 3} onClick={reject}>
              {t("rejectScan")}
            </Button>
            <Button size="sm" disabled={busy || reason.trim().length < 3} onClick={approve}>
              {t("approveScan")}
            </Button>
          </div>
        </>
      ) : (
        <div className="mt-3 flex justify-end">
          <Button size="sm" variant="outline" onClick={onClose}>{t("close")}</Button>
        </div>
      )}
    </section>
  );
}

function finishOrReview(
  operation: SkillScanOperation,
  setPending: (operation: SkillScanOperation) => void,
  onComplete: (skill: InstalledSkill) => void,
  onOpenChange: (open: boolean) => void,
) {
  if (operation.installed_skill) {
    onComplete(operation.installed_skill);
    onOpenChange(false);
  } else {
    setPending(operation);
  }
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
