import Editor from "@monaco-editor/react";
import { FileCode2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useThemeStore } from "@/stores/theme-store";
import type { OpenTab } from "@/stores/workspace-explorer-store";

interface EditorPaneProps {
  tab: OpenTab | null;
  onChange: (path: string, value: string) => void;
}

export function EditorPane({ tab, onChange }: EditorPaneProps) {
  const { t } = useTranslation("workspace");
  const resolvedTheme = useThemeStore((state) => state.resolvedTheme);

  if (!tab) {
    return (
      <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-3 py-12 text-muted-foreground">
        <FileCode2 className="size-8 opacity-40" strokeWidth={1.2} aria-hidden />
        <p className="text-xs">{t("explorer.selectFileHint")}</p>
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div
        key={tab.path}
        className="min-h-0 flex-1 overflow-hidden animate-in fade-in duration-[var(--ds-dur-fast)] ease-out"
      >
        <MonacoEditor
          tab={tab}
          resolvedTheme={resolvedTheme}
          onChange={onChange}
        />
      </div>
    </div>
  );
}

interface MonacoEditorProps {
  tab: OpenTab;
  resolvedTheme: "light" | "dark";
  onChange: (path: string, value: string) => void;
}

function MonacoEditor({ tab, resolvedTheme, onChange }: MonacoEditorProps) {
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
        smoothScrolling: true,
        cursorBlinking: "smooth",
        padding: { top: 10, bottom: 10 },
        renderLineHighlight: "line",
        overviewRulerBorder: false,
        scrollbar: {
          verticalScrollbarSize: 10,
          horizontalScrollbarSize: 10,
        },
      }}
    />
  );
}

function detectLanguage(path: string): string {
  const ext = path.split(".").pop()?.toLowerCase();
  switch (ext) {
    case "ts":
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
    case "py":
      return "python";
    case "go":
      return "go";
    case "toml":
      return "toml";
    case "yml":
    case "yaml":
      return "yaml";
    case "xml":
      return "xml";
    case "sh":
    case "bash":
      return "shell";
    default:
      return "plaintext";
  }
}
