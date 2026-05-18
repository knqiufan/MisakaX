import Editor from "@monaco-editor/react";
import { useThemeStore } from "@/stores/theme-store";
import type { OpenTab } from "@/stores/workspace-explorer-store";

interface EditorPaneProps {
  tab: OpenTab | null;
  onChange: (path: string, value: string) => void;
}

export function EditorPane({ tab, onChange }: EditorPaneProps) {
  const resolvedTheme = useThemeStore((state) => state.resolvedTheme);

  if (!tab) {
    return (
      <div className="flex min-h-0 flex-1 items-center justify-center text-xs text-muted-foreground">
        选择文件以开始编辑
      </div>
    );
  }

  return (
    <Editor
      height="100%"
      path={tab.path}
      language={detectLanguage(tab.path)}
      value={tab.currentContent}
      theme={resolvedTheme === "dark" ? "vs-dark" : "vs"}
      onChange={(value) => onChange(tab.path, value ?? "")}
      options={{
        minimap: { enabled: false },
        fontSize: 13,
        lineHeight: 20,
        wordWrap: "on",
        scrollBeyondLastLine: false,
        automaticLayout: true,
      }}
    />
  );
}

function detectLanguage(path: string): string {
  const ext = path.split(".").pop()?.toLowerCase();
  switch (ext) {
    case "ts":
      return "typescript";
    case "tsx":
      return "typescript";
    case "js":
    case "jsx":
      return "javascript";
    case "md":
    case "markdown":
      return "markdown";
    case "json":
      return "json";
    case "rs":
      return "rust";
    case "css":
      return "css";
    case "html":
      return "html";
    default:
      return "plaintext";
  }
}
