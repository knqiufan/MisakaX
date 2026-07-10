import { useTranslation } from "react-i18next";
import { User, Settings, LogOut } from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/app-store";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface UserMenuProps {
  collapsed: boolean;
}

export function UserMenu({ collapsed }: UserMenuProps) {
  const navigate = useAppStore((s) => s.navigate);
  const { t } = useTranslation("nav");

  const trigger = (
    <button
      className={cn(
        "flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors duration-150 ease-out hover:bg-accent",
        collapsed && "justify-center px-0"
      )}
    >
      <Avatar className="h-7 w-7 shrink-0">
        <AvatarFallback className="bg-primary/10 text-xs text-primary">
          U
        </AvatarFallback>
      </Avatar>
      {!collapsed && (
        <span className="truncate text-muted-foreground">{t("user")}</span>
      )}
    </button>
  );

  const menu = (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>{trigger}</DropdownMenuTrigger>
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
        <DropdownMenuItem onClick={() => navigate({ page: "settings" })}>
          <Settings className="mr-2 h-4 w-4" />
          {t("userMenu.settings")}
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem disabled>
          <LogOut className="mr-2 h-4 w-4" />
          {t("userMenu.logout")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );

  if (collapsed) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <div>{menu}</div>
        </TooltipTrigger>
        <TooltipContent side="right" sideOffset={8}>
          <p>{t("user")}</p>
        </TooltipContent>
      </Tooltip>
    );
  }

  return menu;
}
