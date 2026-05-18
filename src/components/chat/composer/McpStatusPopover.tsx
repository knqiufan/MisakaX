import { useEffect, useMemo, useState } from "react";
import { ChevronDown, Wrench } from "lucide-react";
import { mcpIpc } from "@/lib/ipc";
import type { McpServerInfo, McpToolInfo } from "@/lib/ipc";
import { cn } from "@/lib/utils";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ScrollArea } from "@/components/ui/scroll-area";

interface McpStatusPopoverProps {
  label: string;
  emptyLabel: string;
}

export function McpStatusPopover({
  label,
  emptyLabel,
}: McpStatusPopoverProps) {
  const [open, setOpen] = useState(false);
  const [servers, setServers] = useState<McpServerInfo[]>([]);
  const [tools, setTools] = useState<McpToolInfo[]>([]);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    Promise.all([mcpIpc.listServers(), mcpIpc.listTools()])
      .then(([serverList, toolList]) => {
        if (cancelled) return;
        setServers(serverList);
        setTools(toolList);
      })
      .catch(console.error);
    return () => {
      cancelled = true;
    };
  }, [open]);

  const connectedServers = useMemo(
    () => servers.filter((server) => server.status === "connected"),
    [servers]
  );

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownMenuTrigger asChild>
        <FooterTrigger label={`${label}${connectedServers.length > 0 ? ` (${connectedServers.length})` : ""}`} />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" side="top" className="w-80">
        <DropdownMenuLabel className="flex items-center gap-2 text-xs text-muted-foreground">
          <Wrench className="size-3.5" />
          {label}
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        {connectedServers.length === 0 ? (
          <div className="px-2 py-6 text-center text-xs text-muted-foreground">
            {emptyLabel}
          </div>
        ) : (
          <ScrollArea className="max-h-72">
            <div className="space-y-2 p-1">
              {connectedServers.map((server) => (
                <ServerToolGroup
                  key={server.id}
                  server={server}
                  tools={tools.filter((tool) => tool.server_id === server.id)}
                />
              ))}
            </div>
          </ScrollArea>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function ServerToolGroup({
  server,
  tools,
}: {
  server: McpServerInfo;
  tools: McpToolInfo[];
}) {
  return (
    <section className="rounded-[var(--radius-ui-md)] border border-[color:var(--border-muted)] bg-[color:var(--surface-card)] p-2">
      <div className="mb-1 flex items-center justify-between gap-2">
        <p className="truncate text-xs font-medium text-foreground">{server.name}</p>
        <span className="shrink-0 text-[10px] text-muted-foreground">
          {tools.length}
        </span>
      </div>
      <div className="space-y-1">
        {tools.map((tool) => (
          <div key={`${tool.server_id}:${tool.name}`} className="rounded-sm px-1.5 py-1">
            <p className="truncate text-[11px] text-foreground">{tool.name}</p>
            {tool.description ? (
              <p className="line-clamp-2 text-[10px] leading-snug text-muted-foreground">
                {tool.description}
              </p>
            ) : null}
          </div>
        ))}
      </div>
    </section>
  );
}

function FooterTrigger({ label }: { label: string }) {
  return (
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
  );
}
