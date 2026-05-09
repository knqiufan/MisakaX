use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub working_directory: Option<String>,
    pub project_name: Option<String>,
    pub status: String,
    pub mode: String,
    #[serde(default)]
    pub total_input_tokens: i64,
    #[serde(default)]
    pub total_output_tokens: i64,
    pub last_message_at: Option<String>,
    #[serde(default)]
    pub pinned: bool,
    pub group_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_tokens: Option<u64>,
    #[serde(default)]
    pub cache_creation_tokens: Option<u64>,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub token_usage: Option<String>,
    pub model: Option<String>,
    pub thinking_content: Option<String>,
    pub attachments: Option<String>,
    #[serde(default = "default_message_status")]
    pub status: String,
    pub created_at: String,
}

fn default_message_status() -> String {
    "complete".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_key_encrypted: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub config_json: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    /// 接口兼容模式（仅自定义 Provider 使用）：
    /// `None` = 根据 provider 自动判断
    /// `"openai"` = OpenAI 兼容接口
    /// `"anthropic"` = Anthropic 兼容接口
    #[serde(default)]
    pub api_compat: Option<String>,
}

/// 用户自定义模型（绑定到 router_configs）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomModel {
    pub id: String,
    pub router_config_id: String,
    pub model_id: String,
    pub display_name: String,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub max_tokens: Option<i32>,
    pub context_window: Option<i32>,
    pub created_at: String,
}

/// 创建自定义模型的请求参数
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCustomModel {
    pub model_id: String,
    pub display_name: String,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default)]
    pub supports_thinking: bool,
    pub max_tokens: Option<i32>,
    pub context_window: Option<i32>,
}
