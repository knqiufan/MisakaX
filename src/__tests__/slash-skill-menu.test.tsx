import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SlashSkillMenu } from "@/components/chat/composer/SlashSkillMenu";
import type { InstalledSkill } from "@/lib/ipc";

describe("SlashSkillMenu", () => {
  const originalScrollIntoView = HTMLElement.prototype.scrollIntoView;

  beforeEach(() => {
    // The browser bridge may return a non-function value from scrollIntoView.
    // React must never receive that value as a useEffect cleanup callback.
    Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
      configurable: true,
      value: vi.fn(
        () => ({ queued: true }) as unknown as void,
      ) as unknown as typeof HTMLElement.prototype.scrollIntoView,
    });
  });

  afterEach(() => {
    Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
      configurable: true,
      value: originalScrollIntoView,
    });
  });

  it("changes the active option on hover without treating scrolling as effect cleanup", () => {
    render(<SlashSkillMenuHarness />);

    const reviewOption = screen.getAllByRole("option")[1];
    fireEvent.mouseEnter(reviewOption);

    expect(reviewOption.getAttribute("aria-selected")).toBe("true");
  });
});

function SlashSkillMenuHarness() {
  const [activeIndex, setActiveIndex] = useState(0);
  return (
    <SlashSkillMenu
      open
      query=""
      skills={skills}
      label="Skills"
      activeIndex={activeIndex}
      onActiveIndexChange={setActiveIndex}
      onSelect={vi.fn()}
    />
  );
}

const skills: InstalledSkill[] = [
  installedSkill({ slug: "write-docs", name: "Write docs" }),
  installedSkill({ slug: "review", name: "Review changes" }),
];

function installedSkill(overrides: Partial<InstalledSkill>): InstalledSkill {
  return {
    skill_id: overrides.skill_id ?? `id-${overrides.slug ?? "skill"}`,
    slug: "skill",
    name: "Skill",
    description: "Skill description",
    version: null,
    source_kind: "local",
    source_ref: null,
    source_url: null,
    checksum: "checksum",
    installed_path: "C:/skills/skill",
    enabled: true,
    health: "healthy",
    is_external: false,
    effective_active: true,
    effective_rank: 400,
    conflict: false,
    disabled_reason: null,
    security_state: "passed",
    installed_at: "2026-07-28T00:00:00Z",
    updated_at: "2026-07-28T00:00:00Z",
    ...overrides,
  };
}
