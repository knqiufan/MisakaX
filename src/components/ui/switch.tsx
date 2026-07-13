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
        "peer group/switch relative inline-flex shrink-0 cursor-pointer items-center rounded-full border-2 outline-none transition-colors duration-[var(--ds-dur-fast)] ease-out",
        "after:absolute after:-inset-x-3 after:-inset-y-2 after:content-['']",
        "focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/30",
        "disabled:cursor-not-allowed disabled:opacity-50",
        "data-[state=checked]:border-primary data-[state=checked]:bg-primary",
        "data-[state=unchecked]:border-transparent data-[state=unchecked]:bg-input/90",
        isLarge ? "h-5 w-11" : "h-4 w-7",
        className
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb
        data-slot="switch-thumb"
        className={cn(
          "pointer-events-none block rounded-full bg-background shadow-sm ring-0 transition-transform duration-[var(--ds-dur-normal)] ease-out",
          "dark:bg-foreground dark:data-[state=checked]:bg-primary-foreground",
          isLarge
            ? "h-4 w-6 data-[state=unchecked]:translate-x-0.5 data-[state=checked]:translate-x-[calc(100%-8px)]"
            : "h-3 w-4 data-[state=unchecked]:translate-x-0.5 data-[state=checked]:translate-x-[calc(100%-6px)]"
        )}
      />
    </SwitchPrimitive.Root>
  );
}

export { Switch };
