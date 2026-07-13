import { forwardRef, type ButtonHTMLAttributes } from "react";
import { Plus } from "lucide-react";
import { cn } from "@/lib/utils";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface AttachButtonProps
  extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "type"> {
  label: string;
}

export const AttachButton = forwardRef<HTMLButtonElement, AttachButtonProps>(
  function AttachButton({ label, disabled, className, ...rest }, ref) {
    return (
      <Tooltip delayDuration={2000}>
        <TooltipTrigger asChild>
          <button
            ref={ref}
            type="button"
            disabled={disabled}
            aria-label={label}
            {...rest}
            className={cn(
              "inline-flex size-8 shrink-0 items-center justify-center rounded-full",
              "border border-[color:var(--border-strong)]",
              "bg-[color:var(--surface-card-strong)]",
              "text-muted-foreground transition-colors duration-[var(--ds-dur-fast)]",
              "hover:bg-[color:var(--surface-card-strong)] hover:text-foreground",
              "disabled:cursor-not-allowed disabled:opacity-50",
              className
            )}
          >
            <Plus className="size-3.5" strokeWidth={2} />
          </button>
        </TooltipTrigger>
        <TooltipContent
          side="top"
          className={cn(
            "text-xs duration-150",
            "data-[state=closed]:duration-150"
          )}
        >
          {label}
        </TooltipContent>
      </Tooltip>
    );
  }
);
