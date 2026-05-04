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
  created_at: string;
  updated_at: string;
}
