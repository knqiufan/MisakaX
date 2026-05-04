"use client";

import * as React from "react";
import { Switch as SwitchPrimitive } from "radix-ui";

import { cn } from "@/lib/utils";

function Switch({
  className,
  size = "default",
  ...props
}: React.ComponentProps<typeof SwitchPrimitive.Root> & {
  size?: "sm" | "default";
}) {
  const isLarge = size === "default";

  return (
    <SwitchPrimitive.Root
      data-slot="switch"
      data-size={size}
      className={cn(
        "peer group/switch inline-flex shrink-0 cursor-pointer rounded-full border border-[color:var(--border-strong)] outline-none transition-all duration-[var(--ds-dur-fast)] ease-[var(--ds-ease-spring)] focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/55 disabled:pointer-events-none disabled:opacity-50 data-[state=checked]:border-[color:var(--border-accent-soft)] data-[state=checked]:bg-primary data-[state=unchecked]:bg-[color:var(--surface-control)]",
        isLarge ? "h-6 w-[2.75rem] p-[3px]" : "h-5 w-[2rem] gap-px p-px",
        className
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb
        data-slot="switch-thumb"
        className={cn(
          "pointer-events-none rounded-full bg-background shadow-[0_2px_4px_rgba(0,0,0,0.18)] ring-0 transition-[transform,width,height] duration-[var(--ds-dur-normal)] ease-[var(--ds-ease-spring)] dark:bg-foreground dark:data-[state=checked]:bg-primary-foreground",
          isLarge
            ? "size-[18px] data-[state=checked]:translate-x-[calc(100%-2px)] data-[state=unchecked]:translate-x-0"
            : "size-3.5 data-[state=checked]:translate-x-[calc(100%-2px)] data-[state=unchecked]:translate-x-0",
        )}
      />
    </SwitchPrimitive.Root>
  );
}

export { Switch };
