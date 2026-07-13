import { create } from "zustand";
import type { ThemeMode } from "@/lib/theme";
import {
  resolveTheme,
  applyThemeToDOM,
  applyUIFontSize,
  startSystemThemeListener,
  stopSystemThemeListener,
  parseThemeMode,
} from "@/lib/theme";
import { settingsIpc } from "@/lib/ipc";

interface ThemeState {
  mode: ThemeMode;
  resolvedTheme: "light" | "dark";
  reducedTransparency: boolean;
  uiFontSize: number;
  initialized: boolean;

  setMode: (mode: ThemeMode) => void;
  setReducedTransparency: (value: boolean) => void;
  setUiFontSize: (px: number) => void;
  initialize: () => Promise<void>;
  cleanup: () => void;
}

export const useThemeStore = create<ThemeState>((set, get) => ({
  mode: "system",
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
    let reducedTransparency = false;
    let uiFontSize = 14;

    try {
      const config = await settingsIpc.getAppConfig();
      mode = parseThemeMode(config.theme);
      reducedTransparency = config.reduced_transparency ?? false;
      uiFontSize = config.ui_font_size ?? 14;
    } catch (err) {
      if (import.meta.env.DEV) {
        console.warn("[Theme] Failed to load config, using defaults:", err);
      }
    }

    const resolved = resolveTheme(mode);
    applyThemeToDOM(mode, resolved);
    applyUIFontSize(uiFontSize);

    if (mode === "system") {
      startSystemThemeListener((newResolved) => {
        applyThemeToDOM("system", newResolved);
        set({ resolvedTheme: newResolved });
      });
    }

    set({
      mode,
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
  const { mode, reducedTransparency, uiFontSize } = useThemeStore.getState();
  try {
    const config = await settingsIpc.getAppConfig();
    await settingsIpc.updateAppConfig({
      ...config,
      theme: mode,
      reduced_transparency: reducedTransparency,
      ui_font_size: uiFontSize,
    });
  } catch (err) {
    if (import.meta.env.DEV) {
      console.warn("[Theme] Failed to sync config:", err);
    }
  }
}
