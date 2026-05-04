import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex shrink-0 items-center justify-center gap-2 whitespace-nowrap rounded-[var(--radius-button)] font-medium transition-[transform,box-shadow,background,color,border-color,filter] duration-[var(--ds-dur-fast)] ease-out hover:brightness-[1.02] disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 [&_svg]:shrink-0 outline-none focus-visible:border-ring focus-visible:ring-ring/55 focus-visible:ring-[3px] aria-invalid:ring-destructive/30 dark:aria-invalid:ring-destructive/50 aria-invalid:border-destructive active:scale-[var(--ds-active-scale)] active:transition-none",
  {
    variants: {
      variant: {
        default:
          "border-0 bg-gradient-to-br from-[#62b7ff] to-[#4fe3a3] px-4 py-2 text-sm font-semibold text-[#0b0f1a] shadow-md hover:-translate-y-px hover:shadow-lg active:!brightness-[0.92]",
        destructive:
          "border-0 bg-destructive px-4 py-2 text-sm font-medium text-white shadow-xs hover:bg-destructive/90 focus-visible:ring-destructive/35 dark:focus-visible:ring-destructive/45 dark:bg-destructive/72",
        outline:
          "border border-[color:var(--border-strong)] bg-background/70 px-4 py-2 text-sm shadow-xs hover:bg-accent hover:text-accent-foreground dark:bg-transparent dark:hover:bg-input/45",
        secondary:
          "border-0 bg-secondary px-4 py-2 text-sm shadow-[inset_0_1px_0_rgba(255,255,255,0.06)] hover:bg-secondary/85",
        ghost:
          "border border-transparent bg-transparent px-4 py-2 text-sm text-muted-foreground hover:border-[color:var(--border-strong)] hover:bg-[color:var(--surface-hover)] hover:text-foreground dark:hover:bg-white/8",
        link: "scale-100 border-0 px-2 py-1 text-primary underline-offset-4 hover:underline hover:brightness-110 active:scale-100",
      },
      size: {
        default: "h-9 px-4 py-2 text-sm has-[>svg]:px-3 min-h-9",
        sm: "h-8 gap-2 rounded-[var(--radius-ui-sm)] px-4 py-0 text-xs has-[>svg]:px-2",
        lg: "h-10 px-8 text-sm",
        icon: "size-9 active:scale-[var(--ds-active-scale-sm)]",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

function Button({
  className,
  variant,
  size,
  asChild = false,
  ...props
}: React.ComponentProps<"button"> &
  VariantProps<typeof buttonVariants> & {
    asChild?: boolean;
  }) {
  const Comp = asChild ? Slot : "button";

  return (
    <Comp
      type={asChild ? undefined : "button"}
      data-slot="button"
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  );
}

export { Button, buttonVariants };
