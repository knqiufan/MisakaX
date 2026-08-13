import { invoke } from "./invoke";
import type { UserProfile } from "./types";

export interface ProfileUpdateParams {
  displayName?: string;
  timezoneId?: string;
  weekStart?: 0 | 1;
}

export interface ProfileAvatarResponse {
  profile: UserProfile;
  avatar_data_url: string;
}

export const profileIpc = {
  getCurrent: () => invoke<UserProfile>("profile_get_current"),
  update: (params: ProfileUpdateParams) =>
    invoke<UserProfile>("profile_update", {
      request: {
        display_name: params.displayName,
        timezone_id: params.timezoneId,
        week_start: params.weekStart,
      },
    }),
  getAvatar: () => invoke<string | null>("profile_avatar_get"),
  setAvatar: (filePath: string) =>
    invoke<ProfileAvatarResponse>("profile_avatar_set", { filePath }),
  clearAvatar: () => invoke<UserProfile>("profile_avatar_clear"),
};
