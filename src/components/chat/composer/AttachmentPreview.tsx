import { FileText, X } from "lucide-react";
import { cn } from "@/lib/utils";
import type { PendingAttachment } from "@/stores/composer-store";
import { formatFileSize } from "./attachmentUtils";

interface AttachmentPreviewProps {
  attachments: PendingAttachment[];
  onRemove: (id: string) => void;
}

export function AttachmentPreview({
  attachments,
  onRemove,
}: AttachmentPreviewProps) {
  if (attachments.length === 0) return null;

  return (
    <div className="flex flex-wrap gap-2 px-1 pb-2">
      {attachments.map((attachment) =>
        attachment.kind === "image" ? (
          <ImageAttachmentCard
            key={attachment.id}
            attachment={attachment}
            onRemove={onRemove}
          />
        ) : (
          <TextAttachmentCard
            key={attachment.id}
            attachment={attachment}
            onRemove={onRemove}
          />
        )
      )}
    </div>
  );
}

function ImageAttachmentCard({
  attachment,
  onRemove,
}: {
  attachment: Extract<PendingAttachment, { kind: "image" }>;
  onRemove: (id: string) => void;
}) {
  return (
    <div className="group/thumb relative">
      <img
        src={`data:${attachment.media_type};base64,${attachment.data}`}
        alt={attachment.file_name}
        className={cn(
          "size-16 rounded-[var(--radius-ui-md)] object-cover",
          "border border-[color:var(--border-muted)] bg-[color:var(--surface-card)]"
        )}
      />
      <RemoveAttachmentButton
        label={`Remove ${attachment.file_name}`}
        onClick={() => onRemove(attachment.id)}
      />
      <span className="absolute inset-x-0 bottom-0 truncate rounded-b-[var(--radius-ui-md)] bg-black/50 px-1 py-px text-center text-[9px] text-white">
        {attachment.file_name}
      </span>
    </div>
  );
}

function TextAttachmentCard({
  attachment,
  onRemove,
}: {
  attachment: Extract<PendingAttachment, { kind: "text" }>;
  onRemove: (id: string) => void;
}) {
  return (
    <div
      className={cn(
        "group/thumb relative flex max-w-[220px] items-center gap-2 rounded-[var(--radius-ui-md)]",
        "border border-[color:var(--border-muted)] bg-[color:var(--surface-card)]",
        "px-2 py-2"
      )}
    >
      <div className="flex size-8 shrink-0 items-center justify-center rounded-[var(--radius-ui-sm)] bg-[color:var(--cm-surface-panel-strong)] text-muted-foreground">
        <FileText className="size-4" />
      </div>
      <div className="min-w-0">
        <p className="truncate text-xs font-medium text-foreground">
          {attachment.file_name}
        </p>
        <p className="text-[10px] text-muted-foreground">
          {formatFileSize(attachment.size)}
        </p>
      </div>
      <RemoveAttachmentButton
        label={`Remove ${attachment.file_name}`}
        onClick={() => onRemove(attachment.id)}
      />
    </div>
  );
}

function RemoveAttachmentButton({
  label,
  onClick,
}: {
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "absolute -right-1.5 -top-1.5 flex size-4 items-center justify-center rounded-full",
        "border border-[color:var(--border-muted)] bg-[color:var(--surface-popover)]",
        "text-muted-foreground opacity-0 transition-opacity duration-[var(--ds-dur-fast)]",
        "group-hover/thumb:opacity-100 hover:bg-destructive hover:text-destructive-foreground"
      )}
      aria-label={label}
    >
      <X className="size-2.5" />
    </button>
  );
}
