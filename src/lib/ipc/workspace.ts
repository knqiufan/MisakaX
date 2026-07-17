import { invoke } from "./invoke";

export interface DirectoryInfo {
  path: string;
  name: string;
  exists: boolean;
  readable: boolean;
  writable: boolean;
  file_count: number | null;
}

export interface RecentDirectory {
  id: string;
  path: string;
  display_name: string | null;
  last_used_at: string;
  use_count: number;
}

export interface WorkspacePreference {
  workspace_key: string;
  pinned: boolean;
  hidden: boolean;
}

export const workspaceIpc = {
  browseDirectory: (startPath?: string) =>
    invoke<string | null>("browse_directory", { startPath }),

  validateDirectory: (path: string) =>
    invoke<DirectoryInfo>("validate_directory", { path }),

  getRecentDirectories: (limit?: number) =>
    invoke<RecentDirectory[]>("get_recent_directories", { limit }),

  recordDirectoryUsage: (path: string, displayName?: string) =>
    invoke<void>("record_directory_usage", { path, displayName }),

  removeRecentDirectory: (path: string) =>
    invoke<boolean>("remove_recent_directory", { path }),

  listPreferences: () =>
    invoke<WorkspacePreference[]>("list_workspace_preferences"),

  updatePreference: (
    workspaceKey: string,
    preference: { pinned?: boolean; hidden?: boolean }
  ) =>
    invoke<void>("update_workspace_preference", {
      workspaceKey,
      pinned: preference.pinned,
      hidden: preference.hidden,
    }),
};
