use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::db::models::CustomModel;
use crate::db::repository::CustomModelRepo;

/// 可用模型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub display_name: String,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    #[serde(default)]
    pub model_types: Vec<String>,
    pub is_custom: bool,
    pub max_tokens: Option<i32>,
    pub context_window: Option<i32>,
}

impl ModelInfo {
    pub fn builtin(model_id: &str, display_name: &str, vision: bool, thinking: bool) -> Self {
        Self {
            model_id: model_id.to_string(),
            display_name: display_name.to_string(),
            supports_vision: vision,
            supports_thinking: thinking,
            model_types: infer_model_types(model_id, vision),
            is_custom: false,
            max_tokens: None,
            context_window: None,
        }
    }

    /// 从持久化的自定义模型构造（标记 `is_custom = true`）。
    pub fn from_custom(cm: CustomModel) -> Self {
        let model_types = if cm.model_types.is_empty() {
            infer_model_types(&cm.model_id, cm.supports_vision)
        } else {
            cm.model_types.clone()
        };
        Self {
            model_id: cm.model_id,
            display_name: cm.display_name,
            supports_vision: cm.supports_vision,
            supports_thinking: cm.supports_thinking,
            model_types,
            is_custom: true,
            max_tokens: cm.max_tokens,
            context_window: cm.context_window,
        }
    }
}

/// Consistent fallback for OpenAI-compatible lists, which generally expose no
/// machine-readable capability metadata. Provider-specific metadata overrides
/// this classification whenever it is available.
pub fn infer_model_types(model_id: &str, supports_vision: bool) -> Vec<String> {
    let id = model_id.to_ascii_lowercase();
    if id.contains("embed") {
        return vec!["embedding".to_string()];
    }
    if id.contains("rerank") {
        return vec!["rerank".to_string()];
    }
    if id.contains("tts")
        || id.contains("speech")
        || id.contains("transcribe")
        || id.contains("whisper")
    {
        return vec!["speech".to_string()];
    }
    if id.contains("image") || id.contains("dall-e") || id.contains("sora") {
        return vec!["image".to_string()];
    }
    if supports_vision
        || id.contains("vision")
        || id.contains("vl")
        || id.contains("omni")
        || id.starts_with("gpt-4o")
        || id.starts_with("gpt-4.1")
        || id.starts_with("gpt-5")
        || id.starts_with("claude")
        || id.starts_with("gemini")
    {
        return vec!["multimodal".to_string()];
    }
    vec!["text".to_string()]
}

/// 模型注册表 — 内置模型作为候选兜底；会话只使用用户显式启用的模型
pub struct ModelRegistry;

impl ModelRegistry {
    /// 获取某个 Provider 下所有可用模型（仅启用的自定义模型）
    pub fn available_models(
        conn: &Connection,
        router_config_id: &str,
        _provider: &str,
    ) -> Result<Vec<ModelInfo>> {
        Self::custom_models(conn, router_config_id)
    }

    /// 内置模型列表 — 按 Provider 分类
    pub fn builtin_models(provider: &str) -> Vec<ModelInfo> {
        match provider {
            "openai" => Self::openai_models(),
            "anthropic" => Self::anthropic_models(),
            "google" => Self::gemini_models(),
            _ => vec![],
        }
    }

    fn openai_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo::builtin("gpt-4o", "GPT-4o", true, false),
            ModelInfo::builtin("gpt-4o-mini", "GPT-4o Mini", true, false),
            ModelInfo::builtin("gpt-4.1", "GPT-4.1", true, false),
            ModelInfo::builtin("gpt-4.1-mini", "GPT-4.1 Mini", true, false),
            ModelInfo::builtin("gpt-4.1-nano", "GPT-4.1 Nano", true, false),
            ModelInfo::builtin("o1", "o1", true, true),
            ModelInfo::builtin("o3-mini", "o3-mini", false, true),
            ModelInfo::builtin("o4-mini", "o4-mini", true, true),
        ]
    }

    fn anthropic_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo::builtin("claude-sonnet-4-20250514", "Claude Sonnet 4", true, true),
            ModelInfo::builtin("claude-opus-4-20250514", "Claude Opus 4", true, true),
            ModelInfo::builtin("claude-haiku-4-5-20250514", "Claude Haiku 4.5", true, false),
        ]
    }

    fn gemini_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo::builtin("gemini-2.5-pro-preview-05-06", "Gemini 2.5 Pro", true, true),
            ModelInfo::builtin("gemini-2.5-flash", "Gemini 2.5 Flash", true, true),
            ModelInfo::builtin("gemini-2.0-flash", "Gemini 2.0 Flash", true, false),
        ]
    }

    /// 从 custom_models 表读取用户自定义模型（仅启用的）
    fn custom_models(conn: &Connection, router_config_id: &str) -> Result<Vec<ModelInfo>> {
        let models = CustomModelRepo::list_by_router(conn, router_config_id)?;
        Ok(models
            .into_iter()
            .filter(|m| m.enabled)
            .map(ModelInfo::from_custom)
            .collect())
    }
}
