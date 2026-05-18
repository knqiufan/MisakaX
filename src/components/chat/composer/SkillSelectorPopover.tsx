import { ChevronDown, Sparkles } from "lucide-react";
import { cn } from "@/lib/utils";
import { useComposerStore } from "@/stores/composer-store";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

const PLACEHOLDER_SKILLS = [
  { id: "review", name: "Code Review" },
  { id: "debug", name: "Debugging" },
  { id: "docs", name: "Docs Writer" },
];

interface SkillSelectorPopoverProps {
  label: string;
  comingSoonLabel: string;
}

export function SkillSelectorPopover({
  label,
  comingSoonLabel,
}: SkillSelectorPopoverProps) {
  const { selectedSkillIds, setSelectedSkills } = useComposerStore();

  const toggleSkill = (id: string, checked: boolean) => {
    const next = checked
      ? [...selectedSkillIds, id]
      : selectedSkillIds.filter((skillId) => skillId !== id);
    setSelectedSkills(next);
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className={cn(
            "inline-flex items-center gap-1 rounded-full px-2.5 py-1",
            "bg-[color:var(--cm-surface-panel-strong)] text-[11px] text-muted-foreground",
            "transition-colors duration-[var(--ds-dur-fast)]",
            "hover:bg-[color:var(--cm-surface-panel-solid)] hover:text-foreground"
          )}
        >
          <span className="max-w-[120px] truncate">{label}</span>
          <ChevronDown className="size-3 opacity-70" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" side="top" className="w-64">
        <DropdownMenuLabel className="flex items-center gap-2 text-xs text-muted-foreground">
          <Sparkles className="size-3.5" />
          {label}
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <div className="px-2 py-2 text-xs text-muted-foreground">
          {comingSoonLabel}
        </div>
        {PLACEHOLDER_SKILLS.map((skill) => (
          <DropdownMenuCheckboxItem
            key={skill.id}
            checked={selectedSkillIds.includes(skill.id)}
            disabled
            onCheckedChange={(checked) => toggleSkill(skill.id, checked === true)}
          >
            {skill.name}
          </DropdownMenuCheckboxItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
