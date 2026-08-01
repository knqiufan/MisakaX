import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Group as PanelGroup,
  Panel,
  Separator as PanelResizeHandle,
} from "react-resizable-panels";
import { WorkspaceSelector } from "@/components/chat/workspace/WorkspaceSelector";
import { ChatView } from "@/components/chat/ChatView";
import { NewChatWelcome } from "@/components/chat/NewChatWelcome";
import { WorkspacePanel } from "@/components/chat/workspace/WorkspacePanel";
import {
  PANEL_RESIZE_HANDLE_CLASS,
  PANEL_RESIZE_HANDLE_LINE_CLASS,
} from "@/components/chat/workspace/panelResizeHandle";
import {
  resolveWorkspacePanelPresentation,
  WORKSPACE_PANEL_LAYOUT,
} from "@/components/chat/workspace/workspacePanelLayout";
import { useChatStore } from "@/stores/chat-store";
import {
  useWorkspacePanelStore,
  type WorkspacePanelMode,
} from "@/stores/workspace-panel-store";
import { sessionsIpc } from "@/lib/ipc";
import { FEATURE_FLAGS } from "@/lib/feature-flags";
import { useWorkspaceContext } from "@/hooks/use-workspace-context";
import { resolveWorkspacePanelShortcut } from "@/components/chat/workspace/workspacePanelShortcuts";

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
  const panelOpen = useWorkspacePanelStore((state) => state.open);
  const panelMode = useWorkspacePanelStore((state) => state.mode);
  const panelSize = useWorkspacePanelStore((state) => state.size);
  const bindPanelSession = useWorkspacePanelStore((state) => state.bindSession);
  const togglePanel = useWorkspacePanelStore((state) => state.toggle);
  const closePanel = useWorkspacePanelStore((state) => state.close);
  const setPanelSize = useWorkspacePanelStore((state) => state.setSize);
  const [viewportWidth, setViewportWidth] = useState(() =>
    typeof window === "undefined" ? 1280 : window.innerWidth,
  );
  const workspaceContext = useWorkspaceContext(
    activeSession?.id,
    activeSession?.working_directory,
  );

  const workspaceIdentity = activeSession?.working_directory
    ? `${activeSession.id}\u0000${activeSession.working_directory}`
    : null;
  const workspaceGeneration =
    workspaceContext.identity === workspaceIdentity
      ? (workspaceContext.context?.generation ?? 0)
      : 0;
  const terminalEnabled = FEATURE_FLAGS.workspaceTerminal;
  const effectivePanelMode: WorkspacePanelMode =
    terminalEnabled || panelMode !== "terminal" ? panelMode : "explorer";
  const panelPresentation = resolveWorkspacePanelPresentation(viewportWidth);

  useEffect(() => {
    bindPanelSession(activeSession?.id ?? null, workspaceGeneration);
  }, [activeSession?.id, bindPanelSession, workspaceGeneration]);

  useEffect(() => {
    const handleResize = () => setViewportWidth(window.innerWidth);
    handleResize();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  const handleTogglePanel = useCallback(
    (mode: WorkspacePanelMode) => {
      if (!activeSession?.working_directory) return;
      if (mode === "terminal" && !terminalEnabled) return;
      togglePanel(mode);
    },
    [activeSession?.working_directory, terminalEnabled, togglePanel],
  );

  useEffect(() => {
    const handleShortcut = (event: KeyboardEvent) => {
      if (event.defaultPrevented || !activeSession?.working_directory) return;
      const shortcut = resolveWorkspacePanelShortcut(event, {
        terminalEnabled,
        panelOpen,
        presentation: panelPresentation,
      });
      if (!shortcut) return;
      event.preventDefault();
      if (shortcut === "close") closePanel();
      else handleTogglePanel(shortcut);
    };
    window.addEventListener("keydown", handleShortcut);
    return () => window.removeEventListener("keydown", handleShortcut);
  }, [
    activeSession?.working_directory,
    closePanel,
    handleTogglePanel,
    panelOpen,
    panelPresentation,
    terminalEnabled,
  ]);

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

  const chatView = useMemo(
    () =>
      activeSession ? (
        <ChatView
          session={activeSession}
          onChangeDir={handleChangeWorkingDir}
          onTogglePanel={handleTogglePanel}
          panelOpen={panelOpen}
          panelMode={effectivePanelMode}
          terminalEnabled={terminalEnabled}
        />
      ) : null,
    [
      activeSession,
      effectivePanelMode,
      handleChangeWorkingDir,
      handleTogglePanel,
      panelOpen,
      terminalEnabled,
    ],
  );

  const showPanel =
    panelOpen && Boolean(activeSession?.working_directory) && Boolean(chatView);
  const workspacePanel = activeSession?.working_directory ? (
    <WorkspacePanel
      mode={effectivePanelMode}
      chatSessionId={activeSession.id}
      workspaceGeneration={workspaceGeneration}
      workingDir={activeSession.working_directory}
      onClose={closePanel}
    />
  ) : null;

  return (
    <div className="flex h-full min-w-0 flex-col">
      {!activeSession ? (
        <NewChatWelcome
          onSelectWorkspace={() => openWorkspaceSelector("new-session")}
        />
      ) : (
        <div className="relative flex h-full min-w-0 flex-col">
          {panelPresentation === "split" ? (
            <PanelGroup
              orientation="horizontal"
              id="misakax-chat-workspace"
              onLayoutChanged={(layout) => {
                if (typeof layout.workspace === "number") {
                  setPanelSize(layout.workspace);
                }
              }}
            >
              <Panel
                id="chat"
                defaultSize={`${100 - panelSize}%`}
                minSize={WORKSPACE_PANEL_LAYOUT.chatMinSize}
              >
                {chatView}
              </Panel>
              {showPanel ? (
                <>
                  <PanelResizeHandle className={PANEL_RESIZE_HANDLE_CLASS}>
                    <span className={PANEL_RESIZE_HANDLE_LINE_CLASS} />
                  </PanelResizeHandle>
                  <Panel
                    id="workspace"
                    defaultSize={`${panelSize}%`}
                    minSize={`${WORKSPACE_PANEL_LAYOUT.panelMinSize}%`}
                    maxSize={`${WORKSPACE_PANEL_LAYOUT.panelMaxSize}%`}
                  >
                    {workspacePanel}
                  </Panel>
                </>
              ) : null}
            </PanelGroup>
          ) : (
            <>
              <div className="h-full min-w-0">{chatView}</div>
              {showPanel ? (
                <div
                  className="absolute inset-y-0 right-0 z-30 w-[min(88%,520px)] border-l border-border/40 bg-background shadow-xl"
                  data-testid="workspace-panel-overlay"
                >
                  {workspacePanel}
                </div>
              ) : null}
            </>
          )}
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
