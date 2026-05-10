import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import type { SidecarStatus, SidecarStatusEvent } from "@/lib/ipc/sidecar";
import { getSidecarStatus, restartSidecar } from "@/lib/ipc/sidecar";

interface UseSidecarStatusResult {
  status: SidecarStatus;
  message: string | null;
  restartSidecar: () => Promise<void>;
}

export function useSidecarStatus(): UseSidecarStatusResult {
  const [status, setStatus] = useState<SidecarStatus>("stopped");
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    getSidecarStatus()
      .then(setStatus)
      .catch(() => setStatus("stopped"));

    const unlisten = listen<SidecarStatusEvent>(
      "sidecar:status",
      (event) => {
        setStatus(event.payload.status);
        setMessage(event.payload.message);
      },
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleRestart = useCallback(async () => {
    await restartSidecar();
  }, []);

  return { status, message, restartSidecar: handleRestart };
}
