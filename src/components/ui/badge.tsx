"use client";

import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { Slot } from "radix-ui";

import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-full border border-transparent px-2.5 py-0.5 text-xs font-semibold whitespace-nowrap tracking-tight transition-[color,box-shadow,transform] duration-[var(--ds-dur-fast)] outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/40 aria-invalid:border-destructive dark:aria-invalid:ring-destructive/45 [&>svg]:pointer-events-none [&>svg]:size-3",
  {
    variants: {
      variant: {
        default:
          "border-0 bg-gradient-to-r from-[#62b7ff]/90 to-[#4fe3a3]/90 text-[#0b0f1a] shadow-sm [a&]:hover:brightness-[1.05]",
        secondary:
          "border border-[color:var(--border-subtle)] bg-[color:var(--surface-card-strong)] text-foreground [a&]:hover:bg-[color:var(--surface-control-hover)]",
        destructive:
          "bg-destructive text-white focus-visible:ring-destructive/25 dark:bg-destructive/70 [a&]:hover:bg-destructive/90",
        outline:
          "border-[color:var(--border-strong)] text-foreground shadow-[inset_0_1px_0_rgba(255,255,255,0.04)] [a&]:hover:bg-[color:var(--surface-hover)]",
        ghost: "[a&]:hover:bg-[color:var(--surface-hover)] [a&]:hover:text-foreground",
        link: "border-0 px-0 text-primary underline-offset-4 [a&]:hover:underline",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
);

function Badge({
  className,
  variant = "default",
  asChild = false,
  ...props
}: React.ComponentProps<"span"> &
  VariantProps<typeof badgeVariants> & { asChild?: boolean }) {
  const Comp = asChild ? Slot.Root : "span";

  return (
    <Comp
      data-slot="badge"
      data-variant={variant}
      className={cn(badgeVariants({ variant }), className)}
      {...props}
    />
  );
}

export { Badge, badgeVariants };
