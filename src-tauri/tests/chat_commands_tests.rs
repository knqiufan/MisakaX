#[cfg(feature = "test-private")]
mod resolve_model_spec {
    use misaka_x_lib::commands::chat::resolve_model_spec;

    #[test]
    fn with_override() {
        let spec = resolve_model_spec(Some("config1:gpt-4o"), Some("config2:claude")).unwrap();
        assert_eq!(spec.config_id, "config1");
        assert_eq!(spec.model_id, "gpt-4o");
    }

    #[test]
    fn fallback_to_session() {
        let spec = resolve_model_spec(None, Some("config2:claude-sonnet-4")).unwrap();
        assert_eq!(spec.config_id, "config2");
        assert_eq!(spec.model_id, "claude-sonnet-4");
    }

    #[test]
    fn no_model_returns_error() {
        let result = resolve_model_spec(None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No model specified"));
    }

    #[test]
    fn invalid_format_without_colon() {
        let result = resolve_model_spec(Some("no-colon"), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid model format"));
    }

    #[test]
    fn with_multiple_colons() {
        let spec = resolve_model_spec(Some("config:model:extra"), None).unwrap();
        assert_eq!(spec.config_id, "config");
        assert_eq!(spec.model_id, "model:extra");
    }

    #[test]
    fn empty_config_id() {
        let spec = resolve_model_spec(Some(":gpt-4o"), None).unwrap();
        assert_eq!(spec.config_id, "");
        assert_eq!(spec.model_id, "gpt-4o");
    }

    #[test]
    fn empty_model_id() {
        let spec = resolve_model_spec(Some("config1:"), None).unwrap();
        assert_eq!(spec.config_id, "config1");
        assert_eq!(spec.model_id, "");
    }

    #[test]
    fn override_takes_priority_even_if_session_valid() {
        let spec = resolve_model_spec(
            Some("override-cfg:override-model"),
            Some("session-cfg:session-model"),
        )
        .unwrap();
        assert_eq!(spec.config_id, "override-cfg");
        assert_eq!(spec.model_id, "override-model");
    }
}

#[cfg(feature = "test-private")]
#[test]
fn test_send_message_result_serialize() {
    use misaka_x_lib::commands::chat::SendMessageResult;
    let result = SendMessageResult {
        user_message_id: "u1".to_string(),
        assistant_message_id: "a1".to_string(),
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"user_message_id\":\"u1\""));
    assert!(json.contains("\"assistant_message_id\":\"a1\""));
}

#[cfg(feature = "test-private")]
mod enabled_model_guard {
    use misaka_x_lib::commands::chat::ensure_model_enabled_for_config;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        misaka_x_lib::db::migrations::run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn accepts_enabled_custom_model() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO router_configs (id, name, provider)
             VALUES ('rc-1', 'OpenAI', 'openai')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO custom_models (
                id, router_config_id, model_id, display_name, enabled
             ) VALUES ('cm-1', 'rc-1', 'gpt-4o', 'GPT-4o', 1)",
            [],
        )
        .unwrap();

        let result = ensure_model_enabled_for_config(&conn, "rc-1", "gpt-4o");

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_disabled_custom_model() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO router_configs (id, name, provider)
             VALUES ('rc-1', 'OpenAI', 'openai')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO custom_models (
                id, router_config_id, model_id, display_name, enabled
             ) VALUES ('cm-1', 'rc-1', 'gpt-4o', 'GPT-4o', 0)",
            [],
        )
        .unwrap();

        let result = ensure_model_enabled_for_config(&conn, "rc-1", "gpt-4o");

        assert!(result.is_err());
    }
}
