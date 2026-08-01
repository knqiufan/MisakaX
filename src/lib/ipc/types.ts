import type { ProviderApi, VendorId } from "@/lib/providers/catalog";
import type { ModelInfo } from "./models";

export type CloseBehavior = "ask" | "minimize_to_tray" | "quit";

export interface AppConfig {
  language: string;
  theme: "light" | "dark" | "system";
  /** Legacy field retained for config compatibility; UI no longer applies accent overrides. */
  accent_color: string;
  reduced_transparency: boolean;
  ui_font_size: number;
  default_model: string;
  log_level: string;
  sidecar_port: number;
  auto_start_sidecar: boolean;
  /** Prefer Python Sidecar for chat; false uses Rig fallback */
  use_sidecar: boolean;
  mcp_bridge_port: number;
  close_behavior: CloseBehavior;
}

export interface RouterConfig {
  id: string;
  name: string;
  provider: ProviderApi;
  vendor: VendorId | null;
  api_key_encrypted: string | null;
  model: string | null;
  base_url: string | null;
  config_json: string | null;
  advanced_json: string | null;
  is_active: boolean;
  created_at: string;
  api_compat: ProviderApi | null;
}

export interface AdvancedConfig {
  temperature: number;
  max_tokens: number | null;
  proxy?: string | null;
}

export interface RouterConfigView {
  id: string;
  name: string;
  provider: ProviderApi;
  vendor: VendorId | null;
  api_key_masked: string;
  model: string | null;
  base_url: string | null;
  api_compat: ProviderApi | null;
  config_json: string | null;
  advanced: AdvancedConfig;
  is_active: boolean;
  created_at: string;
}

export interface CreateRouterConfig {
  name: string;
  provider: ProviderApi;
  vendor?: VendorId;
  api_key: string;
  model?: string;
  base_url?: string;
  config_json?: string;
  advanced?: AdvancedConfig;
  is_active?: boolean;
  api_compat?: ProviderApi;
}

export interface CreateCustomModel {
  model_id: string;
  display_name: string;
  supports_vision?: boolean;
  supports_thinking?: boolean;
  model_types?: ModelType[];
  max_tokens?: number | null;
  context_window?: number | null;
  enabled?: boolean;
  sort_order?: number;
  /** Same-provider non-thinking model_id used when thinking is toggled off. */
  thinking_off_model_id?: string | null;
}

export interface CreateRouterConfigWithModels {
  config: CreateRouterConfig;
  models: CreateCustomModel[];
}

export interface UpdateRouterConfig {
  name?: string;
  provider?: ProviderApi;
  vendor?: VendorId;
  api_key?: string;
  model?: string;
  base_url?: string;
  config_json?: string;
  advanced?: AdvancedConfig;
  is_active?: boolean;
  api_compat?: ProviderApi;
}

export interface CustomModel {
  id: string;
  router_config_id: string;
  model_id: string;
  display_name: string;
  supports_vision: boolean;
  supports_thinking: boolean;
  model_types: ModelType[];
  max_tokens: number | null;
  context_window: number | null;
  enabled: boolean;
  sort_order: number;
  thinking_off_model_id?: string | null;
  created_at: string;
}

export type ModelType =
  | "text"
  | "multimodal"
  | "speech"
  | "embedding"
  | "rerank"
  | "image";

export interface FetchModelsResult {
  models: ModelInfo[];
  source: "remote" | "fallback";
  warning?: string;
}

export interface FetchProviderModelsRequest {
  api: ProviderApi;
  vendor: VendorId;
  base_url?: string;
  api_key: string;
}

export interface TestModelRequest {
  api: ProviderApi;
  vendor?: VendorId;
  base_url?: string;
  api_key: string;
  model_id: string;
  advanced?: AdvancedConfig;
}

export interface ModelTestResult {
  success: boolean;
  latency_ms: number;
  message: string;
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
  /** `default` = app-managed ~/.misakax/workspace; `custom` = user-picked path. */
  workspace_kind: "default" | "custom" | string;
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

export type MessageAttachment =
  | {
      kind: "image";
      data: string;
      media_type: string;
      file_name?: string;
    }
  | {
      kind: "text";
      extracted_text: string;
      mime: string;
      file_name: string;
      size: number;
    };

export type ImageAttachment = MessageAttachment;

export interface LlmConfigPayload {
  thinking_enabled?: boolean;
  /** `chat` (default) or `research` — enables DeepAgents task/subagents. */
  agent_mode?: "chat" | "research";
  temperature?: number;
  max_tokens?: number;
}

export interface SendMessageRequest {
  session_id: string;
  content: string;
  attachments?: MessageAttachment[];
  images?: MessageAttachment[];
  model_override?: string;
  llm_config?: LlmConfigPayload;
  selected_skill_ids?: string[];
}

export interface SkillRiskReport {
  has_scripts: boolean;
  has_binary_files: boolean;
  has_allowed_tools: boolean;
  remote_scan_status: string | null;
  notes: string[];
}

export interface InstalledSkill {
  skill_id: string;
  slug: string;
  name: string;
  description: string;
  version: string | null;
  source_kind: string;
  source_ref: string | null;
  source_url: string | null;
  checksum: string;
  installed_path: string;
  enabled: boolean;
  health: "healthy" | "missing" | string;
  /** Discovered from a compatible Agent directory; MisakaX does not own it. */
  is_external: boolean;
  effective_active: boolean;
  effective_rank: number;
  conflict: boolean;
  disabled_reason: string | null;
  security_state: "legacy_allowed" | "pending_user" | "user_allowed" | string;
  risk: SkillRiskReport;
  installed_at: string;
  updated_at: string;
}

export interface SkillActivationMount {
  skill_id: string;
  slug: string;
  path: string;
  artifact_hash: string;
}

export interface SkillActivationView {
  generation: number;
  skills: SkillActivationMount[];
}

export interface SkillManifest {
  name: string;
  description: string;
  license: string | null;
  compatibility: string | null;
  allowed_tools: string | null;
  metadata: Record<string, unknown>;
}

export interface SkillFileNode {
  path: string;
  kind: string;
  size_bytes: number;
}

export interface SkillDetail {
  skill: InstalledSkill;
  manifest: SkillManifest;
  files: SkillFileNode[];
  skill_markdown: string;
}

export interface RemoteSkill {
  provider: "skillhub" | "clawhub" | "modelscope" | string;
  slug: string;
  display_name: string;
  summary: string;
  version: string | null;
  owner: string | null;
  source_url: string;
  topics: string[];
  suspicious: boolean;
}

export interface RemoteSkillDetail {
  skill: RemoteSkill;
  changelog: string | null;
  license: string | null;
  compatibility: string | null;
  risk: SkillRiskReport;
  manifest: SkillManifest | null;
  files: SkillFileNode[];
  skill_markdown: string | null;
}

export interface RemoteSearchPage {
  items: RemoteSkill[];
  next_cursor: string | null;
}

export interface SkillArchiveInspection {
  manifest: SkillManifest;
  files: SkillFileNode[];
  risk: SkillRiskReport;
  checksum: string;
  compressed_size_bytes: number;
  uncompressed_size_bytes: number;
}

export interface SkillInstallResult {
  skill: InstalledSkill;
  replaced_existing: boolean;
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

export type ToolCallStatus =
  | "pending"
  | "running"
  | "complete"
  | "error"
  | "aborted";

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
