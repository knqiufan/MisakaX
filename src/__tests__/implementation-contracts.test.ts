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
  it("enables the terminal after the W5 gate while keeping the narrow-capability flag dormant", () => {
    expect(readFeatureFlags({})).toEqual({
      workspaceTerminal: true,
      narrowWebviewCapabilities: false,
    });
  });

  it("keeps an explicit terminal kill switch", () => {
    const flags = readFeatureFlags({
      VITE_MISAKAX_WORKSPACE_TERMINAL: "false",
    });
    expect(flags.workspaceTerminal).toBe(false);
    expect(flags.narrowWebviewCapabilities).toBe(false);
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
