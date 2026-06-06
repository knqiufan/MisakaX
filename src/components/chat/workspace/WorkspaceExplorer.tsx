import { useCallback, useState } from "react";
import { PanelRightClose, RefreshCw } from "lucide-react";
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
import { EditorColumn } from "./EditorColumn";
import { useFileEditor } from "./useFileEditor";

interface WorkspaceExplorerProps {
  workingDir: string;
}

/**
 * 工作区面板。设计语义：
 * 1. **无打开 tab 时**：仅渲染文件树，占满整个面板高度。
 * 2. **有打开 tab 时**：左文件树、右 Tabs + Monaco，水平可拖拽分栏。
 * 3. **关闭最后一个 tab** 后自动回到状态 1。
 */
export function WorkspaceExplorer({ workingDir }: WorkspaceExplorerProps) {
  const { t } = useTranslation("workspace");
  const { setOpen, openTab, tabs } = useWorkspaceExplorerStore();
  const { activeTab, handleChange, handleSave } = useFileEditor(workingDir);
  const [treeRefreshKey, setTreeRefreshKey] = useState(0);
  const [treeLoading, setTreeLoading] = useState(false);

  const handleTreeLoadingChange = useCallback((loading: boolean) => {
    setTreeLoading(loading);
  }, []);

  const refreshTree = useCallback(() => {
    setTreeRefreshKey((key) => key + 1);
  }, []);

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
  const treeProps = {
    workingDir,
    onOpenFile: openFile,
    refreshKey: treeRefreshKey,
    onLoadingChange: handleTreeLoadingChange,
  };

  const workspaceContent = hasOpenTabs ? (
    <ExplorerSplitLayout
      workingDir={workingDir}
      activeTab={activeTab}
      treeProps={treeProps}
      onChange={handleChange}
      onSave={handleSave}
    />
  ) : (
    <FileTreeView {...treeProps} />
  );

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
        refreshLabel={t("explorer.refreshTree")}
        collapseLabel={t("explorer.collapse")}
        treeLoading={treeLoading}
        onRefresh={refreshTree}
        onCollapse={() => setOpen(false)}
      />
      <div className="flex min-h-0 flex-1 flex-col">{workspaceContent}</div>
    </aside>
  );
}

function ExplorerHeader({
  title,
  refreshLabel,
  collapseLabel,
  treeLoading,
  onRefresh,
  onCollapse,
}: {
  title: string;
  refreshLabel: string;
  collapseLabel: string;
  treeLoading: boolean;
  onRefresh: () => void;
  onCollapse: () => void;
}) {
  return (
    <div className="flex h-10 shrink-0 items-center justify-between border-b border-[color:var(--border-muted)] px-3">
      <h2 className="text-xs font-semibold uppercase tracking-[0.12em] text-muted-foreground">
        {title}
      </h2>
      <div className="flex items-center gap-0.5">
        <Button
          type="button"
          variant="ghost"
          size="icon"
          onClick={onRefresh}
          disabled={treeLoading}
          className="size-7 rounded-[var(--radius-ui-sm)]"
          aria-label={refreshLabel}
        >
          <RefreshCw className={`size-3.5 ${treeLoading ? "animate-spin" : ""}`} />
        </Button>
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
    </div>
  );
}

interface ExplorerSplitLayoutProps {
  workingDir: string;
  activeTab: ReturnType<typeof useFileEditor>["activeTab"];
  treeProps: {
    workingDir: string;
    onOpenFile: (path: string) => void;
    refreshKey: number;
    onLoadingChange: (loading: boolean) => void;
  };
  onChange: ReturnType<typeof useFileEditor>["handleChange"];
  onSave: ReturnType<typeof useFileEditor>["handleSave"];
}

function ExplorerSplitLayout({
  workingDir,
  activeTab,
  treeProps,
  onChange,
  onSave,
}: ExplorerSplitLayoutProps) {
  return (
    <PanelGroup orientation="horizontal" id="misakax-workspace-explorer">
      <Panel id="tree" defaultSize="32%" minSize="22%" maxSize="50%">
        <FileTreeView {...treeProps} />
      </Panel>
      <PanelResizeHandle className="w-px bg-[color:var(--border-muted)] hover:bg-[color:var(--border-strong)] transition-colors duration-[var(--ds-dur-fast)]" />
      <Panel id="editor" minSize="35%">
        <EditorColumn
          workingDir={workingDir}
          activeTab={activeTab}
          onChange={onChange}
          onSave={onSave}
        />
      </Panel>
    </PanelGroup>
  );
}
