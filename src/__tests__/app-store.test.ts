import { describe, it, expect, beforeEach } from "vitest";
import {
  useAppStore,
  resolveLeftColumnWidth,
  SESSION_LIST_MIN_WIDTH,
  SESSION_LIST_DEFAULT_WIDTH,
  LG_BREAKPOINT,
} from "@/stores/app-store";
import { SETTINGS_NAV } from "@/components/settings/nav-config";

describe("useAppStore", () => {
  beforeEach(() => {
    useAppStore.setState({
      route: { page: "chat" },
      sessionListWidth: SESSION_LIST_DEFAULT_WIDTH,
      globalLoading: false,
    });
  });

  it("should have correct initial state", () => {
    const state = useAppStore.getState();
    expect(state.route).toEqual({ page: "chat" });
    expect(state.sessionListWidth).toBe(SESSION_LIST_DEFAULT_WIDTH);
    expect(state.globalLoading).toBe(false);
  });

  it("should navigate to a new route", () => {
    useAppStore.getState().navigate({ page: "settings", tab: "models" });
    expect(useAppStore.getState().route).toEqual({
      page: "settings",
      tab: "models",
    });
  });

  it("should navigate to notifications", () => {
    useAppStore.getState().navigate({ page: "notifications" });
    expect(useAppStore.getState().route).toEqual({ page: "notifications" });
  });

  it("exposes Skills only through the Settings deep link", () => {
    useAppStore.getState().navigate({ page: "settings", tab: "skills" });
    expect(useAppStore.getState().route).toEqual({
      page: "settings",
      tab: "skills",
    });
    expect(SETTINGS_NAV.map((item) => item.id)).toEqual([
      "general",
      "models",
      "mcp",
      "skills",
      "appearance",
      "about",
    ]);
    expect(SETTINGS_NAV.find((item) => item.id === "skills")?.contentWidth).toBe(
      "wide"
    );
  });

  it("should clamp and persist session list width", () => {
    useAppStore.getState().setSessionListWidth(500);
    expect(useAppStore.getState().sessionListWidth).toBe(300);
    useAppStore.getState().setSessionListWidth(100);
    expect(useAppStore.getState().sessionListWidth).toBe(SESSION_LIST_MIN_WIDTH);
  });

  it("should set global loading", () => {
    useAppStore.getState().setGlobalLoading(true);
    expect(useAppStore.getState().globalLoading).toBe(true);
    useAppStore.getState().setGlobalLoading(false);
    expect(useAppStore.getState().globalLoading).toBe(false);
  });
});

describe("resolveLeftColumnWidth", () => {
  it("uses preferred width on wide viewports", () => {
    expect(resolveLeftColumnWidth(240, LG_BREAKPOINT)).toBe(240);
    expect(resolveLeftColumnWidth(280, 1400)).toBe(280);
  });

  it("shrinks toward min width on narrow viewports", () => {
    expect(resolveLeftColumnWidth(240, LG_BREAKPOINT - 1)).toBe(
      SESSION_LIST_MIN_WIDTH
    );
    expect(resolveLeftColumnWidth(180, 800)).toBe(SESSION_LIST_MIN_WIDTH);
  });
});
