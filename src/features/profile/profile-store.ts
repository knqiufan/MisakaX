import { create } from "zustand";

import { profileIpc, type ProfileUpdateParams } from "@/lib/ipc/profile";
import type { UserProfile } from "@/lib/ipc/types";

type LoadStatus = "idle" | "loading" | "success" | "error";

interface ProfileState {
  profile: UserProfile | null;
  status: LoadStatus;
  error: string | null;
  updating: boolean;
  updateError: string | null;
  load: (force?: boolean) => Promise<UserProfile>;
  update: (params: ProfileUpdateParams) => Promise<UserProfile>;
  clearUpdateError: () => void;
}

let activeLoad: Promise<UserProfile> | null = null;

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const useProfileStore = create<ProfileState>((set, get) => ({
  profile: null,
  status: "idle",
  error: null,
  updating: false,
  updateError: null,

  load: async (force = false) => {
    const existing = get().profile;
    if (existing && !force) return existing;
    if (activeLoad && !force) return activeLoad;

    set({ status: "loading", error: null });
    const request = profileIpc
      .getCurrent()
      .then((profile) => {
        set({ profile, status: "success", error: null });
        return profile;
      })
      .catch((error: unknown) => {
        set({ status: "error", error: errorMessage(error) });
        throw error;
      })
      .finally(() => {
        if (activeLoad === request) activeLoad = null;
      });
    activeLoad = request;
    return request;
  },

  update: async (params) => {
    set({ updating: true, updateError: null });
    try {
      const profile = await profileIpc.update(params);
      set({ profile, status: "success", error: null, updating: false });
      return profile;
    } catch (error) {
      set({ updating: false, updateError: errorMessage(error) });
      throw error;
    }
  },

  clearUpdateError: () => set({ updateError: null }),
}));
