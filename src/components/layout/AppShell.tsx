import { useEffect } from "react";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";
import { ContentArea } from "./ContentArea";
import { useAppStore } from "@/stores";
import { useThemeStore } from "@/stores/theme-store";

const TABLET_BREAKPOINT = 1280;

export function AppShell() {
  const setSidebarCollapsed = useAppStore((s) => s.setSidebarCollapsed);
  const initialize = useThemeStore((s) => s.initialize);
  const cleanup = useThemeStore((s) => s.cleanup);

  useEffect(() => {
    initialize();
    return cleanup;
  }, [initialize, cleanup]);

  useEffect(() => {
    const handleResize = () => {
      if (window.innerWidth < TABLET_BREAKPOINT) {
        setSidebarCollapsed(true);
      }
    };

    handleResize();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [setSidebarCollapsed]);

  return (
    <TooltipProvider delayDuration={300}>
      <div className="flex h-screen w-screen overflow-hidden bg-background">
        <Sidebar />
        <div className="flex flex-1 flex-col overflow-hidden">
          <TopBar />
          <main className="flex-1 overflow-auto">
            <ContentArea />
          </main>
        </div>
      </div>
    </TooltipProvider>
  );
}
