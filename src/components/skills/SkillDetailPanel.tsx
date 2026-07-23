import { Fragment, type ReactNode } from "react";
import { ChevronLeft, FileCode2, FileText, Folder, FolderTree, PackageOpen, ShieldCheck, ShieldAlert } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { RemoteSkillDetail, SkillDetail, SkillFileNode, SkillRiskReport } from "@/lib/ipc";

interface SkillDetailPanelProps {
  detail: SkillDetail | RemoteSkillDetail | null;
  actions?: ReactNode;
  onBack: () => void;
}

interface FileTreeNode {
  name: string;
  path: string;
  file?: SkillFileNode;
  children: Map<string, FileTreeNode>;
}

const PANEL_CLASS = "flex min-h-[280px] min-w-0 flex-col overflow-hidden rounded-xl border border-border/70 bg-[color:var(--surface-card)] lg:h-full lg:min-h-0";
const STANDARD_RISK_NOTES = new Set([
  "This skill includes executable scripts; installation does not run them.",
  "This skill includes binary assets; inspect them before use.",
  "This skill declares allowed tools that require a runtime review.",
]);

export function SkillDetailPanel({ detail, actions, onBack }: SkillDetailPanelProps) {
  const { t } = useTranslation("skills");
  if (!detail) {
    return (
      <section className={PANEL_CLASS} aria-label={t("details")}>
        <div className="flex h-12 shrink-0 items-center border-b border-border/60 px-4">
          <h3 className="text-sm font-semibold text-foreground">{t("details")}</h3>
        </div>
        <div className="flex min-h-0 flex-1 flex-col items-center justify-center px-8 py-10 text-center">
          <PackageOpen className="size-5 text-muted-foreground/70" aria-hidden="true" />
          <p className="mt-3 text-sm font-medium text-foreground">{t("detailEmptyTitle")}</p>
          <p className="mt-1 max-w-xs text-xs leading-5 text-muted-foreground">{t("detailEmptyDescription")}</p>
        </div>
      </section>
    );
  }

  const manifest = detail.manifest;
  const isRemote = "changelog" in detail;
  const risk = isRemote ? detail.risk : detail.skill.risk;
  const title = isRemote ? detail.skill.display_name : detail.skill.name;
  const description = isRemote ? detail.skill.summary : detail.skill.description;

  return (
    <section className={PANEL_CLASS} aria-label={title}>
      <div className="flex shrink-0 flex-wrap items-start gap-3 border-b border-border/60 p-4">
        <Button variant="ghost" size="icon-xs" onClick={onBack} aria-label={t("backToList")}>
          <ChevronLeft className="size-3.5" />
        </Button>
        <div className="min-w-0 flex-1 overflow-hidden">
          <h3 className="truncate text-base font-semibold text-foreground" title={title}>{title}</h3>
          <p className="mt-1 break-words text-xs leading-5 text-muted-foreground">{description}</p>
        </div>
        {actions ? <div className="ml-auto flex shrink-0 flex-wrap justify-end gap-1.5">{actions}</div> : null}
      </div>
      <ScrollArea className="min-h-0 min-w-0 flex-1">
        <div className="w-full min-w-0 space-y-5 p-4 pb-8">
          <Metadata detail={detail} />
          {!isRemote && detail.skill.is_external ? (
            <p className="break-words rounded-lg border border-border/60 bg-muted/35 p-3 text-xs leading-5 text-muted-foreground">
              {t("externalReadOnly", { source: detail.skill.source_kind })}
            </p>
          ) : null}
          <RiskSummary risk={risk} />
          <SkillPreview markdown={detail.skill_markdown} remote={isRemote} />
          <FileTree files={detail.files} remote={isRemote} />
          {manifest ? <ManifestMetadata manifest={manifest} /> : null}
        </div>
      </ScrollArea>
    </section>
  );
}

function Metadata({ detail }: { detail: SkillDetail | RemoteSkillDetail }) {
  const { t } = useTranslation("skills");
  const remote = "changelog" in detail;
  const rows = remote
    ? [[t("version"), detail.skill.version], [t("author"), detail.skill.owner], [t("source"), detail.skill.provider], [t("license"), detail.license]]
    : [[t("version"), detail.skill.version], [t("source"), detail.skill.source_kind], [t("installedAt"), detail.skill.installed_at], [t("compatibility"), detail.manifest.compatibility]];
  return (
    <dl className="grid min-w-0 grid-cols-2 gap-x-4 gap-y-3 text-xs">
      {rows.map(([label, value]) => (
        <div key={label} className="min-w-0 overflow-hidden">
          <dt className="text-muted-foreground">{label}</dt>
          <dd className="mt-0.5 break-words text-foreground" title={value ?? undefined}>{value ?? t("unknown")}</dd>
        </div>
      ))}
    </dl>
  );
}

function RiskSummary({ risk }: { risk: SkillRiskReport }) {
  const { t } = useTranslation("skills");
  const notes = [
    ...risk.notes.filter((note) => !STANDARD_RISK_NOTES.has(note)),
    ...riskLabels(risk, t),
  ];
  return (
    <section>
      <h4 className="flex items-center gap-1.5 text-xs font-medium text-foreground"><ShieldCheck className="size-3.5" />{t("risk")}</h4>
      <div className="mt-2 rounded-lg border border-border/60 bg-muted/35 p-3">
        {notes.length === 0 ? <p className="text-xs text-muted-foreground">{t("noRisk")}</p> : <RiskNotes notes={notes} />}
        {risk.remote_scan_status ? <p className="mt-2 text-[11px] text-muted-foreground">{t("riskStatus", { status: risk.remote_scan_status })}</p> : null}
      </div>
    </section>
  );
}

function RiskNotes({ notes }: { notes: string[] }) {
  return (
    <ul className="space-y-1.5 text-xs leading-5 text-muted-foreground">
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

function SkillPreview({ markdown, remote }: { markdown: string | null; remote: boolean }) {
  const { t } = useTranslation("skills");
  return (
    <section className="min-w-0">
      <h4 className="flex items-center gap-1.5 text-xs font-medium text-foreground"><FileCode2 className="size-3.5 shrink-0" />{t("readme")}</h4>
      {markdown ? (
        <pre className="mt-2 w-full min-w-0 max-w-full overflow-x-auto rounded-lg border border-border/60 bg-muted/35 p-3 font-mono text-[11px] leading-5 break-words whitespace-pre-wrap [overflow-wrap:anywhere] text-foreground">
          {markdown}
        </pre>
      ) : (
        <p className="mt-2 break-words rounded-lg border border-dashed border-border/70 bg-muted/20 p-3 text-xs leading-5 text-muted-foreground">
          {t(remote ? "previewUnavailableRemote" : "previewUnavailable")}
        </p>
      )}
    </section>
  );
}

function FileTree({ files, remote }: { files: SkillFileNode[]; remote: boolean }) {
  const { t } = useTranslation("skills");
  const tree = buildFileTree(files);
  return (
    <section className="min-w-0">
      <div className="flex min-w-0 items-center justify-between gap-3">
        <h4 className="flex min-w-0 items-center gap-1.5 text-xs font-medium text-foreground"><FolderTree className="size-3.5 shrink-0" />{t("fileTree")}</h4>
        {files.length > 0 ? <span className="shrink-0 text-[11px] tabular-nums text-muted-foreground">{t("files", { count: files.length })}</span> : null}
      </div>
      <p className="mt-1 text-[11px] leading-4 text-muted-foreground">{t("fileTreeDescription")}</p>
      {tree.length > 0 ? (
        <div className="mt-2 w-full min-w-0 overflow-x-auto rounded-lg border border-border/60 bg-muted/20">
          <ul className="min-w-0 py-1" role="tree" aria-label={t("fileTree")}>
            <FileTreeRows nodes={tree} depth={0} />
          </ul>
        </div>
      ) : (
        <p className="mt-2 break-words rounded-lg border border-dashed border-border/70 bg-muted/20 p-3 text-xs leading-5 text-muted-foreground">
          {t(remote ? "fileTreeUnavailableRemote" : "fileTreeUnavailable")}
        </p>
      )}
    </section>
  );
}

function FileTreeRows({ nodes, depth }: { nodes: FileTreeNode[]; depth: number }) {
  return (
    <>
      {nodes.map((node) => {
        const isDirectory = node.children.size > 0;
        const file = node.file;
        return (
          <Fragment key={node.path}>
            <li
              role="treeitem"
              aria-level={depth + 1}
              className="flex min-h-8 min-w-0 items-center gap-2 border-b border-border/45 px-2.5 py-1 last:border-b-0"
              style={{ paddingLeft: `${10 + depth * 14}px` }}
              title={node.path}
            >
              {isDirectory ? <Folder className="size-3.5 shrink-0 text-muted-foreground" aria-hidden="true" /> : <FileText className="size-3.5 shrink-0 text-muted-foreground" aria-hidden="true" />}
              <span className="min-w-0 flex-1 truncate font-mono text-[11px] text-foreground">{node.name}</span>
              {file ? <span className="shrink-0 text-[10px] tabular-nums text-muted-foreground">{formatFileSize(file.size_bytes)}</span> : null}
              {file ? <span className="max-w-24 shrink-0 truncate rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">{file.kind}</span> : null}
            </li>
            {isDirectory ? <FileTreeRows nodes={sortFileTreeNodes(node.children)} depth={depth + 1} /> : null}
          </Fragment>
        );
      })}
    </>
  );
}

function buildFileTree(files: SkillFileNode[]) {
  const root: FileTreeNode = { name: "", path: "", children: new Map() };
  for (const file of files) {
    const parts = file.path.replace(/\\/g, "/").split("/").filter(Boolean);
    let current = root;
    parts.forEach((name, index) => {
      const path = current.path ? `${current.path}/${name}` : name;
      const existing = current.children.get(name);
      const next = existing ?? { name, path, children: new Map<string, FileTreeNode>() };
      if (!existing) current.children.set(name, next);
      if (index === parts.length - 1) next.file = file;
      current = next;
    });
  }
  return sortFileTreeNodes(root.children);
}

function sortFileTreeNodes(nodes: Map<string, FileTreeNode>) {
  return Array.from(nodes.values()).sort((left, right) => {
    const leftDirectory = left.children.size > 0;
    const rightDirectory = right.children.size > 0;
    if (leftDirectory !== rightDirectory) return leftDirectory ? -1 : 1;
    return left.name.localeCompare(right.name);
  });
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function ManifestMetadata({ manifest }: { manifest: NonNullable<SkillDetail["manifest"]> }) {
  const { t } = useTranslation("skills");
  if (!manifest.allowed_tools) return null;
  return (
    <section>
      <h4 className="text-xs font-medium text-foreground">{t("allowedTools")}</h4>
      <p className="mt-2 break-words rounded-lg border border-border/60 bg-muted/20 p-3 font-mono text-[11px] leading-5 [overflow-wrap:anywhere] text-muted-foreground">{manifest.allowed_tools}</p>
    </section>
  );
}
