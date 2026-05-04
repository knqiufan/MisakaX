import { create } from "zustand";
import type { ThemeMode } from "@/lib/theme";
import {
  resolveTheme,
  applyThemeToDOM,
  applyAccentColor,
  applyUIFontSize,
  startSystemThemeListener,
  stopSystemThemeListener,
} from "@/lib/theme";
import { settingsIpc } from "@/lib/ipc";

interface ThemeState {
  mode: ThemeMode;
  accentColor: string;
  resolvedTheme: "light" | "dark";
  reducedTransparency: boolean;
  uiFontSize: number;
  initialized: boolean;

  setMode: (mode: ThemeMode) => void;
  setAccentColor: (color: string) => void;
  setReducedTransparency: (value: boolean) => void;
  setUiFontSize: (px: number) => void;
  initialize: () => Promise<void>;
  cleanup: () => void;
}

export const useThemeStore = create<ThemeState>((set, get) => ({
  mode: "system",
  accentColor: "#6366F1",
  resolvedTheme: "light",
  reducedTransparency: false,
  uiFontSize: 14,
  initialized: false,

  setMode: (mode) => {
    const resolved = resolveTheme(mode);
    applyThemeToDOM(mode, resolved);

    if (mode === "system") {
      startSystemThemeListener((newResolved) => {
        applyThemeToDOM("system", newResolved);
        set({ resolvedTheme: newResolved });
      });
    } else {
      stopSystemThemeListener();
    }

    set({ mode, resolvedTheme: resolved });
    void syncUiConfigFromThemeState();
  },

  setAccentColor: (color) => {
    applyAccentColor(color);
    set({ accentColor: color });
    void syncUiConfigFromThemeState();
  },

  setReducedTransparency: (value) => {
    set({ reducedTransparency: value });
    void syncUiConfigFromThemeState();
  },

  setUiFontSize: (px) => {
    applyUIFontSize(px);
    set({ uiFontSize: px });
    void syncUiConfigFromThemeState();
  },

  initialize: async () => {
    if (get().initialized) return;

    let mode: ThemeMode = "system";
    let accentColor = "#6366F1";
    let reducedTransparency = false;
    let uiFontSize = 14;

    try {
      const config = await settingsIpc.getAppConfig();
      mode = parseThemeMode(config.theme);
      accentColor = config.accent_color ?? "#6366F1";
      reducedTransparency = config.reduced_transparency ?? false;
      uiFontSize = config.ui_font_size ?? 14;
    } catch (err) {
      if (import.meta.env.DEV) {
        console.warn("[Theme] Failed to load config, using defaults:", err);
      }
    }

    const resolved = resolveTheme(mode);
    applyThemeToDOM(mode, resolved);
    applyAccentColor(accentColor);
    applyUIFontSize(uiFontSize);

    if (mode === "system") {
      startSystemThemeListener((newResolved) => {
        applyThemeToDOM("system", newResolved);
        set({ resolvedTheme: newResolved });
      });
    }

    set({
      mode,
      accentColor,
      reducedTransparency,
      uiFontSize,
      resolvedTheme: resolved,
      initialized: true,
    });
  },

  cleanup: () => {
    stopSystemThemeListener();
  },
}));

async function syncUiConfigFromThemeState(): Promise<void> {
  const { mode, accentColor, reducedTransparency, uiFontSize } =
    useThemeStore.getState();
  try {
    const config = await settingsIpc.getAppConfig();
    await settingsIpc.updateAppConfig({
      ...config,
      theme: mode,
      accent_color: accentColor,
      reduced_transparency: reducedTransparency,
      ui_font_size: uiFontSize,
    });
  } catch (err) {
    if (import.meta.env.DEV) {
      console.warn("[Theme] Failed to sync config:", err);
    }
  }
}

function parseThemeMode(value: string): ThemeMode {
  if (value === "light" || value === "dark" || value === "dim" || value === "system") {
    return value;
  }
  return "system";
}

export { ACCENT_COLORS } from "@/lib/theme";
