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
import {
  PANEL_RESIZE_HANDLE_CLASS,
  PANEL_RESIZE_HANDLE_LINE_CLASS,
} from "@/components/chat/workspace/panelResizeHandle";
import { LEGACY_CHAT_PANEL_LAYOUT } from "@/components/chat/workspace/workspacePanelLayout";
import { useChatStore } from "@/stores/chat-store";
import { useWorkspaceExplorerStore } from "@/stores/workspace-explorer-store";
import { sessionsIpc } from "@/lib/ipc";

export function ChatPage() {
  const activeSession = useChatStore((s) => s.activeSession);
  const showWorkspaceSelector = useChatStore((s) => s.showWorkspaceSelector);
  const workspaceSelectorIntent = useChatStore((s) => s.workspaceSelectorIntent);
  const workspaceSelectorTargetSessionId = useChatStore(
    (s) => s.workspaceSelectorTargetSessionId
  );
  const openWorkspaceSelector = useChatStore((s) => s.openWorkspaceSelector);
  const closeWorkspaceSelector = useChatStore((s) => s.closeWorkspaceSelector);
  const upsertSession = useChatStore((s) => s.upsertSession);
  const updateActiveSessionWorkingDir = useChatStore(
    (s) => s.updateActiveSessionWorkingDir
  );
  const { open: explorerOpen, setOpen: setExplorerOpen } =
    useWorkspaceExplorerStore();

  const handleNewSessionWorkspace = useCallback(
    async (path: string | null) => {
      try {
        const session = await sessionsIpc.create({
          workingDirectory: path ?? undefined,
        });
        upsertSession(session);
        closeWorkspaceSelector();
      } catch (err) {
        console.error("Failed to create session:", err);
      }
    },
    [upsertSession, closeWorkspaceSelector]
  );

  const handleChangeWorkingDir = useCallback(() => {
    if (!activeSession) return;
    openWorkspaceSelector("change-session", activeSession.id);
  }, [activeSession, openWorkspaceSelector]);

  const handleWorkspaceSwitched = useCallback(
    async (path: string | null) => {
      const targetId = workspaceSelectorTargetSessionId;
      if (!targetId) return;
      try {
        await sessionsIpc.updateWorkingDir({
          sessionId: targetId,
          workingDirectory: path ?? undefined,
        });
        const updated = await sessionsIpc.get(targetId);
        if (useChatStore.getState().activeSessionId === targetId) {
          updateActiveSessionWorkingDir(
            updated.working_directory,
            updated.workspace_kind
          );
        }
        closeWorkspaceSelector();
      } catch (err) {
        console.error("Failed to update working directory:", err);
      }
    },
    [
      workspaceSelectorTargetSessionId,
      updateActiveSessionWorkingDir,
      closeWorkspaceSelector,
    ]
  );

  const handleSelectorOpenChange = useCallback(
    (open: boolean) => {
      if (!open) closeWorkspaceSelector();
    },
    [closeWorkspaceSelector]
  );

  const isChangeIntent = workspaceSelectorIntent === "change-session";
  const selectorOnSelect = isChangeIntent
    ? handleWorkspaceSwitched
    : handleNewSessionWorkspace;
  const selectorInitialPath = isChangeIntent
    ? activeSession?.working_directory
    : null;

  return (
    <div className="flex h-full min-w-0 flex-col">
      {!activeSession ? (
        <NewChatWelcome
          onSelectWorkspace={() => openWorkspaceSelector("new-session")}
        />
      ) : (
        <div className="flex h-full min-w-0 flex-col">
          <PanelGroup orientation="horizontal" id="misakax-chat-explorer">
            <Panel
              id="chat"
              defaultSize={LEGACY_CHAT_PANEL_LAYOUT.chatDefaultSize}
              minSize={LEGACY_CHAT_PANEL_LAYOUT.chatMinSize}
            >
              <ChatView
                session={activeSession}
                onChangeDir={handleChangeWorkingDir}
                onToggleExplorer={() => setExplorerOpen(!explorerOpen)}
                explorerOpen={explorerOpen}
              />
            </Panel>
            {explorerOpen && activeSession.working_directory ? (
              <>
                <PanelResizeHandle className={PANEL_RESIZE_HANDLE_CLASS}>
                  <span className={PANEL_RESIZE_HANDLE_LINE_CLASS} />
                </PanelResizeHandle>
                <Panel
                  id="explorer"
                  defaultSize={LEGACY_CHAT_PANEL_LAYOUT.explorerDefaultSize}
                  minSize={LEGACY_CHAT_PANEL_LAYOUT.explorerMinSize}
                  maxSize={LEGACY_CHAT_PANEL_LAYOUT.explorerMaxSize}
                >
                  <WorkspaceExplorer workingDir={activeSession.working_directory} />
                </Panel>
              </>
            ) : null}
          </PanelGroup>
        </div>
      )}
      <WorkspaceSelector
        open={showWorkspaceSelector}
        onOpenChange={handleSelectorOpenChange}
        onSelect={selectorOnSelect}
        initialPath={selectorInitialPath}
      />
    </div>
  );
}
