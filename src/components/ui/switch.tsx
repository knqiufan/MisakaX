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
        "peer group/switch relative inline-flex shrink-0 cursor-pointer items-center rounded-full border border-[color:var(--border-strong)] outline-none transition-colors duration-[var(--ds-dur-fast)] ease-out focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/55 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:border-[color:var(--border-accent-soft)] data-[state=checked]:bg-primary data-[state=unchecked]:bg-[color:var(--surface-control)]",
        "overflow-hidden",
        isLarge ? "h-6 w-11 p-0.5" : "h-5 w-9 p-px",
        className
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb
        data-slot="switch-thumb"
        className={cn(
          "pointer-events-none block rounded-full bg-background shadow-[0_1px_3px_rgba(0,0,0,0.22)] ring-0 transition-transform duration-[var(--ds-dur-normal)] ease-[var(--ds-ease-spring)] will-change-transform dark:bg-foreground dark:data-[state=checked]:bg-primary-foreground",
          isLarge
            ? "size-5 data-[state=unchecked]:translate-x-0 data-[state=checked]:translate-x-5"
            : "size-4 data-[state=unchecked]:translate-x-0 data-[state=checked]:translate-x-[1.125rem]",
        )}
      />
    </SwitchPrimitive.Root>
  );
}

export { Switch };
