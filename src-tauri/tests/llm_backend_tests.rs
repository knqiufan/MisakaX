use misaka_x_lib::db::models::Message;
use misaka_x_lib::services::llm::backend::{
    build_rig_chat_history, build_user_prompt, ImageAttachment,
};
use rig::completion::message::Message as RigMessage;

// ─── ImageAttachment 序列化/反序列化 ───────────────────────────────────

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
fn test_image_attachment_roundtrip() {
    let original = ImageAttachment {
        data: "SGVsbG8gV29ybGQ=".to_string(),
        media_type: "image/webp".to_string(),
        file_name: Some("screenshot.webp".to_string()),
    };
    let json = serde_json::to_string(&original).unwrap();
    let restored: ImageAttachment = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.data, original.data);
    assert_eq!(restored.media_type, original.media_type);
    assert_eq!(restored.file_name, original.file_name);
}

#[test]
fn test_image_attachment_list_serialize() {
    let attachments = vec![
        ImageAttachment {
            data: "img1".to_string(),
            media_type: "image/png".to_string(),
            file_name: None,
        },
        ImageAttachment {
            data: "img2".to_string(),
            media_type: "image/jpeg".to_string(),
            file_name: Some("photo.jpg".to_string()),
        },
    ];
    let json = serde_json::to_string(&attachments).unwrap();
    let restored: Vec<ImageAttachment> = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.len(), 2);
    assert_eq!(restored[0].data, "img1");
    assert_eq!(restored[1].file_name, Some("photo.jpg".to_string()));
}

// ─── build_rig_chat_history ────────────────────────────────────────────

fn make_message(id: &str, role: &str, content: &str, attachments: Option<&str>) -> Message {
    Message {
        id: id.to_string(),
        session_id: "s1".to_string(),
        role: role.to_string(),
        content: content.to_string(),
        token_usage: None,
        model: None,
        thinking_content: None,
        attachments: attachments.map(|s| s.to_string()),
        status: "complete".to_string(),
        tool_calls: None,
        created_at: "2025-01-01T00:00:00".to_string(),
    }
}

#[test]
fn test_build_rig_chat_history_empty() {
    let messages: Vec<Message> = vec![];
    let history = build_rig_chat_history(&messages);
    assert!(history.is_empty());
}

#[test]
fn test_build_rig_chat_history_filters_system_role() {
    let messages = vec![
        make_message("m1", "user", "Hello", None),
        make_message("m2", "assistant", "Hi!", None),
        make_message("m3", "system", "You are helpful", None),
    ];
    let history = build_rig_chat_history(&messages);
    assert_eq!(history.len(), 2);
}

#[test]
fn test_build_rig_chat_history_preserves_order() {
    let messages = vec![
        make_message("m1", "user", "First", None),
        make_message("m2", "assistant", "Second", None),
        make_message("m3", "user", "Third", None),
    ];
    let history = build_rig_chat_history(&messages);
    assert_eq!(history.len(), 3);
}

#[test]
fn test_build_rig_chat_history_with_image_attachments() {
    let attachment_json = r#"[{"data":"base64img","media_type":"image/png","file_name":null}]"#;
    let messages = vec![
        make_message("m1", "user", "Look at this image", Some(attachment_json)),
        make_message("m2", "assistant", "I see the image", None),
    ];

    let history = build_rig_chat_history(&messages);
    assert_eq!(history.len(), 2);

    match &history[0] {
        RigMessage::User { content } => {
            assert_eq!(content.len(), 2);
        }
        _ => panic!("Expected User message"),
    }
}

#[test]
fn test_build_rig_chat_history_with_invalid_attachments_json() {
    let messages = vec![make_message(
        "m1",
        "user",
        "text message",
        Some("not-valid-json"),
    )];

    let history = build_rig_chat_history(&messages);
    assert_eq!(history.len(), 1);

    match &history[0] {
        RigMessage::User { content } => {
            assert_eq!(content.len(), 1);
        }
        _ => panic!("Expected User message"),
    }
}

#[test]
fn test_build_rig_chat_history_with_empty_attachments() {
    let messages = vec![make_message("m1", "user", "text only", Some("[]"))];

    let history = build_rig_chat_history(&messages);
    assert_eq!(history.len(), 1);

    match &history[0] {
        RigMessage::User { content } => {
            assert_eq!(content.len(), 1);
        }
        _ => panic!("Expected User message"),
    }
}

// ─── build_user_prompt ─────────────────────────────────────────────────

#[test]
fn test_build_user_prompt_text_only() {
    let prompt = build_user_prompt("Hello world", &None);
    match &prompt {
        RigMessage::User { content } => {
            assert_eq!(content.len(), 1);
        }
        _ => panic!("Expected User message"),
    }
}

#[test]
fn test_build_user_prompt_with_empty_images() {
    let images: Option<Vec<ImageAttachment>> = Some(vec![]);
    let prompt = build_user_prompt("Hello", &images);
    match &prompt {
        RigMessage::User { content } => {
            assert_eq!(content.len(), 1);
        }
        _ => panic!("Expected User message"),
    }
}

#[test]
fn test_build_user_prompt_with_single_image() {
    let images = Some(vec![ImageAttachment {
        data: "base64data".to_string(),
        media_type: "image/png".to_string(),
        file_name: Some("test.png".to_string()),
    }]);
    let prompt = build_user_prompt("Describe this image", &images);
    match &prompt {
        RigMessage::User { content } => {
            assert_eq!(content.len(), 2);
        }
        _ => panic!("Expected User message"),
    }
}

#[test]
fn test_build_user_prompt_with_multiple_images() {
    let images = Some(vec![
        ImageAttachment {
            data: "img1_base64".to_string(),
            media_type: "image/png".to_string(),
            file_name: None,
        },
        ImageAttachment {
            data: "img2_base64".to_string(),
            media_type: "image/jpeg".to_string(),
            file_name: None,
        },
        ImageAttachment {
            data: "img3_base64".to_string(),
            media_type: "image/webp".to_string(),
            file_name: None,
        },
    ]);
    let prompt = build_user_prompt("Compare these images", &images);
    match &prompt {
        RigMessage::User { content } => {
            assert_eq!(content.len(), 4);
        }
        _ => panic!("Expected User message"),
    }
}

#[test]
fn test_build_user_prompt_none_images() {
    let prompt = build_user_prompt("Simple text", &None);
    match prompt {
        RigMessage::User { .. } => {}
        _ => panic!("Expected User message"),
    }
}
