import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { ScrollArea } from "@/components/ui/scroll-area";
import { fsIpc, type FsEntry } from "@/lib/ipc";
import { FileTreeNode } from "./FileTreeNode";

interface FileTreeViewProps {
  workingDir: string;
  onOpenFile: (path: string) => void;
  refreshKey?: number;
  onLoadingChange?: (loading: boolean) => void;
}

export function FileTreeView({
  workingDir,
  onOpenFile,
  refreshKey = 0,
  onLoadingChange,
}: FileTreeViewProps) {
  const { t } = useTranslation("workspace");
  const [entries, setEntries] = useState<FsEntry[]>([]);
  const [loading, setLoading] = useState(false);

  const loadRoot = useCallback(async () => {
    setLoading(true);
    onLoadingChange?.(true);
    try {
      setEntries(await fsIpc.listDir(workingDir, workingDir));
    } catch (error) {
      console.error("Failed to load workspace tree:", error);
      toast.error(t("explorer.loadTreeFailed"));
    } finally {
      setLoading(false);
      onLoadingChange?.(false);
    }
  }, [workingDir, t, onLoadingChange]);

  useEffect(() => {
    void loadRoot();
  }, [loadRoot, refreshKey]);

  return (
    <section className="flex min-h-0 flex-1 flex-col">
      <ScrollArea className="min-h-0 flex-1">
        <div className="py-1" aria-busy={loading}>
          {entries.map((entry) => (
            <FileTreeNode
              key={entry.path}
              entry={entry}
              depth={0}
              workingDir={workingDir}
              onOpenFile={onOpenFile}
            />
          ))}
        </div>
      </ScrollArea>
    </section>
  );
}
