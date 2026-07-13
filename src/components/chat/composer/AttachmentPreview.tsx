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
    <div className="order-first flex w-full flex-wrap items-center gap-1.5 px-3 pb-0 pt-2.5">
      {attachments.map((attachment) =>
        attachment.kind === "image" ? (
          <ImageAttachmentChip
            key={attachment.id}
            attachment={attachment}
            onRemove={onRemove}
          />
        ) : (
          <TextAttachmentChip
            key={attachment.id}
            attachment={attachment}
            onRemove={onRemove}
          />
        )
      )}
    </div>
  );
}

function ImageAttachmentChip({
  attachment,
  onRemove,
}: {
  attachment: Extract<PendingAttachment, { kind: "image" }>;
  onRemove: (id: string) => void;
}) {
  return (
    <div
      className={cn(
        "group/thumb inline-flex items-center gap-1.5 rounded-full border border-border/40 bg-muted",
        "py-0.5 pl-1.5 pr-1 text-xs font-medium text-foreground"
      )}
    >
      <img
        src={`data:${attachment.media_type};base64,${attachment.data}`}
        alt={attachment.file_name}
        className="h-5 w-5 rounded object-cover"
      />
      <span className="max-w-[120px] truncate">{attachment.file_name}</span>
      <RemoveChipButton
        label={`Remove ${attachment.file_name}`}
        onClick={() => onRemove(attachment.id)}
      />
    </div>
  );
}

function TextAttachmentChip({
  attachment,
  onRemove,
}: {
  attachment: Extract<PendingAttachment, { kind: "text" }>;
  onRemove: (id: string) => void;
}) {
  return (
    <div
      className={cn(
        "group/thumb inline-flex items-center gap-1.5 rounded-full border border-border/40 bg-muted",
        "py-0.5 pl-2 pr-1 text-xs font-medium text-foreground"
      )}
    >
      <FileText className="size-3 shrink-0 text-muted-foreground" />
      <span className="max-w-[160px] truncate">{attachment.file_name}</span>
      <span className="text-[10px] font-normal text-muted-foreground">
        {formatFileSize(attachment.size)}
      </span>
      <RemoveChipButton
        label={`Remove ${attachment.file_name}`}
        onClick={() => onRemove(attachment.id)}
      />
    </div>
  );
}

function RemoveChipButton({
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
      className="rounded-full p-0.5 text-muted-foreground transition-colors duration-[var(--ds-dur-fast)] hover:bg-accent hover:text-foreground"
      aria-label={label}
    >
      <X className="size-3" />
    </button>
  );
}
