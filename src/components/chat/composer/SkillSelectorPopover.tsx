import { useMemo, useState } from "react";
import { ChevronDown, Search, Sparkles } from "lucide-react";
import { cn } from "@/lib/utils";
import { useComposerStore } from "@/stores/composer-store";
import { collectSkills, selectableSkills } from "./composerSegment";
import { useSkillsInventory } from "@/components/skills/useSkillsInventory";
import { Input } from "@/components/ui/input";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

interface SkillSelectorPopoverProps {
  label: string;
}

export function SkillSelectorPopover({
  label,
}: SkillSelectorPopoverProps) {
  const { skills } = useSkillsInventory();
  const { segments, insertInlineSkill, removeSkill } = useComposerStore();
  const [query, setQuery] = useState("");
  const selected = collectSkills(segments);
  const enabledSkills = useMemo(
    () => selectableSkills(skills),
    [skills]
  );
  const matches = enabledSkills.filter((skill) =>
    [skill.name, skill.slug, skill.description]
      .join(" ")
      .toLocaleLowerCase()
      .includes(query.trim().toLocaleLowerCase())
  );

  const toggleSkill = (skillId: string, checked: boolean) => {
    const current = selected.find((skill) => skill.skillId === skillId);
    if (checked && !current) {
      const skill = enabledSkills.find((item) => item.skill_id === skillId);
      if (skill) insertInlineSkill({ id: crypto.randomUUID(), skillId, slug: skill.slug, name: skill.name, description: skill.description });
    }
    if (!checked && current) {
      removeSkill(current.id);
    }
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className={cn(
            "inline-flex items-center gap-1 rounded-full px-2.5 py-1",
            "bg-[color:var(--surface-card-strong)] text-[11px] text-muted-foreground",
            "transition-colors duration-[var(--ds-dur-fast)]",
            "hover:bg-[color:var(--surface-card-strong)] hover:text-foreground"
          )}
        >
          <span className="max-w-[120px] truncate">{label}</span>
          <ChevronDown className="size-3 opacity-70" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" side="top" className="w-72">
        <DropdownMenuLabel className="flex items-center gap-2 text-xs text-muted-foreground">
          <Sparkles className="size-3.5" />
          {label}
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <div className="relative px-1 pb-1">
          <Search className="pointer-events-none absolute top-1/2 left-3 size-3 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={label}
            className="h-8 pl-8 text-xs"
          />
        </div>
        <div className="max-h-56 overflow-y-auto">
        {matches.map((skill) => (
          <DropdownMenuCheckboxItem
            key={skill.skill_id}
            checked={selected.some((item) => item.skillId === skill.skill_id)}
            onCheckedChange={(checked) => toggleSkill(skill.skill_id, checked === true)}
          >
            <span className="min-w-0"><span className="block truncate">{skill.name}</span><span className="block truncate text-[10px] text-muted-foreground">{skill.description}</span></span>
          </DropdownMenuCheckboxItem>
        ))}
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
