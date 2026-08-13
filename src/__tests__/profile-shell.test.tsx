import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const profileMocks = vi.hoisted(() => ({
  getCurrent: vi.fn(),
  update: vi.fn(),
  getAvatar: vi.fn(),
  setAvatar: vi.fn(),
  clearAvatar: vi.fn(),
}));

const { dialogOpen } = vi.hoisted(() => ({ dialogOpen: vi.fn() }));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: dialogOpen }));

vi.mock("@/lib/ipc/profile", () => ({
  profileIpc: profileMocks,
}));

vi.mock("@/pages", () => ({
  ChatPage: () => <div>chat-page</div>,
  ProfilePage: () => <div>profile-page</div>,
  KnowledgePage: () => <div>knowledge-page</div>,
  DashboardPage: () => <div>dashboard-page</div>,
  NotificationsPage: () => <div>notifications-page</div>,
  SettingsPage: () => <div>settings-page</div>,
}));

import { ContentArea } from "@/components/layout/ContentArea";
import { UnifiedTopBar } from "@/components/layout/UnifiedTopBar";
import { UserMenu } from "@/components/layout/UserMenu";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ProfileAvatar, ProfileHeader, profileInitial, validateDisplayName } from "@/features/profile";
import { useProfileStore } from "@/features/profile/profile-store";
import { i18n } from "@/locales/i18n";
import type { UserProfile } from "@/lib/ipc/types";
import { SESSION_LIST_DEFAULT_WIDTH, useAppStore } from "@/stores/app-store";

const PROFILE: UserProfile = {
  profile_id: "local",
  profile_kind: "local",
  display_name: "Misaka User",
  avatar_storage_key: null,
  avatar_sha256: null,
  timezone_mode: "system",
  timezone_id: null,
  week_start: 1,
  created_at: "2026-08-13T00:00:00Z",
  updated_at: "2026-08-13T00:00:00Z",
};

describe("profile shell", () => {
  beforeEach(async () => {
    await i18n.changeLanguage("en");
    useAppStore.setState({
      route: { page: "chat" },
      sessionListWidth: SESSION_LIST_DEFAULT_WIDTH,
      globalLoading: false,
    });
    useProfileStore.setState({
      profile: PROFILE,
      avatarUrl: null,
      status: "success",
      error: null,
      updating: false,
      avatarUpdating: false,
      updateError: null,
    });
    profileMocks.getCurrent.mockReset().mockResolvedValue(PROFILE);
    profileMocks.update.mockReset();
    profileMocks.getAvatar.mockReset().mockResolvedValue(null);
    profileMocks.setAvatar.mockReset();
    profileMocks.clearAvatar.mockReset();
    dialogOpen.mockReset();
  });

  it("routes the enabled user-menu profile item and reads the profile name", async () => {
    render(<UserMenu />);
    const trigger = screen.getByRole("button", { name: /Misaka User/ });
    expect(trigger.textContent).toContain("Misaka User");
    fireEvent.pointerDown(trigger, { button: 0, ctrlKey: false });
    const profileItem = await screen.findByRole("menuitem", { name: "Profile" });
    expect(profileItem.getAttribute("data-disabled")).toBeNull();
    fireEvent.click(profileItem);
    expect(useAppStore.getState().route).toEqual({ page: "profile" });
  });

  it("shows the profile top-bar title, returns to chat, and resolves ContentArea", () => {
    useAppStore.setState({ route: { page: "profile" } });
    const { rerender } = render(<UnifiedTopBar />);
    expect(screen.getByRole("heading", { name: "Profile" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /Back/ }));
    expect(useAppStore.getState().route).toEqual({ page: "chat" });

    useAppStore.setState({ route: { page: "profile" } });
    rerender(<ContentArea />);
    expect(screen.getByText("profile-page")).toBeTruthy();
  });

  it("renders the shared 80px avatar fallback and validates Unicode names", () => {
    const { container } = render(
      <ProfileAvatar displayName="御坂" avatarUrl="missing-avatar.webp" className="size-20" />
    );
    expect(screen.getByText("御")).toBeTruthy();
    expect(screen.getByRole("img", { name: "Avatar for 御坂" })).toBeTruthy();
    expect(container.querySelector('[data-slot="avatar"]')?.className).toContain("size-20");
    expect(profileInitial("", "U")).toBe("U");
    expect(validateDisplayName("   ")).toBe("empty");
    expect(validateDisplayName("😀".repeat(40))).toBeNull();
    expect(validateDisplayName("😀".repeat(41))).toBe("tooLong");
  });

  it("restores focus to the accessible edit trigger when the dialog closes", async () => {
    render(
      <TooltipProvider>
        <ProfileHeader />
      </TooltipProvider>
    );
    const trigger = screen.getByRole("button", { name: "Edit profile" });
    trigger.focus();
    fireEvent.click(trigger);
    fireEvent.click(await screen.findByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(document.activeElement).toBe(trigger));
  });

  it("saves a name in the standard dialog and synchronizes the shared store", async () => {
    const updated = { ...PROFILE, display_name: "Local Agent" };
    profileMocks.update.mockResolvedValue(updated);
    render(
      <TooltipProvider>
        <ProfileHeader />
      </TooltipProvider>
    );

    fireEvent.click(screen.getByRole("button", { name: "Edit profile" }));
    const input = await screen.findByRole("textbox", { name: "Display name" });
    fireEvent.change(input, { target: { value: " Local Agent " } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(profileMocks.update).toHaveBeenCalledWith({ displayName: "Local Agent" }));
    await waitFor(() => expect(useProfileStore.getState().profile?.display_name).toBe("Local Agent"));
    expect(screen.getByRole("heading", { name: "Local Agent" })).toBeTruthy();
  });

  it("selects an avatar copy and removes it through the shared store", async () => {
    dialogOpen.mockResolvedValue("D:\\pictures\\avatar.png");
    profileMocks.setAvatar.mockResolvedValue({
      profile: { ...PROFILE, avatar_storage_key: "avatar-safe.webp", avatar_sha256: "hash" },
      avatar_data_url: "data:image/webp;base64,c2FmZQ==",
    });
    profileMocks.clearAvatar.mockResolvedValue(PROFILE);
    render(
      <TooltipProvider>
        <ProfileHeader />
      </TooltipProvider>
    );

    fireEvent.click(screen.getByRole("button", { name: "Edit profile" }));
    fireEvent.click(await screen.findByRole("button", { name: "Change profile image" }));
    await waitFor(() =>
      expect(profileMocks.setAvatar).toHaveBeenCalledWith("D:\\pictures\\avatar.png")
    );
    expect(useProfileStore.getState().avatarUrl).toBe("data:image/webp;base64,c2FmZQ==");

    fireEvent.click(screen.getByRole("button", { name: "Remove profile image" }));
    await waitFor(() => expect(profileMocks.clearAvatar).toHaveBeenCalledOnce());
    expect(useProfileStore.getState().avatarUrl).toBeNull();
  });
});
