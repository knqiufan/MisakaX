import { PanelRightClose } from "lucide-react";
import { useTranslation } from "react-i18next";
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

/**
 * 工作区面板。设计语义：
 * 1. **无打开 tab 时**：仅渲染文件树，占满整个面板高度，避免下方出现「空 tab + 占位编辑器」的视觉冗余。
 * 2. **有打开 tab 时**：上半部为文件树，下半部为 Tabs + Monaco 编辑器，
 *    使用 react-resizable-panels 提供垂直可拖拽分栏。
 * 3. **关闭最后一个 tab** 后自动回到状态 1（由 `tabs.length === 0` 驱动）。
 */
export function WorkspaceExplorer({ workingDir }: WorkspaceExplorerProps) {
  const { t } = useTranslation("workspace");
  const { setOpen, openTab, tabs } = useWorkspaceExplorerStore();
  const { activeTab, handleChange, handleSave } = useFileEditor(workingDir);

  const openFile = async (path: string) => {
    try {
      const content = await fsIpc.readTextFile(workingDir, path);
      openTab(path, content);
    } catch (error) {
      console.error("Failed to open file:", error);
      toast.error(t("explorer.openFileFailed"));
    }
  };

  const hasOpenTabs = tabs.length > 0;

  return (
    <aside
      className={
        "flex h-full min-w-0 flex-col border-l border-[color:var(--border-muted)] " +
        "bg-[color:var(--surface-sidebar)] " +
        "animate-in fade-in slide-in-from-right-4 ease-out duration-[220ms]"
      }
    >
      <ExplorerHeader
        title={t("explorer.title")}
        collapseLabel={t("explorer.collapse")}
        onCollapse={() => setOpen(false)}
      />
      {hasOpenTabs ? (
        <ExplorerSplitLayout
          workingDir={workingDir}
          activeTab={activeTab}
          onOpenFile={openFile}
          onChange={handleChange}
          onSave={handleSave}
        />
      ) : (
        <FileTreeView workingDir={workingDir} onOpenFile={openFile} />
      )}
    </aside>
  );
}

function ExplorerHeader({
  title,
  collapseLabel,
  onCollapse,
}: {
  title: string;
  collapseLabel: string;
  onCollapse: () => void;
}) {
  return (
    <div className="flex h-10 shrink-0 items-center justify-between border-b border-[color:var(--border-muted)] px-3">
      <h2 className="text-xs font-semibold uppercase tracking-[0.12em] text-muted-foreground">
        {title}
      </h2>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        onClick={onCollapse}
        className="size-7 rounded-[var(--radius-ui-sm)]"
        aria-label={collapseLabel}
      >
        <PanelRightClose className="size-4" />
      </Button>
    </div>
  );
}

interface ExplorerSplitLayoutProps {
  workingDir: string;
  activeTab: ReturnType<typeof useFileEditor>["activeTab"];
  onOpenFile: (path: string) => void;
  onChange: ReturnType<typeof useFileEditor>["handleChange"];
  onSave: ReturnType<typeof useFileEditor>["handleSave"];
}

function ExplorerSplitLayout({
  workingDir,
  activeTab,
  onOpenFile,
  onChange,
  onSave,
}: ExplorerSplitLayoutProps) {
  return (
    <PanelGroup orientation="vertical" id="misakax-workspace-explorer">
      <Panel id="tree" defaultSize="34%" minSize="20%">
        <FileTreeView workingDir={workingDir} onOpenFile={onOpenFile} />
      </Panel>
      <PanelResizeHandle className="h-px bg-[color:var(--border-muted)] hover:bg-[color:var(--border-strong)]" />
      <Panel id="editor" minSize="30%">
        <div className="flex h-full min-h-0 flex-col bg-[color:var(--surface-card)]">
          <EditorTabs onSave={onSave} />
          <EditorPane tab={activeTab} onChange={onChange} />
        </div>
      </Panel>
    </PanelGroup>
  );
}
