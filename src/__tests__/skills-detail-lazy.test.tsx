import type { ReactNode } from "react";
import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type {
  SkillFileEntry,
  SkillFilePage,
  SkillFilePreview,
  SkillScanSummary,
  SkillSummary,
} from "@/lib/ipc";

const mocks = vi.hoisted(() => ({
  listFiles: vi.fn(),
  readFile: vi.fn(),
  getScanSummary: vi.fn(),
}));

vi.mock("@/lib/ipc", async (importOriginal) => {
  const original = await importOriginal<typeof import("@/lib/ipc")>();
  return {
    ...original,
    skillsIpc: {
      ...original.skillsIpc,
      listFiles: mocks.listFiles,
      readFile: mocks.readFile,
      getScanSummary: mocks.getScanSummary,
    },
  };
});

vi.mock("@/components/ui/scroll-area", () => ({
  ScrollArea: ({ children, className }: { children: ReactNode; className?: string }) => (
    <div className={className}>{children}</div>
  ),
}));

vi.mock("sonner", () => ({ toast: { error: vi.fn() } }));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: Record<string, unknown>) => {
      if (key === "files") return `${values?.count} files`;
      if (key === "previewProgress") return `${values?.loaded} / ${values?.total}`;
      return key;
    },
  }),
}));

import { SkillDetailPanel } from "@/components/skills/SkillDetailPanel";

describe("lazy Skill file details", () => {
  beforeEach(() => {
    mocks.listFiles.mockReset();
    mocks.readFile.mockReset();
    mocks.getScanSummary.mockReset();
    mocks.listFiles.mockResolvedValue(filePage([], "folder"));
    mocks.getScanSummary.mockResolvedValue(scanSummary());
  });

  it("keeps the default Files tab body-free until a file is selected", async () => {
    mocks.readFile.mockResolvedValue(textPreview("SKILL.md", "# Instructions"));
    renderPanel([file("SKILL.md", 40)]);

    expect(screen.getAllByRole("tab").map((tab) => tab.textContent)).toEqual([
      "filesTab",
      "securityTab",
      "overviewTab",
    ]);
    expect(screen.getByRole("tab", { name: "filesTab" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByText("selectFileToPreview")).toBeTruthy();
    expect(mocks.readFile).not.toHaveBeenCalled();
    expect(mocks.getScanSummary).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("treeitem", { name: /SKILL.md/ }));
    await screen.findByText("# Instructions");
    expect(mocks.readFile).toHaveBeenCalledWith("skill-id", "SKILL.md", 0);
  });

  it("discards a late file response after the user selects another file", async () => {
    let resolveFirst: (preview: SkillFilePreview) => void = () => undefined;
    mocks.readFile.mockImplementation((_skillId: string, path: string) => {
      if (path === "a.txt") {
        return new Promise<SkillFilePreview>((resolve) => {
          resolveFirst = resolve;
        });
      }
      return Promise.resolve(textPreview("b.txt", "second-file"));
    });
    renderPanel([file("a.txt", 10), file("b.txt", 10)]);

    fireEvent.click(screen.getByRole("treeitem", { name: /a.txt/ }));
    fireEvent.click(screen.getByRole("treeitem", { name: /b.txt/ }));
    await screen.findByText("second-file");
    await act(async () => {
      resolveFirst(textPreview("a.txt", "late-first-file"));
      await Promise.resolve();
    });

    expect(screen.queryByText("late-first-file")).toBeNull();
    expect(screen.getByText("second-file")).toBeTruthy();
  });

  it("continues a large UTF-8 preview in bounded segments", async () => {
    mocks.readFile
      .mockResolvedValueOnce({
        ...textPreview("large.md", "first-segment"),
        next_offset: 204800,
        total_size_bytes: 204810,
      })
      .mockResolvedValueOnce({
        ...textPreview("large.md", "-second-segment"),
        offset: 204800,
        total_size_bytes: 204810,
      });
    renderPanel([file("large.md", 204810)]);

    fireEvent.click(screen.getByRole("treeitem", { name: /large.md/ }));
    await screen.findByText("first-segment");
    fireEvent.click(screen.getByRole("button", { name: "loadMorePreview" }));
    await screen.findByText("first-segment-second-segment");
    expect(mocks.readFile).toHaveBeenLastCalledWith("skill-id", "large.md", 204800);
  });

  it("shows metadata instead of rendering binary or unsupported text", async () => {
    mocks.readFile.mockResolvedValueOnce({
      ...textPreview("image.bin", null),
      kind: "binary",
      encoding: "binary",
      sha256: "abc123",
      total_size_bytes: 1536,
    });
    renderPanel([file("image.bin", 1536, false)]);

    fireEvent.click(screen.getByRole("treeitem", { name: /image.bin/ }));
    await screen.findByText("binaryPreviewUnavailable");
    expect(screen.getByText("abc123")).toBeTruthy();
    expect(screen.queryByRole("img")).toBeNull();
  });

  it("supports roving tree focus, directory expansion, and Enter to open", async () => {
    mocks.listFiles.mockResolvedValue(
      filePage([file("scripts/run.py", 12)], "scripts"),
    );
    mocks.readFile.mockResolvedValue(textPreview("scripts/run.py", "print('safe')"));
    renderPanel([directory("scripts"), file("SKILL.md", 20)]);
    const folder = screen.getByRole("treeitem", { name: /scripts/ });

    folder.focus();
    fireEvent.keyDown(folder, { key: "ArrowRight" });
    const child = await screen.findByRole("treeitem", { name: /run.py/ });
    fireEvent.keyDown(folder, { key: "ArrowRight" });
    expect(document.activeElement).toBe(child);
    fireEvent.keyDown(child, { key: "Enter" });
    await screen.findByText("print('safe')");
  });

  it("loads the scan placeholder only after Security is opened", async () => {
    renderPanel([file("SKILL.md", 20)]);
    expect(mocks.getScanSummary).not.toHaveBeenCalled();

    const securityTab = screen.getByRole("tab", { name: "securityTab" });
    fireEvent.pointerDown(securityTab, { button: 0, ctrlKey: false });
    fireEvent.mouseDown(securityTab, { button: 0 });
    fireEvent.click(securityTab);
    await screen.findByText("scanState.unscanned");
    expect(mocks.getScanSummary).toHaveBeenCalledWith("skill-id");
    expect(screen.getByText("scanPlaceholder")).toBeTruthy();
  });

  it("keeps a 500-entry tree within the interaction performance budget", () => {
    const entries = Array.from({ length: 500 }, (_, index) =>
      file(`file-${String(index).padStart(3, "0")}.txt`, index + 1),
    );
    const started = performance.now();
    renderPanel(entries);
    const elapsed = performance.now() - started;

    expect(screen.getAllByRole("treeitem")).toHaveLength(500);
    expect(elapsed).toBeLessThan(1000);
  });

  it("uses independent responsive tree and preview regions for long source", async () => {
    mocks.readFile.mockResolvedValue(
      textPreview("long.md", `line:${"x".repeat(500)}`),
    );
    const { container } = renderPanel([file("long.md", 505)]);
    fireEvent.click(screen.getByRole("treeitem", { name: /long.md/ }));
    const source = await screen.findByText(/line:/);

    expect(source.tagName).toBe("PRE");
    expect(source.className).toContain("whitespace-pre-wrap");
    expect(source.className).toContain("break-words");
    expect(container.querySelector(".max-md\\:grid-cols-1")).toBeTruthy();
  });
});

function renderPanel(items: SkillFileEntry[]) {
  return render(
    <SkillDetailPanel
      detail={summary()}
      initialFiles={filePage(items)}
      onBack={vi.fn()}
    />,
  );
}

function summary(): SkillSummary {
  return {
    generation: 3,
    body_bytes_transferred: 0,
    skill: {
      skill_id: "skill-id",
      slug: "demo-skill",
      name: "Demo Skill",
      description: "Description",
      version: "1.0.0",
      source_kind: "managed",
      source_ref: null,
      source_url: null,
      checksum: "hash",
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
      installed_at: "2026-08-01T00:00:00Z",
      updated_at: "2026-08-01T00:00:00Z",
    },
    manifest: {
      name: "demo-skill",
      description: "Description",
      license: null,
      compatibility: null,
      allowed_tools: null,
      metadata: {},
    },
  };
}

function filePage(items: SkillFileEntry[], parent: string | null = null): SkillFilePage {
  return {
    skill_id: "skill-id",
    generation: 3,
    parent,
    items,
    next_cursor: null,
  };
}

function file(path: string, size: number, text = true): SkillFileEntry {
  return {
    path,
    name: lastSegment(path, "/"),
    kind: lastSegment(path, "."),
    size_bytes: size,
    is_directory: false,
    is_text_candidate: text,
    is_link: false,
  };
}

function directory(path: string): SkillFileEntry {
  return {
    path,
    name: lastSegment(path, "/"),
    kind: "directory",
    size_bytes: 0,
    is_directory: true,
    is_text_candidate: false,
    is_link: false,
  };
}

function lastSegment(value: string, separator: string) {
  const segments = value.split(separator);
  return segments[segments.length - 1] || value;
}

function textPreview(path: string, content: string | null): SkillFilePreview {
  return {
    skill_id: "skill-id",
    generation: 3,
    path,
    kind: "text",
    encoding: "utf-8",
    content,
    offset: 0,
    next_offset: null,
    total_size_bytes: content?.length ?? 0,
    sha256: null,
  };
}

function scanSummary(): SkillScanSummary {
  return {
    skill_id: "skill-id",
    generation: 3,
    state: "unscanned",
    decision: null,
    max_severity: null,
    finding_counts: {},
    engine_version: null,
    policy_version: null,
    last_scanned_at: null,
    placeholder: true,
  };
}
