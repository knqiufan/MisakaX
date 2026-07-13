import { X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { useToolLogsStore } from "@/stores/tool-logs-store";
import { ToolActionsGroup } from "../message/ToolActionsGroup";

interface ToolLogsPanelProps {
  onClose: () => void;
}

/** Chat-column Tool Logs drawer (decoupled from Explorer; shell-spec §5). */
export function ToolLogsPanel({ onClose }: ToolLogsPanelProps) {
  const { t } = useTranslation("workspace");
  const { t: tChat } = useTranslation("chat");
  const { toolCalls } = useToolLogsStore();

  return (
    <div
      className={
        "flex max-h-[min(40vh,320px)] shrink-0 flex-col " +
        "border-b border-border/40 bg-background"
      }
    >
      <div className="flex h-10 shrink-0 items-center justify-between px-3">
        <h2 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
          {t("explorer.toolLogs")}
        </h2>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          onClick={onClose}
          className="size-7"
          aria-label={t("explorer.collapseToolLogs")}
        >
          <X className="size-3.5" />
        </Button>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto">
        {toolCalls.length === 0 ? (
          <div className="flex items-center justify-center p-4">
            <p className="text-sm text-muted-foreground">
              {tChat("toolLogs.empty")}
            </p>
          </div>
        ) : (
          <div className="p-3">
            <ToolActionsGroup toolCalls={toolCalls} defaultOpen />
          </div>
        )}
      </div>
    </div>
  );
}
