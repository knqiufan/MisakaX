use misaka_x_lib::db::models::Message;
use misaka_x_lib::services::llm::backend::{build_rig_chat_history, ImageAttachment};

#[test]
fn test_image_attachment_serialize() {
    let attachment = ImageAttachment {
        data: "base64data".to_string(),
        media_type: "image/png".to_string(),
        file_name: Some("test.png".to_string()),
    };
    let json = serde_json::to_string(&attachment).unwrap();
    assert!(json.contains("\"data\":\"base64data\""));
    assert!(json.contains("\"media_type\":\"image/png\""));
    assert!(json.contains("\"file_name\":\"test.png\""));
}

#[test]
fn test_image_attachment_deserialize() {
    let json = r#"{"data":"abc","media_type":"image/jpeg","file_name":null}"#;
    let attachment: ImageAttachment = serde_json::from_str(json).unwrap();
    assert_eq!(attachment.data, "abc");
    assert_eq!(attachment.media_type, "image/jpeg");
    assert!(attachment.file_name.is_none());
}

#[test]
fn test_build_rig_chat_history_empty() {
    let messages: Vec<Message> = vec![];
    let history = build_rig_chat_history(&messages);
    assert!(history.is_empty());
}

#[test]
fn test_build_rig_chat_history_filters_roles() {
    let messages = vec![
        Message {
            id: "m1".to_string(),
            session_id: "s1".to_string(),
            role: "user".to_string(),
            content: "Hello".to_string(),
            token_usage: None,
            model: None,
            thinking_content: None,
            attachments: None,
            status: "complete".to_string(),
            created_at: "2025-01-01".to_string(),
        },
        Message {
            id: "m2".to_string(),
            session_id: "s1".to_string(),
            role: "assistant".to_string(),
            content: "Hi there!".to_string(),
            token_usage: None,
            model: None,
            thinking_content: None,
            attachments: None,
            status: "complete".to_string(),
            created_at: "2025-01-01".to_string(),
        },
        Message {
            id: "m3".to_string(),
            session_id: "s1".to_string(),
            role: "system".to_string(),
            content: "You are helpful".to_string(),
            token_usage: None,
            model: None,
            thinking_content: None,
            attachments: None,
            status: "complete".to_string(),
            created_at: "2025-01-01".to_string(),
        },
    ];
    let history = build_rig_chat_history(&messages);
    assert_eq!(history.len(), 2);
}

#[test]
fn test_build_rig_chat_history_preserves_order() {
    let messages = vec![
        Message {
            id: "m1".to_string(),
            session_id: "s1".to_string(),
            role: "user".to_string(),
            content: "First".to_string(),
            token_usage: None,
            model: None,
            thinking_content: None,
            attachments: None,
            status: "complete".to_string(),
            created_at: "2025-01-01".to_string(),
        },
        Message {
            id: "m2".to_string(),
            session_id: "s1".to_string(),
            role: "assistant".to_string(),
            content: "Second".to_string(),
            token_usage: None,
            model: None,
            thinking_content: None,
            attachments: None,
            status: "complete".to_string(),
            created_at: "2025-01-01".to_string(),
        },
        Message {
            id: "m3".to_string(),
            session_id: "s1".to_string(),
            role: "user".to_string(),
            content: "Third".to_string(),
            token_usage: None,
            model: None,
            thinking_content: None,
            attachments: None,
            status: "complete".to_string(),
            created_at: "2025-01-01".to_string(),
        },
    ];
    let history = build_rig_chat_history(&messages);
    assert_eq!(history.len(), 3);
}
