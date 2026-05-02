import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "@/stores/app-store";

describe("useAppStore", () => {
  beforeEach(() => {
    useAppStore.setState({
      route: { page: "chat" },
      sidebarCollapsed: false,
      globalLoading: false,
    });
  });

  it("should have correct initial state", () => {
    const state = useAppStore.getState();
    expect(state.route).toEqual({ page: "chat" });
    expect(state.sidebarCollapsed).toBe(false);
    expect(state.globalLoading).toBe(false);
  });

  it("should navigate to a new route", () => {
    useAppStore.getState().navigate({ page: "settings", tab: "models" });
    expect(useAppStore.getState().route).toEqual({ page: "settings", tab: "models" });
  });

  it("should navigate to notifications", () => {
    useAppStore.getState().navigate({ page: "notifications" });
    expect(useAppStore.getState().route).toEqual({ page: "notifications" });
  });

  it("should toggle sidebar", () => {
    expect(useAppStore.getState().sidebarCollapsed).toBe(false);
    useAppStore.getState().toggleSidebar();
    expect(useAppStore.getState().sidebarCollapsed).toBe(true);
    useAppStore.getState().toggleSidebar();
    expect(useAppStore.getState().sidebarCollapsed).toBe(false);
  });

  it("should set sidebar collapsed directly", () => {
    useAppStore.getState().setSidebarCollapsed(true);
    expect(useAppStore.getState().sidebarCollapsed).toBe(true);
  });

  it("should set global loading", () => {
    useAppStore.getState().setGlobalLoading(true);
    expect(useAppStore.getState().globalLoading).toBe(true);
    useAppStore.getState().setGlobalLoading(false);
    expect(useAppStore.getState().globalLoading).toBe(false);
  });
});
