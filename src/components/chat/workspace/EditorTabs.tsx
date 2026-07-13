import { useMemo, useState } from "react";
import { File, X } from "lucide-react";
import { useTranslation } from "react-i18next";
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
import {
  useWorkspaceExplorerStore,
  type OpenTab,
} from "@/stores/workspace-explorer-store";

interface EditorTabsProps {
  onSave: (path: string) => Promise<void>;
}

export function EditorTabs({ onSave }: EditorTabsProps) {
  const { t } = useTranslation("workspace");
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
      <div className="flex shrink-0 items-center bg-transparent px-2 pb-3 pt-1.5">
        <div
          role="tablist"
          className="flex min-w-0 flex-1 gap-0.5 overflow-x-auto"
        >
          {tabs.map((tab) => (
            <EditorTab
              key={tab.path}
              tab={tab}
              active={tab.path === activePath}
              onActivate={() => setActive(tab.path)}
              onClose={() => requestClose(tab)}
              closeLabel={t("explorer.closeTab", { name: fileName(tab.path) })}
            />
          ))}
        </div>
        {activeName ? (
          <span className="sr-only">
            {t("explorer.activeFile", { name: activeName })}
          </span>
        ) : null}
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

interface EditorTabProps {
  tab: OpenTab;
  active: boolean;
  onActivate: () => void;
  onClose: () => void;
  closeLabel: string;
}

function EditorTab({
  tab,
  active,
  onActivate,
  onClose,
  closeLabel,
}: EditorTabProps) {
  return (
    <div
      className={cn(
        "group flex min-w-[40px] max-w-[160px] flex-1 items-center rounded-full text-sm transition-colors duration-[var(--ds-dur-fast)] ease-out",
        active
          ? "bg-muted text-foreground"
          : "text-muted-foreground hover:bg-muted/50 hover:text-foreground"
      )}
    >
      <button
        type="button"
        role="tab"
        aria-selected={active}
        onClick={onActivate}
        title={tab.path}
        className={cn(
          "flex min-w-0 flex-1 items-center gap-1.5 rounded-full py-2 pl-3 pr-1",
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
        )}
      >
        <span className="relative size-4 shrink-0">
          <File
            className={cn(
              "absolute inset-0 size-4 text-inherit transition-opacity duration-[var(--ds-dur-fast)]",
              "opacity-100 group-hover:opacity-0"
            )}
            aria-hidden
          />
          <span
            role="button"
            tabIndex={0}
            aria-label={closeLabel}
            title={closeLabel}
            onClick={(event) => {
              event.stopPropagation();
              onClose();
            }}
            onKeyDown={(event) => {
              if (event.key !== "Enter" && event.key !== " ") return;
              event.preventDefault();
              event.stopPropagation();
              onClose();
            }}
            className={cn(
              "absolute inset-0 flex items-center justify-center rounded-sm",
              "opacity-0 transition-opacity duration-[var(--ds-dur-fast)]",
              "group-hover:opacity-100 hover:bg-muted"
            )}
          >
            <X className="size-3.5" />
          </span>
        </span>
        <span className="min-w-0 flex-1 truncate text-left text-xs font-medium">
          {fileName(tab.path)}
        </span>
        {tab.dirty ? (
          <span
            aria-hidden
            className="size-1.5 shrink-0 rounded-full bg-[color:var(--status-warning)]"
          />
        ) : null}
      </button>
    </div>
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
  const { t } = useTranslation("workspace");
  return (
    <Dialog open={Boolean(tab)} onOpenChange={(open) => !open && onCancel()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("explorer.unsavedTitle")}</DialogTitle>
          <DialogDescription>
            {tab
              ? t("explorer.unsavedDescription", { name: fileName(tab.path) })
              : ""}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={onCancel}>
            {t("explorer.cancelAction")}
          </Button>
          <Button type="button" variant="outline" onClick={onDiscard}>
            {t("explorer.discardAction")}
          </Button>
          <Button type="button" onClick={onSave}>
            {t("explorer.saveAction")}
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
