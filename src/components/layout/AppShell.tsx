import { useEffect, useState } from "react";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";
import { ContentArea } from "./ContentArea";
import { useAppStore } from "@/stores";
import { useThemeStore } from "@/stores/theme-store";
import { settingsIpc } from "@/lib/ipc";
import { cn } from "@/lib/utils";

const NARROW_BREAKPOINT = 960;

export function AppShell() {
  const setSidebarCollapsed = useAppStore((s) => s.setSidebarCollapsed);
  const initializeTheme = useThemeStore((s) => s.initialize);
  const cleanupTheme = useThemeStore((s) => s.cleanup);
  const reducedTransparency = useThemeStore((s) => s.reducedTransparency);
  const [isWindows, setIsWindows] = useState(false);

  useEffect(() => {
    initializeTheme();
    return cleanupTheme;
  }, [initializeTheme, cleanupTheme]);

  useEffect(() => {
    void settingsIpc.getSystemInfo().then((info) => {
      setIsWindows(info.os.toLowerCase() === "windows");
    });
  }, []);

  useEffect(() => {
    const handleResize = () => {
      if (window.innerWidth < NARROW_BREAKPOINT) {
        setSidebarCollapsed(true);
      }
    };

    handleResize();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [setSidebarCollapsed]);

  return (
    <TooltipProvider delayDuration={300}>
      <div
        className={cn(
          "misaka-app flex h-screen w-screen overflow-hidden bg-background",
          reducedTransparency && "reduced-transparency",
          isWindows && "is-windows"
        )}
      >
        <Sidebar />
        <div className="relative flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
          <TopBar />
          <main className="min-h-0 flex-1 overflow-auto bg-[color:var(--surface-messages)]">
            <ContentArea />
          </main>
        </div>
      </div>
    </TooltipProvider>
  );
}
