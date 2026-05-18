use misaka_x_lib::services::llm::backend::{build_user_prompt, MessageAttachment};
use rig::completion::message::Message as RigMessage;

#[test]
fn deserialize_legacy_image_attachment_array() {
    let legacy = r#"[{"data":"AAA","media_type":"image/png","file_name":"a.png"}]"#;
    let parsed: Vec<MessageAttachment> = serde_json::from_str(legacy).unwrap();
    assert!(matches!(parsed[0], MessageAttachment::Image { .. }));
}

#[test]
fn deserialize_text_attachment_array() {
    let json = r#"[{"kind":"text","extracted_text":"hello","mime":"text/markdown","file_name":"notes.md","size":5}]"#;
    let parsed: Vec<MessageAttachment> = serde_json::from_str(json).unwrap();
    assert!(matches!(parsed[0], MessageAttachment::Text { .. }));
}

#[test]
fn build_user_prompt_prepends_text_attachment_content() {
    let attachments = Some(vec![MessageAttachment::Text {
        extracted_text: "# Notes".to_string(),
        mime: "text/markdown".to_string(),
        file_name: "notes.md".to_string(),
        size: 7,
    }]);

    let prompt = build_user_prompt("Summarize", &attachments);

    match prompt {
        RigMessage::User { content } => {
            let text = format!("{:?}", content);
            assert!(text.contains("notes.md"));
            assert!(text.contains("# Notes"));
            assert!(text.contains("Summarize"));
        }
        _ => panic!("Expected user prompt"),
    }
}
