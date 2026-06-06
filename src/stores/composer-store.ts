import { create } from "zustand";
import {
  createEmptyDocument,
  insertMentionAtCursor,
  removeMentionById,
  updateMentionInSegments,
  type ComposerCursor,
  type ComposerSegment,
} from "@/components/chat/composer/composer-segment";

export type PendingAttachment =
  | {
      id: string;
      kind: "image";
      data: string;
      media_type: string;
      file_name: string;
      size: number;
    }
  | {
      id: string;
      kind: "text";
      extracted_text: string;
      mime: string;
      file_name: string;
      size: number;
    };

export interface PendingFileMention {
  id: string;
  absPath: string;
  relPath: string;
  name: string;
  status: "loading" | "ready" | "error";
  extractedText?: string;
  size?: number;
  mime?: string;
}

interface MentionHydration {
  extractedText: string;
  size: number;
  mime: string;
}

interface ComposerState {
  attachments: PendingAttachment[];
  segments: ComposerSegment[];
  composerCursor: ComposerCursor | null;
  selectedSkillIds: string[];
  focusRequestId: number;
  addAttachment: (attachment: PendingAttachment) => void;
  removeAttachment: (id: string) => void;
  clearAttachments: () => void;
  setSegments: (segments: ComposerSegment[]) => void;
  updateTextSegment: (id: string, value: string) => void;
  setComposerCursor: (cursor: ComposerCursor | null) => void;
  insertInlineMention: (mention: PendingFileMention) => void;
  hydrateMentionContent: (id: string, meta: MentionHydration) => void;
  markMentionError: (id: string) => void;
  removeMention: (id: string) => void;
  clearMentions: () => void;
  setSelectedSkills: (ids: string[]) => void;
}

export const useComposerStore = create<ComposerState>((set) => ({
  attachments: [],
  segments: createEmptyDocument(),
  composerCursor: null,
  selectedSkillIds: [],
  focusRequestId: 0,
  addAttachment: (attachment) =>
    set((state) => ({ attachments: [...state.attachments, attachment] })),
  removeAttachment: (id) =>
    set((state) => ({
      attachments: state.attachments.filter((attachment) => attachment.id !== id),
    })),
  clearAttachments: () => set({ attachments: [] }),
  setSegments: (segments) => set({ segments }),
  updateTextSegment: (id, value) =>
    set((state) => ({
      segments: state.segments.map((seg) =>
        seg.type === "text" && seg.id === id ? { ...seg, value } : seg
      ),
    })),
  setComposerCursor: (cursor) => set({ composerCursor: cursor }),
  insertInlineMention: (mention) =>
    set((state) => {
      const result = insertMentionAtCursor(
        state.segments,
        state.composerCursor,
        mention
      );
      return {
        segments: result.segments,
        composerCursor: result.cursor,
        focusRequestId: state.focusRequestId + 1,
      };
    }),
  hydrateMentionContent: (id, meta) =>
    set((state) => ({
      segments: updateMentionInSegments(state.segments, id, (m) => ({
        ...m,
        status: "ready",
        extractedText: meta.extractedText,
        size: meta.size,
        mime: meta.mime,
      })),
    })),
  markMentionError: (id) =>
    set((state) => ({
      segments: updateMentionInSegments(state.segments, id, (m) => ({
        ...m,
        status: "error",
      })),
    })),
  removeMention: (id) =>
    set((state) => ({
      segments: removeMentionById(state.segments, id),
    })),
  clearMentions: () =>
    set({
      segments: createEmptyDocument(),
      composerCursor: null,
    }),
  setSelectedSkills: (ids) => set({ selectedSkillIds: ids }),
}));
