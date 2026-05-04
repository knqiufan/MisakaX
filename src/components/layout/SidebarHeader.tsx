import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores";
import { Sparkles } from "lucide-react";

export function SidebarHeader() {
  const collapsed = useAppStore((s) => s.sidebarCollapsed);

  return (
    <div
      className={cn(
        "flex h-12 shrink-0 items-center gap-2 border-b border-[color:var(--border-muted)] px-3"
      )}
    >
      <Sparkles className="h-6 w-6 shrink-0 text-[color:var(--text-accent-soft)]" />
      <span
        className={cn(
          "text-sidebar-heading text-foreground transition-opacity duration-[var(--ds-dur-normal)] ease-out",
          collapsed ? "w-0 opacity-0" : "opacity-100"
        )}
      >
        MisakaX
      </span>
    </div>
  );
}
