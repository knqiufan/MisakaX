import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  User,
  LogOut,
  BookOpen,
  LayoutDashboard,
  Bell,
  Settings,
} from "lucide-react";
import { useAppStore } from "@/stores/app-store";
import { ProfileAvatar } from "@/features/profile";
import { useProfileStore } from "@/features/profile/profile-store";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

export function UserMenu() {
  const navigate = useAppStore((s) => s.navigate);
  const { t } = useTranslation("nav");
  const profile = useProfileStore((state) => state.profile);
  const avatarUrl = useProfileStore((state) => state.avatarUrl);
  const loadProfile = useProfileStore((state) => state.load);

  useEffect(() => {
    void loadProfile().catch(() => undefined);
  }, [loadProfile]);

  const displayName = profile?.display_name ?? t("user");

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className="flex h-9 w-full items-center gap-2 rounded-xl px-3 text-[13px] font-normal text-sidebar-foreground outline-none transition-colors duration-150 hover:bg-sidebar-accent/60 focus-visible:ring-2 focus-visible:ring-ring/35"
        >
          <ProfileAvatar
            displayName={displayName}
          avatarUrl={avatarUrl}
            className="size-5"
            fallbackClassName="text-[10px]"
          />
          <span className="truncate">{displayName}</span>
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        side="right"
        align="end"
        sideOffset={8}
        className="w-48"
      >
        <DropdownMenuItem onClick={() => navigate({ page: "profile" })}>
          <User className="mr-2 h-4 w-4" />
          {t("userMenu.profile")}
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => navigate({ page: "notifications" })}>
          <Bell className="mr-2 h-4 w-4" />
          {t("notifications")}
          <span className="ml-auto flex h-5 min-w-5 items-center justify-center rounded-full bg-destructive px-1.5 text-[10px] text-destructive-foreground">
            3
          </span>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => navigate({ page: "settings" })}>
          <Settings className="mr-2 h-4 w-4" />
          {t("settings")}
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={() => navigate({ page: "knowledge" })}>
          <BookOpen className="mr-2 h-4 w-4" />
          {t("knowledge")}
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => navigate({ page: "dashboard" })}>
          <LayoutDashboard className="mr-2 h-4 w-4" />
          {t("dashboard")}
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem disabled>
          <LogOut className="mr-2 h-4 w-4" />
          {t("userMenu.logout")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
