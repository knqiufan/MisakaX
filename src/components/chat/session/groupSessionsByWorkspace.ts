import type { Session } from "@/lib/ipc";
import type { WorkspacePreference } from "@/lib/ipc/workspace";

export interface WorkspaceSessionGroup {
  key: string;
  path: string | null;
  projectName: string | null;
  workspaceKind: string;
  sessions: Session[];
  manualGroups: Map<string, Session[]>;
  ungrouped: Session[];
  pinned: boolean;
}

export interface GroupedSessionsByWorkspace {
  workspaces: WorkspaceSessionGroup[];
}

/** Normalize working_directory for stable grouping keys. */
export function normalizeWorkingDirectory(dir: string | null | undefined): string {
  if (!dir) return "__none__";
  return dir.replace(/\\/g, "/").replace(/\/+$/, "").toLowerCase();
}

export function collapseKeyForGroup(workspaceKey: string, groupName: string): string {
  return `${workspaceKey}::${groupName}`;
}

function pinnedFirst(a: Session, b: Session): number {
  if (a.pinned === b.pinned) return 0;
  return a.pinned ? -1 : 1;
}

function sortPinnedFirst(sessions: Session[]): Session[] {
  return [...sessions].sort(pinnedFirst);
}

function buildManualSplit(sessions: Session[]): {
  manualGroups: Map<string, Session[]>;
  ungrouped: Session[];
} {
  const manualGroups = new Map<string, Session[]>();
  const ungrouped: Session[] = [];

  for (const session of sessions) {
    const groupName = session.group_name?.trim();
    if (groupName) {
      const list = manualGroups.get(groupName) ?? [];
      list.push(session);
      manualGroups.set(groupName, list);
    } else {
      ungrouped.push(session);
    }
  }

  for (const [name, list] of manualGroups) {
    manualGroups.set(name, sortPinnedFirst(list));
  }

  return {
    manualGroups,
    ungrouped: sortPinnedFirst(ungrouped),
  };
}

function workspaceLabel(session: Session): {
  path: string | null;
  projectName: string | null;
  workspaceKind: string;
} {
  return {
    path: session.working_directory,
    projectName: session.project_name,
    workspaceKind: session.workspace_kind || "custom",
  };
}

/**
 * Group sessions by normalized working_directory.
 * Pinned sessions stay inside their workspace (not a global section).
 * Manual group_name becomes a secondary collapsible subgroup.
 */
export function groupSessionsByWorkspace(
  sessions: Session[],
  preferences: readonly WorkspacePreference[] = []
): GroupedSessionsByWorkspace {
  const preferencesByKey = new Map(
    preferences.map((preference) => [preference.workspace_key, preference])
  );
  const buckets = new Map<string, Session[]>();
  const meta = new Map<string, ReturnType<typeof workspaceLabel>>();

  for (const session of sessions) {
    const key = normalizeWorkingDirectory(session.working_directory);
    if (preferencesByKey.get(key)?.hidden) continue;
    const list = buckets.get(key) ?? [];
    list.push(session);
    buckets.set(key, list);
    if (!meta.has(key)) {
      meta.set(key, workspaceLabel(session));
    }
  }

  const workspaces: WorkspaceSessionGroup[] = [];

  for (const [key, bucket] of buckets) {
    const info = meta.get(key)!;
    const sessionsSorted = sortPinnedFirst(bucket);
    const { manualGroups, ungrouped } = buildManualSplit(sessionsSorted);
    workspaces.push({
      key,
      path: info.path,
      projectName: info.projectName,
      workspaceKind: info.workspaceKind,
      sessions: sessionsSorted,
      manualGroups,
      ungrouped,
      pinned: preferencesByKey.get(key)?.pinned ?? false,
    });
  }

  workspaces.sort((a, b) => {
    if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
    const aLabel = a.projectName || a.path || a.key;
    const bLabel = b.projectName || b.path || b.key;
    return aLabel.localeCompare(bLabel);
  });

  return { workspaces };
}
