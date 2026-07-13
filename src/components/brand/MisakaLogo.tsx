import type { ComponentPropsWithoutRef } from "react";
import misakaLogoUrl from "@/assets/brand/misakax-logo.svg";
import { cn } from "@/lib/utils";

type MisakaLogoProps = Omit<ComponentPropsWithoutRef<"img">, "alt" | "src">;

/**
 * MisakaX brand mark sourced from the canonical SVG asset.
 */
export function MisakaLogo({
  className,
  title,
  ...props
}: MisakaLogoProps) {
  return (
    <img
      src={misakaLogoUrl}
      alt={title ?? ""}
      aria-hidden={title ? undefined : true}
      className={cn("shrink-0", className)}
      width={128}
      height={128}
      {...props}
    />
  );
}
