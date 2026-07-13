import { useCallback, useState } from "react";
import { X, RefreshCw } from "lucide-react";
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
        "flex h-full min-w-0 flex-col border-l border-border/40 bg-background " +
        "animate-in fade-in duration-[var(--ds-dur-fast)] ease-out"
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
    <div className="flex h-10 shrink-0 items-center justify-between px-3">
      <h2 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
        {title}
      </h2>
      <div className="flex items-center gap-0.5">
        <Button
          type="button"
          variant="ghost"
          size="icon"
          onClick={onRefresh}
          disabled={treeLoading}
          className="size-7"
          aria-label={refreshLabel}
        >
          <RefreshCw
            className={`size-3.5 ${treeLoading ? "animate-spin" : ""}`}
          />
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          onClick={onCollapse}
          className="size-7"
          aria-label={collapseLabel}
        >
          <X className="size-3.5" />
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
  const { t } = useTranslation("workspace");

  return (
    <PanelGroup orientation="horizontal" id="misakax-workspace-explorer">
      <Panel id="tree" defaultSize="32%" minSize="22%" maxSize="50%">
        <div className="flex h-full min-h-0 flex-col border-r border-border/40">
          <div className="flex h-8 shrink-0 items-center px-3">
            <span className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
              {t("explorer.files")}
            </span>
          </div>
          <div className="min-h-0 flex-1">
            <FileTreeView {...treeProps} />
          </div>
        </div>
      </Panel>
      <PanelResizeHandle className="w-2 bg-transparent transition-colors duration-[var(--ds-dur-fast)] hover:bg-border/60" />
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
