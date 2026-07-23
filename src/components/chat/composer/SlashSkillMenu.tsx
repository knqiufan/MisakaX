import { useEffect, useRef } from "react";
import { Sparkles } from "lucide-react";
import { usePresence } from "@/hooks/usePresence";
import { cn } from "@/lib/utils";
import type { InstalledSkill } from "@/lib/ipc";

interface SlashSkillMenuProps {
  open: boolean;
  query: string;
  skills: InstalledSkill[];
  label: string;
  activeIndex: number;
  onActiveIndexChange: (index: number) => void;
  onSelect: (skill: InstalledSkill) => void;
}

export function SlashSkillMenu({
  open,
  query,
  skills,
  label,
  activeIndex,
  onActiveIndexChange,
  onSelect,
}: SlashSkillMenuProps) {
  const { mounted, exiting, onExitComplete } = usePresence(open);
  const activeRef = useRef<HTMLButtonElement>(null);
  useEffect(() => activeRef.current?.scrollIntoView({ block: "nearest" }), [activeIndex]);
  if (!mounted) return null;
  return (
    <div
      data-state={exiting ? "closed" : "open"}
      onAnimationEnd={(event) => {
        if (event.target === event.currentTarget && exiting) onExitComplete();
      }}
      className={cn(
        "absolute bottom-full left-0 z-50 mb-2 w-[min(360px,calc(100vw-2rem))]",
        "overflow-hidden rounded-xl border border-[color:var(--border-strong)] bg-popover/95 p-1 shadow-lg backdrop-blur-md",
        "duration-150 ease-out data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:slide-in-from-bottom-1",
        "data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:slide-out-to-bottom-1"
      )}
      role="listbox"
      aria-label={label}
    >
      <div className="flex items-center gap-2 px-2 py-1.5 text-[11px] text-muted-foreground">
        <Sparkles className="size-3.5" />/{query}
      </div>
      <div className="max-h-56 overflow-y-auto">
        {skills.map((skill, index) => (
          <button
            key={skill.slug}
            ref={index === activeIndex ? activeRef : undefined}
            type="button"
            role="option"
            aria-selected={index === activeIndex}
            onMouseEnter={() => onActiveIndexChange(index)}
            onClick={() => onSelect(skill)}
            className={cn(
              "flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left transition-colors duration-150",
              index === activeIndex ? "bg-accent text-accent-foreground" : "hover:bg-accent/60"
            )}
          >
            <Sparkles className="size-3.5 shrink-0 text-primary" />
            <span className="min-w-0"><span className="block truncate text-xs font-medium">{skill.name}</span><span className="mt-0.5 block truncate text-[10px] text-muted-foreground">{skill.description}</span></span>
          </button>
        ))}
      </div>
    </div>
  );
}
