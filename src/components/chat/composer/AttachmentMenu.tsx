import type { ReactNode } from "react";
import { FileImage, FileText, FolderOpen, FileClock } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

interface AttachmentMenuProps {
  trigger: ReactNode;
  labels: {
    image: string;
    text: string;
    workspace: string;
    planned: string;
    comingSoon: string;
  };
  onPickImages: () => void;
  onPickText: () => void;
  onPickWorkspace: () => void;
}

export function AttachmentMenu({
  trigger,
  labels,
  onPickImages,
  onPickText,
  onPickWorkspace,
}: AttachmentMenuProps) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>{trigger}</DropdownMenuTrigger>
      <DropdownMenuContent align="start" side="top" className="w-60">
        <DropdownMenuLabel className="text-xs text-muted-foreground">
          {labels.image}
        </DropdownMenuLabel>
        <DropdownMenuItem onSelect={onPickImages}>
          <FileImage className="size-4" />
          {labels.image}
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuLabel className="text-xs text-muted-foreground">
          {labels.text}
        </DropdownMenuLabel>
        <DropdownMenuItem onSelect={onPickText}>
          <FileText className="size-4" />
          Markdown / Text
        </DropdownMenuItem>
        <DropdownMenuItem disabled>
          <FileClock className="size-4" />
          <span className="min-w-0 flex-1 truncate">{labels.planned}</span>
          <span className="text-[10px] text-muted-foreground">{labels.comingSoon}</span>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onSelect={onPickWorkspace}>
          <FolderOpen className="size-4" />
          {labels.workspace}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
