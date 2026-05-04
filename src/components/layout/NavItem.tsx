import type { LucideIcon } from "lucide-react";
import type { Route } from "@/stores/app-store";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface NavItemProps {
  icon: LucideIcon;
  label: string;
  route: Route;
  badge?: number;
}

export function NavItem({ icon: Icon, label, route, badge }: NavItemProps) {
  const currentRoute = useAppStore((s) => s.route);
  const navigate = useAppStore((s) => s.navigate);
  const collapsed = useAppStore((s) => s.sidebarCollapsed);

  const isActive = currentRoute.page === route.page;

  const button = (
    <button
      onClick={() => navigate(route)}
      aria-current={isActive ? "page" : undefined}
      className={cn(
        "relative flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors duration-150 ease-out",
        collapsed && "justify-center px-0",
        isActive
          ? "bg-primary/10 text-primary"
          : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
      )}
    >
      {isActive && (
        <span className="absolute left-0 top-1/2 h-5 w-[3px] -translate-y-1/2 rounded-r-full bg-primary" />
      )}
      <Icon className="h-5 w-5 shrink-0" />
      {!collapsed && <span className="truncate">{label}</span>}
      {badge !== undefined && badge > 0 && (
        <BadgeIndicator count={badge} collapsed={collapsed} />
      )}
    </button>
  );

  if (collapsed) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>{button}</TooltipTrigger>
        <TooltipContent side="right" sideOffset={8}>
          <p>{label}</p>
        </TooltipContent>
      </Tooltip>
    );
  }

  return button;
}

function BadgeIndicator({
  count,
  collapsed,
}: {
  count: number;
  collapsed: boolean;
}) {
  if (collapsed) {
    return (
      <span className="absolute right-1 top-1 h-2 w-2 rounded-full bg-destructive" />
    );
  }

  return (
    <span className="ml-auto flex h-5 min-w-5 items-center justify-center rounded-full bg-destructive px-1.5 text-[10px] font-semibold text-destructive-foreground">
      {count > 99 ? "99+" : count}
    </span>
  );
}
