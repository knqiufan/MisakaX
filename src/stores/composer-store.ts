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

/**
 * 对话中由文件树「@ 引用」加入的工作区文件标签。
 * - `absPath`: 绝对路径，用于去重与后续可能的内容读取；
 * - `relPath`: 相对工作目录的相对路径（POSIX 风格），即 `@xxx/yyy.ts` 中的 `xxx/yyy.ts`；
 * - `name`: 文件名，仅用于 UI 显示。
 */
export interface PendingFileMention {
  id: string;
  absPath: string;
  relPath: string;
  name: string;
}

interface ComposerState {
  attachments: PendingAttachment[];
  mentions: PendingFileMention[];
  selectedSkillIds: string[];
  addAttachment: (attachment: PendingAttachment) => void;
  removeAttachment: (id: string) => void;
  clearAttachments: () => void;
  addMention: (mention: PendingFileMention) => void;
  removeMention: (id: string) => void;
  clearMentions: () => void;
  setSelectedSkills: (ids: string[]) => void;
}

export const useComposerStore = create<ComposerState>((set) => ({
  attachments: [],
  mentions: [],
  selectedSkillIds: [],
  addAttachment: (attachment) =>
    set((state) => ({ attachments: [...state.attachments, attachment] })),
  removeAttachment: (id) =>
    set((state) => ({
      attachments: state.attachments.filter((attachment) => attachment.id !== id),
    })),
  clearAttachments: () => set({ attachments: [] }),
  addMention: (mention) =>
    set((state) => {
      if (state.mentions.some((m) => m.absPath === mention.absPath)) {
        return state;
      }
      return { mentions: [...state.mentions, mention] };
    }),
  removeMention: (id) =>
    set((state) => ({
      mentions: state.mentions.filter((mention) => mention.id !== id),
    })),
  clearMentions: () => set({ mentions: [] }),
  setSelectedSkills: (ids) => set({ selectedSkillIds: ids }),
}));
