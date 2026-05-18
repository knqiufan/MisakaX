import { PanelRightClose } from "lucide-react";
import { toast } from "sonner";
import {
  Group as PanelGroup,
  Panel,
  Separator as PanelResizeHandle,
} from "react-resizable-panels";
import { Button } from "@/components/ui/button";
import { fsIpc } from "@/lib/ipc";
import { useWorkspaceExplorerStore } from "@/stores/workspace-explorer-store";
import { FileTreeView } from "./FileTreeView";
import { EditorTabs } from "./EditorTabs";
import { EditorPane } from "./EditorPane";
import { useFileEditor } from "./useFileEditor";

interface WorkspaceExplorerProps {
  workingDir: string;
}

export function WorkspaceExplorer({ workingDir }: WorkspaceExplorerProps) {
  const { setOpen, openTab } = useWorkspaceExplorerStore();
  const { activeTab, handleChange, handleSave } = useFileEditor(workingDir);

  const openFile = async (path: string) => {
    try {
      const content = await fsIpc.readTextFile(workingDir, path);
      openTab(path, content);
    } catch (error) {
      console.error("Failed to open file:", error);
      toast.error("无法打开文件");
    }
  };

  return (
    <aside className="flex h-full min-w-0 flex-col border-l border-[color:var(--border-muted)] bg-[color:var(--surface-sidebar)]">
      <div className="flex h-10 shrink-0 items-center justify-between border-b border-[color:var(--border-muted)] px-3">
        <h2 className="text-xs font-semibold uppercase tracking-[0.12em] text-muted-foreground">
          Workspace
        </h2>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          onClick={() => setOpen(false)}
          className="size-7 rounded-[var(--radius-ui-sm)]"
          aria-label="Collapse workspace explorer"
        >
          <PanelRightClose className="size-4" />
        </Button>
      </div>
      <PanelGroup orientation="vertical" id="misakax-workspace-explorer">
        <Panel id="tree" defaultSize="34%" minSize="20%">
          <FileTreeView workingDir={workingDir} onOpenFile={openFile} />
        </Panel>
        <PanelResizeHandle className="h-px bg-[color:var(--border-muted)] hover:bg-[color:var(--border-strong)]" />
        <Panel id="editor" minSize="30%">
          <div className="flex h-full min-h-0 flex-col">
            <EditorTabs onSave={handleSave} />
            <EditorPane tab={activeTab} onChange={handleChange} />
          </div>
        </Panel>
      </PanelGroup>
    </aside>
  );
}
