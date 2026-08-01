import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { CheckCircle2, ChevronDown, Download, Loader2, PackageOpen, ShieldAlert } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
import type { InstalledSkill, RemoteSkill } from "@/lib/ipc";

const INITIAL_BATCH_SIZE = 10;
const LOAD_MORE_DELAY_MS = 160;
const SOURCE_ORDER = [
  "local",
  "skillhub",
  "clawhub",
  "modelscope",
  "codex",
  "claude",
  "cursor",
] as const;

type ListSkill = InstalledSkill | RemoteSkill;

interface SkillsListProps {
  installed: InstalledSkill[];
  remote: RemoteSkill[];
  view: "installed" | "discover";
  loading: boolean;
  query: string;
  selectedKey: string | null;
  pendingSkillIds?: Set<string>;
  onSelectInstalled: (skill: InstalledSkill) => void;
  onToggleInstalled?: (skill: InstalledSkill, enabled: boolean) => void;
  onSelectRemote: (skill: RemoteSkill) => void;
  onInstall: (skill: RemoteSkill) => void;
}

export function SkillsList({
  installed,
  remote,
  view,
  loading,
  query,
  selectedKey,
  pendingSkillIds = new Set<string>(),
  onSelectInstalled,
  onToggleInstalled,
  onSelectRemote,
  onInstall,
}: SkillsListProps) {
  const { t } = useTranslation("skills");
  const installedItems = useMemo(
    () => filterInstalled(installed, query).sort(compareInstalledBySource),
    [installed, query],
  );
  const installedSourceCounts = useMemo(() => sourceCounts(installedItems), [installedItems]);
  const remoteItems = useMemo(() => filterRemote(remote, query), [remote, query]);
  const items: ListSkill[] = view === "installed" ? installedItems : remoteItems;
  const fingerprint = useMemo(
    () => `${view}:${items.map(listKey).join("|")}`,
    [items, view],
  );
  const { visibleItems, hasMore, isLoadingMore, sentinelRef } = useIncrementalItems<ListSkill>(
    items,
    fingerprint,
  );
  const title = view === "installed"
    ? t("installedSkills")
    : query.trim()
      ? t("searchResults")
      : t("popularSkills");

  return (
    <section
      aria-label={title}
      className="flex min-h-[280px] min-w-0 flex-col overflow-hidden rounded-xl border border-border/70 bg-[color:var(--surface-card)] lg:h-full lg:min-h-0"
    >
      <div className="flex h-12 shrink-0 items-center justify-between gap-3 border-b border-border/60 px-4">
        <h3 className="min-w-0 truncate text-sm font-semibold text-foreground">{title}</h3>
        <span className="shrink-0 text-[11px] tabular-nums text-muted-foreground">
          {t("skillsVisible", { visible: visibleItems.length, total: items.length })}
        </span>
      </div>
      <ScrollArea className="min-h-0 min-w-0 flex-1">
        <div className="w-full min-w-0 space-y-3 p-3">
          {loading ? <ListLoading /> : null}
          {!loading && items.length === 0 ? <ListEmpty view={view} query={query} /> : null}
          {!loading && items.length > 0 && view === "installed" ? (
            <InstalledSourceGroups
              skills={visibleItems.filter(isInstalledSkill)}
              sourceCounts={installedSourceCounts}
              selectedKey={selectedKey}
              pendingSkillIds={pendingSkillIds}
              onSelect={onSelectInstalled}
              onToggle={onToggleInstalled}
            />
          ) : null}
          {!loading && items.length > 0 && view === "discover" ? (
            <ul className="space-y-1.5" role="list">
              {visibleItems.filter(isRemoteSkill).map((skill) => (
                <li key={`${skill.provider}:${skill.slug}`}>
                  <RemoteListItem
                    skill={skill}
                    selected={selectedKey === remoteKey(skill)}
                    onSelect={() => onSelectRemote(skill)}
                    onInstall={() => onInstall(skill)}
                  />
                </li>
              ))}
            </ul>
          ) : null}
          {!loading && hasMore ? (
            <div
              ref={sentinelRef}
              className="flex min-h-10 items-center justify-center gap-2 py-2 text-xs text-muted-foreground"
              role="status"
              aria-live="polite"
            >
              {isLoadingMore ? <Loader2 className="size-3.5 animate-spin" /> : <ChevronDown className="size-3.5" />}
              <span>{isLoadingMore ? t("loadingMore") : t("scrollToLoadMore")}</span>
            </div>
          ) : null}
        </div>
      </ScrollArea>
    </section>
  );
}

function InstalledSourceGroups({
  skills,
  sourceCounts,
  selectedKey,
  pendingSkillIds,
  onSelect,
  onToggle,
}: {
  skills: InstalledSkill[];
  sourceCounts: Map<string, number>;
  selectedKey: string | null;
  pendingSkillIds: Set<string>;
  onSelect: (skill: InstalledSkill) => void;
  onToggle?: (skill: InstalledSkill, enabled: boolean) => void;
}) {
  return (
    <div className="space-y-4">
      {groupInstalledBySource(skills).map((group) => (
        <section key={group.source} aria-labelledby={`skills-source-${group.source}`}>
          <SourceGroupHeader source={group.source} count={sourceCounts.get(group.source) ?? group.skills.length} />
          <ul className="mt-1.5 space-y-1.5" role="list">
            {group.skills.map((skill) => (
              <li key={skill.skill_id}>
                <InstalledListItem
                  skill={skill}
                  selected={selectedKey === installedKey(skill)}
                  pending={pendingSkillIds.has(skill.skill_id)}
                  onSelect={() => onSelect(skill)}
                  onToggle={onToggle ? (enabled) => onToggle(skill, enabled) : undefined}
                />
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

function SourceGroupHeader({ source, count }: { source: string; count: number }) {
  const { t } = useTranslation("skills");
  return (
    <div className="flex h-7 items-center justify-between gap-2 px-1">
      <h4
        id={`skills-source-${source}`}
        className="truncate text-[11px] font-semibold tracking-wide text-muted-foreground"
      >
        {sourceLabel(source, t)}
      </h4>
      <span className="rounded-full bg-muted px-2 py-0.5 text-[10px] tabular-nums text-muted-foreground">
        {t("skillsCount", { count })}
      </span>
    </div>
  );
}

function InstalledListItem({
  skill,
  selected,
  pending,
  onSelect,
  onToggle,
}: {
  skill: InstalledSkill;
  selected: boolean;
  pending: boolean;
  onSelect: () => void;
  onToggle?: (enabled: boolean) => void;
}) {
  const { t } = useTranslation("skills");
  const healthy = skill.health === "healthy" && [
    "legacy_allowed",
    "passed",
    "warnings",
    "approved",
  ].includes(skill.security_state);
  return (
    <div
      aria-current={selected ? "true" : undefined}
      className={cn(
        "flex min-h-16 w-full min-w-0 items-start gap-2 overflow-hidden rounded-lg border px-2 py-2 text-left transition-colors duration-150",
        selected
          ? "border-primary/30 bg-primary/8"
          : "border-border/60 bg-background/45 hover:bg-muted/60",
      )}
    >
      <button
        type="button"
        onClick={onSelect}
        className="flex min-h-11 min-w-0 flex-1 cursor-pointer items-start gap-3 rounded-md px-1 py-0.5 text-left focus-visible:ring-[3px] focus-visible:ring-ring/30 focus-visible:outline-none"
      >
        <span className="mt-0.5 shrink-0">
          <StatusIcon healthy={healthy} />
        </span>
        <SkillText name={skill.name} description={skill.description} />
      </button>
      <div className="ml-auto flex min-h-11 shrink-0 flex-col items-end justify-center gap-1 px-1">
        <span className="text-[10px] text-muted-foreground">{skill.version ?? t("unknown")}</span>
        {onToggle ? (
          <Switch
            checked={skill.enabled}
            disabled={pending || !healthy}
            aria-label={t("enableSkill", { name: skill.name })}
            aria-busy={pending}
            onCheckedChange={onToggle}
          />
        ) : (
          <span className={cn("text-[10px]", skill.enabled && healthy ? "text-primary" : "text-muted-foreground")}>
            {skill.enabled ? t("enabled") : t("disabled")}
          </span>
        )}
      </div>
    </div>
  );
}

function RemoteListItem({
  skill,
  selected,
  onSelect,
  onInstall,
}: {
  skill: RemoteSkill;
  selected: boolean;
  onSelect: () => void;
  onInstall: () => void;
}) {
  const { t } = useTranslation("skills");
  return (
    <div
      className={cn(
        "flex min-h-16 min-w-0 items-start gap-3 overflow-hidden rounded-lg border px-3 py-2.5 transition-colors duration-150",
        selected ? "border-primary/30 bg-primary/8" : "border-border/60 bg-background/45 hover:bg-muted/60",
      )}
    >
      <button
        type="button"
        onClick={onSelect}
        aria-current={selected ? "true" : undefined}
        className="flex min-w-0 flex-1 cursor-pointer items-start gap-3 overflow-hidden text-left focus-visible:ring-[3px] focus-visible:ring-ring/30 focus-visible:outline-none"
      >
        <PackageOpen className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
        <SkillText name={skill.display_name} description={skill.summary} />
      </button>
      <Button
        size="xs"
        className="mt-0.5 shrink-0"
        onClick={onInstall}
        aria-label={`${t("install")} ${skill.display_name}`}
      >
        <Download className="size-3" />
        {t("install")}
      </Button>
    </div>
  );
}

function SkillText({ name, description }: { name: string; description: string }) {
  return (
    <span className="min-w-0 flex-1 overflow-hidden">
      <span className="block truncate text-sm font-medium text-foreground" title={name}>{name}</span>
      <span className="mt-0.5 line-clamp-2 break-words text-xs leading-4 text-muted-foreground" title={description}>
        {description}
      </span>
    </span>
  );
}

function StatusIcon({ healthy }: { healthy: boolean }) {
  return healthy ? (
    <CheckCircle2 className="size-4 shrink-0 text-primary" aria-hidden="true" />
  ) : (
    <ShieldAlert className="size-4 shrink-0 text-destructive" aria-hidden="true" />
  );
}

function ListLoading() {
  const { t } = useTranslation("skills");
  return (
    <div className="space-y-1.5" aria-label={t("loadingSkills")}>
      {Array.from({ length: 5 }, (_, index) => (
        <div key={index} className="h-16 animate-pulse rounded-lg bg-muted/60" />
      ))}
    </div>
  );
}

function ListEmpty({ view, query }: { view: "installed" | "discover"; query: string }) {
  const { t } = useTranslation("skills");
  const hasQuery = query.trim().length > 0;
  return (
    <div className="rounded-lg border border-dashed border-border p-8 text-center">
      <PackageOpen className="mx-auto size-5 text-muted-foreground/70" />
      <p className="mt-2 text-sm font-medium">
        {t(hasQuery ? "emptySearchTitle" : view === "installed" ? "emptyInstalledTitle" : "popularUnavailableTitle")}
      </p>
      <p className="mt-1 text-xs text-muted-foreground">
        {t(hasQuery ? "emptySearchDescription" : view === "installed" ? "emptyInstalledDescription" : "popularUnavailableDescription")}
      </p>
    </div>
  );
}

function useIncrementalItems<T>(items: T[], fingerprint: string) {
  const [visibleCount, setVisibleCount] = useState(INITIAL_BATCH_SIZE);
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const sentinelRef = useRef<HTMLDivElement>(null);
  const timerRef = useRef<number | null>(null);

  const clearLoadingTimer = useCallback(() => {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  useEffect(() => {
    clearLoadingTimer();
    setVisibleCount(INITIAL_BATCH_SIZE);
    setIsLoadingMore(false);
  }, [clearLoadingTimer, fingerprint]);

  useEffect(() => clearLoadingTimer, [clearLoadingTimer]);

  const loadMore = useCallback(() => {
    if (isLoadingMore || visibleCount >= items.length) return;
    setIsLoadingMore(true);
    timerRef.current = window.setTimeout(() => {
      setVisibleCount((count) => Math.min(count + INITIAL_BATCH_SIZE, items.length));
      setIsLoadingMore(false);
      timerRef.current = null;
    }, LOAD_MORE_DELAY_MS);
  }, [isLoadingMore, items.length, visibleCount]);

  useEffect(() => {
    const sentinel = sentinelRef.current;
    if (!sentinel || !hasIntersectionObserver() || visibleCount >= items.length) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) loadMore();
      },
      { rootMargin: "120px 0px" },
    );
    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [items.length, loadMore, visibleCount]);

  return {
    visibleItems: items.slice(0, visibleCount),
    hasMore: visibleCount < items.length,
    isLoadingMore,
    sentinelRef,
  };
}

function hasIntersectionObserver() {
  return typeof window !== "undefined" && "IntersectionObserver" in window;
}

function groupInstalledBySource(skills: InstalledSkill[]) {
  const groups = new Map<string, InstalledSkill[]>();
  for (const skill of skills) {
    const source = normalizedSource(skill.source_kind);
    const group = groups.get(source) ?? [];
    group.push(skill);
    groups.set(source, group);
  }
  return Array.from(groups, ([source, groupedSkills]) => ({ source, skills: groupedSkills })).sort(
    (left, right) => sourceRank(left.source) - sourceRank(right.source) || left.source.localeCompare(right.source),
  );
}

function sourceCounts(skills: InstalledSkill[]) {
  const counts = new Map<string, number>();
  for (const skill of skills) {
    const source = normalizedSource(skill.source_kind);
    counts.set(source, (counts.get(source) ?? 0) + 1);
  }
  return counts;
}

function compareInstalledBySource(left: InstalledSkill, right: InstalledSkill) {
  const leftSource = normalizedSource(left.source_kind);
  const rightSource = normalizedSource(right.source_kind);
  return sourceRank(leftSource) - sourceRank(rightSource)
    || leftSource.localeCompare(rightSource)
    || left.name.localeCompare(right.name);
}

function normalizedSource(source: string) {
  return source.trim().toLocaleLowerCase() || "other";
}

function sourceRank(source: string) {
  const rank = SOURCE_ORDER.indexOf(source as (typeof SOURCE_ORDER)[number]);
  return rank === -1 ? SOURCE_ORDER.length : rank;
}

function sourceLabel(source: string, t: (key: string) => string) {
  const keys: Record<string, string> = {
    local: "sourceMisakaX",
    skillhub: "sourceSkillhub",
    clawhub: "sourceClawhub",
    modelscope: "sourceModelscope",
    codex: "sourceCodex",
    claude: "sourceClaude",
    cursor: "sourceCursor",
  };
  return keys[source] ? t(keys[source]) : t("sourceOther");
}

function filterInstalled(skills: InstalledSkill[], query: string) {
  return skills.filter((skill) => matchesQuery(query, skill.name, skill.description, skill.slug));
}

function filterRemote(skills: RemoteSkill[], query: string) {
  return skills.filter((skill) => matchesQuery(query, skill.display_name, skill.summary, skill.slug));
}

function matchesQuery(query: string, ...fields: string[]) {
  const normalized = query.trim().toLocaleLowerCase();
  return !normalized || fields.some((field) => field.toLocaleLowerCase().includes(normalized));
}

function listKey(skill: ListSkill) {
  return "display_name" in skill ? remoteKey(skill) : installedKey(skill);
}

function isInstalledSkill(skill: ListSkill): skill is InstalledSkill {
  return "source_kind" in skill;
}

function isRemoteSkill(skill: ListSkill): skill is RemoteSkill {
  return "provider" in skill;
}

export function installedKey(skill: InstalledSkill) {
  return `installed:${skill.skill_id}`;
}

export function remoteKey(skill: RemoteSkill) {
  return `remote:${skill.provider}:${skill.slug}`;
}
