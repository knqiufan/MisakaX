use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::repository::MessageRepo;
use rusqlite::Connection;

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();

    conn.execute("INSERT INTO sessions (id, title) VALUES ('s1', 'Test')", [])
        .unwrap();
    conn
}

/// 使用显式时间戳插入消息，避免 CURRENT_TIMESTAMP 的秒级精度问题
fn insert_message_at(
    conn: &Connection,
    id: &str,
    session_id: &str,
    role: &str,
    content: &str,
    timestamp: &str,
    attachments: Option<&str>,
) {
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, attachments, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'complete', ?6)",
        rusqlite::params![id, session_id, role, content, attachments, timestamp],
    )
    .unwrap();
}

fn insert_assistant_at(
    conn: &Connection,
    id: &str,
    session_id: &str,
    content: &str,
    model: &str,
    timestamp: &str,
) {
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, model, status, created_at)
         VALUES (?1, ?2, 'assistant', ?3, ?4, 'complete', ?5)",
        rusqlite::params![id, session_id, content, model, timestamp],
    )
    .unwrap();
}

// ─── 基本插入/查询 ────────────────────────────────────────────────────

#[test]
fn test_insert_and_find_user_message() {
    let conn = setup_db();
    MessageRepo::insert_user_message(&conn, "m1", "s1", "Hello world", None).unwrap();

    let messages = MessageRepo::find_recent(&conn, "s1", 10).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].id, "m1");
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[0].content, "Hello world");
    assert_eq!(messages[0].status, "complete");
    assert!(messages[0].attachments.is_none());
}

#[test]
fn test_insert_user_message_with_attachments() {
    let conn = setup_db();
    let attachments = r#"[{"data":"base64","media_type":"image/png","file_name":"test.png"}]"#;
    MessageRepo::insert_user_message(&conn, "m1", "s1", "Look at this", Some(attachments))
        .unwrap();

    let messages = MessageRepo::find_recent(&conn, "s1", 10).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].attachments.as_deref(), Some(attachments));
}

#[test]
fn test_insert_assistant_placeholder_and_update() {
    let conn = setup_db();
    MessageRepo::insert_assistant_placeholder(&conn, "m1", "s1", "gpt-4o").unwrap();

    let messages = MessageRepo::find_recent(&conn, "s1", 10).unwrap();
    assert_eq!(messages[0].status, "streaming");
    assert_eq!(messages[0].content, "");
    assert_eq!(messages[0].model, Some("gpt-4o".to_string()));

    MessageRepo::update_assistant_content(
        &conn,
        "m1",
        "Hello! I'm an AI assistant.",
        Some("Let me think about this..."),
        Some(r#"{"input_tokens":100,"output_tokens":50,"total_tokens":150}"#),
        false,
    )
    .unwrap();

    let messages = MessageRepo::find_recent(&conn, "s1", 10).unwrap();
    assert_eq!(messages[0].status, "complete");
    assert_eq!(messages[0].content, "Hello! I'm an AI assistant.");
    assert_eq!(
        messages[0].thinking_content.as_deref(),
        Some("Let me think about this...")
    );
    assert!(messages[0].token_usage.is_some());
}

#[test]
fn test_update_assistant_content_aborted() {
    let conn = setup_db();
    MessageRepo::insert_assistant_placeholder(&conn, "m1", "s1", "gpt-4o").unwrap();
    MessageRepo::update_assistant_content(&conn, "m1", "Partial response...", None, None, true)
        .unwrap();

    let messages = MessageRepo::find_recent(&conn, "s1", 10).unwrap();
    assert_eq!(messages[0].status, "aborted");
    assert_eq!(messages[0].content, "Partial response...");
}

// ─── find_recent 分页与排序 ───────────────────────────────────────────

#[test]
fn test_find_recent_with_limit() {
    let conn = setup_db();
    for i in 0..10 {
        insert_message_at(
            &conn,
            &format!("m{i}"),
            "s1",
            "user",
            &format!("Message {i}"),
            &format!("2025-01-01T00:{i:02}:00"),
            None,
        );
    }

    let messages = MessageRepo::find_recent(&conn, "s1", 3).unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].content, "Message 7");
    assert_eq!(messages[2].content, "Message 9");
}

#[test]
fn test_find_recent_returns_chronological_order() {
    let conn = setup_db();
    insert_message_at(&conn, "m1", "s1", "user", "First", "2025-01-01T00:01:00", None);
    insert_assistant_at(&conn, "m2", "s1", "Response", "gpt-4o", "2025-01-01T00:02:00");
    insert_message_at(&conn, "m3", "s1", "user", "Second", "2025-01-01T00:03:00", None);

    let messages = MessageRepo::find_recent(&conn, "s1", 10).unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[0].content, "First");
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(messages[2].role, "user");
    assert_eq!(messages[2].content, "Second");
}

// ─── find_before 分页 ──────────────────────────────────────────────────

#[test]
fn test_find_before_pagination() {
    let conn = setup_db();
    for i in 0..5 {
        insert_message_at(
            &conn,
            &format!("m{i}"),
            "s1",
            "user",
            &format!("Message {i}"),
            &format!("2025-01-01T00:{i:02}:00"),
            None,
        );
    }

    let before = MessageRepo::find_before(&conn, "s1", "m3", 10).unwrap();
    assert_eq!(before.len(), 3);
    assert_eq!(before[0].id, "m0");
    assert_eq!(before[2].id, "m2");
}

#[test]
fn test_find_before_with_limit() {
    let conn = setup_db();
    for i in 0..10 {
        insert_message_at(
            &conn,
            &format!("m{i}"),
            "s1",
            "user",
            &format!("Message {i}"),
            &format!("2025-01-01T00:{i:02}:00"),
            None,
        );
    }

    let before = MessageRepo::find_before(&conn, "s1", "m9", 3).unwrap();
    assert_eq!(before.len(), 3);
    assert_eq!(before[2].id, "m8");
}

// ─── regeneration context ──────────────────────────────────────────────

#[test]
fn test_find_regeneration_context() {
    let conn = setup_db();
    insert_message_at(&conn, "m1", "s1", "user", "First user msg", "2025-01-01T00:01:00", None);
    insert_assistant_at(&conn, "m2", "s1", "First response", "gpt-4o", "2025-01-01T00:02:00");
    insert_message_at(
        &conn,
        "m3",
        "s1",
        "user",
        "Second user msg",
        "2025-01-01T00:03:00",
        None,
    );
    insert_assistant_at(&conn, "m4", "s1", "Second response", "gpt-4o", "2025-01-01T00:04:00");

    let ctx = MessageRepo::find_regeneration_context(&conn, "s1", "m4").unwrap();
    assert_eq!(ctx.user_content, "Second user msg");
    assert_eq!(ctx.user_msg_id, "m3");
    assert!(ctx.user_attachments.is_none());
    assert_eq!(ctx.messages_before.len(), 2);
    assert_eq!(ctx.messages_before[0].id, "m1");
    assert_eq!(ctx.messages_before[1].id, "m2");
}

#[test]
fn test_find_regeneration_context_with_attachments() {
    let conn = setup_db();
    let attachments = r#"[{"data":"img","media_type":"image/png","file_name":null}]"#;
    insert_message_at(
        &conn,
        "m1",
        "s1",
        "user",
        "Look at this",
        "2025-01-01T00:01:00",
        Some(attachments),
    );
    insert_assistant_at(&conn, "m2", "s1", "I see an image", "gpt-4o", "2025-01-01T00:02:00");

    let ctx = MessageRepo::find_regeneration_context(&conn, "s1", "m2").unwrap();
    assert_eq!(ctx.user_content, "Look at this");
    assert_eq!(ctx.user_attachments.as_deref(), Some(attachments));
}

// ─── delete_from ───────────────────────────────────────────────────────

#[test]
fn test_delete_from() {
    let conn = setup_db();
    insert_message_at(&conn, "m1", "s1", "user", "Keep me", "2025-01-01T00:01:00", None);
    insert_assistant_at(&conn, "m2", "s1", "Keep me too", "gpt-4o", "2025-01-01T00:02:00");
    insert_message_at(
        &conn,
        "m3",
        "s1",
        "user",
        "Delete from here",
        "2025-01-01T00:03:00",
        None,
    );
    insert_assistant_at(&conn, "m4", "s1", "Delete me", "gpt-4o", "2025-01-01T00:04:00");

    MessageRepo::delete_from(&conn, "s1", "m3").unwrap();

    let messages = MessageRepo::find_recent(&conn, "s1", 10).unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].id, "m1");
    assert_eq!(messages[1].id, "m2");
}

// ─── 会话隔离 ──────────────────────────────────────────────────────────

#[test]
fn test_different_sessions_are_isolated() {
    let conn = setup_db();
    conn.execute("INSERT INTO sessions (id, title) VALUES ('s2', 'Test2')", [])
        .unwrap();

    MessageRepo::insert_user_message(&conn, "m1", "s1", "Session 1", None).unwrap();
    MessageRepo::insert_user_message(&conn, "m2", "s2", "Session 2", None).unwrap();

    let s1_messages = MessageRepo::find_recent(&conn, "s1", 10).unwrap();
    let s2_messages = MessageRepo::find_recent(&conn, "s2", 10).unwrap();

    assert_eq!(s1_messages.len(), 1);
    assert_eq!(s1_messages[0].content, "Session 1");
    assert_eq!(s2_messages.len(), 1);
    assert_eq!(s2_messages[0].content, "Session 2");
}
