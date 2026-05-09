import { useTranslation } from "react-i18next";
import { FolderOpen, FolderX, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface WorkspaceBarProps {
  workingDir: string | null;
  onChangeDir: () => void;
}

export function WorkspaceBar({ workingDir, onChangeDir }: WorkspaceBarProps) {
  const { t } = useTranslation("workspace");

  if (!workingDir) {
    return (
      <div className="flex items-center gap-2 border-b border-border/50 bg-muted/30 px-4 py-1.5">
        <FolderX className="h-3.5 w-3.5 text-muted-foreground/70" />
        <span className="text-xs text-muted-foreground">
          {t("noWorkingDir")}
        </span>
        <Button
          variant="link"
          size="sm"
          onClick={onChangeDir}
          className="h-auto px-1 py-0 text-xs"
        >
          {t("selectDir")}
        </Button>
      </div>
    );
  }

  const dirName = extractDirName(workingDir);

  return (
    <div className="flex items-center gap-2 border-b border-border/50 bg-muted/30 px-4 py-1.5">
      <FolderOpen className="h-3.5 w-3.5 shrink-0 text-primary/80" />
      <Tooltip>
        <TooltipTrigger asChild>
          <span className="max-w-[280px] truncate text-xs text-muted-foreground">
            {workingDir}
          </span>
        </TooltipTrigger>
        <TooltipContent side="bottom" className="max-w-sm text-xs">
          {workingDir}
        </TooltipContent>
      </Tooltip>
      <span className="text-xs text-muted-foreground/50">·</span>
      <span className="text-xs font-medium text-foreground/80">{dirName}</span>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            onClick={onChangeDir}
            className="ml-auto h-5 w-5"
          >
            <RefreshCw className="h-3 w-3 text-muted-foreground" />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom" className="text-xs">
          {t("switchDir")}
        </TooltipContent>
      </Tooltip>
    </div>
  );
}

function extractDirName(path: string): string {
  const normalized = path.replace(/\\/g, "/").replace(/\/+$/, "");
  const parts = normalized.split("/");
  return parts[parts.length - 1] || path;
}
