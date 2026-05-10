import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Plug,
  Plus,
  RefreshCw,
  Trash2,
  Power,
  PowerOff,
  ChevronRight,
  Wrench,
  ShieldCheck,
  RotateCcw,
} from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { mcpIpc } from "@/lib/ipc/mcp";
import type {
  McpServerInfo,
  McpToolInfo,
  McpServerStatusType,
  ToolPermission,
} from "@/lib/ipc";
import type { McpTransportInput } from "@/lib/ipc/mcp";

export function McpSettings() {
  const { t } = useTranslation("settings");
  const [servers, setServers] = useState<McpServerInfo[]>([]);
  const [tools, setTools] = useState<McpToolInfo[]>([]);
  const [permissions, setPermissions] = useState<ToolPermission[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const [s, tl, p] = await Promise.all([
        mcpIpc.listServers(),
        mcpIpc.listTools(),
        mcpIpc.listPermissions(),
      ]);
      setServers(s);
      setTools(tl);
      setPermissions(p);
    } catch (err) {
      console.error("Failed to load MCP data:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    const unlisten = listen("mcp:server_status", () => {
      refresh();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [refresh]);

  return (
    <div className="w-full min-w-0 space-y-6">
      <SectionHeader
        title={t("mcp.servers")}
        action={
          <AddServerDialog onAdded={refresh} />
        }
      />

      {loading && servers.length === 0 ? (
        <LoadingPlaceholder />
      ) : servers.length === 0 ? (
        <EmptyState message={t("mcp.noServers")} />
      ) : (
        <div className="space-y-2">
          {servers.map((server) => (
            <ServerCard
              key={server.id}
              server={server}
              tools={tools.filter((tl) => tl.server_id === server.id)}
              onRefresh={refresh}
            />
          ))}
        </div>
      )}

      {permissions.length > 0 && (
        <>
          <SectionHeader title={t("mcp.permissions")} />
          <PermissionsList permissions={permissions} onRefresh={refresh} />
        </>
      )}
    </div>
  );
}

// ─── Server Card ──────────────────────────────────────────────────────

function ServerCard({
  server,
  tools,
  onRefresh,
}: {
  server: McpServerInfo;
  tools: McpToolInfo[];
  onRefresh: () => void;
}) {
  const { t } = useTranslation("settings");
  const [expanded, setExpanded] = useState(false);
  const [busy, setBusy] = useState(false);

  const isConnected = server.status === "connected";

  const handleConnect = async () => {
    setBusy(true);
    try {
      await mcpIpc.restartServer(server.id);
      onRefresh();
    } catch (err) {
      console.error("Connect failed:", err);
    } finally {
      setBusy(false);
    }
  };

  const handleDisconnect = async () => {
    setBusy(true);
    try {
      await mcpIpc.disconnectServer(server.id);
      onRefresh();
    } catch (err) {
      console.error("Disconnect failed:", err);
    } finally {
      setBusy(false);
    }
  };

  const handleRestart = async () => {
    setBusy(true);
    try {
      await mcpIpc.restartServer(server.id);
      onRefresh();
    } catch (err) {
      console.error("Restart failed:", err);
    } finally {
      setBusy(false);
    }
  };

  const handleDelete = async () => {
    setBusy(true);
    try {
      await mcpIpc.removeServerConfig(server.id);
      onRefresh();
    } catch (err) {
      console.error("Delete failed:", err);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div
      className={cn(
        "rounded-[var(--radius-ui-lg)] border bg-[color:var(--surface-card)] px-4 py-3",
        isConnected
          ? "border-emerald-500/20"
          : "border-[color:var(--border-muted)]"
      )}
    >
      <div className="flex items-center gap-3">
        <StatusDot status={server.status} />
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-medium text-foreground">
            {server.name}
          </p>
          <p className="text-xs text-muted-foreground">
            {server.transport_type} · {server.tools_count} {t("mcp.tools")}
          </p>
        </div>
        <div className="flex items-center gap-1">
          {isConnected ? (
            <IconBtn
              icon={PowerOff}
              label={t("mcp.disconnect")}
              onClick={handleDisconnect}
              disabled={busy}
            />
          ) : (
            <IconBtn
              icon={Power}
              label={t("mcp.connect")}
              onClick={handleConnect}
              disabled={busy}
            />
          )}
          <IconBtn
            icon={RefreshCw}
            label={t("mcp.restart")}
            onClick={handleRestart}
            disabled={busy}
          />
          <IconBtn
            icon={Trash2}
            label={t("mcp.delete")}
            onClick={handleDelete}
            disabled={busy}
            destructive
          />
        </div>
      </div>

      {tools.length > 0 && (
        <Collapsible open={expanded} onOpenChange={setExpanded}>
          <CollapsibleTrigger
            className={cn(
              "mt-2 flex items-center gap-1.5 rounded-md px-2 py-1",
              "text-xs text-muted-foreground/70",
              "transition-colors duration-[var(--ds-dur-fast)]",
              "hover:bg-[color:var(--surface-hover)] hover:text-muted-foreground"
            )}
          >
            <Wrench className="size-3" />
            <span>{t("mcp.viewTools")}</span>
            <ChevronRight
              className={cn(
                "size-3 transition-transform duration-[var(--ds-dur-fast)]",
                expanded && "rotate-90"
              )}
            />
          </CollapsibleTrigger>
          <CollapsibleContent>
            <div className="mt-2 space-y-1 pl-2">
              {tools.map((tool) => (
                <ToolRow key={`${tool.server_id}-${tool.name}`} tool={tool} />
              ))}
            </div>
          </CollapsibleContent>
        </Collapsible>
      )}
    </div>
  );
}

function ToolRow({ tool }: { tool: McpToolInfo }) {
  return (
    <div className="flex items-start gap-2 rounded-md px-2 py-1.5 text-xs">
      <Wrench className="mt-0.5 size-3 shrink-0 text-muted-foreground/50" />
      <div className="min-w-0">
        <span className="font-mono font-medium text-foreground/90">
          {tool.name}
        </span>
        {tool.description && (
          <p className="mt-0.5 text-muted-foreground/70">{tool.description}</p>
        )}
      </div>
    </div>
  );
}

// ─── Add Server Dialog ────────────────────────────────────────────────

function AddServerDialog({ onAdded }: { onAdded: () => void }) {
  const { t } = useTranslation("settings");
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [transportType, setTransportType] = useState<"stdio" | "http" | "sse">(
    "stdio"
  );
  const [command, setCommand] = useState("");
  const [args, setArgs] = useState("");
  const [url, setUrl] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const reset = () => {
    setName("");
    setTransportType("stdio");
    setCommand("");
    setArgs("");
    setUrl("");
  };

  const handleSubmit = async () => {
    if (!name.trim()) return;
    setSubmitting(true);

    let transport: McpTransportInput;
    if (transportType === "stdio") {
      if (!command.trim()) return;
      transport = {
        type: "stdio",
        command: command.trim(),
        args: args
          .split(/\s+/)
          .filter(Boolean),
      };
    } else {
      if (!url.trim()) return;
      transport = { type: transportType, url: url.trim() };
    }

    const config = {
      id: crypto.randomUUID(),
      name: name.trim(),
      transport,
      auto_connect: true,
      env: {},
    };

    try {
      await mcpIpc.addServerConfig(config);
      await mcpIpc.connectServer(config);
      onAdded();
      setOpen(false);
      reset();
    } catch (err) {
      console.error("Add server failed:", err);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm">
          <Plus className="mr-1.5 size-3.5" />
          {t("mcp.addServer")}
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("mcp.addServer")}</DialogTitle>
          <DialogDescription>{t("mcp.addServerDesc")}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label>{t("mcp.serverName")}</Label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t("mcp.serverNamePlaceholder")}
            />
          </div>

          <div className="space-y-2">
            <Label>{t("mcp.transportType")}</Label>
            <div className="flex gap-2">
              {(["stdio", "http", "sse"] as const).map((tt) => (
                <button
                  key={tt}
                  type="button"
                  onClick={() => setTransportType(tt)}
                  className={cn(
                    "rounded-[var(--radius-button)] border px-3 py-1.5 text-xs font-medium transition-colors duration-[var(--ds-dur-fast)]",
                    transportType === tt
                      ? "border-primary bg-primary/10 text-primary"
                      : "border-[color:var(--border-muted)] text-muted-foreground hover:bg-[color:var(--surface-hover)]"
                  )}
                >
                  {tt.toUpperCase()}
                </button>
              ))}
            </div>
          </div>

          {transportType === "stdio" ? (
            <>
              <div className="space-y-2">
                <Label>{t("mcp.command")}</Label>
                <Input
                  value={command}
                  onChange={(e) => setCommand(e.target.value)}
                  placeholder="npx -y @modelcontextprotocol/server-filesystem"
                  className="font-mono text-xs"
                />
              </div>
              <div className="space-y-2">
                <Label>{t("mcp.arguments")}</Label>
                <Input
                  value={args}
                  onChange={(e) => setArgs(e.target.value)}
                  placeholder="/path/to/directory"
                  className="font-mono text-xs"
                />
              </div>
            </>
          ) : (
            <div className="space-y-2">
              <Label>{t("mcp.url")}</Label>
              <Input
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="http://localhost:3000/mcp"
                className="font-mono text-xs"
              />
            </div>
          )}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setOpen(false)}
          >
            {t("mcp.cancel")}
          </Button>
          <Button
            size="sm"
            onClick={handleSubmit}
            disabled={submitting || !name.trim()}
          >
            {submitting ? t("mcp.adding") : t("mcp.add")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

// ─── Permissions List ─────────────────────────────────────────────────

function PermissionsList({
  permissions,
  onRefresh,
}: {
  permissions: ToolPermission[];
  onRefresh: () => void;
}) {
  const { t } = useTranslation("settings");

  const handleReset = async (serverId: string, toolName: string) => {
    try {
      await mcpIpc.resetPermission(serverId, toolName);
      onRefresh();
    } catch (err) {
      console.error("Reset permission failed:", err);
    }
  };

  return (
    <div className="space-y-1">
      {permissions.map((perm) => (
        <div
          key={`${perm.server_id}-${perm.tool_name}`}
          className="flex items-center gap-3 rounded-[var(--radius-ui-md)] border border-[color:var(--border-muted)] bg-[color:var(--surface-card)] px-3 py-2"
        >
          <ShieldCheck className="size-3.5 shrink-0 text-muted-foreground/60" />
          <div className="min-w-0 flex-1">
            <span className="font-mono text-xs text-foreground/90">
              {perm.tool_name}
            </span>
            <span className="ml-2 text-xs text-muted-foreground/60">
              ({perm.server_id})
            </span>
          </div>
          <span
            className={cn(
              "rounded-full px-2 py-0.5 text-[0.625rem] font-medium",
              perm.policy === "allow"
                ? "bg-emerald-500/10 text-emerald-500"
                : "bg-destructive/10 text-destructive"
            )}
          >
            {perm.policy}
          </span>
          <button
            type="button"
            onClick={() => handleReset(perm.server_id, perm.tool_name)}
            title={t("mcp.resetPermission")}
            className={cn(
              "inline-flex size-6 items-center justify-center rounded-md",
              "text-muted-foreground/60",
              "transition-colors duration-[var(--ds-dur-fast)]",
              "hover:bg-[color:var(--surface-hover)] hover:text-foreground"
            )}
          >
            <RotateCcw className="size-3" />
          </button>
        </div>
      ))}
    </div>
  );
}

// ─── Shared Components ────────────────────────────────────────────────

function SectionHeader({
  title,
  action,
}: {
  title: string;
  action?: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between">
      <h3 className="text-sm font-semibold text-foreground">{title}</h3>
      {action}
    </div>
  );
}

function StatusDot({ status }: { status: McpServerStatusType }) {
  const color =
    status === "connected"
      ? "bg-emerald-500"
      : status === "connecting"
        ? "bg-amber-400 animate-pulse"
        : "bg-muted-foreground/40";

  return <span className={cn("size-2 shrink-0 rounded-full", color)} />;
}

function IconBtn({
  icon: Icon,
  label,
  onClick,
  disabled,
  destructive,
}: {
  icon: typeof Plug;
  label: string;
  onClick: () => void;
  disabled?: boolean;
  destructive?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      title={label}
      aria-label={label}
      disabled={disabled}
      className={cn(
        "inline-flex size-7 items-center justify-center rounded-md",
        "transition-colors duration-[var(--ds-dur-fast)]",
        "disabled:pointer-events-none disabled:opacity-40",
        destructive
          ? "text-muted-foreground/60 hover:bg-destructive/10 hover:text-destructive"
          : "text-muted-foreground/60 hover:bg-[color:var(--surface-hover)] hover:text-foreground"
      )}
    >
      <Icon className="size-3.5" />
    </button>
  );
}

function LoadingPlaceholder() {
  return (
    <div className="flex min-h-32 items-center justify-center">
      <RefreshCw className="size-5 animate-spin text-muted-foreground/40" />
    </div>
  );
}

function EmptyState({ message }: { message: string }) {
  return (
    <div className="flex min-h-32 flex-col items-center justify-center gap-2 rounded-[var(--radius-ui-lg)] border border-dashed border-[color:var(--border-strong)] bg-[color:var(--surface-card)]/40 px-4 py-6">
      <Plug className="h-8 w-8 text-muted-foreground/40" />
      <p className="text-sm text-muted-foreground">{message}</p>
    </div>
  );
}
