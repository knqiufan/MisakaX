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
      type="button"
      onClick={() => navigate(route)}
      aria-current={isActive ? "page" : undefined}
      className={cn(
        "relative flex w-full items-center gap-3 px-3 py-2 text-[0.8125rem] font-semibold leading-snug outline-none transition-[color,background-color,box-shadow,transform] duration-[var(--ds-dur-fast)] [transition-timing-function:var(--ds-ease-out)] focus-visible:ring-2 focus-visible:ring-[color:var(--border-accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-transparent",
        "rounded-[var(--radius-button)] border border-transparent",
        collapsed && "justify-center px-0",
        isActive
          ? "border-[color:var(--border-accent-soft)] bg-[color:var(--surface-active)] text-foreground shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]"
          : "text-muted-foreground hover:bg-[color:var(--surface-hover)] hover:text-foreground"
      )}
    >
      <Icon
        className={cn(
          "h-5 w-5 shrink-0",
          isActive && "text-[color:var(--text-accent-soft)]"
        )}
      />
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
    <span className="bg-destructive text-destructive-foreground ml-auto flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-meta">
      {count > 99 ? "99+" : count}
    </span>
  );
}
