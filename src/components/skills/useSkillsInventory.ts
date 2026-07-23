import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { skillsIpc } from "@/lib/ipc";
import type { InstalledSkill } from "@/lib/ipc";

export function useSkillsInventory() {
  const [skills, setSkills] = useState<InstalledSkill[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      setSkills(await skillsIpc.listInstalled());
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    const unlisten = listen("skills:changed", () => {
      void refresh();
    });
    return () => {
      void unlisten.then((dispose) => dispose());
    };
  }, [refresh]);

  return { skills, loading, error, refresh };
}

function errorMessage(reason: unknown) {
  return reason instanceof Error ? reason.message : String(reason);
}
