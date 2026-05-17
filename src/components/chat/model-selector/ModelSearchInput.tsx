import { Search, X } from "lucide-react";
import { useEffect, useRef } from "react";
import { cn } from "@/lib/utils";

interface ModelSearchInputProps {
  value: string;
  placeholder: string;
  clearLabel: string;
  onChange: (value: string) => void;
}

export function ModelSearchInput({
  value,
  placeholder,
  clearLabel,
  onChange,
}: ModelSearchInputProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <div className="relative">
      <Search className="absolute left-3 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
      <input
        ref={inputRef}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        className={cn(
          "h-8 w-full rounded-[var(--radius-ui-md)] border",
          "border-[color:var(--border-strong)] bg-[color:var(--surface-control)]/55",
          "px-8 text-xs text-foreground outline-none",
          "placeholder:text-muted-foreground/65",
          "focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/35",
        )}
      />
      {value && (
        <button
          type="button"
          aria-label={clearLabel}
          onClick={() => onChange("")}
          className="absolute right-2 top-1/2 -translate-y-1/2 rounded-md p-1 text-muted-foreground hover:bg-[color:var(--surface-hover)] hover:text-foreground"
        >
          <X className="size-3" />
        </button>
      )}
    </div>
  );
}
