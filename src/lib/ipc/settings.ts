import { invoke } from "./invoke";
import type { AppConfig, SystemInfo } from "./types";

export const settingsIpc = {
  getAppConfig: () => invoke<AppConfig>("get_app_config"),

  updateAppConfig: (config: AppConfig) =>
    invoke<void>("update_app_config", { config }),

  getSetting: (key: string) =>
    invoke<string | null>("get_setting", { key }),

  setSetting: (key: string, value: string) =>
    invoke<void>("set_setting", { key, value }),

  getAllSettings: () =>
    invoke<Record<string, string>>("get_all_settings"),

  getSystemInfo: () => invoke<SystemInfo>("get_system_info"),
};
