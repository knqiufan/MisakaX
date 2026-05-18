import { useCallback, useState } from "react";
import { ChevronRight, File, Folder, FolderOpen } from "lucide-react";
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

  return (
    <div>
      <button
        type="button"
        onClick={handleClick}
        className={cn(
          "flex h-7 w-full items-center gap-1 rounded-sm pr-2 text-left text-xs",
          "text-muted-foreground transition-colors duration-[var(--ds-dur-fast)]",
          "hover:bg-[color:var(--surface-hover)] hover:text-foreground"
        )}
        style={{ paddingLeft: 8 + depth * 14 }}
      >
        {entry.is_dir ? (
          <ChevronRight
            className={cn("size-3 transition-transform", expanded && "rotate-90")}
          />
        ) : (
          <span className="size-3" />
        )}
        {entry.is_dir ? (
          expanded ? <FolderOpen className="size-3.5" /> : <Folder className="size-3.5" />
        ) : (
          <File className="size-3.5" />
        )}
        <span className="min-w-0 flex-1 truncate">{entry.name}</span>
      </button>
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
