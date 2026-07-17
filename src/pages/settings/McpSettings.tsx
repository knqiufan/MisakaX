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
import {
  SettingsCard,
  SettingsSectionHeader,
  SettingsPartitionLabel,
  StatusPill,
} from "@/components/settings";
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
    <div className="space-y-6">
      <SettingsSectionHeader
        title={t("mcp.title")}
        action={<AddServerDialog onAdded={refresh} />}
      />

      <div className="space-y-3">
        <SettingsPartitionLabel>{t("mcp.servers")}</SettingsPartitionLabel>
        {loading && servers.length === 0 ? (
          <LoadingPlaceholder />
        ) : servers.length === 0 ? (
          <EmptyState message={t("mcp.noServers")} />
        ) : (
          <div className="space-y-3">
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
      </div>

      {permissions.length > 0 ? (
        <div className="space-y-3">
          <SettingsPartitionLabel>
            {t("mcp.permissions")}
          </SettingsPartitionLabel>
          <PermissionsList permissions={permissions} onRefresh={refresh} />
        </div>
      ) : null}
    </div>
  );
}

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
    <SettingsCard compact>
      <div className="flex items-center gap-3">
        <div className="min-w-0 flex-1 space-y-1">
          <div className="flex flex-wrap items-center gap-2">
            <p className="truncate text-sm font-medium text-foreground">
              {server.name}
            </p>
            <StatusPill
              tone={statusTone(server.status)}
              label={formatServerStatus(server.status)}
            />
          </div>
          <p className="text-xs text-muted-foreground">
            {server.transport_type} · {server.tools_count} {t("mcp.tools")}
          </p>
        </div>
        <div className="flex items-center gap-0.5">
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

      {tools.length > 0 ? (
        <Collapsible open={expanded} onOpenChange={setExpanded}>
          <CollapsibleTrigger
            className={cn(
              "mt-3 flex items-center gap-1.5 rounded-md px-2 py-1",
              "text-xs text-muted-foreground",
              "transition-colors duration-[var(--ds-dur-fast)]",
              "hover:bg-muted/40 hover:text-foreground"
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
            <div className="mt-2 space-y-1 rounded-md bg-muted/40 px-2 py-1">
              {tools.map((tool) => (
                <ToolRow key={`${tool.server_id}-${tool.name}`} tool={tool} />
              ))}
            </div>
          </CollapsibleContent>
        </Collapsible>
      ) : null}
    </SettingsCard>
  );
}

function statusTone(
  status: McpServerStatusType
): "available" | "needs-config" | "error" | "unknown" {
  if (status === "connected") return "available";
  if (status === "connecting") return "needs-config";
  if (typeof status === "object" && "error" in status) return "error";
  return "unknown";
}

function formatServerStatus(status: McpServerStatusType): string {
  if (typeof status === "object" && "error" in status) {
    return status.error || "error";
  }
  return status;
}

function ToolRow({ tool }: { tool: McpToolInfo }) {
  return (
    <div className="flex items-start gap-2 rounded-md px-2 py-1.5 text-xs">
      <Wrench className="mt-0.5 size-3 shrink-0 text-muted-foreground/50" />
      <div className="min-w-0">
        <span className="font-mono font-medium text-foreground/90">
          {tool.name}
        </span>
        {tool.description ? (
          <p className="mt-0.5 text-muted-foreground/70">{tool.description}</p>
        ) : null}
      </div>
    </div>
  );
}

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
  const [headers, setHeaders] = useState("{}");
  const [configMode, setConfigMode] = useState<"form" | "json">("form");
  const [jsonConfig, setJsonConfig] = useState("");
  const [configError, setConfigError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const reset = () => {
    setName("");
    setTransportType("stdio");
    setCommand("");
    setArgs("");
    setUrl("");
    setHeaders("{}");
    setConfigMode("form");
    setJsonConfig("");
    setConfigError(null);
  };

  const handleSubmit = async () => {
    let config: { id: string; name: string; transport: McpTransportInput; auto_connect: boolean; env: Record<string, string> };
    try {
      config = configMode === "json"
        ? parseJsonServerConfig(jsonConfig)
        : createFormServerConfig({ name, transportType, command, args, url, headers });
      setConfigError(null);
    } catch (error) {
      setConfigError(String(error));
      return;
    }

    setSubmitting(true);

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
            <Label>{t("mcp.configMode")}</Label>
            <div className="inline-flex rounded-md bg-muted p-0.5">
              {(["form", "json"] as const).map((mode) => (
                <button
                  key={mode}
                  type="button"
                  onClick={() => {
                    setConfigMode(mode);
                    setConfigError(null);
                  }}
                  className={cn(
                    "cursor-pointer rounded-sm px-2.5 py-1 text-xs transition-colors duration-[var(--ds-dur-fast)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/45",
                    configMode === mode
                      ? "bg-background text-foreground shadow-sm"
                      : "text-muted-foreground hover:text-foreground",
                  )}
                  aria-pressed={configMode === mode}
                >
                  {t(mode === "form" ? "mcp.formMode" : "mcp.jsonMode")}
                </button>
              ))}
            </div>
          </div>

          {configMode === "json" ? (
            <div className="space-y-2">
              <Label>{t("mcp.jsonConfig")}</Label>
              <textarea
                value={jsonConfig}
                onChange={(event) => setJsonConfig(event.target.value)}
                placeholder={t("mcp.jsonConfigPlaceholder")}
                spellCheck={false}
                className="min-h-56 w-full rounded-[var(--radius-ui-md)] border border-[color:var(--border-strong)] bg-[color:var(--surface-item)] px-3 py-2 font-mono text-xs outline-none transition-[color,box-shadow] duration-[var(--ds-dur-fast)] focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/45"
              />
            </div>
          ) : (
            <>
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
            <div className="inline-flex rounded-md bg-muted p-0.5">
              {(["stdio", "http", "sse"] as const).map((tt) => (
                <button
                  key={tt}
                  type="button"
                  onClick={() => setTransportType(tt)}
                  className={cn(
                    "cursor-pointer rounded-sm px-2.5 py-1 text-xs transition-colors duration-[var(--ds-dur-fast)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/45",
                    transportType === tt
                      ? "bg-background text-foreground shadow-sm"
                      : "text-muted-foreground hover:text-foreground"
                  )}
                  aria-pressed={transportType === tt}
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
            <>
              <div className="space-y-2">
                <Label>{t("mcp.url")}</Label>
                <Input
                  value={url}
                  onChange={(e) => setUrl(e.target.value)}
                  placeholder="http://localhost:3000/mcp"
                  className="font-mono text-xs"
                />
              </div>
              {transportType === "http" ? (
                <div className="space-y-2">
                  <Label>{t("mcp.headers")}</Label>
                  <textarea
                    value={headers}
                    onChange={(event) => setHeaders(event.target.value)}
                    placeholder={t("mcp.headersPlaceholder")}
                    spellCheck={false}
                    className="min-h-24 w-full rounded-[var(--radius-ui-md)] border border-[color:var(--border-strong)] bg-[color:var(--surface-item)] px-3 py-2 font-mono text-xs outline-none transition-[color,box-shadow] duration-[var(--ds-dur-fast)] focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/45"
                  />
                </div>
              ) : null}
            </>
          )}
            </>
          )}
          {configError ? (
            <p className="text-xs text-destructive">{configError}</p>
          ) : null}
        </div>

        <DialogFooter>
          <Button variant="outline" size="sm" onClick={() => setOpen(false)}>
            {t("mcp.cancel")}
          </Button>
          <Button
            size="sm"
            onClick={handleSubmit}
            disabled={submitting || (configMode === "form" && !name.trim())}
          >
            {submitting ? t("mcp.adding") : t("mcp.add")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

type ServerDraft = {
  name: string;
  transportType: "stdio" | "http" | "sse";
  command: string;
  args: string;
  url: string;
  headers: string;
};

function createFormServerConfig(draft: ServerDraft) {
  if (!draft.name.trim()) throw new Error("Server name is required.");
  let transport: McpTransportInput;
  if (draft.transportType === "stdio") {
    if (!draft.command.trim()) throw new Error("Command is required.");
    transport = {
      type: "stdio",
      command: draft.command.trim(),
      args: draft.args.split(/\s+/).filter(Boolean),
    };
  } else {
    if (!draft.url.trim()) throw new Error("URL is required.");
    transport = {
      type: draft.transportType,
      url: draft.url.trim(),
      ...(draft.transportType === "http" ? { headers: parseStringRecord(draft.headers) } : {}),
    };
  }
  return { id: crypto.randomUUID(), name: draft.name.trim(), transport, auto_connect: true, env: {} };
}

function parseJsonServerConfig(value: string) {
  let parsed: unknown;
  try {
    parsed = JSON.parse(value);
  } catch {
    throw new Error("Invalid JSON configuration.");
  }
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("Invalid JSON configuration.");
  }
  const root = parsed as Record<string, unknown>;
  const servers = isRecord(root.mcpServers) ? root.mcpServers : root;
  const entries = typeof root.command === "string" || typeof root.url === "string"
    ? [[typeof root.name === "string" ? root.name : "MCP Server", root] as const]
    : Object.entries(servers);
  if (entries.length !== 1) throw new Error("JSON configuration must contain exactly one MCP server.");
  const [fallbackName, rawConfig] = entries[0];
  if (!isRecord(rawConfig)) throw new Error("Invalid JSON configuration.");
  const source = isRecord(rawConfig.transport) ? rawConfig.transport : rawConfig;
  const name = typeof rawConfig.name === "string" ? rawConfig.name : fallbackName;
  const env = parseStringRecord(rawConfig.env);
  if (typeof source.command === "string") {
    return {
      id: typeof rawConfig.id === "string" ? rawConfig.id : crypto.randomUUID(),
      name,
      transport: {
        type: "stdio" as const,
        command: source.command,
        args: Array.isArray(source.args) ? source.args.filter((arg): arg is string => typeof arg === "string") : [],
      },
      auto_connect: rawConfig.auto_connect !== false,
      env,
    };
  }
  if (typeof source.url === "string") {
    const type: "http" | "sse" = source.type === "sse" ? "sse" : "http";
    return {
      id: typeof rawConfig.id === "string" ? rawConfig.id : crypto.randomUUID(),
      name,
      transport: { type, url: source.url, headers: parseStringRecord(source.headers) },
      auto_connect: rawConfig.auto_connect !== false,
      env,
    };
  }
  throw new Error("JSON configuration needs either command or url.");
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

function parseStringRecord(value: unknown): Record<string, string> {
  const parsed = typeof value === "string" ? JSON.parse(value) : value ?? {};
  if (!isRecord(parsed) || Object.values(parsed).some((item) => typeof item !== "string")) {
    throw new Error("Headers and environment values must be a JSON object of strings.");
  }
  return parsed as Record<string, string>;
}

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
    <SettingsCard divided>
      {permissions.map((perm) => (
        <div
          key={`${perm.server_id}-${perm.tool_name}`}
          className="flex items-center gap-3 py-3"
        >
          <ShieldCheck className="size-3.5 shrink-0 text-muted-foreground/60" />
          <div className="min-w-0 flex-1">
            <span className="font-mono text-xs text-foreground/90">
              {perm.tool_name}
            </span>
            <span className="ml-2 text-[10px] text-muted-foreground">
              ({perm.server_id})
            </span>
          </div>
          <StatusPill
            tone={perm.policy === "allow" ? "available" : "error"}
            label={perm.policy}
          />
          <button
            type="button"
            onClick={() => handleReset(perm.server_id, perm.tool_name)}
            title={t("mcp.resetPermission")}
            aria-label={t("mcp.resetPermission")}
            className={cn(
              "inline-flex size-7 items-center justify-center rounded-md",
              "text-muted-foreground/60",
              "transition-colors duration-[var(--ds-dur-fast)]",
              "hover:bg-muted/50 hover:text-foreground"
            )}
          >
            <RotateCcw className="size-3" />
          </button>
        </div>
      ))}
    </SettingsCard>
  );
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
          : "text-muted-foreground/60 hover:bg-muted/50 hover:text-foreground"
      )}
    >
      <Icon className="size-3.5" />
    </button>
  );
}

function LoadingPlaceholder() {
  return (
    <div className="flex justify-center rounded-lg border border-dashed border-border/50 bg-card/50 p-10">
      <RefreshCw className="size-5 animate-spin text-muted-foreground" />
    </div>
  );
}

function EmptyState({ message }: { message: string }) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 rounded-lg border border-border/50 bg-card p-10 text-center">
      <Plug className="size-8 text-muted-foreground opacity-40" aria-hidden />
      <p className="text-sm text-muted-foreground">{message}</p>
    </div>
  );
}
