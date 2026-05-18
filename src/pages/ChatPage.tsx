import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { MessageSquare, Plus } from "lucide-react";
import {
  Group as PanelGroup,
  Panel,
  Separator as PanelResizeHandle,
} from "react-resizable-panels";
import { Button } from "@/components/ui/button";
import { WorkspaceSelector } from "@/components/chat/WorkspaceSelector";
import { ChatView } from "@/components/chat/ChatView";
import { SessionPanel } from "@/components/chat/SessionPanel";
import { WorkspaceExplorer } from "@/components/chat/workspace-explorer/WorkspaceExplorer";
import { useChatStore } from "@/stores/chat-store";
import { useWorkspaceExplorerStore } from "@/stores/workspace-explorer-store";
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
  const { open: explorerOpen, setOpen: setExplorerOpen } =
    useWorkspaceExplorerStore();

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
    <div className="flex h-full">
      <SessionPanel onNewSession={handleNewSession} />

      <div className="flex min-w-0 flex-1 flex-col">
        {activeSession ? (
          <>
            <PanelGroup orientation="horizontal" id="misakax-chat-explorer">
              <Panel id="chat" defaultSize={explorerOpen ? "70%" : "100%"} minSize="40%">
                <ChatView
                  session={activeSession}
                  onChangeDir={handleChangeWorkingDir}
                  onToggleExplorer={() => setExplorerOpen(true)}
                  explorerOpen={explorerOpen}
                />
              </Panel>
              {explorerOpen && activeSession.working_directory ? (
                <>
                  <PanelResizeHandle className="w-px bg-[color:var(--border-muted)] hover:bg-[color:var(--border-strong)]" />
                  <Panel id="explorer" defaultSize="30%" minSize="18%" maxSize="55%">
                    <WorkspaceExplorer workingDir={activeSession.working_directory} />
                  </Panel>
                </>
              ) : null}
            </PanelGroup>
            <WorkspaceSelector
              open={showWorkspaceSelector}
              onOpenChange={setShowWorkspaceSelector}
              onSelect={handleWorkspaceSwitched}
              initialPath={activeSession.working_directory}
            />
          </>
        ) : (
          <WelcomeView
            showWorkspaceSelector={showWorkspaceSelector}
            setShowWorkspaceSelector={setShowWorkspaceSelector}
            onNewSession={handleNewSession}
            onWorkspaceSelected={handleWorkspaceSelected}
            t={t}
          />
        )}
      </div>
    </div>
  );
}

function WelcomeView({
  showWorkspaceSelector,
  setShowWorkspaceSelector,
  onNewSession,
  onWorkspaceSelected,
  t,
}: {
  showWorkspaceSelector: boolean;
  setShowWorkspaceSelector: (show: boolean) => void;
  onNewSession: () => void;
  onWorkspaceSelected: (path: string | null) => Promise<void>;
  t: ReturnType<typeof useTranslation>["t"];
}) {
  return (
    <>
      <div className="flex h-full flex-col items-center justify-center gap-6 px-4 text-center">
        <div className="flex size-16 items-center justify-center rounded-[var(--radius-ui-xl)] border border-[color:var(--border-subtle)] bg-[color:var(--surface-card)] shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]">
          <MessageSquare className="size-8 text-muted-foreground/60" strokeWidth={1.15} />
        </div>
        <div className="max-w-md space-y-2">
          <h2 className="text-lg font-semibold tracking-tight text-foreground">
            {t("common:appName")}
          </h2>
          <p className="text-sm leading-relaxed text-muted-foreground">
            {t("workspace:description")}
          </p>
        </div>
        <Button onClick={onNewSession} className="mt-2 gap-2" size="lg">
          <Plus className="h-4 w-4" />
          {t("workspace:selectWorkingDir")}
        </Button>
      </div>
      <WorkspaceSelector
        open={showWorkspaceSelector}
        onOpenChange={setShowWorkspaceSelector}
        onSelect={onWorkspaceSelected}
      />
    </>
  );
}
