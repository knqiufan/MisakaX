import { useEffect, useRef, useState } from "react";

const EXIT_FALLBACK_MS = 200;

/**
 * Keeps children mounted through the close animation.
 * When `open` becomes false, stays mounted until `onExitComplete` is called
 * (typically from `onAnimationEnd` on the exiting element).
 */
export function usePresence(open: boolean): {
  mounted: boolean;
  exiting: boolean;
  onExitComplete: () => void;
} {
  const [mounted, setMounted] = useState(open);
  const [exiting, setExiting] = useState(false);
  const exitTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearExitTimer = () => {
    if (exitTimerRef.current !== null) {
      clearTimeout(exitTimerRef.current);
      exitTimerRef.current = null;
    }
  };

  useEffect(() => {
    if (open) {
      clearExitTimer();
      setMounted(true);
      setExiting(false);
      return;
    }
    if (!mounted) return;
    setExiting(true);
    clearExitTimer();
    exitTimerRef.current = setTimeout(() => {
      setMounted(false);
      setExiting(false);
      exitTimerRef.current = null;
    }, EXIT_FALLBACK_MS);
    return clearExitTimer;
  }, [open, mounted]);

  const onExitComplete = () => {
    if (!open) {
      clearExitTimer();
      setMounted(false);
      setExiting(false);
    }
  };

  return { mounted, exiting, onExitComplete };
}
