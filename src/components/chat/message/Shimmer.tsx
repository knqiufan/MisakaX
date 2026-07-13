import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface ShimmerProps {
  children: ReactNode;
  className?: string;
  durationSec?: number;
}

/** Thinking-label shimmer (docs/ui/06 §9). */
export function Shimmer({
  children,
  className,
  durationSec = 1,
}: ShimmerProps) {
  return (
    <span
      className={cn("misaka-shimmer", className)}
      style={{ animationDuration: `${durationSec}s` }}
    >
      {children}
    </span>
  );
}
