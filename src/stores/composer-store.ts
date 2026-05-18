import { create } from "zustand";

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

interface ComposerState {
  attachments: PendingAttachment[];
  selectedSkillIds: string[];
  addAttachment: (attachment: PendingAttachment) => void;
  removeAttachment: (id: string) => void;
  clearAttachments: () => void;
  setSelectedSkills: (ids: string[]) => void;
}

export const useComposerStore = create<ComposerState>((set) => ({
  attachments: [],
  selectedSkillIds: [],
  addAttachment: (attachment) =>
    set((state) => ({ attachments: [...state.attachments, attachment] })),
  removeAttachment: (id) =>
    set((state) => ({
      attachments: state.attachments.filter((attachment) => attachment.id !== id),
    })),
  clearAttachments: () => set({ attachments: [] }),
  setSelectedSkills: (ids) => set({ selectedSkillIds: ids }),
}));
