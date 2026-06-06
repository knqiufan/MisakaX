import { X } from "lucide-react";
import { cn } from "@/lib/utils";

export interface PendingImage {
  id: string;
  data: string;
  mime_type: string;
  name: string;
  size: number;
}

interface ImagePreviewProps {
  images: PendingImage[];
  onRemove: (id: string) => void;
}

export function ImagePreview({ images, onRemove }: ImagePreviewProps) {
  if (images.length === 0) return null;

  return (
    <div className="flex flex-wrap gap-2 px-1 pb-2">
      {images.map((img) => (
        <ImageThumbnail key={img.id} image={img} onRemove={onRemove} />
      ))}
    </div>
  );
}

function ImageThumbnail({
  image,
  onRemove,
}: {
  image: PendingImage;
  onRemove: (id: string) => void;
}) {
  return (
    <div className="group/thumb relative">
      <img
        src={`data:${image.mime_type};base64,${image.data}`}
        alt={image.name}
        className={cn(
          "size-16 rounded-[var(--radius-ui-md)] object-cover",
          "border border-[color:var(--border-muted)]",
          "bg-[color:var(--surface-card)]"
        )}
      />
      <button
        type="button"
        onClick={() => onRemove(image.id)}
        className={cn(
          "absolute -right-1.5 -top-1.5",
          "flex size-4 items-center justify-center rounded-full",
          "bg-[color:var(--surface-popover)] text-muted-foreground",
          "border border-[color:var(--border-muted)]",
          "opacity-0 transition-opacity duration-[var(--ds-dur-fast)]",
          "group-hover/thumb:opacity-100",
          "hover:bg-destructive hover:text-destructive-foreground"
        )}
        aria-label={`Remove ${image.name}`}
      >
        <X className="size-2.5" />
      </button>
      <span
        className={cn(
          "absolute bottom-0 left-0 right-0",
          "truncate rounded-b-[var(--radius-ui-md)] px-1 py-px",
          "bg-black/50 text-center text-[9px] text-white"
        )}
      >
        {image.name}
      </span>
    </div>
  );
}
