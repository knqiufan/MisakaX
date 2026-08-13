import { beforeEach, describe, expect, it, vi } from "vitest";

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));

import { profileIpc } from "@/lib/ipc/profile";

describe("profile IPC", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("loads the current local profile", async () => {
    mockInvoke.mockResolvedValue({ profile_id: "local", display_name: "User" });
    await profileIpc.getCurrent();
    expect(mockInvoke).toHaveBeenCalledWith("profile_get_current", undefined);
  });

  it("normalizes update input into the Rust request DTO", async () => {
    mockInvoke.mockResolvedValue({ profile_id: "local", display_name: "Misaka" });
    await profileIpc.update({ displayName: "Misaka", weekStart: 1 });
    expect(mockInvoke).toHaveBeenCalledWith("profile_update", {
      request: {
        display_name: "Misaka",
        timezone_id: undefined,
        week_start: 1,
      },
    });
  });

  it("uses dedicated avatar commands without exposing storage paths", async () => {
    mockInvoke.mockResolvedValue(null);
    await profileIpc.getAvatar();
    await profileIpc.setAvatar("D:\\pictures\\avatar.png");
    await profileIpc.clearAvatar();
    expect(mockInvoke.mock.calls).toEqual([
      ["profile_avatar_get", undefined],
      ["profile_avatar_set", { filePath: "D:\\pictures\\avatar.png" }],
      ["profile_avatar_clear", undefined],
    ]);
  });
});
