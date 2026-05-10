export interface AppConfig {
  language: string;
  theme: "light" | "dark" | "dim" | "system";
  accent_color: string;
  reduced_transparency: boolean;
  ui_font_size: number;
  default_model: string;
  log_level: string;
  sidecar_port: number;
  auto_start_sidecar: boolean;
}

export interface RouterConfig {
  id: string;
  name: string;
  provider: string;
  api_key_encrypted: string | null;
  model: string | null;
  base_url: string | null;
  config_json: string | null;
  is_active: boolean;
  created_at: string;
}

export interface RouterConfigView {
  id: string;
  name: string;
  provider: string;
  api_key_masked: string;
  model: string | null;
  base_url: string | null;
  is_active: boolean;
  created_at: string;
}

export interface CreateRouterConfig {
  name: string;
  provider: string;
  api_key: string;
  model?: string;
  base_url?: string;
  config_json?: string;
  is_active?: boolean;
}

export interface UpdateRouterConfig {
  name?: string;
  provider?: string;
  api_key?: string;
  model?: string;
  base_url?: string;
  config_json?: string;
  is_active?: boolean;
}

export interface ConnectionTestResult {
  success: boolean;
  message: string;
  models?: string[];
}

export interface SystemInfo {
  app_version: string;
  os: string;
  arch: string;
  data_dir: string;
  db_size_bytes: number;
}

export interface Session {
  id: string;
  title: string | null;
  model: string | null;
  system_prompt: string | null;
  working_directory: string | null;
  project_name: string | null;
  status: string;
  mode: string;
  total_input_tokens: number;
  total_output_tokens: number;
  last_message_at: string | null;
  pinned: boolean;
  group_name: string | null;
  created_at: string;
  updated_at: string;
}

export type MessageRole = "user" | "assistant" | "system";
export type MessageStatus = "streaming" | "complete" | "error" | "stopped" | "aborted";

export interface Message {
  id: string;
  session_id: string;
  role: MessageRole;
  content: string;
  token_usage: string | null;
  model: string | null;
  thinking_content: string | null;
  attachments: string | null;
  status: MessageStatus;
  created_at: string;
  tool_calls?: ToolCall[];
}

export interface TokenUsage {
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number | null;
  cache_creation_tokens: number | null;
  total_tokens: number;
}

export interface ImageAttachment {
  type: string;
  data: string;
  mime_type: string;
}

export interface SendMessageRequest {
  session_id: string;
  content: string;
  images?: ImageAttachment[];
  model_override?: string;
  llm_config?: Record<string, unknown>;
}

export interface SendMessageResult {
  user_message_id: string;
  assistant_message_id: string;
}

export interface StreamTokenPayload {
  session_id: string;
  message_id: string;
  delta: string;
  content_type: "text" | "thinking";
}

export interface StreamCompletePayload {
  session_id: string;
  message_id: string;
  usage: TokenUsage | null;
  model: string | null;
}

export interface StreamErrorPayload {
  session_id: string;
  message_id: string | null;
  error: string;
}

export interface UpdateSessionParams {
  id: string;
  title?: string;
  model?: string;
  pinned?: boolean;
  status?: string;
}

export interface MessageSearchResult {
  id: string;
  session_id: string;
  session_title: string | null;
  role: string;
  snippet: string;
  created_at: string;
}

export interface ExportData {
  version: number;
  exported_at: string;
  app: string;
  sessions: ExportSession[];
}

export interface ExportSession {
  session: Session;
  messages: Message[];
}

export interface ImportResult {
  imported_count: number;
  skipped_count: number;
  errors: string[];
}

// ─── MCP Types ───────────────────────────────────────────────────────

export type ToolCallStatus = "pending" | "running" | "complete" | "error";

export interface ToolCall {
  id: string;
  server_id: string;
  server_name: string;
  tool_name: string;
  arguments: Record<string, unknown>;
  result: unknown | null;
  status: ToolCallStatus;
  error: string | null;
  started_at: number | null;
  completed_at: number | null;
}

export type McpServerStatusType =
  | "disconnected"
  | "connecting"
  | "connected"
  | { error: string };

export interface McpServerInfo {
  id: string;
  name: string;
  transport_type: string;
  status: McpServerStatusType;
  tools_count: number;
  auto_connect: boolean;
}

export interface McpToolInfo {
  server_id: string;
  name: string;
  description: string | null;
  input_schema: Record<string, unknown>;
}

export type ToolPermissionPolicy = "allow" | "deny" | "ask";

export interface ToolPermission {
  server_id: string;
  tool_name: string;
  policy: ToolPermissionPolicy;
  updated_at: string;
}

export interface ToolCallRequestEvent {
  request_id: string;
  server_id: string;
  server_name: string;
  tool_name: string;
  arguments: Record<string, unknown>;
}
