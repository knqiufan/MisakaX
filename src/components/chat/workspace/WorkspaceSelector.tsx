import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  Folder,
  FolderOpen,
  Clock,
  ChevronRight,
  Globe,
  X,
  Check,
  AlertCircle,
} from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { workspaceIpc } from "@/lib/ipc/workspace";
import type { DirectoryInfo, RecentDirectory } from "@/lib/ipc/workspace";

interface WorkspaceSelectorProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelect: (path: string | null) => void;
  initialPath?: string | null;
}

export function WorkspaceSelector({
  open,
  onOpenChange,
  onSelect,
  initialPath,
}: WorkspaceSelectorProps) {
  const { t } = useTranslation("workspace");
  const { t: tc } = useTranslation("common");

  const [recentDirs, setRecentDirs] = useState<RecentDirectory[]>([]);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [validationInfo, setValidationInfo] = useState<DirectoryInfo | null>(
    null
  );
  const [loading, setLoading] = useState(false);
  const [browsing, setBrowsing] = useState(false);

  const loadRecentDirs = useCallback(async () => {
    try {
      const dirs = await workspaceIpc.getRecentDirectories(20);
      setRecentDirs(dirs);
    } catch (err) {
      console.error("Failed to load recent directories:", err);
    }
  }, []);

  useEffect(() => {
    if (open) {
      loadRecentDirs();
      setSelectedPath(initialPath ?? null);
      setValidationInfo(null);
    }
  }, [open, initialPath, loadRecentDirs]);

  useEffect(() => {
    if (!selectedPath) {
      setValidationInfo(null);
      return;
    }
    const abortCtrl = new AbortController();
    workspaceIpc
      .validateDirectory(selectedPath)
      .then((info) => {
        if (!abortCtrl.signal.aborted) setValidationInfo(info);
      })
      .catch(() => {
        if (!abortCtrl.signal.aborted) setValidationInfo(null);
      });
    return () => abortCtrl.abort();
  }, [selectedPath]);

  async function handleBrowse() {
    setBrowsing(true);
    try {
      const picked = await workspaceIpc.browseDirectory(
        selectedPath ?? undefined
      );
      if (picked) {
        setSelectedPath(picked);
      }
    } catch (err) {
      console.error("Browse failed:", err);
    } finally {
      setBrowsing(false);
    }
  }

  function handleSelectRecent(dir: RecentDirectory) {
    setSelectedPath(dir.path);
  }

  async function handleRemoveRecent(
    e: React.MouseEvent,
    dir: RecentDirectory
  ) {
    e.stopPropagation();
    try {
      await workspaceIpc.removeRecentDirectory(dir.path);
      setRecentDirs((prev) => prev.filter((d) => d.path !== dir.path));
      if (selectedPath === dir.path) {
        setSelectedPath(null);
      }
    } catch (err) {
      console.error("Failed to remove directory:", err);
    }
  }

  async function handleConfirm() {
    if (!selectedPath) return;
    setLoading(true);
    try {
      await workspaceIpc.recordDirectoryUsage(
        selectedPath,
        extractDirName(selectedPath)
      );
      onSelect(selectedPath);
      onOpenChange(false);
    } catch (err) {
      console.error("Failed to record usage:", err);
      onSelect(selectedPath);
      onOpenChange(false);
    } finally {
      setLoading(false);
    }
  }

  function handleSkip() {
    onSelect(null);
    onOpenChange(false);
  }

  const isValid =
    validationInfo?.exists && validationInfo?.readable;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("selectWorkingDir")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>

        <div className="flex flex-col gap-3">
          <RecentDirectoryList
            dirs={recentDirs}
            selectedPath={selectedPath}
            onSelect={handleSelectRecent}
            onRemove={handleRemoveRecent}
            t={t}
          />

          <Separator />

          <ActionButtons
            browsing={browsing}
            onBrowse={handleBrowse}
            t={t}
          />

          <Separator />

          <SkipOption onSkip={handleSkip} t={t} />

          {selectedPath && (
            <SelectionStatus
              selectedPath={selectedPath}
              validationInfo={validationInfo}
              t={t}
            />
          )}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
          >
            {tc("cancel")}
          </Button>
          <Button
            onClick={handleConfirm}
            disabled={!selectedPath || loading || !isValid}
          >
            {loading ? tc("loading") : tc("confirm")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

// ─── 子组件 ─────────────────────────────────────────────────────────

function RecentDirectoryList({
  dirs,
  selectedPath,
  onSelect,
  onRemove,
  t,
}: {
  dirs: RecentDirectory[];
  selectedPath: string | null;
  onSelect: (dir: RecentDirectory) => void;
  onRemove: (e: React.MouseEvent, dir: RecentDirectory) => void;
  t: (key: string, opts?: Record<string, unknown>) => string;
}) {
  if (dirs.length === 0) {
    return (
      <div className="flex flex-col items-center gap-1 py-6 text-center text-sm text-muted-foreground">
        <Folder className="h-8 w-8 opacity-40" />
        <p>{t("noRecentDirs")}</p>
        <p className="text-xs">{t("noRecentDirsHint")}</p>
      </div>
    );
  }

  return (
    <div>
      <p className="mb-2 flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
        <Clock className="h-3.5 w-3.5" />
        {t("recentDirs")}
      </p>
      <ScrollArea className="max-h-48">
        <div className="flex flex-col gap-1">
          {dirs.map((dir) => (
            <RecentDirItem
              key={dir.path}
              dir={dir}
              isSelected={selectedPath === dir.path}
              onSelect={onSelect}
              onRemove={onRemove}
              t={t}
            />
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}

function RecentDirItem({
  dir,
  isSelected,
  onSelect,
  onRemove,
  t,
}: {
  dir: RecentDirectory;
  isSelected: boolean;
  onSelect: (dir: RecentDirectory) => void;
  onRemove: (e: React.MouseEvent, dir: RecentDirectory) => void;
  t: (key: string, opts?: Record<string, unknown>) => string;
}) {
  const displayName = dir.display_name || extractDirName(dir.path);
  const timeAgo = formatRelativeTime(dir.last_used_at, t);

  return (
    <button
      type="button"
      onClick={() => onSelect(dir)}
      className={`group flex w-full items-center gap-2 rounded-md px-2.5 py-2 text-left text-sm transition-colors ${
        isSelected
          ? "bg-accent text-accent-foreground"
          : "hover:bg-accent/50"
      }`}
    >
      <FolderOpen className="h-4 w-4 shrink-0 text-muted-foreground" />
      <div className="flex min-w-0 flex-1 flex-col">
        <span className="truncate font-medium">{displayName}</span>
        <span className="truncate text-xs text-muted-foreground">
          {dir.path}
        </span>
      </div>
      <span className="shrink-0 text-xs text-muted-foreground">
        {timeAgo}
      </span>
      {isSelected && (
        <Check className="h-4 w-4 shrink-0 text-primary" />
      )}
      <Tooltip>
        <TooltipTrigger asChild>
          <span
            role="button"
            tabIndex={0}
            onClick={(e) => onRemove(e, dir)}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                onRemove(
                  e as unknown as React.MouseEvent,
                  dir
                );
              }
            }}
            className="shrink-0 rounded-sm p-0.5 opacity-0 transition-opacity hover:bg-destructive/10 hover:text-destructive group-hover:opacity-100"
          >
            <X className="h-3.5 w-3.5" />
          </span>
        </TooltipTrigger>
        <TooltipContent side="left" className="text-xs">
          {t("remove")}
        </TooltipContent>
      </Tooltip>
    </button>
  );
}

function ActionButtons({
  browsing,
  onBrowse,
  t,
}: {
  browsing: boolean;
  onBrowse: () => void;
  t: (key: string) => string;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <Button
        variant="outline"
        className="justify-start gap-2"
        onClick={onBrowse}
        disabled={browsing}
      >
        <FolderOpen className="h-4 w-4" />
        {browsing ? t("validating") : t("browseLocal")}
        <ChevronRight className="ml-auto h-4 w-4 text-muted-foreground" />
      </Button>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="outline"
            className="justify-start gap-2 opacity-50"
            disabled
          >
            <Globe className="h-4 w-4" />
            {t("remoteConnect")}
            <Badge variant="secondary" className="ml-auto text-[10px]">
              {t("remoteComingSoon")}
            </Badge>
          </Button>
        </TooltipTrigger>
        <TooltipContent>{t("remoteComingSoon")}</TooltipContent>
      </Tooltip>
    </div>
  );
}

function SkipOption({
  onSkip,
  t,
}: {
  onSkip: () => void;
  t: (key: string) => string;
}) {
  return (
    <button
      type="button"
      onClick={onSkip}
      className="w-full rounded-md px-2.5 py-2 text-left text-sm text-muted-foreground transition-colors hover:bg-accent/50 hover:text-foreground"
    >
      {t("skipNoDir")}
    </button>
  );
}

function SelectionStatus({
  selectedPath,
  validationInfo,
  t,
}: {
  selectedPath: string;
  validationInfo: DirectoryInfo | null;
  t: (key: string, opts?: Record<string, unknown>) => string;
}) {
  if (!validationInfo) {
    return (
      <div className="flex items-center gap-2 rounded-md bg-muted/50 px-3 py-2 text-xs text-muted-foreground">
        <span className="animate-pulse">{t("validating")}</span>
      </div>
    );
  }

  const hasWarning =
    !validationInfo.exists ||
    !validationInfo.readable;

  return (
    <div
      className={`flex items-center gap-2 rounded-md px-3 py-2 text-xs ${
        hasWarning
          ? "bg-destructive/10 text-destructive"
          : "bg-primary/5 text-foreground"
      }`}
    >
      {hasWarning ? (
        <AlertCircle className="h-3.5 w-3.5 shrink-0" />
      ) : (
        <Check className="h-3.5 w-3.5 shrink-0 text-primary" />
      )}
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className="truncate font-medium">
          {extractDirName(selectedPath)}
        </span>
        <span className="truncate opacity-70">{selectedPath}</span>
        {!validationInfo.exists && (
          <span>{t("dirNotExist")}</span>
        )}
        {validationInfo.exists && !validationInfo.readable && (
          <span>{t("dirNotReadable")}</span>
        )}
        {validationInfo.exists &&
          validationInfo.readable &&
          !validationInfo.writable && (
            <span className="text-yellow-600 dark:text-yellow-400">
              {t("dirNotWritable")}
            </span>
          )}
        {validationInfo.file_count != null && validationInfo.exists && (
          <span className="opacity-60">
            {t("fileCount", { count: validationInfo.file_count })}
          </span>
        )}
      </div>
    </div>
  );
}

// ─── 工具函数 ─────────────────────────────────────────────────────────

function extractDirName(path: string): string {
  const normalized = path.replace(/\\/g, "/").replace(/\/+$/, "");
  const parts = normalized.split("/");
  return parts[parts.length - 1] || path;
}

function formatRelativeTime(
  isoDate: string,
  t: (key: string, opts?: Record<string, unknown>) => string
): string {
  const date = new Date(isoDate);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) return t("today");
  if (diffDays < 7) return t("daysAgo", { count: diffDays });
  const weeks = Math.floor(diffDays / 7);
  return t("weeksAgo", { count: weeks });
}
