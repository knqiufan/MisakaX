import { useCallback, useState } from "react";
import { ChevronRight, Copy, File, Folder, FolderOpen, FolderSearch } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { fsIpc, type FsEntry } from "@/lib/ipc";
import { cn } from "@/lib/utils";

interface FileTreeNodeProps {
  entry: FsEntry;
  depth: number;
  workingDir: string;
  onOpenFile: (path: string) => void;
}

export function FileTreeNode({
  entry,
  depth,
  workingDir,
  onOpenFile,
}: FileTreeNodeProps) {
  const { t } = useTranslation("workspace");
  const [expanded, setExpanded] = useState(false);
  const [children, setChildren] = useState<FsEntry[]>([]);
  const [loaded, setLoaded] = useState(false);

  const handleClick = useCallback(async () => {
    if (!entry.is_dir) {
      onOpenFile(entry.path);
      return;
    }
    const nextExpanded = !expanded;
    setExpanded(nextExpanded);
    if (loaded || !nextExpanded) return;
    try {
      const list = await fsIpc.listDir(workingDir, entry.path);
      setChildren(list);
      setLoaded(true);
    } catch (error) {
      console.error("Failed to load directory:", error);
    }
  }, [entry, expanded, loaded, onOpenFile, workingDir]);

  const handleReveal = useCallback(async () => {
    try {
      await fsIpc.revealInExplorer(workingDir, entry.path);
    } catch (error) {
      console.error("Failed to reveal in explorer:", error);
      toast.error(t("explorer.openInExplorerFailed"));
    }
  }, [entry.path, workingDir, t]);

  const handleCopyPath = useCallback(async () => {
    try {
      await writeText(entry.path);
      toast.success(t("explorer.pathCopied"));
    } catch (error) {
      console.error("Failed to copy path:", error);
    }
  }, [entry.path, t]);

  return (
    <div>
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <button
            type="button"
            onClick={handleClick}
            className={cn(
              "flex h-7 w-full items-center gap-1 rounded-sm pr-2 text-left text-xs",
              "text-muted-foreground transition-colors duration-[var(--ds-dur-fast)]",
              "hover:bg-[color:var(--surface-hover)] hover:text-foreground",
              "data-[state=open]:bg-[color:var(--surface-hover)] data-[state=open]:text-foreground"
            )}
            style={{ paddingLeft: 8 + depth * 14 }}
          >
            <NodeIcons entry={entry} expanded={expanded} />
            <span className="min-w-0 flex-1 truncate">{entry.name}</span>
          </button>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem onSelect={handleReveal}>
            <FolderSearch className="size-4" />
            {t("explorer.openInExplorer")}
          </ContextMenuItem>
          <ContextMenuSeparator />
          <ContextMenuItem onSelect={handleCopyPath}>
            <Copy className="size-4" />
            {t("explorer.copyPath")}
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
      {expanded
        ? children.map((child) => (
            <FileTreeNode
              key={child.path}
              entry={child}
              depth={depth + 1}
              workingDir={workingDir}
              onOpenFile={onOpenFile}
            />
          ))
        : null}
    </div>
  );
}

function NodeIcons({ entry, expanded }: { entry: FsEntry; expanded: boolean }) {
  if (!entry.is_dir) {
    return (
      <>
        <span className="size-3" />
        <File className="size-3.5" />
      </>
    );
  }
  return (
    <>
      <ChevronRight
        className={cn("size-3 transition-transform", expanded && "rotate-90")}
      />
      {expanded ? <FolderOpen className="size-3.5" /> : <Folder className="size-3.5" />}
    </>
  );
}
