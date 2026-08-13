import { useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import type { EChartsOption } from "echarts";

import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

interface EChartCanvasProps {
  option: unknown;
  ariaLabel: string;
  className?: string;
  fallback: ReactNode;
  onError?: () => void;
}

function cssVariableName(value: string): string | null {
  const match = /^var\((--[^),\s]+)(?:,[^)]+)?\)$/.exec(value.trim());
  return match?.[1] ?? null;
}

export function resolveChartCssVariables<T>(value: T, styles: CSSStyleDeclaration): T {
  if (typeof value === "string") {
    const variableName = cssVariableName(value);
    if (!variableName) return value;
    const resolved = styles.getPropertyValue(variableName).trim();
    return (resolved || value) as T;
  }
  if (Array.isArray(value)) {
    return value.map((item) => resolveChartCssVariables(item, styles)) as T;
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [
        key,
        resolveChartCssVariables(item, styles),
      ])
    ) as T;
  }
  return value;
}

export function EChartCanvas({
  option,
  ariaLabel,
  className,
  fallback,
  onError,
}: EChartCanvasProps) {
  const elementRef = useRef<HTMLDivElement>(null);
  const [loading, setLoading] = useState(true);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;
    let disposed = false;
    let instance: {
      resize: () => void;
      dispose: () => void;
      setOption: (nextOption: EChartsOption, settings?: object) => void;
    } | null = null;
    let resizeObserver: ResizeObserver | null = null;
    let themeObserver: MutationObserver | null = null;

    setLoading(true);
    setFailed(false);
    void import("echarts")
      .then((echarts) => {
        if (disposed) return;
        const chart = echarts.init(element, undefined, { renderer: "canvas" });
        const applyOption = () => {
          const resolved = resolveChartCssVariables(
            option,
            getComputedStyle(element)
          ) as EChartsOption;
          chart.setOption(resolved, { notMerge: true, lazyUpdate: true });
        };
        applyOption();
        instance = chart;
        if (typeof ResizeObserver !== "undefined") {
          resizeObserver = new ResizeObserver(() => chart.resize());
          resizeObserver.observe(element);
        }
        if (typeof MutationObserver !== "undefined") {
          themeObserver = new MutationObserver(applyOption);
          themeObserver.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ["class", "style"],
          });
        }
        setLoading(false);
      })
      .catch(() => {
        if (disposed) return;
        setLoading(false);
        setFailed(true);
        onError?.();
      });

    return () => {
      disposed = true;
      resizeObserver?.disconnect();
      themeObserver?.disconnect();
      instance?.dispose();
    };
  }, [onError, option]);

  if (failed) return <>{fallback}</>;

  return (
    <div className={cn("relative min-w-0", className)} data-chart-state={loading ? "loading" : "ready"}>
      {loading ? <Skeleton className="absolute inset-0" /> : null}
      <div
        ref={elementRef}
        className="size-full min-w-0"
        role="img"
        aria-label={ariaLabel}
        aria-busy={loading}
      />
    </div>
  );
}
