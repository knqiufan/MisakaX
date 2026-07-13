import type { SVGProps } from "react";
import { cn } from "@/lib/utils";

/** 5×5 diffusion-dot brand mark (docs/ui/01 §8.3). */
export function MonolithIcon({
  className,
  ...props
}: SVGProps<SVGSVGElement>) {
  const opacities = [1, 0.82, 0.58, 0.34, 0.1] as const;
  const cells: { cx: number; cy: number; opacity: number }[] = [];

  for (let row = 0; row < 5; row += 1) {
    for (let col = 0; col < 5; col += 1) {
      const dist = Math.max(Math.abs(row - 2), Math.abs(col - 2));
      cells.push({
        cx: col * 119 + 59.5,
        cy: row * 119 + 59.5,
        opacity: opacities[dist] ?? 0.1,
      });
    }
  }

  return (
    <svg
      viewBox="-30 -30 655 655"
      fill="currentColor"
      aria-hidden
      className={cn("shrink-0", className)}
      {...props}
    >
      {cells.map((cell) => (
        <circle
          key={`${cell.cx}-${cell.cy}`}
          cx={cell.cx}
          cy={cell.cy}
          r={42}
          opacity={cell.opacity}
        />
      ))}
    </svg>
  );
}
