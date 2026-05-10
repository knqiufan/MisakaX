use rusqlite::Connection;
use misaka_x_lib::db::migrations::run_migrations;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    conn
}

#[test]
fn test_migration_v1() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();

    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM _schema_version", [], |row| row.get(0))
        .unwrap();
    assert!(version >= 1);

    conn.execute(
        "INSERT INTO sessions (id, title) VALUES ('s1', 'Test Session')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content) VALUES ('m1', 's1', 'user', 'hello')",
        [],
    )
    .unwrap();
}

#[test]
fn test_migration_v2_adds_custom_models_table() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO router_configs (id, name, provider) VALUES ('rc1', 'Test', 'openai')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO custom_models (id, router_config_id, model_id, display_name)
         VALUES ('cm1', 'rc1', 'gpt-4o-ft', 'Fine-tuned')",
        [],
    )
    .unwrap();

    let display_name: String = conn
        .query_row(
            "SELECT display_name FROM custom_models WHERE id = 'cm1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(display_name, "Fine-tuned");
}

#[test]
fn test_migration_v2_adds_api_compat_column() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO router_configs (id, name, provider, api_compat)
         VALUES ('rc1', 'DeepSeek', 'deepseek', 'openai')",
        [],
    )
    .unwrap();

    let compat: Option<String> = conn
        .query_row(
            "SELECT api_compat FROM router_configs WHERE id = 'rc1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(compat, Some("openai".to_string()));
}

#[test]
fn test_migration_v2_adds_session_token_columns() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO sessions (id, title, total_input_tokens, total_output_tokens)
         VALUES ('s1', 'Test', 100, 200)",
        [],
    )
    .unwrap();

    let (input, output): (i32, i32) = conn
        .query_row(
            "SELECT total_input_tokens, total_output_tokens FROM sessions WHERE id = 's1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(input, 100);
    assert_eq!(output, 200);
}

#[test]
fn test_migration_v2_adds_message_columns() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO sessions (id) VALUES ('s1')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, model, thinking_content, status)
         VALUES ('m1', 's1', 'assistant', 'Hello!', 'gpt-4o', 'I need to greet the user.', 'complete')",
        [],
    )
    .unwrap();

    let (model, thinking, status): (Option<String>, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT model, thinking_content, status FROM messages WHERE id = 'm1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(model, Some("gpt-4o".to_string()));
    assert_eq!(thinking, Some("I need to greet the user.".to_string()));
    assert_eq!(status, Some("complete".to_string()));
}

#[test]
fn test_migration_v2_creates_recent_directories() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO recent_directories (id, path, display_name, use_count)
         VALUES ('rd1', '/home/user/project', 'project', 3)",
        [],
    )
    .unwrap();

    let count: i32 = conn
        .query_row(
            "SELECT use_count FROM recent_directories WHERE id = 'rd1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_migration_idempotent() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();
    run_migrations(&conn).unwrap();

    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM _schema_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 3);
}

#[test]
fn test_custom_models_cascade_delete() {
    let conn = create_test_db();
    run_migrations(&conn).unwrap();

    conn.execute(
        "INSERT INTO router_configs (id, name, provider) VALUES ('rc1', 'Test', 'openai')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO custom_models (id, router_config_id, model_id, display_name)
         VALUES ('cm1', 'rc1', 'model-1', 'Model 1')",
        [],
    )
    .unwrap();

    conn.execute("DELETE FROM router_configs WHERE id = 'rc1'", [])
        .unwrap();

    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM custom_models WHERE router_config_id = 'rc1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0, "Custom models should be cascade deleted");
}
