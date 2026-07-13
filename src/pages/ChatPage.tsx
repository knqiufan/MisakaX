import { useCallback } from "react";
import {
  Group as PanelGroup,
  Panel,
  Separator as PanelResizeHandle,
} from "react-resizable-panels";
import { WorkspaceSelector } from "@/components/chat/workspace/WorkspaceSelector";
import { ChatView } from "@/components/chat/ChatView";
import { NewChatWelcome } from "@/components/chat/NewChatWelcome";
import { WorkspaceExplorer } from "@/components/chat/workspace/WorkspaceExplorer";
import { useChatStore } from "@/stores/chat-store";
import { useWorkspaceExplorerStore } from "@/stores/workspace-explorer-store";
import { sessionsIpc } from "@/lib/ipc";

export function ChatPage() {
  const {
    activeSession,
    showWorkspaceSelector,
    setShowWorkspaceSelector,
    setActiveSessionData,
    updateActiveSessionWorkingDir,
  } = useChatStore();
  const { open: explorerOpen, setOpen: setExplorerOpen } =
    useWorkspaceExplorerStore();

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
    <div className="flex h-full min-w-0 flex-col">
      {!activeSession ? (
        <NewChatWelcome
          onSelectWorkspace={() => setShowWorkspaceSelector(true)}
        />
      ) : (
        <div className="flex h-full min-w-0 flex-col">
          <PanelGroup orientation="horizontal" id="misakax-chat-explorer">
            <Panel id="chat" defaultSize="70%" minSize="55%">
              <ChatView
                session={activeSession}
                onChangeDir={handleChangeWorkingDir}
                onToggleExplorer={() => setExplorerOpen(true)}
                explorerOpen={explorerOpen}
              />
            </Panel>
            {explorerOpen && activeSession.working_directory ? (
              <>
                <PanelResizeHandle className="w-[2px] bg-border transition-colors duration-[var(--ds-dur-fast)] hover:bg-foreground/20" />
                <Panel id="explorer" defaultSize="30%" minSize="18%" maxSize="55%">
                  <WorkspaceExplorer workingDir={activeSession.working_directory} />
                </Panel>
              </>
            ) : null}
          </PanelGroup>
        </div>
      )}
      <WorkspaceSelector
        open={showWorkspaceSelector}
        onOpenChange={setShowWorkspaceSelector}
        onSelect={activeSession ? handleWorkspaceSwitched : handleWorkspaceSelected}
        initialPath={activeSession?.working_directory}
      />
    </div>
  );
}
