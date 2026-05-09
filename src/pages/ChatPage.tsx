import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { MessageSquare, Plus } from "lucide-react";
import { EmptyState } from "@/components/layout/EmptyState";
import { Button } from "@/components/ui/button";
import { WorkspaceSelector } from "@/components/chat/WorkspaceSelector";
import { WorkspaceBar } from "@/components/chat/WorkspaceBar";
import { useChatStore } from "@/stores/chat-store";
import { sessionsIpc } from "@/lib/ipc";

export function ChatPage() {
  const { t } = useTranslation();
  const {
    activeSession,
    showWorkspaceSelector,
    setShowWorkspaceSelector,
    setActiveSessionData,
    updateActiveSessionWorkingDir,
  } = useChatStore();

  const handleNewSession = useCallback(() => {
    setShowWorkspaceSelector(true);
  }, [setShowWorkspaceSelector]);

  const handleWorkspaceSelected = useCallback(
    async (path: string | null) => {
      try {
        const session = await sessionsIpc.create({
          workingDirectory: path ?? undefined,
        });
        setActiveSessionData(session);
      } catch (err) {
        console.error("Failed to create session:", err);
      }
    },
    [setActiveSessionData]
  );

  const handleChangeWorkingDir = useCallback(() => {
    setShowWorkspaceSelector(true);
  }, [setShowWorkspaceSelector]);

  const handleWorkspaceSwitched = useCallback(
    async (path: string | null) => {
      if (!activeSession) return;
      try {
        await sessionsIpc.updateWorkingDir({
          sessionId: activeSession.id,
          workingDirectory: path ?? undefined,
        });
        updateActiveSessionWorkingDir(path);
      } catch (err) {
        console.error("Failed to update working directory:", err);
      }
    },
    [activeSession, updateActiveSessionWorkingDir]
  );

  return (
    <div className="flex h-full flex-col">
      {activeSession ? (
        <>
          <WorkspaceBar
            workingDir={activeSession.working_directory}
            onChangeDir={handleChangeWorkingDir}
          />
          <div className="flex flex-1 flex-col items-center justify-center">
            <EmptyState
              icon={MessageSquare}
              title={t("nav:chat")}
              description={t("comingSoon")}
            />
          </div>
          <WorkspaceSelector
            open={showWorkspaceSelector}
            onOpenChange={setShowWorkspaceSelector}
            onSelect={handleWorkspaceSwitched}
            initialPath={activeSession.working_directory}
          />
        </>
      ) : (
        <>
          <div className="flex h-full flex-col items-center justify-center gap-4">
            <EmptyState
              icon={MessageSquare}
              title={t("nav:chat")}
              description={t("comingSoon")}
            />
            <Button onClick={handleNewSession} className="gap-2">
              <Plus className="h-4 w-4" />
              {t("common:newSession")}
            </Button>
          </div>
          <WorkspaceSelector
            open={showWorkspaceSelector}
            onOpenChange={setShowWorkspaceSelector}
            onSelect={handleWorkspaceSelected}
          />
        </>
      )}
    </div>
  );
}
