import { useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { fsIpc } from "@/lib/ipc";
import { useWorkspaceExplorerStore } from "@/stores/workspace-explorer-store";

export function useFileEditor(workingDir: string) {
  const { t } = useTranslation("workspace");
  const { activePath, tabs, updateContent, markSaved } = useWorkspaceExplorerStore();

  const activeTab = tabs.find((tab) => tab.path === activePath) ?? null;

  const handleChange = useCallback(
    (path: string, value: string) => {
      updateContent(path, value);
    },
    [updateContent]
  );

  const handleSave = useCallback(
    async (path: string) => {
      const tab = useWorkspaceExplorerStore
        .getState()
        .tabs.find((candidate) => candidate.path === path);
      if (!tab) return;
      try {
        await fsIpc.writeTextFile(workingDir, path, tab.currentContent);
        markSaved(path);
        const name = path.replace(/\\/g, "/").split("/").pop() ?? path;
        toast.success(t("explorer.fileSaved", { name }));
      } catch (error) {
        console.error("Failed to save file:", error);
        toast.error(t("explorer.saveFailed"));
      }
    },
    [markSaved, workingDir, t]
  );

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || event.key.toLowerCase() !== "s") {
        return;
      }
      const tab = useWorkspaceExplorerStore.getState().tabs.find((item) => item.path === activePath);
      if (!tab?.dirty) return;
      event.preventDefault();
      void handleSave(tab.path);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [activePath, handleSave]);

  return { activeTab, handleChange, handleSave };
}
