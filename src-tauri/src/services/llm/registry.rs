use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// 可用模型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub display_name: String,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub is_custom: bool,
    pub max_tokens: Option<i32>,
    pub context_window: Option<i32>,
}

impl ModelInfo {
    pub fn builtin(
        model_id: &str,
        display_name: &str,
        vision: bool,
        thinking: bool,
    ) -> Self {
        Self {
            model_id: model_id.to_string(),
            display_name: display_name.to_string(),
            supports_vision: vision,
            supports_thinking: thinking,
            is_custom: false,
            max_tokens: None,
            context_window: None,
        }
    }
}

/// 模型注册表 — 合并内置模型与用户自定义模型
pub struct ModelRegistry;

impl ModelRegistry {
    /// 获取某个 Provider 下所有可用模型（内置 + 自定义）
    pub fn available_models(
        conn: &Connection,
        router_config_id: &str,
        provider: &str,
    ) -> Result<Vec<ModelInfo>> {
        let mut models = Self::builtin_models(provider);
        let custom = Self::custom_models(conn, router_config_id)?;
        models.extend(custom);
        Ok(models)
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
            ModelInfo::builtin(
                "claude-sonnet-4-20250514",
                "Claude Sonnet 4",
                true,
                true,
            ),
            ModelInfo::builtin(
                "claude-opus-4-20250514",
                "Claude Opus 4",
                true,
                true,
            ),
            ModelInfo::builtin(
                "claude-haiku-4-5-20250514",
                "Claude Haiku 4.5",
                true,
                false,
            ),
        ]
    }

    fn gemini_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo::builtin(
                "gemini-2.5-pro-preview-05-06",
                "Gemini 2.5 Pro",
                true,
                true,
            ),
            ModelInfo::builtin(
                "gemini-2.5-flash",
                "Gemini 2.5 Flash",
                true,
                true,
            ),
            ModelInfo::builtin(
                "gemini-2.0-flash",
                "Gemini 2.0 Flash",
                true,
                false,
            ),
        ]
    }

    /// 从 custom_models 表读取用户自定义模型
    fn custom_models(
        conn: &Connection,
        router_config_id: &str,
    ) -> Result<Vec<ModelInfo>> {
        let mut stmt = conn.prepare(
            "SELECT model_id, display_name, supports_vision, supports_thinking,
                    max_tokens, context_window
             FROM custom_models
             WHERE router_config_id = ?1
             ORDER BY created_at ASC",
        )?;

        let models = stmt
            .query_map([router_config_id], |row| {
                Ok(ModelInfo {
                    model_id: row.get(0)?,
                    display_name: row.get(1)?,
                    supports_vision: row.get::<_, i32>(2)? != 0,
                    supports_thinking: row.get::<_, i32>(3)? != 0,
                    is_custom: true,
                    max_tokens: row.get(4)?,
                    context_window: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();

        conn.execute_batch(
            "CREATE TABLE router_configs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider TEXT NOT NULL,
                api_key_encrypted TEXT,
                model TEXT,
                base_url TEXT,
                config_json TEXT,
                is_active INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                api_compat TEXT DEFAULT NULL
            );
            CREATE TABLE custom_models (
                id TEXT PRIMARY KEY,
                router_config_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                display_name TEXT NOT NULL,
                supports_vision INTEGER DEFAULT 0,
                supports_thinking INTEGER DEFAULT 0,
                max_tokens INTEGER,
                context_window INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (router_config_id) REFERENCES router_configs(id) ON DELETE CASCADE,
                UNIQUE (router_config_id, model_id)
            );",
        )
        .unwrap();

        conn.execute(
            "INSERT INTO router_configs (id, name, provider) VALUES (?1, ?2, ?3)",
            rusqlite::params!["rc-1", "My OpenAI", "openai"],
        )
        .unwrap();

        conn
    }

    #[test]
    fn test_builtin_openai_models() {
        let models = ModelRegistry::builtin_models("openai");
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.model_id == "gpt-4o"));
        assert!(models.iter().all(|m| !m.is_custom));
    }

    #[test]
    fn test_builtin_anthropic_models() {
        let models = ModelRegistry::builtin_models("anthropic");
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.model_id.contains("claude-sonnet")));
        assert!(models.iter().any(|m| m.supports_thinking));
    }

    #[test]
    fn test_builtin_gemini_models() {
        let models = ModelRegistry::builtin_models("google");
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.model_id.contains("gemini")));
    }

    #[test]
    fn test_unknown_provider_has_no_builtins() {
        let models = ModelRegistry::builtin_models("my-custom-provider");
        assert!(models.is_empty());
    }

    #[test]
    fn test_custom_models_empty_by_default() {
        let conn = setup_test_db();
        let models = ModelRegistry::available_models(&conn, "rc-1", "openai").unwrap();
        let custom_count = models.iter().filter(|m| m.is_custom).count();
        assert_eq!(custom_count, 0);
    }

    #[test]
    fn test_custom_models_included_in_available() {
        let conn = setup_test_db();

        conn.execute(
            "INSERT INTO custom_models (id, router_config_id, model_id, display_name,
             supports_vision, supports_thinking)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["cm-1", "rc-1", "gpt-4o-ft", "My Fine-tuned GPT-4o", 1, 0],
        )
        .unwrap();

        let models = ModelRegistry::available_models(&conn, "rc-1", "openai").unwrap();
        let custom: Vec<_> = models.iter().filter(|m| m.is_custom).collect();
        assert_eq!(custom.len(), 1);
        assert_eq!(custom[0].model_id, "gpt-4o-ft");
        assert_eq!(custom[0].display_name, "My Fine-tuned GPT-4o");
        assert!(custom[0].supports_vision);
        assert!(!custom[0].supports_thinking);
    }

    #[test]
    fn test_custom_models_scoped_to_config() {
        let conn = setup_test_db();

        conn.execute(
            "INSERT INTO router_configs (id, name, provider) VALUES (?1, ?2, ?3)",
            rusqlite::params!["rc-2", "Other Provider", "openai"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO custom_models (id, router_config_id, model_id, display_name)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["cm-1", "rc-2", "other-model", "Other Model"],
        )
        .unwrap();

        let models = ModelRegistry::available_models(&conn, "rc-1", "openai").unwrap();
        let custom_count = models.iter().filter(|m| m.is_custom).count();
        assert_eq!(custom_count, 0, "Custom models from rc-2 should not appear for rc-1");
    }

    #[test]
    fn test_model_info_builtin_constructor() {
        let info = ModelInfo::builtin("gpt-4o", "GPT-4o", true, false);
        assert_eq!(info.model_id, "gpt-4o");
        assert_eq!(info.display_name, "GPT-4o");
        assert!(info.supports_vision);
        assert!(!info.supports_thinking);
        assert!(!info.is_custom);
        assert!(info.max_tokens.is_none());
    }
}
