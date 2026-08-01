import { describe, expect, it } from "vitest";
import { readFeatureFlags } from "@/lib/feature-flags";
import type {
  AppErrorCode,
  DomainEvent,
  ScanDecision,
  ScanState,
  WorkspacePanelMode,
} from "@/lib/ipc/contracts";

describe("implementation contracts", () => {
  it("keeps S0/W0 feature switches disabled by default", () => {
    expect(readFeatureFlags({})).toEqual({
      skillsSettingsTabV2: false,
      skillsLazyFilePreview: false,
      skillsSecurityGate: false,
      skillsActivationView: false,
      skillsDeepScanner: false,
      workspaceContextBadge: false,
      workspaceTerminal: false,
      narrowWebviewCapabilities: false,
    });
  });

  it("only enables explicit true values", () => {
    const flags = readFeatureFlags({
      VITE_MISAKAX_WORKSPACE_TERMINAL: "TRUE",
      VITE_MISAKAX_SKILLS_SECURITY_GATE: "1",
      VITE_MISAKAX_SKILLS_SETTINGS_TAB_V2: "yes",
    });
    expect(flags.workspaceTerminal).toBe(true);
    expect(flags.skillsSecurityGate).toBe(true);
    expect(flags.skillsSettingsTabV2).toBe(false);
  });

  it("freezes the cross-layer wire values", () => {
    const error: AppErrorCode = "SKILL_SCAN_REQUIRED";
    const state: ScanState = "review_required";
    const decision: ScanDecision = "allow_with_ack";
    const mode: WorkspacePanelMode = "terminal";
    const event: DomainEvent<WorkspacePanelMode> = {
      eventId: "event-1",
      aggregateId: "terminal-1",
      generation: 4,
      occurredAt: "2026-08-01T00:00:00Z",
      payload: mode,
    };

    expect({ error, state, decision, event }).toMatchObject({
      error: "SKILL_SCAN_REQUIRED",
      state: "review_required",
      decision: "allow_with_ack",
      event: { generation: 4, payload: "terminal" },
    });
  });
});
