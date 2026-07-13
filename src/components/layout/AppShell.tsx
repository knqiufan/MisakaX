import { useCallback, useEffect, useState } from "react";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Sidebar } from "./Sidebar";
import { ContentArea } from "./ContentArea";
import { UnifiedTopBar } from "./UnifiedTopBar";
import { ResizeGutter } from "./ResizeGutter";
import { SessionPanel } from "@/components/chat/session/SessionPanel";
import {
  useAppStore,
  LG_BREAKPOINT,
  SESSION_LIST_DEFAULT_WIDTH,
} from "@/stores/app-store";
import { useThemeStore } from "@/stores/theme-store";
import { useChatStore } from "@/stores/chat-store";
import { settingsIpc } from "@/lib/ipc";
import { cn } from "@/lib/utils";
import { i18n } from "@/locales/i18n";

export function AppShell() {
  const route = useAppStore((s) => s.route);
  const sessionListOpen = useAppStore((s) => s.sessionListOpen);
  const sessionListWidth = useAppStore((s) => s.sessionListWidth);
  const setSessionListOpen = useAppStore((s) => s.setSessionListOpen);
  const setSessionListWidth = useAppStore((s) => s.setSessionListWidth);
  const setSidebarCollapsed = useAppStore((s) => s.setSidebarCollapsed);
  const initializeTheme = useThemeStore((s) => s.initialize);
  const cleanupTheme = useThemeStore((s) => s.cleanup);
  const reducedTransparency = useThemeStore((s) => s.reducedTransparency);
  const setShowWorkspaceSelector = useChatStore((s) => s.setShowWorkspaceSelector);
  const [isWindows, setIsWindows] = useState(false);

  useEffect(() => {
    initializeTheme();
    return cleanupTheme;
  }, [initializeTheme, cleanupTheme]);

  useEffect(() => {
    void settingsIpc.getAppConfig().then((config) => {
      if (config.language && config.language !== i18n.language) {
        i18n.changeLanguage(config.language);
      }
    });
  }, []);

  useEffect(() => {
    void settingsIpc.getSystemInfo().then((info) => {
      setIsWindows(info.os.toLowerCase() === "windows");
    });
  }, []);

  useEffect(() => {
    const applyBreakpoint = () => {
      const narrow = window.innerWidth < LG_BREAKPOINT;
      setSidebarCollapsed(narrow);
      setSessionListOpen(!narrow);
    };

    applyBreakpoint();
    window.addEventListener("resize", applyBreakpoint);
    return () => window.removeEventListener("resize", applyBreakpoint);
  }, [setSidebarCollapsed, setSessionListOpen]);

  const handleNewSession = useCallback(() => {
    setShowWorkspaceSelector(true);
  }, [setShowWorkspaceSelector]);

  const handleSessionListResize = useCallback(
    (deltaX: number) => {
      setSessionListWidth(sessionListWidth + deltaX);
    },
    [sessionListWidth, setSessionListWidth]
  );

  const showSessionList = route.page === "chat" && sessionListOpen;

  return (
    <TooltipProvider delayDuration={300}>
      <div
        data-app-shell
        className={cn(
          "misaka-app flex h-screen w-screen flex-col overflow-hidden bg-background",
          reducedTransparency && "reduced-transparency",
          isWindows && "is-windows"
        )}
      >
        <UnifiedTopBar />
        <div
          data-app-content-row
          className="flex min-h-0 flex-1 overflow-hidden"
        >
          <Sidebar />
          {showSessionList ? (
            <>
              <div
                className="flex h-full shrink-0 flex-col overflow-hidden border-r border-sidebar-border"
                style={{ width: sessionListWidth }}
              >
                <SessionPanel onNewSession={handleNewSession} />
              </div>
              <ResizeGutter
                onResize={handleSessionListResize}
                onReset={() => setSessionListWidth(SESSION_LIST_DEFAULT_WIDTH)}
              />
            </>
          ) : null}
          <main className="relative flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden bg-background">
            <ContentArea />
          </main>
        </div>
      </div>
    </TooltipProvider>
  );
}
