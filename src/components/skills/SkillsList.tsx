import { CheckCircle2, Download, PackageOpen, ShieldAlert } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { InstalledSkill, RemoteSkill } from "@/lib/ipc";

interface SkillsListProps {
  installed: InstalledSkill[];
  remote: RemoteSkill[];
  view: "installed" | "discover";
  loading: boolean;
  query: string;
  selectedKey: string | null;
  onSelectInstalled: (skill: InstalledSkill) => void;
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
  onSelectInstalled,
  onSelectRemote,
  onInstall,
}: SkillsListProps) {
  const items = view === "installed" ? filterInstalled(installed, query) : filterRemote(remote, query);
  if (loading) return <ListLoading />;
  if (items.length === 0) return <ListEmpty view={view} query={query} />;
  return (
    <div className="space-y-1.5">
      {items.map((skill) =>
        "display_name" in skill ? (
          <RemoteListItem
            key={`${skill.provider}:${skill.slug}`}
            skill={skill}
            selected={selectedKey === remoteKey(skill)}
            onSelect={() => onSelectRemote(skill)}
            onInstall={() => onInstall(skill)}
          />
        ) : (
          <InstalledListItem
            key={skill.slug}
            skill={skill}
            selected={selectedKey === installedKey(skill)}
            onSelect={() => onSelectInstalled(skill)}
          />
        )
      )}
    </div>
  );
}

function InstalledListItem({
  skill,
  selected,
  onSelect,
}: {
  skill: InstalledSkill;
  selected: boolean;
  onSelect: () => void;
}) {
  const { t } = useTranslation("skills");
  const healthy = skill.health === "healthy";
  return (
    <button
      type="button"
      onClick={onSelect}
      className={cn(
        "flex w-full items-center gap-3 rounded-xl border px-3 py-2.5 text-left transition-colors duration-150",
        selected ? "border-primary/30 bg-primary/8" : "border-border/60 bg-[color:var(--surface-card)] hover:bg-muted/60"
      )}
    >
      <StatusIcon healthy={healthy} />
      <SkillText name={skill.name} description={skill.description} />
      <div className="ml-auto flex shrink-0 flex-col items-end gap-1">
        <span className="text-[10px] text-muted-foreground">{skill.version ?? t("unknown")}</span>
        {skill.is_external ? <span className="text-[10px] text-muted-foreground">{t("externalSource", { source: skill.source_kind })}</span> : null}
        <span className={cn("text-[10px]", skill.enabled && healthy ? "text-primary" : "text-muted-foreground")}>
          {skill.enabled ? t("enabled") : t("disabled")}
        </span>
      </div>
    </button>
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
        "flex items-center gap-3 rounded-xl border px-3 py-2.5 transition-colors duration-150",
        selected ? "border-primary/30 bg-primary/8" : "border-border/60 bg-[color:var(--surface-card)] hover:bg-muted/60"
      )}
    >
      <button type="button" onClick={onSelect} className="flex min-w-0 flex-1 items-center gap-3 text-left">
        <PackageOpen className="size-4 shrink-0 text-muted-foreground" />
        <SkillText name={skill.display_name} description={skill.summary} />
      </button>
      <Button size="xs" onClick={onInstall} aria-label={`${t("install")} ${skill.display_name}`}>
        <Download className="size-3" />
        {t("install")}
      </Button>
    </div>
  );
}

function SkillText({ name, description }: { name: string; description: string }) {
  return (
    <span className="min-w-0">
      <span className="block truncate text-sm font-medium text-foreground">{name}</span>
      <span className="mt-0.5 block truncate text-xs text-muted-foreground">{description}</span>
    </span>
  );
}

function StatusIcon({ healthy }: { healthy: boolean }) {
  return healthy ? (
    <CheckCircle2 className="size-4 shrink-0 text-primary" />
  ) : (
    <ShieldAlert className="size-4 shrink-0 text-destructive" />
  );
}

function ListLoading() {
  return <div className="space-y-1.5">{Array.from({ length: 5 }, (_, index) => <div key={index} className="h-16 animate-pulse rounded-xl bg-muted/60" />)}</div>;
}

function ListEmpty({ view, query }: { view: "installed" | "discover"; query: string }) {
  const { t } = useTranslation("skills");
  const hasQuery = query.trim().length > 0;
  return (
    <div className="rounded-xl border border-dashed border-border p-8 text-center">
      <PackageOpen className="mx-auto size-5 text-muted-foreground/70" />
      <p className="mt-2 text-sm font-medium">{t(hasQuery ? "emptySearchTitle" : view === "installed" ? "emptyInstalledTitle" : "popularUnavailableTitle")}</p>
      <p className="mt-1 text-xs text-muted-foreground">{t(hasQuery ? "emptySearchDescription" : view === "installed" ? "emptyInstalledDescription" : "popularUnavailableDescription")}</p>
    </div>
  );
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

export function installedKey(skill: InstalledSkill) {
  return `installed:${skill.slug}`;
}

export function remoteKey(skill: RemoteSkill) {
  return `remote:${skill.provider}:${skill.slug}`;
}
