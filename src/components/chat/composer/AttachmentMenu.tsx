import type { ReactNode } from "react";
import { FileImage, FileText, FolderOpen } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

interface AttachmentMenuProps {
  trigger: ReactNode;
  labels: {
    image: string;
    text: string;
    workspace: string;
  };
  onPickImages: () => void;
  onPickText: () => void;
  onPickWorkspace: () => void;
}

/**
 * Composer 附件下拉菜单。
 *
 * UI 约定（与设计规范一致）：
 * - 仅保留实际可用的入口，不展示「即将支持」等占位项；
 * - 不使用 `DropdownMenuLabel` 作为分组标题，菜单只承载选项；
 * - 每个 item 统一为 `图标 + 文案` 排版（图标 `size-4` + 8px gap），
 *   保持上下行宽度与对齐一致；
 * - 触发器（trigger）由调用方传入，本组件只承担菜单部分。
 */
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
      <DropdownMenuContent align="start" side="top" className="w-56">
        <DropdownMenuItem onSelect={onPickImages}>
          <FileImage className="size-4" />
          {labels.image}
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={onPickText}>
          <FileText className="size-4" />
          {labels.text}
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
