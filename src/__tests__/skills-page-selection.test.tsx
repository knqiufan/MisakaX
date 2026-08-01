import type { ButtonHTMLAttributes, ReactNode } from "react";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { RemoteSkillDetail, SkillFilePage, SkillSummary } from "@/lib/ipc";

const mocks = vi.hoisted(() => ({
  getSummary: vi.fn(),
  listFiles: vi.fn(),
  readFile: vi.fn(),
  refresh: vi.fn(),
  searchRemote: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ save: vi.fn() }));

vi.mock("sonner", () => ({
  toast: { error: vi.fn(), success: vi.fn() },
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({ children, ...props }: ButtonHTMLAttributes<HTMLButtonElement> & { children?: ReactNode }) => (
    <button type="button" {...props}>{children}</button>
  ),
}));

vi.mock("@/components/skills/SkillsToolbar", () => ({
  SkillsToolbar: ({ onViewChange }: { onViewChange: (view: "installed" | "discover") => void }) => (
    <button type="button" onClick={() => onViewChange("discover")}>Discover</button>
  ),
}));

vi.mock("@/components/skills/SkillsList", () => ({
  installedKey: (skill: { slug: string }) => `installed:${skill.slug}`,
  remoteKey: (skill: { provider: string; slug: string }) => `remote:${skill.provider}:${skill.slug}`,
  SkillsList: ({ onSelectInstalled }: { onSelectInstalled: (skill: Record<string, unknown>) => void }) => (
    <button
      type="button"
      onClick={() => onSelectInstalled({
        slug: "demo-skill",
        skill_id: "demo-skill-id",
        name: "Demo Skill",
        description: "Demo",
        version: "1.0.0",
        source_kind: "local",
        enabled: true,
        is_external: false,
      })}
    >
      Open Skill
    </button>
  ),
}));

vi.mock("@/components/skills/SkillDetailPanel", () => ({
  SkillDetailPanel: ({ detail }: { detail: SkillSummary | RemoteSkillDetail | null }) => (
    <output data-testid="detail-state">{detail ? "loaded" : "empty"}</output>
  ),
}));

vi.mock("@/components/skills/SkillInstallDialogs", () => ({
  UploadSkillDialog: () => null,
  ModelScopeImportDialog: () => null,
  RemoteInstallDialog: () => null,
  UninstallSkillDialog: () => null,
}));

vi.mock("@/components/skills/useSkillsInventory", () => ({
  useSkillsInventory: () => ({
    skills: [],
    loading: false,
    error: null,
    refresh: mocks.refresh,
  }),
}));

vi.mock("@/lib/ipc", () => ({
  skillsIpc: {
    getSummary: mocks.getSummary,
    listFiles: mocks.listFiles,
    readFile: mocks.readFile,
    getRemoteDetail: vi.fn(),
    searchRemote: mocks.searchRemote,
    setEnabled: vi.fn(),
    exportInstalled: vi.fn(),
    downloadRemote: vi.fn(),
  },
}));

import { SkillsSettingsFeature } from "@/components/skills/SkillsSettingsFeature";

describe("Skills Settings detail selection", () => {
  beforeEach(() => {
    mocks.getSummary.mockReset();
    mocks.listFiles.mockReset();
    mocks.readFile.mockReset();
    mocks.refresh.mockReset();
    mocks.searchRemote.mockReset();
    mocks.refresh.mockResolvedValue(undefined);
    mocks.searchRemote.mockResolvedValue({ items: [] });
    mocks.listFiles.mockResolvedValue(filePageFixture());
  });

  it("clears a loaded installed detail when switching to Discover", async () => {
    mocks.getSummary.mockResolvedValue(detailFixture());
    render(<SkillsSettingsFeature />);

    fireEvent.click(screen.getByText("Open Skill"));
    await waitFor(() => expect(screen.getByTestId("detail-state").textContent).toBe("loaded"));
    expect(mocks.getSummary).toHaveBeenCalledWith("demo-skill-id");
    expect(mocks.listFiles).toHaveBeenCalledTimes(1);
    expect(mocks.readFile).not.toHaveBeenCalled();

    fireEvent.click(screen.getByText("Discover"));
    expect(screen.getByTestId("detail-state").textContent).toBe("empty");
  });

  it("does not restore a late detail response after switching context", async () => {
    let resolveDetail: (value: SkillSummary) => void = () => undefined;
    mocks.getSummary.mockReturnValue(new Promise<SkillSummary>((resolve) => { resolveDetail = resolve; }));
    render(<SkillsSettingsFeature />);

    fireEvent.click(screen.getByText("Open Skill"));
    fireEvent.click(screen.getByText("Discover"));
    await act(async () => {
      resolveDetail(detailFixture());
      await Promise.resolve();
    });

    expect(screen.getByTestId("detail-state").textContent).toBe("empty");
  });
});

function detailFixture(): SkillSummary {
  return {
    generation: 11,
    skill: {
      skill_id: "demo-skill-id",
      slug: "demo-skill",
      name: "Demo Skill",
      description: "Demo",
      version: "1.0.0",
      source_kind: "local",
      source_ref: null,
      source_url: null,
      checksum: "checksum",
      installed_path: "C:/skills/demo-skill",
      enabled: true,
      health: "healthy",
      is_external: false,
      effective_active: true,
      effective_rank: 400,
      conflict: false,
      disabled_reason: null,
      security_state: "legacy_allowed",
      risk: {
        has_scripts: false,
        has_binary_files: false,
        has_allowed_tools: false,
        remote_scan_status: null,
        notes: [],
      },
      installed_at: "2026-07-23T00:00:00Z",
      updated_at: "2026-07-23T00:00:00Z",
    },
    manifest: {
      name: "demo-skill",
      description: "Demo",
      license: null,
      compatibility: null,
      allowed_tools: null,
      metadata: {},
    },
    body_bytes_transferred: 0,
  };
}

function filePageFixture(): SkillFilePage {
  return {
    skill_id: "demo-skill-id",
    generation: 11,
    parent: null,
    items: [],
    next_cursor: null,
  };
}
