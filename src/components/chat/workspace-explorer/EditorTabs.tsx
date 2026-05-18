import { useMemo, useState } from "react";
import { X } from "lucide-react";
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
import { useWorkspaceExplorerStore, type OpenTab } from "@/stores/workspace-explorer-store";

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
      <div
        className={cn(
          "flex h-10 shrink-0 items-center gap-1 overflow-x-auto px-2 pt-1.5",
          "border-b border-[color:var(--border-muted)] bg-[color:var(--surface-topbar)]"
        )}
      >
        {tabs.length === 0 ? (
          <span className="px-2 text-xs text-muted-foreground">
            {t("explorer.noFileOpen")}
          </span>
        ) : (
          tabs.map((tab) => (
            <EditorTab
              key={tab.path}
              tab={tab}
              active={tab.path === activePath}
              onActivate={() => setActive(tab.path)}
              onClose={() => requestClose(tab)}
              closeLabel={t("explorer.closeTab", { name: fileName(tab.path) })}
            />
          ))
        )}
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

function EditorTab({ tab, active, onActivate, onClose, closeLabel }: EditorTabProps) {
  return (
    <button
      type="button"
      onClick={onActivate}
      title={tab.path}
      className={cn(
        "group relative flex h-8 max-w-[200px] shrink-0 items-center gap-1.5 rounded-t-[var(--radius-ui-md)]",
        "border border-b-0 px-2.5 text-xs",
        "transition-colors duration-[var(--ds-dur-fast)] ease-out",
        active
          ? "border-[color:var(--border-muted)] bg-[color:var(--surface-card)] text-foreground"
          : "border-transparent text-muted-foreground hover:bg-[color:var(--surface-hover)] hover:text-foreground"
      )}
    >
      <span className="truncate font-medium">{fileName(tab.path)}</span>
      {tab.dirty ? (
        <span
          aria-hidden
          className="ms-0.5 inline-block size-1.5 rounded-full bg-primary/80"
        />
      ) : null}
      <span
        role="button"
        tabIndex={0}
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
          "ms-1 flex size-4 items-center justify-center rounded-[var(--radius-ui-xs)]",
          "text-muted-foreground/80 transition-colors duration-[var(--ds-dur-fast)]",
          "opacity-0 hover:bg-[color:var(--surface-control-hover)] hover:text-foreground",
          "group-hover:opacity-100",
          active && "opacity-100"
        )}
        aria-label={closeLabel}
      >
        <X className="size-3" />
      </span>
    </button>
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
