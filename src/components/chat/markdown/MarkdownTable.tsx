import {
  useCallback,
  useRef,
  type ComponentPropsWithoutRef,
  type ReactNode,
} from "react";
import { Copy, ImageIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ExtraProps } from "streamdown";

type TableProps = ComponentPropsWithoutRef<"table"> & ExtraProps;

function extractMarkdownTable(table: HTMLTableElement): string {
  const rows = Array.from(table.querySelectorAll("tr"));
  return rows
    .map((row) => {
      const cells = Array.from(row.querySelectorAll("th,td")).map((cell) =>
        cell.textContent?.replace(/\|/g, "\\|").trim() ?? ""
      );
      return `| ${cells.join(" | ")} |`;
    })
    .join("\n");
}

/** Table card with toolbar (docs/ui/06 §4). */
export function MarkdownTable({
  children,
  className,
  ...props
}: TableProps) {
  const tableRef = useRef<HTMLTableElement>(null);

  const copyMarkdown = useCallback(() => {
    const el = tableRef.current;
    if (!el) return;
    const md = extractMarkdownTable(el);
    void navigator.clipboard.writeText(md);
  }, []);

  return (
    <div className="my-4 overflow-hidden rounded-xl bg-muted/20">
      <div className="flex items-center justify-end gap-1 bg-muted/30 px-2 py-1">
        <button
          type="button"
          onClick={copyMarkdown}
          aria-label="Copy table as Markdown"
          title="Copy Markdown"
          className={cn(
            "inline-flex h-7 w-7 items-center justify-center rounded-md",
            "text-xs text-muted-foreground transition-colors duration-[var(--ds-dur-fast)]",
            "hover:bg-muted hover:text-foreground"
          )}
        >
          <Copy className="size-3.5" />
        </button>
        <button
          type="button"
          disabled
          aria-label="Export PNG (coming soon)"
          title="Export PNG"
          className={cn(
            "inline-flex h-7 w-7 items-center justify-center rounded-md",
            "text-xs text-muted-foreground/40"
          )}
        >
          <ImageIcon className="size-3.5" />
        </button>
      </div>
      <div className="overflow-x-auto px-3 pb-3 pt-2">
        <table
          ref={tableRef}
          className={cn("w-full border-collapse text-sm", className)}
          {...props}
        >
          {children as ReactNode}
        </table>
      </div>
    </div>
  );
}
