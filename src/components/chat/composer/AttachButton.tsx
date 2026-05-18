import { Plus } from "lucide-react";
import { cn } from "@/lib/utils";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface AttachButtonProps {
  onClick: () => void;
  label: string;
  disabled?: boolean;
}

export function AttachButton({ onClick, label, disabled }: AttachButtonProps) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type="button"
          onClick={onClick}
          disabled={disabled}
          aria-label={label}
          className={cn(
            "inline-flex size-9 shrink-0 items-center justify-center rounded-full",
            "border border-[color:var(--cm-border-strong)]",
            "bg-[color:var(--cm-surface-panel-strong)]",
            "text-muted-foreground transition-colors duration-[var(--ds-dur-fast)]",
            "hover:bg-[color:var(--cm-surface-panel-solid)] hover:text-foreground",
            "disabled:cursor-not-allowed disabled:opacity-50"
          )}
        >
          <Plus className="size-4" strokeWidth={2} />
        </button>
      </TooltipTrigger>
      <TooltipContent side="top" className="text-xs">
        {label}
      </TooltipContent>
    </Tooltip>
  );
}
