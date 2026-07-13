import { useTranslation } from "react-i18next";
import { useToolLogsStore } from "@/stores/tool-logs-store";
import { ToolActionsGroup } from "../message/ToolActionsGroup";

export function ToolLogsPanel() {
  const { t } = useTranslation("chat");
  const { toolCalls } = useToolLogsStore();

  if (toolCalls.length === 0) {
    return (
      <div className="flex h-full items-center justify-center p-4">
        <p className="text-sm text-muted-foreground">
          {t("toolLogs.empty", { defaultValue: "No tool calls yet" })}
        </p>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto p-3">
      <ToolActionsGroup toolCalls={toolCalls} defaultOpen />
    </div>
  );
}
