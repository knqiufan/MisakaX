import { ChevronLeft, FileCode2, FolderTree, ShieldCheck, ShieldAlert } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { RemoteSkillDetail, SkillDetail, SkillFileNode, SkillRiskReport } from "@/lib/ipc";

interface SkillDetailPanelProps {
  detail: SkillDetail | RemoteSkillDetail | null;
  actions?: React.ReactNode;
  onBack: () => void;
}

export function SkillDetailPanel({ detail, actions, onBack }: SkillDetailPanelProps) {
  const { t } = useTranslation("skills");
  if (!detail) {
    return (
      <div className="flex h-full items-center justify-center rounded-xl border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
        {t("details")}
      </div>
    );
  }
  const manifest = detail.manifest;
  const isRemote = "changelog" in detail;
  const risk = isRemote ? detail.risk : detail.skill.risk;
  return (
    <div className="flex h-full min-h-0 flex-col rounded-xl border border-border/70 bg-[color:var(--surface-card)]">
      <div className="flex items-start gap-3 border-b border-border/60 p-4">
        <Button variant="ghost" size="icon-xs" onClick={onBack} aria-label={t("backToList")}>
          <ChevronLeft className="size-3.5" />
        </Button>
        <div className="min-w-0 flex-1">
          <h3 className="truncate text-base font-semibold">{isRemote ? detail.skill.display_name : detail.skill.name}</h3>
          <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{isRemote ? detail.skill.summary : detail.skill.description}</p>
        </div>
        <div className="flex shrink-0 flex-wrap justify-end gap-1.5">{actions}</div>
      </div>
      <ScrollArea className="min-h-0 flex-1">
        <div className="space-y-5 p-4">
          <Metadata detail={detail} />
          {!isRemote && detail.skill.is_external ? <p className="rounded-lg border border-border/60 bg-muted/35 p-3 text-xs text-muted-foreground">{t("externalReadOnly", { source: detail.skill.source_kind })}</p> : null}
          <RiskSummary risk={risk} />
          <SkillPreview markdown={detail.skill_markdown} />
          <FileTree files={detail.files} />
          {manifest ? <ManifestMetadata manifest={manifest} /> : null}
        </div>
      </ScrollArea>
    </div>
  );
}

function Metadata({ detail }: { detail: SkillDetail | RemoteSkillDetail }) {
  const { t } = useTranslation("skills");
  const remote = "changelog" in detail;
  const rows = remote
    ? [[t("version"), detail.skill.version], [t("author"), detail.skill.owner], [t("source"), detail.skill.provider], [t("license"), detail.license]]
    : [[t("version"), detail.skill.version], [t("source"), detail.skill.source_kind], [t("installedAt"), detail.skill.installed_at], [t("compatibility"), detail.manifest.compatibility]];
  return (
    <dl className="grid grid-cols-2 gap-x-4 gap-y-2 text-xs">
      {rows.map(([label, value]) => (
        <div key={label}>
          <dt className="text-muted-foreground">{label}</dt>
          <dd className="mt-0.5 truncate text-foreground">{value ?? t("unknown")}</dd>
        </div>
      ))}
    </dl>
  );
}

function RiskSummary({ risk }: { risk: SkillRiskReport }) {
  const { t } = useTranslation("skills");
  const notes = [...risk.notes, ...riskLabels(risk, t)];
  return (
    <section>
      <h4 className="flex items-center gap-1.5 text-xs font-medium"><ShieldCheck className="size-3.5" />{t("risk")}</h4>
      <div className="mt-2 rounded-lg border border-border/60 bg-muted/35 p-3">
        {notes.length === 0 ? <p className="text-xs text-muted-foreground">{t("noRisk")}</p> : <RiskNotes notes={notes} />}
        {risk.remote_scan_status ? <p className="mt-2 text-[11px] text-muted-foreground">{t("riskStatus", { status: risk.remote_scan_status })}</p> : null}
      </div>
    </section>
  );
}

function RiskNotes({ notes }: { notes: string[] }) {
  return (
    <ul className="space-y-1.5 text-xs text-muted-foreground">
      {Array.from(new Set(notes)).map((note) => <li key={note} className="flex gap-1.5"><ShieldAlert className="mt-0.5 size-3 shrink-0 text-muted-foreground" />{note}</li>)}
    </ul>
  );
}

function riskLabels(risk: SkillRiskReport, t: (key: string) => string) {
  return [
    risk.has_scripts ? t("riskScripts") : null,
    risk.has_binary_files ? t("riskBinary") : null,
    risk.has_allowed_tools ? t("riskTools") : null,
  ].filter((note): note is string => Boolean(note));
}

function SkillPreview({ markdown }: { markdown: string | null }) {
  const { t } = useTranslation("skills");
  if (!markdown) return null;
  return (
    <section>
      <h4 className="flex items-center gap-1.5 text-xs font-medium"><FileCode2 className="size-3.5" />{t("readme")}</h4>
      <pre className="mt-2 max-h-72 overflow-auto rounded-lg border border-border/60 bg-muted/35 p-3 font-mono text-[11px] leading-5 text-foreground whitespace-pre-wrap">{markdown}</pre>
    </section>
  );
}

function FileTree({ files }: { files: SkillFileNode[] }) {
  const { t } = useTranslation("skills");
  if (files.length === 0) return null;
  return (
    <details>
      <summary className="flex cursor-pointer list-none items-center gap-1.5 text-xs font-medium"><FolderTree className="size-3.5" />{t("fileTree")} · {t("files", { count: files.length })}</summary>
      <ul className="mt-2 space-y-1 rounded-lg border border-border/60 bg-muted/20 p-2 font-mono text-[11px] text-muted-foreground">
        {files.map((file) => <li key={file.path} className="truncate">{file.path}</li>)}
      </ul>
    </details>
  );
}

function ManifestMetadata({ manifest }: { manifest: NonNullable<SkillDetail["manifest"]> }) {
  const { t } = useTranslation("skills");
  if (!manifest.allowed_tools) return null;
  return <p className="text-xs text-muted-foreground">{t("compatibility")}: {manifest.compatibility ?? t("unknown")}</p>;
}
