import { useCallback, useRef, useState, type PointerEvent } from "react";
import { cn } from "@/lib/utils";

export const RESIZE_GUTTER_WIDTH_PX = 8;

interface ResizeGutterProps {
  onResize: (deltaX: number) => void;
  onResizeEnd?: () => void;
  onReset?: () => void;
  className?: string;
}

export function ResizeGutter({
  onResize,
  onResizeEnd,
  onReset,
  className,
}: ResizeGutterProps) {
  const [hovering, setHovering] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [cursorY, setCursorY] = useState(0.5);
  const lastXRef = useRef(0);
  const gutterRef = useRef<HTMLDivElement>(null);

  const handlePointerDown = useCallback(
    (e: PointerEvent<HTMLDivElement>) => {
      e.preventDefault();
      lastXRef.current = e.clientX;
      setDragging(true);
      e.currentTarget.setPointerCapture(e.pointerId);
      updateCursorY(e);
    },
    []
  );

  const updateCursorY = (e: PointerEvent<HTMLDivElement>) => {
    const el = gutterRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    if (rect.height <= 0) return;
    setCursorY(Math.min(1, Math.max(0, (e.clientY - rect.top) / rect.height)));
  };

  const handlePointerMove = useCallback(
    (e: PointerEvent<HTMLDivElement>) => {
      updateCursorY(e);
      if (!dragging) return;
      const delta = e.clientX - lastXRef.current;
      lastXRef.current = e.clientX;
      if (delta !== 0) onResize(delta);
    },
    [dragging, onResize]
  );

  const handlePointerUp = useCallback(
    (e: PointerEvent<HTMLDivElement>) => {
      if (!dragging) return;
      setDragging(false);
      try {
        e.currentTarget.releasePointerCapture(e.pointerId);
      } catch {
        /* already released */
      }
      onResizeEnd?.();
    },
    [dragging, onResizeEnd]
  );

  const peakAlpha = dragging ? 0.36 : hovering ? 0.24 : 0;
  const peakPercent = `${Math.round(cursorY * 100)}%`;

  return (
    <div
      ref={gutterRef}
      role="separator"
      aria-orientation="vertical"
      tabIndex={0}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerCancel={handlePointerUp}
      onDoubleClick={() => onReset?.()}
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => {
        if (!dragging) setHovering(false);
      }}
      className={cn(
        "relative z-10 flex shrink-0 touch-none cursor-col-resize items-stretch justify-center",
        className
      )}
      style={{ width: RESIZE_GUTTER_WIDTH_PX }}
    >
      <div
        className="w-0.5 self-stretch opacity-0 transition-opacity duration-150"
        style={{
          opacity: peakAlpha > 0 ? 1 : 0,
          background: `linear-gradient(
            to bottom,
            transparent 0%,
            color-mix(in oklch, var(--foreground) ${Math.round(peakAlpha * 100)}%, transparent) ${peakPercent},
            transparent 100%
          )`,
        }}
      />
    </div>
  );
}
