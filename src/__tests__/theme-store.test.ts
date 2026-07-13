import { describe, it, expect, beforeEach, vi } from "vitest";
import { useThemeStore } from "@/stores/theme-store";

vi.mock("@/lib/theme", () => ({
  resolveTheme: vi.fn((mode: string) => {
    if (mode === "dark") return "dark";
    return "light";
  }),
  applyThemeToDOM: vi.fn(),
  applyUIFontSize: vi.fn(),
  startSystemThemeListener: vi.fn(),
  stopSystemThemeListener: vi.fn(),
  parseThemeMode: vi.fn((value: string) => {
    if (value === "light" || value === "dark" || value === "system") return value;
    if (value === "dim") return "dark";
    return "system";
  }),
}));

vi.mock("@/lib/ipc", () => ({
  settingsIpc: {
    getAppConfig: vi.fn().mockResolvedValue({
      language: "en",
      theme: "dark",
      accent_color: "#6366F1",
      reduced_transparency: false,
      ui_font_size: 14,
      default_model: "claude-sonnet-4-20250514",
      log_level: "info",
      sidecar_port: 9527,
      auto_start_sidecar: false,
      use_sidecar: true,
      mcp_bridge_port: 9528,
    }),
    updateAppConfig: vi.fn().mockResolvedValue(undefined),
  },
}));

import {
  resolveTheme,
  applyThemeToDOM,
  applyUIFontSize,
  startSystemThemeListener,
  stopSystemThemeListener,
  parseThemeMode,
} from "@/lib/theme";

describe("useThemeStore", () => {
  beforeEach(() => {
    useThemeStore.setState({
      mode: "system",
      resolvedTheme: "light",
      reducedTransparency: false,
      uiFontSize: 14,
      initialized: false,
    });
    vi.clearAllMocks();
  });

  it("should have correct initial state", () => {
    const state = useThemeStore.getState();
    expect(state.mode).toBe("system");
    expect(state.resolvedTheme).toBe("light");
    expect(state.reducedTransparency).toBe(false);
    expect(state.uiFontSize).toBe(14);
    expect(state.initialized).toBe(false);
  });

  it("should set mode and resolve theme", () => {
    useThemeStore.getState().setMode("dark");

    const state = useThemeStore.getState();
    expect(state.mode).toBe("dark");
    expect(state.resolvedTheme).toBe("dark");
    expect(resolveTheme).toHaveBeenCalledWith("dark");
    expect(applyThemeToDOM).toHaveBeenCalledWith("dark", "dark");
    expect(stopSystemThemeListener).toHaveBeenCalled();
  });

  it("should start system listener when mode is system", () => {
    useThemeStore.getState().setMode("system");

    expect(startSystemThemeListener).toHaveBeenCalled();
  });

  it("should stop system listener when mode is not system", () => {
    useThemeStore.getState().setMode("light");

    expect(stopSystemThemeListener).toHaveBeenCalled();
    expect(startSystemThemeListener).not.toHaveBeenCalled();
  });

  it("should set reduced transparency", () => {
    useThemeStore.getState().setReducedTransparency(true);
    expect(useThemeStore.getState().reducedTransparency).toBe(true);
  });

  it("should set UI font size", () => {
    useThemeStore.getState().setUiFontSize(18);

    expect(useThemeStore.getState().uiFontSize).toBe(18);
    expect(applyUIFontSize).toHaveBeenCalledWith(18);
  });

  it("should initialize from config", async () => {
    await useThemeStore.getState().initialize();

    const state = useThemeStore.getState();
    expect(state.initialized).toBe(true);
    expect(state.mode).toBe("dark");
    expect(state.resolvedTheme).toBe("dark");
    expect(parseThemeMode).toHaveBeenCalledWith("dark");
  });

  it("should not reinitialize if already initialized", async () => {
    await useThemeStore.getState().initialize();
    vi.clearAllMocks();

    await useThemeStore.getState().initialize();

    expect(resolveTheme).not.toHaveBeenCalled();
  });

  it("should cleanup system theme listener", () => {
    useThemeStore.getState().cleanup();
    expect(stopSystemThemeListener).toHaveBeenCalled();
  });
});
