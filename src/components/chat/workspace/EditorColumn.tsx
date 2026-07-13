import { EditorTabs } from "./EditorTabs";
import { EditorFileInfo } from "./EditorFileInfo";
import { EditorPane } from "./EditorPane";
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
  return (
    <div className="flex h-full min-h-0 flex-col bg-background">
      <EditorTabs onSave={onSave} />
      <EditorFileInfo
        workingDir={workingDir}
        tab={activeTab}
        onSave={onSave}
      />
      <EditorPane tab={activeTab} onChange={onChange} />
    </div>
  );
}
