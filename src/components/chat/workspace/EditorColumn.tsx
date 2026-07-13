import {
  EditorTabs,
  EDITOR_TABPANEL_ID,
  editorTabDomId,
} from "./EditorTabs";
import { EditorFileInfo } from "./EditorFileInfo";
import { EditorPane } from "./EditorPane";
import { useWorkspaceExplorerStore } from "@/stores/workspace-explorer-store";
import type { useFileEditor } from "./useFileEditor";

interface EditorColumnProps {
  workingDir: string;
  activeTab: ReturnType<typeof useFileEditor>["activeTab"];
  onChange: ReturnType<typeof useFileEditor>["handleChange"];
  onSave: ReturnType<typeof useFileEditor>["handleSave"];
}

export function EditorColumn({
  workingDir,
  activeTab,
  onChange,
  onSave,
}: EditorColumnProps) {
  const activePath = useWorkspaceExplorerStore((s) => s.activePath);
  const labelledBy = activePath ? editorTabDomId(activePath) : undefined;

  return (
    <div className="flex h-full min-h-0 flex-col bg-background">
      <EditorTabs onSave={onSave} />
      <div
        id={EDITOR_TABPANEL_ID}
        role="tabpanel"
        aria-labelledby={labelledBy}
        className="flex min-h-0 flex-1 flex-col focus-visible:outline-none"
      >
        <EditorFileInfo
          workingDir={workingDir}
          tab={activeTab}
          onSave={onSave}
        />
        <EditorPane tab={activeTab} onChange={onChange} />
      </div>
    </div>
  );
}
