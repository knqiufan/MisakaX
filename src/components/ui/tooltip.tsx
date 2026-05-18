import * as React from "react";
import { Tooltip as TooltipPrimitive } from "radix-ui";

import { cn } from "@/lib/utils";

function TooltipProvider({
  delayDuration = 0,
  ...props
}: React.ComponentProps<typeof TooltipPrimitive.Provider>) {
  return (
    <TooltipPrimitive.Provider
      data-slot="tooltip-provider"
      delayDuration={delayDuration}
      {...props}
    />
  );
}

function Tooltip({
  ...props
}: React.ComponentProps<typeof TooltipPrimitive.Root>) {
  return <TooltipPrimitive.Root data-slot="tooltip" {...props} />;
}

function TooltipTrigger({
  ...props
}: React.ComponentProps<typeof TooltipPrimitive.Trigger>) {
  return <TooltipPrimitive.Trigger data-slot="tooltip-trigger" {...props} />;
}

function TooltipContent({
  className,
  sideOffset = 6,
  children,
  ...props
}: React.ComponentProps<typeof TooltipPrimitive.Content>) {
  return (
    <TooltipPrimitive.Portal>
      <TooltipPrimitive.Content
        data-slot="tooltip-content"
        sideOffset={sideOffset}
        className={cn(
          "z-[var(--ds-layer-modal)] pointer-events-none w-fit max-w-[260px]",
          "origin-(--radix-tooltip-content-transform-origin)",
          "rounded-[var(--radius-ui-md)]",
          "border border-[color:var(--border-strong)]/70",
          "bg-[color:var(--surface-popover,var(--popover))]/95 backdrop-blur-md",
          "px-2 py-1 text-[12px] font-medium leading-[18px] tracking-tight",
          "text-popover-foreground/95 text-balance",
          "shadow-[0_10px_24px_-12px_rgba(0,0,0,0.45),0_2px_6px_-1px_rgba(0,0,0,0.25)]",
          "animate-in fade-in-0 ease-out duration-[120ms]",
          "data-[side=bottom]:slide-in-from-top-1 data-[side=left]:slide-in-from-right-1",
          "data-[side=right]:slide-in-from-left-1 data-[side=top]:slide-in-from-bottom-1",
          "data-[state=closed]:animate-out data-[state=closed]:fade-out-0",
          className
        )}
        {...props}
      >
        {children}
      </TooltipPrimitive.Content>
    </TooltipPrimitive.Portal>
  );
}

export { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider };
