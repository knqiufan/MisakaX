import { create } from "zustand";
import type { ThemeMode } from "@/lib/theme";
import {
  resolveTheme,
  applyThemeToDOM,
  applyAccentColor,
  startSystemThemeListener,
  stopSystemThemeListener,
  ACCENT_COLORS,
} from "@/lib/theme";
import { settingsIpc } from "@/lib/ipc";

interface ThemeState {
  mode: ThemeMode;
  accentColor: string;
  resolvedTheme: "light" | "dark";
  initialized: boolean;

  setMode: (mode: ThemeMode) => void;
  setAccentColor: (color: string) => void;
  initialize: () => Promise<void>;
  cleanup: () => void;
}

export const useThemeStore = create<ThemeState>((set, get) => ({
  mode: "system",
  accentColor: "#6366F1",
  resolvedTheme: "light",
  initialized: false,

  setMode: (mode) => {
    const resolved = resolveTheme(mode);
    applyThemeToDOM(resolved);

    if (mode === "system") {
      startSystemThemeListener((newResolved) => {
        applyThemeToDOM(newResolved);
        set({ resolvedTheme: newResolved });
      });
    } else {
      stopSystemThemeListener();
    }

    set({ mode, resolvedTheme: resolved });
    syncThemeToConfig(mode, get().accentColor);
  },

  setAccentColor: (color) => {
    applyAccentColor(color);
    set({ accentColor: color });
    syncThemeToConfig(get().mode, color);
  },

  initialize: async () => {
    if (get().initialized) return;

    let mode: ThemeMode = "system";
    let accentColor = "#6366F1";

    try {
      const config = await settingsIpc.getAppConfig();
      mode = parseThemeMode(config.theme);
      accentColor = config.accent_color || "#6366F1";
    } catch (err) {
      if (import.meta.env.DEV) {
        console.warn("[Theme] Failed to load config, using defaults:", err);
      }
    }

    const resolved = resolveTheme(mode);
    applyThemeToDOM(resolved);
    applyAccentColor(accentColor);

    if (mode === "system") {
      startSystemThemeListener((newResolved) => {
        applyThemeToDOM(newResolved);
        set({ resolvedTheme: newResolved });
      });
    }

    set({ mode, accentColor, resolvedTheme: resolved, initialized: true });
  },

  cleanup: () => {
    stopSystemThemeListener();
  },
}));

function parseThemeMode(value: string): ThemeMode {
  if (value === "light" || value === "dark" || value === "system") return value;
  return "system";
}

async function syncThemeToConfig(mode: ThemeMode, accentColor: string): Promise<void> {
  try {
    const config = await settingsIpc.getAppConfig();
    await settingsIpc.updateAppConfig({
      ...config,
      theme: mode,
      accent_color: accentColor,
    });
  } catch (err) {
    if (import.meta.env.DEV) {
      console.warn("[Theme] Failed to sync config:", err);
    }
  }
}

export { ACCENT_COLORS };
