import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ShieldAlert, Server, Wrench } from "lucide-react";
import { cn } from "@/lib/utils";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import type { ToolCallRequestEvent } from "@/lib/ipc";
import { mcpIpc } from "@/lib/ipc/mcp";
import { CodeBlock } from "./message/CodeBlock";

interface ToolApprovalDialogProps {
  request: ToolCallRequestEvent | null;
  onDismiss: () => void;
}

export function ToolApprovalDialog({
  request,
  onDismiss,
}: ToolApprovalDialogProps) {
  const { t } = useTranslation("chat");
  const [countdown, setCountdown] = useState(60);
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (!request) return;
    setCountdown(60);
    const timer = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          clearInterval(timer);
          handleDeny(false);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
    return () => clearInterval(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [request?.request_id]);

  const handleApprove = useCallback(
    async (remember: boolean) => {
      if (!request || isSubmitting) return;
      setIsSubmitting(true);
      try {
        await mcpIpc.approveToolCall(request.request_id, remember);
      } finally {
        setIsSubmitting(false);
        onDismiss();
      }
    },
    [request, isSubmitting, onDismiss]
  );

  const handleDeny = useCallback(
    async (remember: boolean) => {
      if (!request || isSubmitting) return;
      setIsSubmitting(true);
      try {
        await mcpIpc.denyToolCall(request.request_id, remember);
      } finally {
        setIsSubmitting(false);
        onDismiss();
      }
    },
    [request, isSubmitting, onDismiss]
  );

  if (!request) return null;

  return (
    <Dialog open={!!request} onOpenChange={() => handleDeny(false)}>
      <DialogContent showCloseButton={false} className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <ShieldAlert className="size-5 text-amber-500" />
            {t("toolApproval.title")}
          </DialogTitle>
          <DialogDescription>
            {t("toolApproval.description")}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <InfoRow
            icon={<Wrench className="size-3.5" />}
            label={t("toolApproval.tool")}
            value={request.tool_name}
          />
          <InfoRow
            icon={<Server className="size-3.5" />}
            label={t("toolApproval.server")}
            value={request.server_name}
          />
          <div>
            <p className="mb-1 text-xs font-medium text-muted-foreground">
              {t("toolApproval.arguments")}
            </p>
            <div className="max-h-48 overflow-auto rounded-[var(--radius-ui-md)] border border-[color:var(--border-muted)]">
              <CodeBlock
                code={JSON.stringify(request.arguments, null, 2)}
                language="json"
              />
            </div>
          </div>
        </div>

        <DialogFooter className="flex-col gap-2 sm:flex-row">
          <Button
            variant="destructive"
            size="sm"
            onClick={() => handleDeny(false)}
            disabled={isSubmitting}
          >
            {t("toolApproval.deny")}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => handleApprove(false)}
            disabled={isSubmitting}
          >
            {t("toolApproval.allowOnce")}
          </Button>
          <Button
            variant="default"
            size="sm"
            onClick={() => handleApprove(true)}
            disabled={isSubmitting}
          >
            {t("toolApproval.alwaysAllow")}
          </Button>
        </DialogFooter>

        <p
          className={cn(
            "text-center text-xs text-muted-foreground/60",
            countdown <= 10 && "text-destructive/80"
          )}
        >
          {t("toolApproval.countdown", { seconds: countdown })}
        </p>
      </DialogContent>
    </Dialog>
  );
}

function InfoRow({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-center gap-2 text-sm">
      <span className="text-muted-foreground/60">{icon}</span>
      <span className="font-medium text-muted-foreground">{label}:</span>
      <span className="font-mono text-foreground">{value}</span>
    </div>
  );
}
