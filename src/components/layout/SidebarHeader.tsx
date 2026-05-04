import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores";
import { Sparkles } from "lucide-react";

export function SidebarHeader() {
  const collapsed = useAppStore((s) => s.sidebarCollapsed);

  return (
    <div className="flex h-12 items-center gap-2 border-b border-border px-3">
      <Sparkles className="h-6 w-6 shrink-0 text-primary" />
      <span
        className={cn(
          "text-base font-bold text-foreground transition-opacity duration-200",
          collapsed ? "w-0 opacity-0" : "opacity-100"
        )}
      >
        MisakaX
      </span>
    </div>
  );
}
