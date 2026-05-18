import { useCallback, useEffect, useState } from "react";
import { RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { fsIpc, type FsEntry } from "@/lib/ipc";
import { FileTreeNode } from "./FileTreeNode";

interface FileTreeViewProps {
  workingDir: string;
  onOpenFile: (path: string) => void;
}

export function FileTreeView({ workingDir, onOpenFile }: FileTreeViewProps) {
  const { t } = useTranslation("workspace");
  const [entries, setEntries] = useState<FsEntry[]>([]);
  const [loading, setLoading] = useState(false);

  const loadRoot = useCallback(async () => {
    setLoading(true);
    try {
      setEntries(await fsIpc.listDir(workingDir, workingDir));
    } catch (error) {
      console.error("Failed to load workspace tree:", error);
      toast.error(t("explorer.loadTreeFailed"));
    } finally {
      setLoading(false);
    }
  }, [workingDir, t]);

  useEffect(() => {
    void loadRoot();
  }, [loadRoot]);

  return (
    <section className="flex min-h-0 flex-1 flex-col">
      <div className="flex h-9 items-center justify-between border-b border-[color:var(--border-muted)] px-3">
        <span className="text-xs font-medium text-muted-foreground">
          {t("explorer.treeTitle")}
        </span>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          onClick={loadRoot}
          disabled={loading}
          className="size-7 rounded-[var(--radius-ui-sm)]"
          aria-label={t("explorer.refreshTree")}
        >
          <RefreshCw className={`size-3.5 ${loading ? "animate-spin" : ""}`} />
        </Button>
      </div>
      <ScrollArea className="min-h-0 flex-1">
        <div className="py-1">
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
