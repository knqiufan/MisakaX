#[cfg(feature = "test-private")]
#[test]
fn test_resolve_model_spec_with_override() {
    let spec = misaka_x_lib::commands::chat::resolve_model_spec(
        Some("config1:gpt-4o"),
        Some("config2:claude"),
    )
    .unwrap();
    assert_eq!(spec.config_id, "config1");
    assert_eq!(spec.model_id, "gpt-4o");
}

#[cfg(feature = "test-private")]
#[test]
fn test_resolve_model_spec_fallback_to_session() {
    let spec =
        misaka_x_lib::commands::chat::resolve_model_spec(None, Some("config2:claude-sonnet-4"))
            .unwrap();
    assert_eq!(spec.config_id, "config2");
    assert_eq!(spec.model_id, "claude-sonnet-4");
}

#[cfg(feature = "test-private")]
#[test]
fn test_resolve_model_spec_no_model() {
    let result = misaka_x_lib::commands::chat::resolve_model_spec(None, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No model specified"));
}

#[cfg(feature = "test-private")]
#[test]
fn test_resolve_model_spec_invalid_format() {
    let result = misaka_x_lib::commands::chat::resolve_model_spec(Some("no-colon"), None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid model format"));
}

#[cfg(feature = "test-private")]
#[test]
fn test_resolve_model_spec_with_multiple_colons() {
    let spec =
        misaka_x_lib::commands::chat::resolve_model_spec(Some("config:model:extra"), None).unwrap();
    assert_eq!(spec.config_id, "config");
    assert_eq!(spec.model_id, "model:extra");
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
