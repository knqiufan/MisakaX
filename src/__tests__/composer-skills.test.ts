import { describe, expect, it } from "vitest";
import {
  getSlashSkillQuery,
  handleSegmentBackspace,
  replaceSlashQueryWithSkill,
  reconcileActiveSkillSegments,
  selectableSkills,
  selectedSkillIds,
  type ComposerSegment,
} from "@/components/chat/composer/composerSegment";

const skill = {
  id: "skill-1",
  skillId: "stable-skill-id",
  slug: "code-review",
  name: "Code review",
  description: "Review a change",
};

describe("composer Skill segments", () => {
  it("derives selected skill ids in document order", () => {
    const segments: ComposerSegment[] = [
      { type: "text", id: "text-1", value: "Please " },
      { type: "skill", skill },
      { type: "text", id: "text-2", value: " inspect this" },
    ];
    expect(selectedSkillIds(segments)).toEqual(["stable-skill-id"]);
  });

  it("keeps stable identity independent from the slug snapshot", () => {
    const segments: ComposerSegment[] = [
      { type: "skill", skill },
      { type: "text", id: "text-1", value: "Run it" },
    ];
    expect(selectedSkillIds(segments)).toEqual(["stable-skill-id"]);
  });

  it("removes a chip when its stable Skill source is no longer active", () => {
    const segments: ComposerSegment[] = [
      { type: "text", id: "text-1", value: "Run " },
      { type: "skill", skill },
      { type: "text", id: "text-2", value: " now" },
    ];
    const reconciled = reconcileActiveSkillSegments(segments, new Set());
    expect(reconciled.removed).toEqual([skill]);
    expect(reconciled.segments).toEqual([
      { type: "text", id: "text-1", value: "Run  now" },
    ]);
  });

  it("filters disabled and unhealthy inventory entries from selector and slash sources", () => {
    const base = {
      skill_id: "base-id",
      name: "Code review",
      description: "Review a change",
      version: null,
      source_kind: "local",
      source_ref: null,
      source_url: null,
      checksum: "checksum",
      installed_path: "C:/skills/code-review",
      is_external: false,
      effective_active: true,
      effective_rank: 400,
      conflict: false,
      disabled_reason: null,
      security_state: "passed",
      installed_at: "",
      updated_at: "",
    };
    const inventory = [
      { ...base, skill_id: "enabled-id", slug: "enabled", enabled: true, health: "healthy" },
      { ...base, skill_id: "disabled-id", slug: "disabled", enabled: false, health: "healthy", effective_active: false },
      { ...base, skill_id: "missing-id", slug: "missing", enabled: true, health: "missing", effective_active: false },
    ];

    expect(selectableSkills(inventory).map((item) => item.slug)).toEqual(["enabled"]);
  });

  it("opens slash matching only at a whitespace boundary", () => {
    const segments: ComposerSegment[] = [{ type: "text", id: "text-1", value: "use /code" }];
    expect(getSlashSkillQuery(segments, { segmentId: "text-1", offset: 9 })).toMatchObject({
      start: 4,
      end: 9,
      query: "code",
    });

    const noBoundary: ComposerSegment[] = [{ type: "text", id: "text-2", value: "path/code" }];
    expect(getSlashSkillQuery(noBoundary, { segmentId: "text-2", offset: 9 })).toBeNull();
  });

  it("replaces a slash query with an inline Skill chip", () => {
    const segments: ComposerSegment[] = [{ type: "text", id: "text-1", value: "Run /code now" }];
    const slash = getSlashSkillQuery(segments, { segmentId: "text-1", offset: 9 });
    expect(slash).not.toBeNull();
    const result = replaceSlashQueryWithSkill(segments, slash!, skill);
    expect(result.segments.map((segment) => segment.type)).toEqual(["text", "skill", "text"]);
    expect(result.segments[0]).toMatchObject({ type: "text", value: "Run " });
    expect(result.segments[1]).toMatchObject({ type: "skill", skill: { slug: "code-review" } });
    expect(result.segments[2]).toMatchObject({ type: "text", value: " now" });
  });

  it("removes a Skill chip with Backspace at its following text boundary", () => {
    const segments: ComposerSegment[] = [
      { type: "text", id: "text-1", value: "Run " },
      { type: "skill", skill },
      { type: "text", id: "text-2", value: " now" },
    ];
    const result = handleSegmentBackspace(segments, { segmentId: "text-2", offset: 0 });
    expect(result.handled).toBe(true);
    expect(result.segments).toEqual([{ type: "text", id: "text-1", value: "Run  now" }]);
  });
});
