import { useCallback, useEffect, useState } from "react";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ContentArea } from "./ContentArea";
import { UnifiedTopBar } from "./UnifiedTopBar";
import { ResizeGutter } from "./ResizeGutter";
import { SessionPanel } from "@/components/chat/session/SessionPanel";
import { SettingsSidebar } from "@/components/settings/SettingsSidebar";
import {
  useAppStore,
  LG_BREAKPOINT,
  SESSION_LIST_DEFAULT_WIDTH,
  resolveLeftColumnWidth,
} from "@/stores/app-store";
import { useThemeStore } from "@/stores/theme-store";
import { useChatStore } from "@/stores/chat-store";
import { settingsIpc } from "@/lib/ipc";
import { cn } from "@/lib/utils";
import { i18n } from "@/locales/i18n";

export function AppShell() {
  const route = useAppStore((s) => s.route);
  const sessionListWidth = useAppStore((s) => s.sessionListWidth);
  const setSessionListWidth = useAppStore((s) => s.setSessionListWidth);
  const initializeTheme = useThemeStore((s) => s.initialize);
  const cleanupTheme = useThemeStore((s) => s.cleanup);
  const reducedTransparency = useThemeStore((s) => s.reducedTransparency);
  const setShowWorkspaceSelector = useChatStore(
    (s) => s.setShowWorkspaceSelector
  );
  const [isWindows, setIsWindows] = useState(false);
  const [viewportWidth, setViewportWidth] = useState(() =>
    typeof window === "undefined" ? LG_BREAKPOINT : window.innerWidth
  );

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
    const onResize = () => setViewportWidth(window.innerWidth);
    onResize();
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  const handleNewSession = useCallback(() => {
    setShowWorkspaceSelector(true);
  }, [setShowWorkspaceSelector]);

  const displayWidth = resolveLeftColumnWidth(sessionListWidth, viewportWidth);

  const handleLeftColumnResize = useCallback(
    (deltaX: number) => {
      setSessionListWidth(displayWidth + deltaX);
    },
    [displayWidth, setSessionListWidth]
  );

  const showLeftColumn = route.page === "chat" || route.page === "settings";

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
          {showLeftColumn ? (
            <>
              <div
                className="flex h-full shrink-0 flex-col overflow-hidden border-r border-sidebar-border"
                style={{ width: displayWidth }}
              >
                {route.page === "settings" ? (
                  <SettingsSidebar />
                ) : (
                  <SessionPanel onNewSession={handleNewSession} />
                )}
              </div>
              <ResizeGutter
                onResize={handleLeftColumnResize}
                onReset={() =>
                  setSessionListWidth(SESSION_LIST_DEFAULT_WIDTH)
                }
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
