import { toWorkspaceRelativePath } from "../composer/attachment-utils";

interface EditorBreadcrumbProps {
  workingDir: string;
  filePath: string;
}

export function EditorBreadcrumb({ workingDir, filePath }: EditorBreadcrumbProps) {
  const relativePath = toWorkspaceRelativePath(workingDir, filePath);

  return (
    <div
      className="flex h-7 shrink-0 items-center border-b border-[color:var(--border-muted)] px-3"
      title={relativePath}
    >
      <span className="truncate text-[11px] text-muted-foreground">{relativePath}</span>
    </div>
  );
}
