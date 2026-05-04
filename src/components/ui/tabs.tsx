import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { Tabs as TabsPrimitive } from "radix-ui";

import { cn } from "@/lib/utils";

function Tabs({
  className,
  orientation = "horizontal",
  ...props
}: React.ComponentProps<typeof TabsPrimitive.Root>) {
  return (
    <TabsPrimitive.Root
      data-slot="tabs"
      data-orientation={orientation}
      orientation={orientation}
      className={cn(
        "group/tabs flex gap-3 data-[orientation=horizontal]:flex-col",
        className
      )}
      {...props}
    />
  );
}

const tabsListVariants = cva(
  "group/tabs-list inline-flex min-h-9 w-fit shrink-0 items-center gap-1.5 rounded-full border border-[color:var(--border-subtle)] bg-[color:var(--surface-control)] p-1 shadow-[inset_0_1px_0_rgba(255,255,255,0.04)] backdrop-blur-sm group-data-[orientation=horizontal]/tabs:mx-0 group-data-[orientation=vertical]/tabs:h-fit group-data-[orientation=vertical]/tabs:flex-col",
  {
    variants: {
      variant: {
        pill: "",
        line: "min-h-auto gap-0 rounded-none border-0 bg-transparent p-0 shadow-none backdrop-blur-0 flex-col rounded-none md:flex-row",
      },
    },
    defaultVariants: {
      variant: "pill",
    },
  }
);

function TabsList({
  className,
  variant = "pill",
  ...props
}: React.ComponentProps<typeof TabsPrimitive.List> &
  VariantProps<typeof tabsListVariants>) {
  return (
    <TabsPrimitive.List
      data-slot="tabs-list"
      data-variant={variant}
      className={cn(tabsListVariants({ variant }), className)}
      {...props}
    />
  );
}

function TabsTrigger({
  className,
  ...props
}: React.ComponentProps<typeof TabsPrimitive.Trigger>) {
  return (
    <TabsPrimitive.Trigger
      data-slot="tabs-trigger"
      className={cn(
        "relative inline-flex h-8 flex-1 items-center justify-center gap-1.5 rounded-full px-3 text-[0.8125rem] font-semibold whitespace-nowrap text-muted-foreground transition-[background,color,box-shadow,transform] duration-[var(--ds-dur-fast)] ease-out hover:text-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/55 focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50 dark:text-muted-foreground",
        "data-[state=active]:border data-[state=active]:border-[color:var(--border-subtle)] data-[state=active]:bg-[color:var(--surface-control-hover)] data-[state=active]:text-foreground data-[state=active]:shadow-[inset_0_-1px_0_rgba(0,0,0,0.15)] dark:data-[state=active]:shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]",
        "group-data-[variant=line]/tabs-list:border-0 group-data-[variant=line]/tabs-list:rounded-none group-data-[variant=line]/tabs-list:data-[state=active]:rounded-md group-data-[variant=line]/tabs-list:bg-transparent group-data-[variant=line]/tabs-list:data-[state=active]:bg-transparent group-data-[variant=line]/tabs-list:data-[state=active]:shadow-none group-data-[variant=line]/tabs-list:data-[state=active]:underline group-data-[variant=line]/tabs-list:data-[state=active]:decoration-[color:var(--border-accent)] group-data-[variant=line]/tabs-list:data-[state=active]:decoration-[2px]",
        "[&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
        className
      )}
      {...props}
    />
  );
}

function TabsContent({
  className,
  ...props
}: React.ComponentProps<typeof TabsPrimitive.Content>) {
  return (
    <TabsPrimitive.Content
      data-slot="tabs-content"
      className={cn(
        "mt-4 flex-1 outline-none data-[state=active]:animate-in data-[state=active]:fade-in-0 data-[state=active]:duration-[var(--ds-dur-fast)]",
        className
      )}
      {...props}
    />
  );
}

export { Tabs, TabsList, TabsTrigger, TabsContent, tabsListVariants };
