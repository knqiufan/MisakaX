import { FileCode2, Save } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { toWorkspaceRelativePath } from "../composer/attachmentUtils";
import type { OpenTab } from "@/stores/workspace-explorer-store";

interface EditorFileInfoProps {
  workingDir: string;
  tab: OpenTab | null;
  onSave: (path: string) => Promise<void>;
}

/** Preview-style file info row under Tab bar (docs/ui/03-workspace.md §6.1). */
export function EditorFileInfo({
  workingDir,
  tab,
  onSave,
}: EditorFileInfoProps) {
  const { t } = useTranslation("workspace");

  if (!tab) {
    return (
      <div className="flex h-12 shrink-0 items-center border-b border-border/40 bg-background px-3 pb-1">
        <span className="text-xs text-muted-foreground">
          {t("explorer.noFileOpen")}
        </span>
      </div>
    );
  }

  const name = fileName(tab.path);
  const relativePath = toWorkspaceRelativePath(workingDir, tab.path);
  const language = detectLanguageLabel(tab.path);

  return (
    <div className="flex h-12 shrink-0 items-center gap-1 border-b border-border/40 bg-background px-3 pb-1">
      <div className="min-w-0 flex-1 space-y-0.5">
        <span className="block truncate text-xs font-medium text-foreground">
          {name}
        </span>
        <div className="flex min-w-0 items-center gap-1.5">
          <span
            className="max-w-[260px] truncate font-mono text-[10px] text-muted-foreground/60"
            title={relativePath}
          >
            {relativePath}
          </span>
          {language ? (
            <>
              <span className="text-[10px] text-muted-foreground/40" aria-hidden>
                ·
              </span>
              <span className="text-[10px] text-muted-foreground/60">
                {language}
              </span>
            </>
          ) : null}
        </div>
      </div>
      {tab.dirty ? (
        <span
          aria-hidden
          className="size-1.5 shrink-0 rounded-full bg-[color:var(--status-warning)]"
        />
      ) : null}
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className={cn(
          "size-7 text-muted-foreground/80",
          "hover:bg-muted/50 hover:text-foreground"
        )}
        disabled={!tab.dirty}
        aria-label={t("explorer.saveAction")}
        title={t("explorer.saveAction")}
        onClick={() => onSave(tab.path)}
      >
        <Save className="size-3.5" />
      </Button>
      <FileCode2
        className="size-3.5 shrink-0 text-muted-foreground/50"
        aria-hidden
      />
    </div>
  );
}

function fileName(path: string): string {
  return path.replace(/\\/g, "/").split("/").pop() ?? path;
}

function detectLanguageLabel(path: string): string | null {
  const ext = path.split(".").pop()?.toLowerCase();
  if (!ext) return null;
  return ext;
}
