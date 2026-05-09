use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::repository::{SessionRepo, WorkspaceRepo};
use rusqlite::Connection;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();
    conn
}

// ─── create ─────────────────────────────────────────────────────────

#[test]
fn create_session_without_working_directory() {
    let conn = create_test_db();

    let session = SessionRepo::create(&conn, "sess-1", Some("Test Chat"), None, None).unwrap();

    assert_eq!(session.id, "sess-1");
    assert_eq!(session.title, Some("Test Chat".to_string()));
    assert_eq!(session.working_directory, None);
    assert_eq!(session.project_name, None);
    assert_eq!(session.status, "active");
    assert_eq!(session.mode, "agent");
}

#[test]
fn create_session_with_working_directory() {
    let conn = create_test_db();

    let session = SessionRepo::create(
        &conn,
        "sess-2",
        None,
        Some("config1:gpt-4"),
        Some("/home/user/my-project"),
    )
    .unwrap();

    assert_eq!(session.id, "sess-2");
    assert_eq!(session.working_directory, Some("/home/user/my-project".to_string()));
    assert_eq!(session.project_name, Some("my-project".to_string()));
    assert_eq!(session.model, Some("config1:gpt-4".to_string()));
}

#[test]
fn create_session_extracts_project_name_from_path() {
    let conn = create_test_db();

    let session = SessionRepo::create(
        &conn,
        "sess-3",
        None,
        None,
        Some("D:\\code\\Misaka-Tauri"),
    )
    .unwrap();

    assert_eq!(session.project_name, Some("Misaka-Tauri".to_string()));
}

#[test]
fn create_session_persists_to_sqlite() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "sess-persist", Some("Persisted"), None, Some("/tmp/project"))
        .unwrap();

    let loaded = SessionRepo::find_by_id(&conn, "sess-persist").unwrap();
    assert_eq!(loaded.title, Some("Persisted".to_string()));
    assert_eq!(loaded.working_directory, Some("/tmp/project".to_string()));
    assert_eq!(loaded.project_name, Some("project".to_string()));
}

// ─── update_working_directory ────────────────────────────────────────

#[test]
fn update_working_directory_sets_new_dir() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "sess-upd", None, None, None).unwrap();

    SessionRepo::update_working_directory(&conn, "sess-upd", Some("/new/path/project-x"))
        .unwrap();

    let session = SessionRepo::find_by_id(&conn, "sess-upd").unwrap();
    assert_eq!(session.working_directory, Some("/new/path/project-x".to_string()));
    assert_eq!(session.project_name, Some("project-x".to_string()));
}

#[test]
fn update_working_directory_can_clear() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "sess-clr", None, None, Some("/initial/dir")).unwrap();

    SessionRepo::update_working_directory(&conn, "sess-clr", None).unwrap();

    let session = SessionRepo::find_by_id(&conn, "sess-clr").unwrap();
    assert_eq!(session.working_directory, None);
    assert_eq!(session.project_name, None);
}

// ─── 工作目录绑定与 recent_directories 集成 ─────────────────────────

#[test]
fn workspace_binding_records_usage_in_recent_directories() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "sess-rec", None, None, Some("/projects/awesome")).unwrap();

    let id = uuid::Uuid::new_v4().to_string();
    WorkspaceRepo::record_usage(&conn, &id, "/projects/awesome", Some("awesome")).unwrap();

    let recent = WorkspaceRepo::find_recent(&conn, 10).unwrap();
    assert!(!recent.is_empty());
    assert!(recent.iter().any(|d| d.path == "/projects/awesome"));
}

// ─── update_stats ───────────────────────────────────────────────────

#[test]
fn update_stats_increments_token_counts() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "sess-stats", None, None, None).unwrap();

    SessionRepo::update_stats(&conn, "sess-stats", Some(100), Some(50)).unwrap();
    SessionRepo::update_stats(&conn, "sess-stats", Some(200), Some(100)).unwrap();

    let total_input: i64 = conn
        .query_row(
            "SELECT total_input_tokens FROM sessions WHERE id = 'sess-stats'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let total_output: i64 = conn
        .query_row(
            "SELECT total_output_tokens FROM sessions WHERE id = 'sess-stats'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(total_input, 300);
    assert_eq!(total_output, 150);
}

#[test]
fn update_stats_without_tokens_updates_timestamp() {
    let conn = create_test_db();

    SessionRepo::create(&conn, "sess-ts", None, None, None).unwrap();

    SessionRepo::update_stats(&conn, "sess-ts", None, None).unwrap();

    let last: Option<String> = conn
        .query_row(
            "SELECT last_message_at FROM sessions WHERE id = 'sess-ts'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(last.is_some());
}
