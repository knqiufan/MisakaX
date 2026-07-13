import { useTranslation } from "react-i18next";
import {
  User,
  LogOut,
  Sparkles,
  BookOpen,
  LayoutDashboard,
} from "lucide-react";
import { useAppStore } from "@/stores/app-store";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
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

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className="flex h-9 w-full items-center gap-2 rounded-xl px-3 text-[13px] font-normal text-sidebar-foreground outline-none transition-colors duration-150 hover:bg-sidebar-accent/60 focus-visible:ring-2 focus-visible:ring-ring/35"
        >
          <Avatar className="size-5 shrink-0">
            <AvatarFallback className="bg-primary/10 text-[10px] text-primary">
              U
            </AvatarFallback>
          </Avatar>
          <span className="truncate">{t("user")}</span>
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        side="right"
        align="end"
        sideOffset={8}
        className="w-48"
      >
        <DropdownMenuItem disabled>
          <User className="mr-2 h-4 w-4" />
          {t("userMenu.profile")}
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => navigate({ page: "skills" })}>
          <Sparkles className="mr-2 h-4 w-4" />
          {t("skills")}
        </DropdownMenuItem>
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
