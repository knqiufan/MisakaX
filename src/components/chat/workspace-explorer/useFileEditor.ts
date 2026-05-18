import { useCallback, useEffect } from "react";
import { toast } from "sonner";
import { fsIpc } from "@/lib/ipc";
import { useWorkspaceExplorerStore } from "@/stores/workspace-explorer-store";

export function useFileEditor(workingDir: string) {
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
        toast.success("文件已保存");
      } catch (error) {
        console.error("Failed to save file:", error);
        toast.error("保存失败");
      }
    },
    [markSaved, workingDir]
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
