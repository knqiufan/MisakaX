import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { MessageSquare, Plus } from "lucide-react";
import {
  Group as PanelGroup,
  Panel,
  Separator as PanelResizeHandle,
} from "react-resizable-panels";
import { Button } from "@/components/ui/button";
import { WorkspaceSelector } from "@/components/chat/workspace/WorkspaceSelector";
import { ChatView } from "@/components/chat/ChatView";
import { SessionPanel } from "@/components/chat/session/SessionPanel";
import { WorkspaceExplorer } from "@/components/chat/workspace/WorkspaceExplorer";
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
    <PanelGroup
      orientation="horizontal"
      id="misakax-shell"
      className="flex h-full"
    >
      <Panel id="sessions" defaultSize="18%" minSize="14%" maxSize="32%">
        <SessionPanel onNewSession={handleNewSession} />
      </Panel>
      <PanelResizeHandle className="w-[2px] bg-[color:var(--border-muted)] hover:bg-[color:var(--border-strong)] transition-colors duration-[var(--ds-dur-fast)]" />
      <Panel id="main" minSize="50%">
        <MainArea
          activeSession={activeSession}
          explorerOpen={explorerOpen}
          showWorkspaceSelector={showWorkspaceSelector}
          setShowWorkspaceSelector={setShowWorkspaceSelector}
          setExplorerOpen={setExplorerOpen}
          onChangeWorkingDir={handleChangeWorkingDir}
          onWorkspaceSwitched={handleWorkspaceSwitched}
          onNewSession={handleNewSession}
          onWorkspaceSelected={handleWorkspaceSelected}
          t={t}
        />
      </Panel>
    </PanelGroup>
  );
}

interface MainAreaProps {
  activeSession: ReturnType<typeof useChatStore.getState>["activeSession"];
  explorerOpen: boolean;
  showWorkspaceSelector: boolean;
  setShowWorkspaceSelector: (show: boolean) => void;
  setExplorerOpen: (open: boolean) => void;
  onChangeWorkingDir: () => void;
  onWorkspaceSwitched: (path: string | null) => Promise<void>;
  onNewSession: () => void;
  onWorkspaceSelected: (path: string | null) => Promise<void>;
  t: ReturnType<typeof useTranslation>["t"];
}

function MainArea({
  activeSession,
  explorerOpen,
  showWorkspaceSelector,
  setShowWorkspaceSelector,
  setExplorerOpen,
  onChangeWorkingDir,
  onWorkspaceSwitched,
  onNewSession,
  onWorkspaceSelected,
  t,
}: MainAreaProps) {
  if (!activeSession) {
    return (
      <WelcomeView
        showWorkspaceSelector={showWorkspaceSelector}
        setShowWorkspaceSelector={setShowWorkspaceSelector}
        onNewSession={onNewSession}
        onWorkspaceSelected={onWorkspaceSelected}
        t={t}
      />
    );
  }

  return (
    <div className="flex h-full min-w-0 flex-col">
      <PanelGroup orientation="horizontal" id="misakax-chat-explorer">
        <Panel id="chat" defaultSize="70%" minSize="55%">
          <ChatView
            session={activeSession}
            onChangeDir={onChangeWorkingDir}
            onToggleExplorer={() => setExplorerOpen(true)}
            explorerOpen={explorerOpen}
          />
        </Panel>
        {explorerOpen && activeSession.working_directory ? (
          <>
            <PanelResizeHandle className="w-[2px] bg-[color:var(--border-muted)] hover:bg-[color:var(--border-strong)] transition-colors duration-[var(--ds-dur-fast)]" />
            <Panel id="explorer" defaultSize="30%" minSize="18%" maxSize="55%">
              <WorkspaceExplorer workingDir={activeSession.working_directory} />
            </Panel>
          </>
        ) : null}
      </PanelGroup>
      <WorkspaceSelector
        open={showWorkspaceSelector}
        onOpenChange={setShowWorkspaceSelector}
        onSelect={onWorkspaceSwitched}
        initialPath={activeSession.working_directory}
      />
    </div>
  );
}

interface WelcomeViewProps {
  showWorkspaceSelector: boolean;
  setShowWorkspaceSelector: (show: boolean) => void;
  onNewSession: () => void;
  onWorkspaceSelected: (path: string | null) => Promise<void>;
  t: ReturnType<typeof useTranslation>["t"];
}

function WelcomeView({
  showWorkspaceSelector,
  setShowWorkspaceSelector,
  onNewSession,
  onWorkspaceSelected,
  t,
}: WelcomeViewProps) {
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
