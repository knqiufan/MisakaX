import { File, FileCode2, FileText, Loader2, AlertTriangle } from "lucide-react";
import { cn } from "@/lib/utils";

interface FileTypeStyle {
  icon: typeof File;
  colorClass: string;
}

const EXTENSION_STYLES: Record<string, FileTypeStyle> = {
  tsx: { icon: FileCode2, colorClass: "text-sky-400" },
  jsx: { icon: FileCode2, colorClass: "text-sky-400" },
  ts: { icon: FileCode2, colorClass: "text-blue-400" },
  js: { icon: FileCode2, colorClass: "text-blue-400" },
  rs: { icon: FileCode2, colorClass: "text-orange-400" },
  py: { icon: FileCode2, colorClass: "text-yellow-400" },
  md: { icon: FileText, colorClass: "text-muted-foreground" },
  markdown: { icon: FileText, colorClass: "text-muted-foreground" },
};

function resolveFileTypeStyle(fileName: string): FileTypeStyle {
  const ext = fileName.split(".").pop()?.toLowerCase() ?? "";
  return EXTENSION_STYLES[ext] ?? { icon: File, colorClass: "text-muted-foreground" };
}

interface FileTypeIconProps {
  name: string;
  className?: string;
}

export function FileTypeIcon({ name, className }: FileTypeIconProps) {
  const { icon: Icon, colorClass } = resolveFileTypeStyle(name);
  return <Icon className={cn("size-3.5 shrink-0", colorClass, className)} />;
}

export function getFileTypeColorClass(fileName: string): string {
  return resolveFileTypeStyle(fileName).colorClass;
}

interface ChipStatusIconProps {
  isLoading: boolean;
  isError: boolean;
  fileName: string;
}

export function ChipStatusIcon({ isLoading, isError, fileName }: ChipStatusIconProps) {
  if (isLoading) {
    return <Loader2 className="size-3.5 shrink-0 animate-spin text-muted-foreground" />;
  }
  if (isError) {
    return <AlertTriangle className="size-3.5 shrink-0 text-destructive" />;
  }
  return <FileTypeIcon name={fileName} />;
}
