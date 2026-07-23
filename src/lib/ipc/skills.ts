import { invoke } from "./invoke";
import type {
  InstalledSkill,
  RemoteSearchPage,
  RemoteSkillDetail,
  SkillArchiveInspection,
  SkillDetail,
  SkillInstallResult,
} from "./types";

export const skillsIpc = {
  listInstalled: () => invoke<InstalledSkill[]>("skills_list_installed"),

  getDetail: (slug: string) =>
    invoke<SkillDetail>("skills_get_detail", { slug }),

  inspectArchive: (path: string) =>
    invoke<SkillArchiveInspection>("skills_inspect_archive", { path }),

  installArchive: (path: string) =>
    invoke<SkillInstallResult>("skills_install_archive", { path }),

  searchRemote: (provider: string, query: string, limit?: number) =>
    invoke<RemoteSearchPage>("skills_search_remote", { provider, query, limit }),

  getRemoteDetail: (provider: string, slug: string) =>
    invoke<RemoteSkillDetail>("skills_get_remote_detail", { provider, slug }),

  installRemote: (provider: string, slug: string, version?: string) =>
    invoke<SkillInstallResult>("skills_install_remote", { provider, slug, version }),

  importModelScope: (reference: string) =>
    invoke<SkillInstallResult>("skills_import_modelscope", { reference }),

  exportInstalled: (slug: string, destination: string) =>
    invoke<void>("skills_export_installed", { slug, destination }),

  downloadRemote: (
    provider: string,
    slug: string,
    destination: string,
    version?: string
  ) =>
    invoke<void>("skills_download_remote", {
      provider,
      slug,
      version,
      destination,
    }),

  setEnabled: (slug: string, enabled: boolean) =>
    invoke<void>("skills_set_enabled", { slug, enabled }),

  uninstall: (slug: string) => invoke<void>("skills_uninstall", { slug }),
};
