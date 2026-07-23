import { Search, Upload, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

export type SkillsView = "installed" | "discover";

interface SkillsToolbarProps {
  view: SkillsView;
  query: string;
  source: string;
  busy: boolean;
  onViewChange: (view: SkillsView) => void;
  onQueryChange: (query: string) => void;
  onSourceChange: (source: string) => void;
  onUpload: () => void;
  onModelScope: () => void;
  onRefresh: () => void;
}

export function SkillsToolbar({
  view,
  query,
  source,
  busy,
  onViewChange,
  onQueryChange,
  onSourceChange,
  onUpload,
  onModelScope,
  onRefresh,
}: SkillsToolbarProps) {
  const { t } = useTranslation("skills");
  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-xl font-semibold tracking-tight">{t("title")}</h2>
          <p className="mt-1 text-sm text-muted-foreground">{t("description")}</p>
        </div>
        <div className="flex items-center gap-2">
          <Button size="sm" variant="outline" onClick={onRefresh} disabled={busy}>
            <RefreshCw className={cn("size-3.5", busy && "animate-spin")} />
            {t("refresh")}
          </Button>
          <Button size="sm" variant="outline" onClick={onModelScope}>
            {t("sourceModelscope")}
          </Button>
          <Button size="sm" onClick={onUpload}>
            <Upload className="size-3.5" />
            {t("upload")}
          </Button>
        </div>
      </div>
      <div className="flex flex-col gap-2 md:flex-row">
        <ViewTabs value={view} onChange={onViewChange} />
        <div className="relative min-w-0 flex-1">
          <Search className="pointer-events-none absolute top-1/2 left-3 size-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={query}
            onChange={(event) => onQueryChange(event.target.value)}
            placeholder={t("searchPlaceholder")}
            className="pl-9"
            aria-label={t("searchPlaceholder")}
          />
        </div>
        {view === "discover" ? (
          <SourceTabs value={source} onChange={onSourceChange} />
        ) : null}
      </div>
    </div>
  );
}

function ViewTabs({
  value,
  onChange,
}: {
  value: SkillsView;
  onChange: (view: SkillsView) => void;
}) {
  const { t } = useTranslation("skills");
  return (
    <div className="flex shrink-0 rounded-full bg-muted p-0.5" role="tablist">
      {(["installed", "discover"] as const).map((view) => (
        <button
          key={view}
          type="button"
          role="tab"
          aria-selected={value === view}
          onClick={() => onChange(view)}
          className={cn(
            "rounded-full px-3 py-1.5 text-xs font-medium transition-colors duration-150",
            value === view ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"
          )}
        >
          {t(view)}
        </button>
      ))}
    </div>
  );
}

function SourceTabs({ value, onChange }: { value: string; onChange: (source: string) => void }) {
  const { t } = useTranslation("skills");
  const sources = [
    ["all", t("allSources")],
    ["skillhub", t("sourceSkillhub")],
    ["clawhub", t("sourceClawhub")],
  ];
  return (
    <select
      value={value}
      onChange={(event) => onChange(event.target.value)}
      className="h-9 rounded-full border border-border bg-background px-3 text-xs outline-none focus-visible:ring-[3px] focus-visible:ring-ring/30"
      aria-label={t("source")}
    >
      {sources.map(([id, label]) => (
        <option key={id} value={id}>{label}</option>
      ))}
    </select>
  );
}
