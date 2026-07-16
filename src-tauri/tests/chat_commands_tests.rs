#[cfg(feature = "test-private")]
mod resolve_model_spec {
    use misaka_x_lib::services::chat::resolve_model_spec;

    #[test]
    fn with_override() {
        let spec =
            resolve_model_spec(Some("config1:gpt-4o"), Some("config2:claude")).expect("valid spec");
        assert_eq!(spec.config_id, "config1");
        assert_eq!(spec.model_id, "gpt-4o");
    }

    #[test]
    fn fallback_to_session() {
        let spec = resolve_model_spec(None, Some("config2:claude-sonnet-4")).expect("valid spec");
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
        let spec = resolve_model_spec(Some("config:model:extra"), None).expect("valid spec");
        assert_eq!(spec.config_id, "config");
        assert_eq!(spec.model_id, "model:extra");
    }

    #[test]
    fn empty_config_id() {
        let spec = resolve_model_spec(Some(":gpt-4o"), None).expect("valid spec");
        assert_eq!(spec.config_id, "");
        assert_eq!(spec.model_id, "gpt-4o");
    }

    #[test]
    fn empty_model_id() {
        let spec = resolve_model_spec(Some("config1:"), None).expect("valid spec");
        assert_eq!(spec.config_id, "config1");
        assert_eq!(spec.model_id, "");
    }

    #[test]
    fn override_takes_priority_even_if_session_valid() {
        let spec = resolve_model_spec(
            Some("override-cfg:override-model"),
            Some("session-cfg:session-model"),
        )
        .expect("valid spec");
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
    let json = serde_json::to_string(&result).expect("SendMessageResult should serialize to JSON");
    assert!(json.contains("\"user_message_id\":\"u1\""));
    assert!(json.contains("\"assistant_message_id\":\"a1\""));
}

#[cfg(feature = "test-private")]
mod agent_message_builders {
    use misaka_x_lib::db::models::{Message, RouterConfig, Session};
    use misaka_x_lib::services::chat::{
        build_agent_chat_request, build_agent_messages, build_title_prompt, sanitize_session_title,
    };

    fn sample_session() -> Session {
        Session {
            id: "s1".to_string(),
            title: Some("t".to_string()),
            model: Some("cfg:model".to_string()),
            system_prompt: Some("Be helpful".to_string()),
            working_directory: Some("D:/code".to_string()),
            project_name: None,
            status: "active".to_string(),
            mode: "agent".to_string(),
            total_input_tokens: 0,
            total_output_tokens: 0,
            last_message_at: None,
            pinned: false,
            group_name: None,
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        }
    }

    fn sample_router() -> RouterConfig {
        RouterConfig {
            id: "cfg".to_string(),
            name: "OpenAI Compat".to_string(),
            provider: "custom".to_string(),
            vendor: Some("openai".to_string()),
            api_key_encrypted: None,
            model: Some("gpt-4o".to_string()),
            base_url: Some("https://api.example.com/v1".to_string()),
            config_json: None,
            advanced_json: None,
            is_active: true,
            created_at: "now".to_string(),
            api_compat: Some("openai".to_string()),
        }
    }

    fn sample_message(role: &str, content: &str) -> Message {
        Message {
            id: "m1".to_string(),
            session_id: "s1".to_string(),
            role: role.to_string(),
            content: content.to_string(),
            token_usage: None,
            model: None,
            thinking_content: None,
            attachments: None,
            status: "complete".to_string(),
            tool_calls: None,
            created_at: "now".to_string(),
        }
    }

    #[test]
    fn build_agent_messages_includes_system_and_history() {
        let session = sample_session();
        let history = vec![
            sample_message("user", "hi"),
            sample_message("assistant", "hello"),
        ];
        let messages = build_agent_messages(&session, &history);
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, "Be helpful");
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[2].role, "assistant");
    }

    #[test]
    fn build_agent_chat_request_sets_working_dir_and_session() {
        let session = sample_session();
        let history = vec![sample_message("user", "prior")];
        let router = sample_router();
        let req = build_agent_chat_request(
            &session,
            &history,
            "next",
            "gpt-4o",
            None,
            &router,
            "sk-test",
        );
        assert_eq!(req.session_id.as_deref(), Some("s1"));
        assert_eq!(req.working_dir.as_deref(), Some("D:/code"));
        assert_eq!(req.config.model.as_deref(), Some("gpt-4o"));
        assert_eq!(req.config.provider.as_deref(), Some("custom"));
        assert_eq!(req.config.api_compat.as_deref(), Some("openai"));
        assert_eq!(
            req.config.base_url.as_deref(),
            Some("https://api.example.com/v1")
        );
        assert_eq!(req.config.api_key.as_deref(), Some("sk-test"));
        assert!(req.config.stream);
        assert_eq!(
            req.messages
                .last()
                .expect("request should have messages")
                .content,
            "next"
        );
    }

    #[test]
    fn title_prompt_and_sanitize() {
        let prompt = build_title_prompt("How do I fix SSE?");
        assert!(prompt.contains("How do I fix SSE?"));
        assert_eq!(
            sanitize_session_title("  \"Fix SSE parser\"  "),
            "Fix SSE parser"
        );
        assert_eq!(sanitize_session_title("   "), "New Chat");
    }
}

#[cfg(feature = "test-private")]
mod enabled_model_guard {
    use misaka_x_lib::services::chat::ensure_model_enabled_for_config;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory sqlite");
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .expect("enable foreign_keys pragma");
        misaka_x_lib::db::migrations::run_migrations(&conn).expect("run migrations");
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
        .expect("insert router_config fixture");
        conn.execute(
            "INSERT INTO custom_models (
                id, router_config_id, model_id, display_name, enabled
             ) VALUES ('cm-1', 'rc-1', 'gpt-4o', 'GPT-4o', 1)",
            [],
        )
        .expect("insert enabled custom_model fixture");

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
        .expect("insert router_config fixture");
        conn.execute(
            "INSERT INTO custom_models (
                id, router_config_id, model_id, display_name, enabled
             ) VALUES ('cm-1', 'rc-1', 'gpt-4o', 'GPT-4o', 0)",
            [],
        )
        .expect("insert disabled custom_model fixture");

        let result = ensure_model_enabled_for_config(&conn, "rc-1", "gpt-4o");

        assert!(result.is_err());
    }
}
