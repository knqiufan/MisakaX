use misaka_x_lib::crypto;
use misaka_x_lib::db::models::CreateCustomModel;
use misaka_x_lib::db::repository::{CustomModelRepo, RouterConfigRepo};
use misaka_x_lib::services::llm::registry::{ModelInfo, ModelRegistry};
use rusqlite::Connection;

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    misaka_x_lib::db::migrations::run_migrations(&conn).unwrap();
    conn
}

fn insert_router_config(conn: &Connection, id: &str, provider: &str) {
    let encrypted = crypto::encrypt("sk-test-key-fake-12345678").unwrap();
    RouterConfigRepo::insert(
        conn,
        id,
        "Test Provider",
        provider,
        &encrypted,
        None,
        None,
        None,
        true,
        None,
    )
    .unwrap();
}

// ─── CustomModelRepo tests ───────────────────────────────────────────

#[test]
fn test_insert_custom_model() {
    let conn = setup_test_db();
    insert_router_config(&conn, "rc-1", "openai");

    let model = CreateCustomModel {
        model_id: "ft-gpt-4o".to_string(),
        display_name: "Fine-tuned GPT-4o".to_string(),
        supports_vision: true,
        supports_thinking: false,
        max_tokens: Some(4096),
        context_window: Some(128000),
        enabled: true,
        sort_order: 0,
            thinking_off_model_id: None,
    };

    CustomModelRepo::insert(&conn, "cm-1", "rc-1", &model).unwrap();

    let models = ModelRegistry::available_models(&conn, "rc-1", "openai").unwrap();
    let custom: Vec<_> = models.iter().filter(|m| m.is_custom).collect();
    assert_eq!(custom.len(), 1);
    assert_eq!(custom[0].model_id, "ft-gpt-4o");
    assert_eq!(custom[0].display_name, "Fine-tuned GPT-4o");
    assert!(custom[0].supports_vision);
    assert!(!custom[0].supports_thinking);
    assert_eq!(custom[0].max_tokens, Some(4096));
    assert_eq!(custom[0].context_window, Some(128000));
}

#[test]
fn test_insert_custom_model_minimal() {
    let conn = setup_test_db();
    insert_router_config(&conn, "rc-1", "openai");

    let model = CreateCustomModel {
        model_id: "my-model".to_string(),
        display_name: "My Model".to_string(),
        supports_vision: false,
        supports_thinking: false,
        max_tokens: None,
        context_window: None,
        enabled: true,
        sort_order: 0,
            thinking_off_model_id: None,
    };

    CustomModelRepo::insert(&conn, "cm-1", "rc-1", &model).unwrap();

    let models = ModelRegistry::available_models(&conn, "rc-1", "openai").unwrap();
    let custom: Vec<_> = models.iter().filter(|m| m.is_custom).collect();
    assert_eq!(custom.len(), 1);
    assert_eq!(custom[0].max_tokens, None);
    assert_eq!(custom[0].context_window, None);
}

#[test]
fn test_delete_custom_model() {
    let conn = setup_test_db();
    insert_router_config(&conn, "rc-1", "openai");

    let model = CreateCustomModel {
        model_id: "to-delete".to_string(),
        display_name: "Delete Me".to_string(),
        supports_vision: false,
        supports_thinking: false,
        max_tokens: None,
        context_window: None,
        enabled: true,
        sort_order: 0,
            thinking_off_model_id: None,
    };

    CustomModelRepo::insert(&conn, "cm-del", "rc-1", &model).unwrap();
    CustomModelRepo::delete(&conn, "cm-del").unwrap();

    let models = ModelRegistry::available_models(&conn, "rc-1", "openai").unwrap();
    let custom_count = models.iter().filter(|m| m.is_custom).count();
    assert_eq!(custom_count, 0);
}

#[test]
fn test_delete_custom_model_not_found() {
    let conn = setup_test_db();
    let result = CustomModelRepo::delete(&conn, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_custom_models_unique_constraint() {
    let conn = setup_test_db();
    insert_router_config(&conn, "rc-1", "openai");

    let model = CreateCustomModel {
        model_id: "duplicate".to_string(),
        display_name: "First".to_string(),
        supports_vision: false,
        supports_thinking: false,
        max_tokens: None,
        context_window: None,
        enabled: true,
        sort_order: 0,
            thinking_off_model_id: None,
    };

    CustomModelRepo::insert(&conn, "cm-1", "rc-1", &model).unwrap();

    let model2 = CreateCustomModel {
        model_id: "duplicate".to_string(),
        display_name: "Second".to_string(),
        supports_vision: false,
        supports_thinking: false,
        max_tokens: None,
        context_window: None,
        enabled: true,
        sort_order: 0,
            thinking_off_model_id: None,
    };

    let result = CustomModelRepo::insert(&conn, "cm-2", "rc-1", &model2);
    assert!(
        result.is_err(),
        "Should fail on duplicate (router_config_id, model_id)"
    );
}

#[test]
fn test_multiple_custom_models_per_provider() {
    let conn = setup_test_db();
    insert_router_config(&conn, "rc-1", "openai");

    for i in 0..5 {
        let model = CreateCustomModel {
            model_id: format!("model-{i}"),
            display_name: format!("Model {i}"),
            supports_vision: false,
            supports_thinking: false,
            max_tokens: None,
            context_window: None,
            enabled: true,
            sort_order: i,
            thinking_off_model_id: None,
        };
        CustomModelRepo::insert(&conn, &format!("cm-{i}"), "rc-1", &model).unwrap();
    }

    let models = ModelRegistry::available_models(&conn, "rc-1", "openai").unwrap();
    let custom_count = models.iter().filter(|m| m.is_custom).count();
    assert_eq!(custom_count, 5);
}

#[test]
fn test_available_models_only_returns_enabled_custom_models() {
    let conn = setup_test_db();
    insert_router_config(&conn, "rc-1", "openai");

    let enabled = CreateCustomModel {
        model_id: "enabled-model".to_string(),
        display_name: "Enabled Model".to_string(),
        supports_vision: false,
        supports_thinking: false,
        max_tokens: None,
        context_window: None,
        enabled: true,
        sort_order: 2,
            thinking_off_model_id: None,
    };
    let disabled = CreateCustomModel {
        model_id: "disabled-model".to_string(),
        display_name: "Disabled Model".to_string(),
        supports_vision: false,
        supports_thinking: false,
        max_tokens: None,
        context_window: None,
        enabled: false,
        sort_order: 1,
            thinking_off_model_id: None,
    };

    CustomModelRepo::insert(&conn, "cm-enabled", "rc-1", &enabled).unwrap();
    CustomModelRepo::insert(&conn, "cm-disabled", "rc-1", &disabled).unwrap();

    let models = ModelRegistry::available_models(&conn, "rc-1", "openai").unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].model_id, "enabled-model");
}

#[test]
fn test_list_and_replace_custom_models() {
    let mut conn = setup_test_db();
    insert_router_config(&conn, "rc-1", "openai");

    let models = vec![
        CreateCustomModel {
            model_id: "model-b".to_string(),
            display_name: "Model B".to_string(),
            supports_vision: false,
            supports_thinking: false,
            max_tokens: None,
            context_window: None,
            enabled: true,
            sort_order: 2,
                thinking_off_model_id: None,
    },
        CreateCustomModel {
            model_id: "model-a".to_string(),
            display_name: "Model A".to_string(),
            supports_vision: true,
            supports_thinking: false,
            max_tokens: Some(4096),
            context_window: Some(8192),
            enabled: false,
            sort_order: 1,
                thinking_off_model_id: None,
    },
    ];

    CustomModelRepo::replace_all(&mut conn, "rc-1", &models).unwrap();
    let stored = CustomModelRepo::list_by_router(&conn, "rc-1").unwrap();

    assert_eq!(stored.len(), 2);
    assert_eq!(stored[0].model_id, "model-a");
    assert!(!stored[0].enabled);
    assert_eq!(stored[1].model_id, "model-b");
}

// ─── ModelInfo structure tests ───────────────────────────────────────

#[test]
fn test_model_info_serialize() {
    let info = ModelInfo::builtin("gpt-4o", "GPT-4o", true, false);
    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("\"model_id\":\"gpt-4o\""));
    assert!(json.contains("\"display_name\":\"GPT-4o\""));
    assert!(json.contains("\"supports_vision\":true"));
    assert!(json.contains("\"supports_thinking\":false"));
    assert!(json.contains("\"is_custom\":false"));
}

#[test]
fn test_model_info_deserialize() {
    let json = r#"{
        "model_id": "custom-1",
        "display_name": "Custom Model",
        "supports_vision": true,
        "supports_thinking": true,
        "is_custom": true,
        "max_tokens": 8192,
        "context_window": null
    }"#;
    let info: ModelInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.model_id, "custom-1");
    assert!(info.supports_vision);
    assert!(info.supports_thinking);
    assert!(info.is_custom);
    assert_eq!(info.max_tokens, Some(8192));
    assert!(info.context_window.is_none());
}
