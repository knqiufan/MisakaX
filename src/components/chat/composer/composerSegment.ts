import type { InstalledSkill } from "@/lib/ipc";
import type { PendingFileMention, PendingSkill } from "@/stores/composer-store";

export type ComposerSegment =
  | { type: "text"; id: string; value: string }
  | { type: "mention"; mention: PendingFileMention }
  | { type: "skill"; skill: PendingSkill };

export interface ComposerCursor {
  segmentId: string;
  offset: number;
}

export function createEmptyDocument(): ComposerSegment[] {
  return [{ type: "text", id: crypto.randomUUID(), value: "" }];
}

export function getSegmentId(segment: ComposerSegment): string {
  if (segment.type === "text") return segment.id;
  return segment.type === "mention" ? segment.mention.id : segment.skill.id;
}

export function collectMentions(segments: ComposerSegment[]): PendingFileMention[] {
  return segments
    .filter((s): s is Extract<ComposerSegment, { type: "mention" }> => s.type === "mention")
    .map((s) => s.mention);
}

export function collectSkills(segments: ComposerSegment[]): PendingSkill[] {
  return segments
    .filter((s): s is Extract<ComposerSegment, { type: "skill" }> => s.type === "skill")
    .map((s) => s.skill);
}

export function selectedSkillIds(segments: ComposerSegment[]): string[] {
  return collectSkills(segments).map((skill) => skill.skillId);
}

export function selectableSkills(skills: InstalledSkill[]): InstalledSkill[] {
  return skills.filter((skill) => skill.effective_active);
}

export function reconcileActiveSkillSegments(
  segments: ComposerSegment[],
  activeSkillIds: ReadonlySet<string>
): { segments: ComposerSegment[]; removed: PendingSkill[] } {
  const removed = collectSkills(segments).filter(
    (skill) => !activeSkillIds.has(skill.skillId)
  );
  const next = removed.reduce(
    (current, skill) => removeSkillById(current, skill.id),
    segments
  );
  return { segments: next, removed };
}

export function getDocumentPlainText(segments: ComposerSegment[]): string {
  return segments
    .filter((s): s is Extract<ComposerSegment, { type: "text" }> => s.type === "text")
    .map((s) => s.value)
    .join("");
}

export function countSendableFromSegments(segments: ComposerSegment[]): number {
  return collectMentions(segments).filter((m) => m.status !== "error").length;
}

export function hasMentionAbsPath(segments: ComposerSegment[], absPath: string): boolean {
  return collectMentions(segments).some((m) => m.absPath === absPath);
}

export function buildOutgoingFromSegments(segments: ComposerSegment[]): string {
  const parts: string[] = [];
  for (const seg of segments) {
    if (seg.type === "text") {
      parts.push(seg.value);
      continue;
    }
    if (seg.type === "mention" && seg.mention.status !== "error") {
      parts.push(`@${seg.mention.relPath}`);
    }
  }
  return parts.join("").trim();
}

export function insertMentionAtCursor(
  segments: ComposerSegment[],
  cursor: ComposerCursor | null,
  mention: PendingFileMention
): { segments: ComposerSegment[]; cursor: ComposerCursor } {
  if (hasMentionAbsPath(segments, mention.absPath)) {
    return { segments, cursor: cursor ?? defaultCursor(segments) };
  }
  return insertReferenceAtCursor(segments, cursor, { type: "mention", mention }, mention.id);
}

export function insertSkillAtCursor(
  segments: ComposerSegment[],
  cursor: ComposerCursor | null,
  skill: PendingSkill
): { segments: ComposerSegment[]; cursor: ComposerCursor } {
  if (collectSkills(segments).some((item) => item.skillId === skill.skillId)) {
    return { segments, cursor: cursor ?? defaultCursor(segments) };
  }
  return insertReferenceAtCursor(segments, cursor, { type: "skill", skill }, skill.id);
}

function insertReferenceAtCursor(
  segments: ComposerSegment[],
  cursor: ComposerCursor | null,
  reference: Exclude<ComposerSegment, { type: "text" }>,
  referenceId: string
): { segments: ComposerSegment[]; cursor: ComposerCursor } {
  const { index, offset } = resolveInsertPosition(segments, cursor);
  const target = segments[index];
  if (target.type !== "text") {
    return insertAtEnd(segments, reference, referenceId);
  }

  const before = target.value.slice(0, offset);
  const after = target.value.slice(offset);
  const next: ComposerSegment[] = [
    ...segments.slice(0, index),
    { type: "text", id: target.id, value: before },
    reference,
    { type: "text", id: crypto.randomUUID(), value: after },
    ...segments.slice(index + 1),
  ];

  const merged = normalizeSegments(next);
  const afterTextId = findTextSegmentAfterReference(merged, referenceId);
  return {
    segments: merged,
    cursor: { segmentId: afterTextId, offset: 0 },
  };
}

export function handleSegmentBackspace(
  segments: ComposerSegment[],
  cursor: ComposerCursor
): { segments: ComposerSegment[]; cursor: ComposerCursor; handled: boolean } {
  const index = segments.findIndex((s) => getSegmentId(s) === cursor.segmentId);
  if (index < 0) return { segments, cursor, handled: false };

  const current = segments[index];
  if (current.type !== "text" || cursor.offset > 0) {
    return { segments, cursor, handled: false };
  }

  if (index === 0) return { segments, cursor, handled: false };

  const previous = segments[index - 1];
  if (isReference(previous)) {
    return removeReferenceAndMerge(segments, index - 1, index);
  }

  if (previous.type === "text") {
    const mergedValue = previous.value + current.value;
    const next = [
      ...segments.slice(0, index - 1),
      { type: "text" as const, id: previous.id, value: mergedValue },
      ...segments.slice(index + 1),
    ];
    return {
      segments: normalizeSegments(next),
      cursor: { segmentId: previous.id, offset: previous.value.length },
      handled: true,
    };
  }

  return { segments, cursor, handled: false };
}

export function handleSegmentDelete(
  segments: ComposerSegment[],
  cursor: ComposerCursor
): { segments: ComposerSegment[]; cursor: ComposerCursor; handled: boolean } {
  const index = segments.findIndex((s) => getSegmentId(s) === cursor.segmentId);
  if (index < 0) return { segments, cursor, handled: false };

  const current = segments[index];
  if (current.type !== "text" || cursor.offset < current.value.length) {
    return { segments, cursor, handled: false };
  }

  const nextSeg = segments[index + 1];
  if (!nextSeg || !isReference(nextSeg)) {
    return { segments, cursor, handled: false };
  }

  return removeReferenceAndMerge(segments, index + 1, index);
}

export function removeMentionById(
  segments: ComposerSegment[],
  mentionId: string
): ComposerSegment[] {
  const mentionIndex = segments.findIndex(
    (s) => s.type === "mention" && s.mention.id === mentionId
  );
  if (mentionIndex < 0) return segments;
  const textIndex = findAdjacentTextIndex(segments, mentionIndex);
  return removeReferenceAndMerge(segments, mentionIndex, textIndex).segments;
}

export function removeSkillById(segments: ComposerSegment[], skillId: string): ComposerSegment[] {
  const skillIndex = segments.findIndex(
    (s) => s.type === "skill" && s.skill.id === skillId
  );
  if (skillIndex < 0) return segments;
  const textIndex = findAdjacentTextIndex(segments, skillIndex);
  return removeReferenceAndMerge(segments, skillIndex, textIndex).segments;
}

export function updateMentionInSegments(
  segments: ComposerSegment[],
  mentionId: string,
  updater: (mention: PendingFileMention) => PendingFileMention
): ComposerSegment[] {
  return segments.map((seg) =>
    seg.type === "mention" && seg.mention.id === mentionId
      ? { type: "mention", mention: updater(seg.mention) }
      : seg
  );
}

export function findLastTextSegmentIndex(segments: ComposerSegment[]): number {
  return findLastTextIndex(segments);
}

export function clearAllTextContent(
  segments: ComposerSegment[]
): { segments: ComposerSegment[]; cursor: ComposerCursor } {
  const cleared = segments.map((seg) =>
    seg.type === "text" ? { ...seg, value: "" } : seg
  );
  const next = normalizeSegments(cleared);
  return { segments: next, cursor: defaultCursor(next) };
}

function resolveInsertPosition(
  segments: ComposerSegment[],
  cursor: ComposerCursor | null
): { index: number; offset: number } {
  if (cursor) {
    const index = segments.findIndex((s) => getSegmentId(s) === cursor.segmentId);
    if (index >= 0 && segments[index].type === "text") {
      return { index, offset: cursor.offset };
    }
  }
  const index = findLastTextIndex(segments);
  const seg = segments[index];
  const offset = seg.type === "text" ? seg.value.length : 0;
  return { index, offset };
}

function insertAtEnd(
  segments: ComposerSegment[],
  reference: Exclude<ComposerSegment, { type: "text" }>,
  referenceId: string
): { segments: ComposerSegment[]; cursor: ComposerCursor } {
  const index = findLastTextIndex(segments);
  const target = segments[index];
  if (target.type !== "text") {
    const next: ComposerSegment[] = [
      ...segments,
      reference,
      { type: "text", id: crypto.randomUUID(), value: "" },
    ];
    const merged = normalizeSegments(next);
    return {
      segments: merged,
      cursor: { segmentId: findLastTextSegmentId(merged), offset: 0 },
    };
  }

  return insertReferenceAtCursor(
    segments,
    { segmentId: target.id, offset: target.value.length },
    reference,
    referenceId
  );
}

function removeReferenceAndMerge(
  segments: ComposerSegment[],
  mentionIndex: number,
  textIndex: number
): { segments: ComposerSegment[]; cursor: ComposerCursor; handled: boolean } {
  const textSeg = segments[textIndex];
  if (textSeg.type !== "text") {
    return { segments, cursor: defaultCursor(segments), handled: false };
  }

  const beforeText = mentionIndex > 0 ? segments[mentionIndex - 1] : null;
  const afterText =
    mentionIndex < segments.length - 1 ? segments[mentionIndex + 1] : null;

  let next = segments.filter((_, i) => i !== mentionIndex);

  if (beforeText?.type === "text" && afterText?.type === "text") {
    const mergedId = beforeText.id;
    const mergedValue = beforeText.value + afterText.value;
    next = next
      .filter((s) => getSegmentId(s) !== afterText.id)
      .map((s) =>
        s.type === "text" && s.id === mergedId
          ? { type: "text" as const, id: mergedId, value: mergedValue }
          : s
      );
    return {
      segments: normalizeSegments(next),
      cursor: { segmentId: mergedId, offset: beforeText.value.length },
      handled: true,
    };
  }

  if (beforeText?.type === "text") {
    return {
      segments: normalizeSegments(next),
      cursor: { segmentId: beforeText.id, offset: beforeText.value.length },
      handled: true,
    };
  }

  if (afterText?.type === "text") {
    return {
      segments: normalizeSegments(next),
      cursor: { segmentId: afterText.id, offset: 0 },
      handled: true,
    };
  }

  return {
    segments: normalizeSegments(next),
    cursor: { segmentId: textSeg.id, offset: 0 },
    handled: true,
  };
}

function normalizeSegments(segments: ComposerSegment[]): ComposerSegment[] {
  const merged: ComposerSegment[] = [];
  for (const seg of segments) {
    const last = merged[merged.length - 1];
    if (seg.type === "text" && last?.type === "text") {
      merged[merged.length - 1] = {
        type: "text",
        id: last.id,
        value: last.value + seg.value,
      };
    } else {
      merged.push(seg);
    }
  }
  const base = merged.length > 0 ? merged : createEmptyDocument();
  return pruneEmptyTextSegments(base);
}

/** 移除 mention 之间的空 text 段，避免渲染出宽 textarea 把 chip 撑开。保留末尾 text 段供输入。 */
function pruneEmptyTextSegments(segments: ComposerSegment[]): ComposerSegment[] {
  const lastTextIdx = findLastTextIndex(segments);
  const pruned = segments.filter((seg, i) => {
    if (seg.type !== "text") return true;
    if (i === lastTextIdx) return true;
    return seg.value.length > 0;
  });
  return pruned.length > 0 ? pruned : createEmptyDocument();
}

function findLastTextIndex(segments: ComposerSegment[]): number {
  for (let i = segments.length - 1; i >= 0; i--) {
    if (segments[i].type === "text") return i;
  }
  return 0;
}

function findLastTextSegmentId(segments: ComposerSegment[]): string {
  for (let i = segments.length - 1; i >= 0; i--) {
    const segment = segments[i];
    if (segment.type === "text") return segment.id;
  }
  const first = segments[0];
  return first.type === "text" ? first.id : "";
}

function findTextSegmentAfterReference(
  segments: ComposerSegment[],
  referenceId: string
): string {
  const index = segments.findIndex(
    (s) => getSegmentId(s) === referenceId
  );
  if (index >= 0 && index + 1 < segments.length) {
    const next = segments[index + 1];
    if (next.type === "text") return next.id;
  }
  return findLastTextSegmentId(segments);
}

export interface SlashSkillQuery {
  segmentId: string;
  start: number;
  end: number;
  query: string;
}

export function getSlashSkillQuery(
  segments: ComposerSegment[],
  cursor: ComposerCursor | null
): SlashSkillQuery | null {
  if (!cursor) return null;
  const segment = segments.find(
    (item) => item.type === "text" && item.id === cursor.segmentId
  );
  if (!segment || segment.type !== "text") return null;
  const beforeCursor = segment.value.slice(0, cursor.offset);
  const match = /(^|\s)\/([a-z0-9-]*)$/i.exec(beforeCursor);
  if (!match) return null;
  return {
    segmentId: segment.id,
    start: beforeCursor.length - match[2].length - 1,
    end: cursor.offset,
    query: match[2],
  };
}

export function replaceSlashQueryWithSkill(
  segments: ComposerSegment[],
  slash: SlashSkillQuery,
  skill: PendingSkill
): { segments: ComposerSegment[]; cursor: ComposerCursor } {
  const next = segments.map((segment) => {
    if (segment.type !== "text" || segment.id !== slash.segmentId) return segment;
    return {
      ...segment,
      value: segment.value.slice(0, slash.start) + segment.value.slice(slash.end),
    };
  });
  return insertSkillAtCursor(next, { segmentId: slash.segmentId, offset: slash.start }, skill);
}

function isReference(
  segment: ComposerSegment
): segment is Exclude<ComposerSegment, { type: "text" }> {
  return segment.type === "mention" || segment.type === "skill";
}

function defaultCursor(segments: ComposerSegment[]): ComposerCursor {
  const segmentId = findLastTextSegmentId(segments);
  const seg = segments.find((s) => s.type === "text" && s.id === segmentId);
  return {
    segmentId,
    offset: seg?.type === "text" ? seg.value.length : 0,
  };
}

function findAdjacentTextIndex(segments: ComposerSegment[], mentionIndex: number): number {
  const after = segments[mentionIndex + 1];
  if (after?.type === "text") return mentionIndex + 1;
  const before = segments[mentionIndex - 1];
  if (before?.type === "text") return mentionIndex - 1;
  return findLastTextIndex(segments);
}
