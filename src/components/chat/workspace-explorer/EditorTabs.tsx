import { useMemo, useState } from "react";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { cn } from "@/lib/utils";
import { useWorkspaceExplorerStore, type OpenTab } from "@/stores/workspace-explorer-store";

interface EditorTabsProps {
  onSave: (path: string) => Promise<void>;
}

export function EditorTabs({ onSave }: EditorTabsProps) {
  const { tabs, activePath, setActive, closeTab } = useWorkspaceExplorerStore();
  const [pendingClose, setPendingClose] = useState<OpenTab | null>(null);

  const activeName = useMemo(() => fileName(activePath), [activePath]);

  const requestClose = (tab: OpenTab) => {
    if (tab.dirty) {
      setPendingClose(tab);
      return;
    }
    closeTab(tab.path);
  };

  return (
    <>
      <div className="flex h-9 shrink-0 items-center overflow-x-auto border-b border-[color:var(--border-muted)] bg-[color:var(--surface-topbar)]">
        {tabs.length === 0 ? (
          <span className="px-3 text-xs text-muted-foreground">No file open</span>
        ) : (
          tabs.map((tab) => (
            <button
              key={tab.path}
              type="button"
              onClick={() => setActive(tab.path)}
              className={cn(
                "group flex h-full max-w-[180px] items-center gap-2 border-r px-3 text-xs",
                "border-[color:var(--border-muted)] transition-colors duration-[var(--ds-dur-fast)]",
                tab.path === activePath
                  ? "bg-[color:var(--surface-card)] text-foreground"
                  : "text-muted-foreground hover:bg-[color:var(--surface-hover)] hover:text-foreground"
              )}
              title={tab.path}
            >
              <span className="truncate">{fileName(tab.path)}</span>
              {tab.dirty ? <span className="text-sm leading-none">•</span> : null}
              <span
                role="button"
                tabIndex={0}
                onClick={(event) => {
                  event.stopPropagation();
                  requestClose(tab);
                }}
                className="rounded-sm p-0.5 text-muted-foreground hover:bg-[color:var(--surface-hover)] hover:text-foreground"
                aria-label={`Close ${fileName(tab.path)}`}
              >
                <X className="size-3" />
              </span>
            </button>
          ))
        )}
        {activeName ? <span className="sr-only">Active file: {activeName}</span> : null}
      </div>
      <UnsavedDialog
        tab={pendingClose}
        onCancel={() => setPendingClose(null)}
        onDiscard={() => {
          if (pendingClose) closeTab(pendingClose.path);
          setPendingClose(null);
        }}
        onSave={async () => {
          if (!pendingClose) return;
          await onSave(pendingClose.path);
          closeTab(pendingClose.path);
          setPendingClose(null);
        }}
      />
    </>
  );
}

function UnsavedDialog({
  tab,
  onCancel,
  onDiscard,
  onSave,
}: {
  tab: OpenTab | null;
  onCancel: () => void;
  onDiscard: () => void;
  onSave: () => Promise<void>;
}) {
  return (
    <Dialog open={Boolean(tab)} onOpenChange={(open) => !open && onCancel()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>保存更改？</DialogTitle>
          <DialogDescription>
            {tab ? `${fileName(tab.path)} 有未保存的更改。` : ""}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={onCancel}>
            取消
          </Button>
          <Button type="button" variant="outline" onClick={onDiscard}>
            丢弃
          </Button>
          <Button type="button" onClick={onSave}>
            保存
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function fileName(path: string | null): string {
  if (!path) return "";
  return path.replace(/\\/g, "/").split("/").pop() ?? path;
}
