import type { ReactNode } from "react";
import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SkillDetailPanel } from "@/components/skills/SkillDetailPanel";
import { SkillsList } from "@/components/skills/SkillsList";
import type { InstalledSkill, RemoteSkillDetail, SkillFilePage, SkillRiskReport, SkillSummary } from "@/lib/ipc";

vi.mock("@/components/ui/scroll-area", () => ({
  ScrollArea: ({ children, className }: { children: ReactNode; className?: string }) => (
    <div className={className}>{children}</div>
  ),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: Record<string, unknown>) => {
      if (key === "skillsVisible") return `${values?.visible} of ${values?.total}`;
      if (key === "skillsCount") return `${values?.count} Skills`;
      if (key === "files") return `${values?.count} Files`;
      return {
        installedSkills: "Installed Skills",
        popularSkills: "Popular Skills",
        searchResults: "Search results",
        loadingSkills: "Loading Skills",
        loadingMore: "Loading more Skills",
        scrollToLoadMore: "Scroll to load more",
        sourceCodex: "Codex",
        sourceClaude: "Claude",
        sourceOther: "Other sources",
        enabled: "Enabled",
        disabled: "Disabled",
        unknown: "Unknown",
        detailEmptyTitle: "Select a Skill",
        detailEmptyDescription: "Choose a Skill from the list.",
        details: "Details",
        backToList: "Back to list",
        version: "Version",
        author: "Author",
        source: "Source",
        license: "License",
        installedAt: "Installed",
        compatibility: "Compatibility",
        risk: "Safety notes",
        noRisk: "No additional notices.",
        readme: "SKILL.md",
        fileTree: "File structure",
        fileTreeDescription: "Relative package paths, file types, and sizes.",
        fileTreeUnavailable: "This installed Skill has no files available to inspect.",
        fileTreeUnavailableRemote: "Registry metadata is available, but package files could not be previewed.",
        previewUnavailable: "SKILL.md is not available to preview.",
        previewUnavailableRemote: "Registry metadata is available, but SKILL.md could not be previewed.",
        allowedTools: "Declared tools",
      }[key] ?? key;
    },
  }),
}));

let intersectionCallback: IntersectionObserverCallback | undefined;

class IntersectionObserverMock {
  constructor(callback: IntersectionObserverCallback) {
    intersectionCallback = callback;
  }

  observe() {}
  unobserve() {}
  disconnect() {}
  takeRecords() { return []; }
}

describe("Skills repository panels", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    intersectionCallback = undefined;
    vi.stubGlobal("IntersectionObserver", IntersectionObserverMock);
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("groups installed Skills by source and appends the next ten items after reaching the list end", () => {
    render(
      <SkillsList
        installed={Array.from({ length: 12 }, (_, index) => installedSkill({
          slug: `skill-${index + 1}`,
          name: `Skill ${String(index + 1).padStart(2, "0")}`,
          source_kind: index < 6 ? "codex" : "claude",
        }))}
        remote={[]}
        view="installed"
        loading={false}
        query=""
        selectedKey={null}
        onSelectInstalled={vi.fn()}
        onSelectRemote={vi.fn()}
        onInstall={vi.fn()}
      />,
    );

    expect(screen.getByText("Codex")).toBeTruthy();
    expect(screen.getByText("Claude")).toBeTruthy();
    expect(screen.getByText("10 of 12")).toBeTruthy();
    expect(screen.queryByText("Skill 11")).toBeNull();

    act(() => {
      intersectionCallback?.([{ isIntersecting: true } as IntersectionObserverEntry], {} as IntersectionObserver);
    });
    expect(screen.getByText("Loading more Skills")).toBeTruthy();

    act(() => {
      vi.advanceTimersByTime(160);
    });
    expect(screen.getByText("Skill 11")).toBeTruthy();
    expect(screen.getByText("Skill 12")).toBeTruthy();
  });

  it("exposes an always-visible accessible enable switch", () => {
    const onToggleInstalled = vi.fn();
    render(
      <SkillsList
        installed={[installedSkill()]}
        remote={[]}
        view="installed"
        loading={false}
        query=""
        selectedKey={null}
        pendingSkillIds={new Set()}
        onSelectInstalled={vi.fn()}
        onToggleInstalled={onToggleInstalled}
        onSelectRemote={vi.fn()}
        onInstall={vi.fn()}
      />,
    );

    const toggle = screen.getByRole("switch", { name: "enableSkill" });
    expect(toggle.getAttribute("aria-checked")).toBe("true");
    fireEvent.click(toggle);
    expect(onToggleInstalled).toHaveBeenCalledWith(expect.objectContaining({ skill_id: "example-skill-id" }), false);
  });

  it("renders a hierarchical package file tree with file sizes", () => {
    render(<SkillDetailPanel detail={installedDetail()} initialFiles={rootFiles()} onBack={vi.fn()} />);

    expect(screen.getByText("scripts")).toBeTruthy();
    expect(screen.getByText("SKILL.md")).toBeTruthy();
  });

  it("does not render or request a SKILL.md body before a file is selected", () => {
    render(
      <SkillDetailPanel
        detail={installedDetail()}
        initialFiles={rootFiles()}
        onBack={vi.fn()}
      />,
    );

    expect(screen.getByText("selectFileToPreview")).toBeTruthy();
    expect(screen.queryByText(/name: example-skill/)).toBeNull();
  });

  it("explains why a remote package has no previewable files", () => {
    render(<SkillDetailPanel detail={remoteDetailWithoutPreview()} onBack={vi.fn()} />);

    expect(screen.getByText("remoteFilesRequireInstall")).toBeTruthy();
  });
});

function installedSkill(overrides: Partial<InstalledSkill> = {}): InstalledSkill {
  return {
    skill_id: "example-skill-id",
    slug: "example-skill",
    name: "Example Skill",
    description: "Example description",
    version: "1.0.0",
    source_kind: "local",
    source_ref: null,
    source_url: null,
    checksum: "checksum",
    installed_path: "C:/skills/example-skill",
    enabled: true,
    health: "healthy",
    is_external: false,
    effective_active: true,
    effective_rank: 400,
    conflict: false,
    disabled_reason: null,
    security_state: "passed",
    installed_at: "2026-07-23T00:00:00Z",
    updated_at: "2026-07-23T00:00:00Z",
    ...overrides,
  };
}

function installedDetail(): SkillSummary {
  return {
    generation: 7,
    skill: installedSkill(),
    manifest: {
      name: "example-skill",
      description: "Example description",
      license: null,
      compatibility: null,
      allowed_tools: null,
      metadata: {},
    },
    body_bytes_transferred: 0,
  };
}

function rootFiles(): SkillFilePage {
  return {
    skill_id: "example-skill-id",
    generation: 7,
    parent: null,
    next_cursor: null,
    items: [
      { path: "scripts", name: "scripts", kind: "directory", size_bytes: 0, is_directory: true, is_text_candidate: false, is_link: false },
      { path: "SKILL.md", name: "SKILL.md", kind: "skill-manifest", size_bytes: 240, is_directory: false, is_text_candidate: true, is_link: false },
    ],
  };
}

function remoteDetailWithoutPreview(): RemoteSkillDetail {
  return {
    skill: {
      provider: "skillhub",
      slug: "remote-skill",
      display_name: "Remote Skill",
      summary: "Remote summary",
      version: "1.0.0",
      owner: "owner",
      source_url: "https://example.com/remote-skill",
      topics: [],
      suspicious: false,
    },
    changelog: null,
    license: null,
    compatibility: null,
    risk: noRisk(),
    manifest: null,
    files: [],
  };
}

function noRisk(): SkillRiskReport {
  return {
    has_scripts: false,
    has_binary_files: false,
    has_allowed_tools: false,
    remote_scan_status: null,
    notes: [],
  };
}
