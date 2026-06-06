import { EditorTabs } from "./EditorTabs";
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
    <div
      className={
        "flex h-full min-h-0 flex-col bg-[color:var(--surface-card)] " +
        "animate-in fade-in slide-in-from-right-2 ease-out duration-[var(--ds-dur-slow)]"
      }
    >
      <EditorTabs onSave={onSave} />
      <EditorPane
        workingDir={workingDir}
        tab={activeTab}
        onChange={onChange}
      />
    </div>
  );
}
