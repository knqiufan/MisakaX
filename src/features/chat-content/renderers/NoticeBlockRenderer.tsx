import { AlertTriangle, CircleAlert, Info } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { BlockRendererProps } from "../renderer-registry";
import { readNoticePayload } from "../types";

export function NoticeBlockRenderer({ block }: BlockRendererProps) {
  const { t } = useTranslation("chat");
  const payload = readNoticePayload(block);
  const severity = payload?.severity ?? (block.status === "failed" ? "error" : "info");
  const Icon = severity === "error" ? AlertTriangle : severity === "warning" ? CircleAlert : Info;
  const message = payload ? t(payload.message_key, payload.params) : t(block.fallback.message_key || "richContent.blockUnavailable", block.fallback.params);
  return (
    <div
      className={
        severity === "error"
          ? "my-3 flex gap-2 rounded-lg border border-destructive/40 bg-destructive/8 px-3 py-2 text-sm text-destructive"
          : "my-3 flex gap-2 rounded-lg border border-border/50 bg-muted/35 px-3 py-2 text-sm text-muted-foreground"
      }
      role={severity === "error" ? "alert" : "status"}
    >
      <Icon className="mt-0.5 size-4 shrink-0" aria-hidden />
      <span>{message}</span>
    </div>
  );
}
