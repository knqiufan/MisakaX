import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent,
  type ReactNode,
} from "react";
import {
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  FileText,
  Folder,
  PackageOpen,
  ShieldAlert,
  ShieldCheck,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { skillsIpc } from "@/lib/ipc";
import { cn } from "@/lib/utils";
import type {
  RemoteSkillDetail,
  SkillFileEntry,
  SkillFilePage,
  SkillFilePreview,
  SkillRiskReport,
  SkillScanSummary,
  SkillSummary,
} from "@/lib/ipc";

interface SkillDetailPanelProps {
  detail: SkillSummary | RemoteSkillDetail | null;
  initialFiles?: SkillFilePage | null;
  actions?: ReactNode;
  onBack: () => void;
}

interface FlatTreeRow {
  entry: SkillFileEntry;
  level: number;
  parent: string;
}

const PANEL_CLASS =
  "flex min-h-[340px] min-w-0 flex-col overflow-hidden rounded-xl border border-border/70 bg-[color:var(--surface-card)] lg:h-full lg:min-h-0";
const STANDARD_RISK_NOTES = new Set([
  "This skill includes executable scripts; installation does not run them.",
  "This skill includes binary assets; inspect them before use.",
  "This skill declares allowed tools that require a runtime review.",
]);

export function SkillDetailPanel({
  detail,
  initialFiles,
  actions,
  onBack,
}: SkillDetailPanelProps) {
  const { t } = useTranslation("skills");
  const [activeTab, setActiveTab] = useState("files");
  const [scanSummary, setScanSummary] = useState<SkillScanSummary | null>(null);
  const [scanLoading, setScanLoading] = useState(false);
  const scanRequestId = useRef(0);
  const detailId = detailIdentity(detail);

  useEffect(() => {
    setActiveTab("files");
    setScanSummary(null);
    setScanLoading(false);
    scanRequestId.current += 1;
  }, [detailId]);

  const handleTabChange = useCallback(
    (value: string) => {
      setActiveTab(value);
      if (value !== "security" || !detail || !("generation" in detail) || scanSummary) {
        return;
      }
      const requestId = ++scanRequestId.current;
      setScanLoading(true);
      void skillsIpc
        .getScanSummary(detail.skill.skill_id)
        .then((summary) => {
          if (
            requestId === scanRequestId.current &&
            summary.generation === detail.generation
          ) {
            setScanSummary(summary);
          }
        })
        .catch((error) => {
          if (requestId === scanRequestId.current) toast.error(String(error));
        })
        .finally(() => {
          if (requestId === scanRequestId.current) setScanLoading(false);
        });
    },
    [detail, scanSummary],
  );

  if (!detail) return <EmptyDetail />;

  const installed = "generation" in detail;
  const title = installed ? detail.skill.name : detail.skill.display_name;
  const description = installed ? detail.skill.description : detail.skill.summary;

  return (
    <section className={PANEL_CLASS} aria-label={title}>
      <div className="flex shrink-0 flex-wrap items-start gap-3 border-b border-border/60 p-4">
        <Button
          variant="ghost"
          size="icon-xs"
          onClick={onBack}
          aria-label={t("backToList")}
        >
          <ChevronLeft className="size-3.5" />
        </Button>
        <div className="min-w-0 flex-1 overflow-hidden">
          <h3 className="truncate text-base font-semibold text-foreground" title={title}>
            {title}
          </h3>
          <p className="mt-1 line-clamp-2 break-words text-xs leading-5 text-muted-foreground">
            {description}
          </p>
          <p className="mt-1 truncate text-[11px] text-muted-foreground" title={sourceSummary(detail)}>
            {sourceSummary(detail)}
          </p>
        </div>
        {actions ? (
          <div className="ml-auto flex shrink-0 flex-wrap items-center justify-end gap-1.5">
            {actions}
          </div>
        ) : null}
      </div>
      <Tabs
        value={activeTab}
        onValueChange={handleTabChange}
        className="min-h-0 min-w-0 flex-1 gap-0"
      >
        <div className="shrink-0 border-b border-border/60 px-4 py-2">
          <TabsList variant="line" aria-label={t("detailTabs")}>
            <TabsTrigger value="files">{t("filesTab")}</TabsTrigger>
            <TabsTrigger value="security">{t("securityTab")}</TabsTrigger>
            <TabsTrigger value="overview">{t("overviewTab")}</TabsTrigger>
          </TabsList>
        </div>
        <TabsContent value="files" className="mt-0 min-h-0 min-w-0 flex-1 overflow-hidden">
          {installed ? (
            <SkillFileBrowser detail={detail} initialFiles={initialFiles ?? null} />
          ) : (
            <RemoteFilesEmpty />
          )}
        </TabsContent>
        <TabsContent value="security" className="mt-0 min-h-0 flex-1 overflow-hidden">
          <ScrollArea className="h-full min-h-0">
            <div className="space-y-5 p-4 pb-8">
              <SecurityPanel
                risk={installed ? detail.skill.risk : detail.risk}
                scanSummary={scanSummary}
                loading={scanLoading}
                remote={!installed}
              />
            </div>
          </ScrollArea>
        </TabsContent>
        <TabsContent value="overview" className="mt-0 min-h-0 flex-1 overflow-hidden">
          <ScrollArea className="h-full min-h-0">
            <div className="space-y-5 p-4 pb-8">
              <Overview detail={detail} />
            </div>
          </ScrollArea>
        </TabsContent>
      </Tabs>
    </section>
  );
}

function EmptyDetail() {
  const { t } = useTranslation("skills");
  return (
    <section className={PANEL_CLASS} aria-label={t("details")}>
      <div className="flex h-12 shrink-0 items-center border-b border-border/60 px-4">
        <h3 className="text-sm font-semibold text-foreground">{t("details")}</h3>
      </div>
      <div className="flex min-h-0 flex-1 flex-col items-center justify-center px-8 py-10 text-center">
        <PackageOpen className="size-5 text-muted-foreground/70" aria-hidden="true" />
        <p className="mt-3 text-sm font-medium text-foreground">{t("detailEmptyTitle")}</p>
        <p className="mt-1 max-w-xs text-xs leading-5 text-muted-foreground">
          {t("detailEmptyDescription")}
        </p>
      </div>
    </section>
  );
}

function SkillFileBrowser({
  detail,
  initialFiles,
}: {
  detail: SkillSummary;
  initialFiles: SkillFilePage | null;
}) {
  const { t } = useTranslation("skills");
  const [children, setChildren] = useState<Map<string, SkillFileEntry[]>>(() => new Map());
  const [nextCursors, setNextCursors] = useState<Map<string, string | null>>(() => new Map());
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set());
  const [loadingDirectories, setLoadingDirectories] = useState<Set<string>>(() => new Set());
  const [focusedPath, setFocusedPath] = useState<string | null>(null);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [preview, setPreview] = useState<SkillFilePreview | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const fileRequestId = useRef(0);
  const detailContext = useRef("");
  const rowRefs = useRef(new Map<string, HTMLButtonElement>());

  useEffect(() => {
    const rootItems =
      initialFiles?.generation === detail.generation &&
      initialFiles.skill_id === detail.skill.skill_id
        ? initialFiles.items
        : [];
    setChildren(new Map([["", rootItems]]));
    setNextCursors(new Map([["", initialFiles?.next_cursor ?? null]]));
    setExpanded(new Set());
    setLoadingDirectories(new Set());
    setFocusedPath(rootItems[0]?.path ?? null);
    setSelectedPath(null);
    setPreview(null);
    setPreviewError(null);
    setPreviewLoading(false);
    detailContext.current = `${detail.skill.skill_id}:${detail.generation}`;
    fileRequestId.current += 1;
  }, [detail.generation, detail.skill.skill_id, initialFiles]);

  const rows = useMemo(() => flattenTree(children, expanded), [children, expanded]);

  useEffect(() => {
    if (focusedPath) rowRefs.current.get(focusedPath)?.focus();
  }, [focusedPath]);

  const loadDirectory = useCallback(
    async (entry: SkillFileEntry) => {
      if (!entry.is_directory || children.has(entry.path)) return;
      const requestContext = `${detail.skill.skill_id}:${detail.generation}`;
      setLoadingDirectories((current) => new Set(current).add(entry.path));
      try {
        const page = await skillsIpc.listFiles(
          detail.skill.skill_id,
          entry.path,
          undefined,
          500,
        );
        if (
          requestContext !== detailContext.current ||
          page.generation !== detail.generation
        ) {
          return;
        }
        setChildren((current) => new Map(current).set(entry.path, page.items));
        setNextCursors((current) => new Map(current).set(entry.path, page.next_cursor));
      } catch (error) {
        toast.error(String(error));
      } finally {
        setLoadingDirectories((current) => {
          const next = new Set(current);
          next.delete(entry.path);
          return next;
        });
      }
    },
    [children, detail.generation, detail.skill.skill_id],
  );

  const loadNextDirectoryPage = useCallback(
    async (parent: string, cursor: string) => {
      const requestContext = `${detail.skill.skill_id}:${detail.generation}`;
      setLoadingDirectories((current) => new Set(current).add(parent));
      try {
        const page = await skillsIpc.listFiles(
          detail.skill.skill_id,
          parent || undefined,
          cursor,
          500,
        );
        if (
          requestContext !== detailContext.current ||
          page.generation !== detail.generation
        ) {
          return;
        }
        setChildren((current) => {
          const next = new Map(current);
          next.set(parent, [...(next.get(parent) ?? []), ...page.items]);
          return next;
        });
        setNextCursors((current) => new Map(current).set(parent, page.next_cursor));
      } catch (error) {
        toast.error(String(error));
      } finally {
        setLoadingDirectories((current) => {
          const next = new Set(current);
          next.delete(parent);
          return next;
        });
      }
    },
    [detail.generation, detail.skill.skill_id],
  );

  const toggleDirectory = useCallback(
    (entry: SkillFileEntry) => {
      if (entry.is_link) return;
      const willExpand = !expanded.has(entry.path);
      setExpanded((current) => {
        const next = new Set(current);
        if (willExpand) next.add(entry.path);
        else next.delete(entry.path);
        return next;
      });
      if (willExpand) void loadDirectory(entry);
    },
    [expanded, loadDirectory],
  );

  const selectFile = useCallback(
    async (entry: SkillFileEntry) => {
      if (entry.is_directory) {
        toggleDirectory(entry);
        return;
      }
      if (entry.is_link) return;
      const requestId = ++fileRequestId.current;
      setSelectedPath(entry.path);
      setPreview(null);
      setPreviewError(null);
      setPreviewLoading(true);
      try {
        const loaded = await skillsIpc.readFile(detail.skill.skill_id, entry.path, 0);
        if (
          requestId === fileRequestId.current &&
          loaded.generation === detail.generation &&
          loaded.path === entry.path
        ) {
          setPreview(loaded);
        }
      } catch (error) {
        if (requestId === fileRequestId.current) setPreviewError(String(error));
      } finally {
        if (requestId === fileRequestId.current) setPreviewLoading(false);
      }
    },
    [detail.generation, detail.skill.skill_id, toggleDirectory],
  );

  const continuePreview = useCallback(async () => {
    if (!preview?.next_offset || !selectedPath) return;
    const requestId = ++fileRequestId.current;
    setPreviewLoading(true);
    setPreviewError(null);
    try {
      const loaded = await skillsIpc.readFile(
        detail.skill.skill_id,
        selectedPath,
        preview.next_offset,
      );
      if (
        requestId === fileRequestId.current &&
        loaded.generation === detail.generation &&
        loaded.path === selectedPath
      ) {
        setPreview((current) =>
          current
            ? {
                ...loaded,
                offset: 0,
                content: `${current.content ?? ""}${loaded.content ?? ""}`,
              }
            : loaded,
        );
      }
    } catch (error) {
      if (requestId === fileRequestId.current) setPreviewError(String(error));
    } finally {
      if (requestId === fileRequestId.current) setPreviewLoading(false);
    }
  }, [detail.generation, detail.skill.skill_id, preview, selectedPath]);

  const handleTreeKeyDown = useCallback(
    (event: KeyboardEvent<HTMLButtonElement>, row: FlatTreeRow) => {
      const index = rows.findIndex((candidate) => candidate.entry.path === row.entry.path);
      const moveTo = (nextIndex: number) => {
        const next = rows[nextIndex];
        if (next) setFocusedPath(next.entry.path);
      };
      switch (event.key) {
        case "ArrowDown":
          event.preventDefault();
          moveTo(Math.min(rows.length - 1, index + 1));
          break;
        case "ArrowUp":
          event.preventDefault();
          moveTo(Math.max(0, index - 1));
          break;
        case "Home":
          event.preventDefault();
          moveTo(0);
          break;
        case "End":
          event.preventDefault();
          moveTo(rows.length - 1);
          break;
        case "ArrowRight":
          if (!row.entry.is_directory) break;
          event.preventDefault();
          if (!expanded.has(row.entry.path)) toggleDirectory(row.entry);
          else moveTo(index + 1);
          break;
        case "ArrowLeft":
          event.preventDefault();
          if (row.entry.is_directory && expanded.has(row.entry.path)) {
            toggleDirectory(row.entry);
          } else if (row.parent) {
            setFocusedPath(row.parent);
          }
          break;
        case "Enter":
        case " ":
          event.preventDefault();
          void selectFile(row.entry);
          break;
      }
    },
    [expanded, rows, selectFile, toggleDirectory],
  );

  return (
    <div className="grid h-full min-h-0 min-w-0 grid-cols-[minmax(13rem,0.78fr)_minmax(0,1.22fr)] max-md:grid-cols-1 max-md:grid-rows-[minmax(12rem,0.8fr)_minmax(14rem,1.2fr)]">
      <div className="flex min-h-0 min-w-0 flex-col border-r border-border/60 max-md:border-r-0 max-md:border-b">
        <div className="flex h-9 shrink-0 items-center justify-between px-3 text-[11px] text-muted-foreground">
          <span>{t("fileTree")}</span>
          <span>{t("files", { count: rows.filter((row) => !row.entry.is_directory).length })}</span>
        </div>
        <ScrollArea className="min-h-0 flex-1">
          {rows.length ? (
            <>
              <div role="tree" aria-label={t("fileTree")} className="min-w-0 py-1">
                {rows.map((row) => {
                const directory = row.entry.is_directory;
                const isExpanded = directory && expanded.has(row.entry.path);
                return (
                  <button
                    key={row.entry.path}
                    ref={(node) => {
                      if (node) rowRefs.current.set(row.entry.path, node);
                      else rowRefs.current.delete(row.entry.path);
                    }}
                    type="button"
                    role="treeitem"
                    aria-level={row.level}
                    aria-expanded={directory ? isExpanded : undefined}
                    aria-selected={!directory && selectedPath === row.entry.path}
                    tabIndex={focusedPath === row.entry.path ? 0 : -1}
                    disabled={row.entry.is_link}
                    onFocus={() => setFocusedPath(row.entry.path)}
                    onClick={() => void selectFile(row.entry)}
                    onKeyDown={(event) => handleTreeKeyDown(event, row)}
                    title={row.entry.path}
                    className={cn(
                      "flex min-h-8 w-full min-w-0 cursor-pointer items-center gap-1.5 pr-2 text-left font-mono text-[11px] transition-colors duration-150 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring/55 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50",
                      selectedPath === row.entry.path
                        ? "bg-primary/10 text-foreground"
                        : "text-muted-foreground hover:bg-muted/60 hover:text-foreground",
                    )}
                    style={{ paddingLeft: `${8 + (row.level - 1) * 14}px` }}
                  >
                    {directory ? (
                      loadingDirectories.has(row.entry.path) ? (
                        <span className="size-3.5 animate-pulse rounded-sm bg-muted" />
                      ) : isExpanded ? (
                        <ChevronDown className="size-3.5 shrink-0" />
                      ) : (
                        <ChevronRight className="size-3.5 shrink-0" />
                      )
                    ) : (
                      <span className="size-3.5 shrink-0" />
                    )}
                    {directory ? (
                      <Folder className="size-3.5 shrink-0" />
                    ) : (
                      <FileText className="size-3.5 shrink-0" />
                    )}
                    <span className="min-w-0 flex-1 truncate">{row.entry.name}</span>
                    {!directory ? (
                      <span className="shrink-0 text-[10px] tabular-nums">
                        {formatFileSize(row.entry.size_bytes)}
                      </span>
                    ) : null}
                  </button>
                );
                })}
              </div>
              {Array.from(nextCursors.entries())
                .filter(([parent, cursor]) => Boolean(cursor) && (!parent || expanded.has(parent)))
                .map(([parent, cursor]) => (
                  <button
                    key={`more:${parent}`}
                    type="button"
                    className="mx-2 my-1 min-h-8 cursor-pointer rounded-md px-2 text-left text-[11px] text-primary hover:bg-muted/60 focus-visible:ring-2 focus-visible:ring-ring/55 focus-visible:outline-none"
                    disabled={loadingDirectories.has(parent)}
                    onClick={() => void loadNextDirectoryPage(parent, cursor!)}
                  >
                    {t("loadMoreFiles", { parent: parent || t("rootDirectory") })}
                  </button>
                ))}
            </>
          ) : (
            <p className="m-3 rounded-lg border border-dashed border-border/70 p-3 text-xs text-muted-foreground">
              {t("fileTreeUnavailable")}
            </p>
          )}
        </ScrollArea>
      </div>
      <FilePreviewPane
        selectedPath={selectedPath}
        preview={preview}
        loading={previewLoading}
        error={previewError}
        onContinue={continuePreview}
      />
    </div>
  );
}

function FilePreviewPane({
  selectedPath,
  preview,
  loading,
  error,
  onContinue,
}: {
  selectedPath: string | null;
  preview: SkillFilePreview | null;
  loading: boolean;
  error: string | null;
  onContinue: () => void;
}) {
  const { t } = useTranslation("skills");
  return (
    <div className="flex min-h-0 min-w-0 flex-col bg-background/25">
      <div className="flex h-9 shrink-0 items-center border-b border-border/45 px-3">
        <span className="truncate font-mono text-[11px] text-muted-foreground" title={selectedPath ?? undefined}>
          {selectedPath ?? t("filePreview")}
        </span>
      </div>
      <ScrollArea className="min-h-0 min-w-0 flex-1">
        {!selectedPath ? (
          <PreviewEmpty text={t("selectFileToPreview")} />
        ) : loading && !preview ? (
          <div aria-label={t("loadingPreview")} className="space-y-2 p-4">
            {Array.from({ length: 7 }, (_, index) => (
              <div key={index} className="h-3 animate-pulse rounded bg-muted/70" style={{ width: `${70 + (index % 3) * 10}%` }} />
            ))}
          </div>
        ) : error ? (
          <div role="alert" className="m-4 rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-xs text-destructive">
            {error}
          </div>
        ) : preview?.kind === "text" ? (
          <div className="min-w-0">
            <pre className="min-w-full whitespace-pre-wrap break-words p-4 font-mono text-[11px] leading-5 text-foreground [overflow-wrap:anywhere]">
              {preview.content}
            </pre>
            <PreviewProgress preview={preview} loading={loading} onContinue={onContinue} />
          </div>
        ) : preview ? (
          <UnsupportedPreview preview={preview} />
        ) : null}
      </ScrollArea>
    </div>
  );
}

function PreviewProgress({
  preview,
  loading,
  onContinue,
}: {
  preview: SkillFilePreview;
  loading: boolean;
  onContinue: () => void;
}) {
  const { t } = useTranslation("skills");
  const loaded = preview.next_offset ?? preview.total_size_bytes;
  return (
    <div className="sticky bottom-0 flex min-h-10 items-center justify-between gap-3 border-t border-border/60 bg-background/95 px-3 py-2 text-[11px] text-muted-foreground">
      <span>{t("previewProgress", { loaded: formatFileSize(loaded), total: formatFileSize(preview.total_size_bytes) })}</span>
      {preview.next_offset !== null ? (
        <Button size="xs" variant="outline" disabled={loading} onClick={onContinue}>
          {loading ? t("loadingPreview") : t("loadMorePreview")}
        </Button>
      ) : null}
    </div>
  );
}

function UnsupportedPreview({ preview }: { preview: SkillFilePreview }) {
  const { t } = useTranslation("skills");
  return (
    <div className="m-4 rounded-lg border border-border/70 bg-muted/25 p-4">
      <p className="text-sm font-medium text-foreground">
        {t(preview.kind === "binary" ? "binaryPreviewUnavailable" : "encodingPreviewUnavailable")}
      </p>
      <dl className="mt-3 grid grid-cols-[auto_1fr] gap-x-3 gap-y-2 text-xs">
        <dt className="text-muted-foreground">{t("fileSize")}</dt>
        <dd>{formatFileSize(preview.total_size_bytes)}</dd>
        <dt className="text-muted-foreground">{t("encoding")}</dt>
        <dd>{preview.encoding ?? t("unknown")}</dd>
        {preview.sha256 ? (
          <>
            <dt className="text-muted-foreground">SHA-256</dt>
            <dd className="min-w-0 break-all font-mono text-[10px]">{preview.sha256}</dd>
          </>
        ) : null}
      </dl>
    </div>
  );
}

function PreviewEmpty({ text }: { text: string }) {
  return (
    <div className="flex min-h-52 items-center justify-center p-6 text-center text-xs text-muted-foreground">
      {text}
    </div>
  );
}

function RemoteFilesEmpty() {
  const { t } = useTranslation("skills");
  return <PreviewEmpty text={t("remoteFilesRequireInstall")} />;
}

function SecurityPanel({
  risk,
  scanSummary,
  loading,
  remote,
}: {
  risk: SkillRiskReport;
  scanSummary: SkillScanSummary | null;
  loading: boolean;
  remote: boolean;
}) {
  const { t } = useTranslation("skills");
  const notes = [
    ...risk.notes.filter((note) => !STANDARD_RISK_NOTES.has(note)),
    ...riskLabels(risk, t),
  ];
  return (
    <section>
      <h4 className="flex items-center gap-1.5 text-sm font-medium text-foreground">
        <ShieldCheck className="size-4" />
        {t("securitySummary")}
      </h4>
      <div className="mt-3 rounded-lg border border-border/60 bg-muted/30 p-4">
        {loading ? (
          <div className="h-12 animate-pulse rounded bg-muted/70" />
        ) : scanSummary ? (
          <>
            <p className="text-sm font-medium text-foreground">{t(`scanState.${scanSummary.state}`)}</p>
            {scanSummary.placeholder ? (
              <p className="mt-1 text-xs leading-5 text-muted-foreground">{t("scanPlaceholder")}</p>
            ) : null}
          </>
        ) : remote ? (
          <p className="text-xs leading-5 text-muted-foreground">{t("remoteSecurityMetadata")}</p>
        ) : null}
      </div>
      <div className="mt-4 rounded-lg border border-border/60 bg-muted/20 p-3">
        {notes.length === 0 ? (
          <p className="text-xs text-muted-foreground">{t("noRisk")}</p>
        ) : (
          <ul className="space-y-1.5 text-xs leading-5 text-muted-foreground">
            {Array.from(new Set(notes)).map((note) => (
              <li key={note} className="flex gap-1.5">
                <ShieldAlert className="mt-0.5 size-3 shrink-0" />
                {note}
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}

function Overview({ detail }: { detail: SkillSummary | RemoteSkillDetail }) {
  const { t } = useTranslation("skills");
  const installed = "generation" in detail;
  const rows: Array<[string, string | null | undefined]> = installed
    ? [
        [t("version"), detail.skill.version],
        [t("source"), detail.skill.source_kind],
        [t("health"), detail.skill.health],
        [t("license"), detail.manifest.license],
        [t("compatibility"), detail.manifest.compatibility],
        [t("installedAt"), detail.skill.installed_at],
      ]
    : [
        [t("version"), detail.skill.version],
        [t("author"), detail.skill.owner],
        [t("source"), detail.skill.provider],
        [t("license"), detail.license],
        [t("compatibility"), detail.compatibility],
      ];
  const allowedTools = installed ? detail.manifest.allowed_tools : detail.manifest?.allowed_tools;
  return (
    <>
      <dl className="grid min-w-0 grid-cols-2 gap-x-4 gap-y-3 text-xs">
        {rows.map(([label, value]) => (
          <div key={label} className="min-w-0 overflow-hidden">
            <dt className="text-muted-foreground">{label}</dt>
            <dd className="mt-0.5 break-words text-foreground" title={value ?? undefined}>
              {value ?? t("unknown")}
            </dd>
          </div>
        ))}
      </dl>
      {installed && detail.skill.is_external ? (
        <p className="break-words rounded-lg border border-border/60 bg-muted/35 p-3 text-xs leading-5 text-muted-foreground">
          {t("externalReadOnly", { source: detail.skill.source_kind })}
        </p>
      ) : null}
      {allowedTools ? (
        <section>
          <h4 className="text-xs font-medium text-foreground">{t("allowedTools")}</h4>
          <p className="mt-2 break-words rounded-lg border border-border/60 bg-muted/20 p-3 font-mono text-[11px] leading-5 text-muted-foreground [overflow-wrap:anywhere]">
            {allowedTools}
          </p>
        </section>
      ) : null}
    </>
  );
}

function flattenTree(
  children: Map<string, SkillFileEntry[]>,
  expanded: Set<string>,
): FlatTreeRow[] {
  const rows: FlatTreeRow[] = [];
  const visit = (parent: string, level: number) => {
    for (const entry of children.get(parent) ?? []) {
      rows.push({ entry, level, parent });
      if (entry.is_directory && expanded.has(entry.path)) visit(entry.path, level + 1);
    }
  };
  visit("", 1);
  return rows;
}

function riskLabels(risk: SkillRiskReport, t: (key: string) => string) {
  return [
    risk.has_scripts ? t("riskScripts") : null,
    risk.has_binary_files ? t("riskBinary") : null,
    risk.has_allowed_tools ? t("riskTools") : null,
  ].filter((note): note is string => Boolean(note));
}

function detailIdentity(detail: SkillSummary | RemoteSkillDetail | null) {
  if (!detail) return "empty";
  return "generation" in detail
    ? `${detail.skill.skill_id}:${detail.generation}`
    : `${detail.skill.provider}:${detail.skill.slug}`;
}

function sourceSummary(detail: SkillSummary | RemoteSkillDetail) {
  return "generation" in detail
    ? `${detail.skill.source_kind} · ${detail.skill.version ?? "unknown"} · ${detail.skill.health} · ${detail.skill.installed_path}`
    : `${detail.skill.provider} · ${detail.skill.version ?? detail.skill.owner ?? ""}`;
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} KiB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
}
