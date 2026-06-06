import { cn } from "@/lib/utils";

interface StreamingIndicatorProps {
  className?: string;
}

export function StreamingIndicator({ className }: StreamingIndicatorProps) {
  return (
    <div
      className={cn("flex items-center gap-1 px-1 py-2", className)}
      role="status"
      aria-label="Generating response"
    >
      <span className="size-1.5 animate-pulse rounded-full bg-muted-foreground/60 [animation-delay:0ms]" />
      <span className="size-1.5 animate-pulse rounded-full bg-muted-foreground/60 [animation-delay:150ms]" />
      <span className="size-1.5 animate-pulse rounded-full bg-muted-foreground/60 [animation-delay:300ms]" />
    </div>
  );
}
