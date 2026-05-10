import { invoke } from "./invoke";
import type {
  McpServerInfo,
  McpToolInfo,
  ToolPermission,
} from "./types";

export interface McpServerConfigInput {
  id: string;
  name: string;
  transport: McpTransportInput;
  auto_connect?: boolean;
  env?: Record<string, string>;
}

export type McpTransportInput =
  | { type: "stdio"; command: string; args?: string[] }
  | { type: "http"; url: string; headers?: Record<string, string> }
  | { type: "sse"; url: string; headers?: Record<string, string> };

export const mcpIpc = {
  listServers: () => invoke<McpServerInfo[]>("mcp_list_servers"),

  connectServer: (config: McpServerConfigInput) =>
    invoke<McpServerInfo>("mcp_connect_server", { config }),

  disconnectServer: (serverId: string) =>
    invoke<void>("mcp_disconnect_server", { serverId }),

  restartServer: (serverId: string) =>
    invoke<McpServerInfo>("mcp_restart_server", { serverId }),

  listTools: () => invoke<McpToolInfo[]>("mcp_list_tools"),

  callTool: (serverId: string, toolName: string, arguments_: Record<string, unknown>) =>
    invoke<unknown>("mcp_call_tool", { serverId, toolName, arguments: arguments_ }),

  addServerConfig: (config: McpServerConfigInput) =>
    invoke<void>("mcp_add_server_config", { config }),

  removeServerConfig: (serverId: string) =>
    invoke<void>("mcp_remove_server_config", { serverId }),

  approveToolCall: (requestId: string, remember: boolean) =>
    invoke<void>("mcp_approve_tool_call", { requestId, remember }),

  denyToolCall: (requestId: string, remember: boolean) =>
    invoke<void>("mcp_deny_tool_call", { requestId, remember }),

  listPermissions: () => invoke<ToolPermission[]>("mcp_list_permissions"),

  resetPermission: (serverId: string, toolName: string) =>
    invoke<void>("mcp_reset_permission", { serverId, toolName }),
};
