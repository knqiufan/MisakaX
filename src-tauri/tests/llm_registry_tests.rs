use misaka_x_lib::services::llm::{ModelInfo, ModelRegistry};
use rusqlite::Connection;

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
            enabled INTEGER DEFAULT 1,
            sort_order INTEGER DEFAULT 0,
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
    assert_eq!(
        custom_count, 0,
        "Custom models from rc-2 should not appear for rc-1"
    );
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
